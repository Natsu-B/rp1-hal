//! All-channel single-block DMA, bounded cancellation and controller recovery.
#![no_std]
#![no_main]

#[path = "../../status-nonce.rs"]
mod status_nonce;

use core::{cell::UnsafeCell, fmt};
use rp1_hal::{
    clocks::Clocks,
    dma::{BusAddr, Channel, Error, Lease, Pool, Transfer, width_log2},
    log,
    pac::dw_axi_dmac::{BASE, CHANNEL_BASE, CHANNEL_STRIDE, ch, common, int},
    timer::{Deadline, Duration, Instant, Timer, WaitTimer},
};
use rp1_rt::{Proc0, Proc1, entry};

rp1_rt::bind!();

const SIZES: [usize; 3] = [4, 64, 4096];
const REPEATS: u32 = 1_000;
const CAPACITY: usize = 4096 + 32;
const CANARY: u8 = 0xa5;
const CANCEL_OFFSET: usize = 17;
const CANCEL_BYTES: usize = 4096;
const TIMEOUT: Duration = Duration::millis(100);
const STOP_LIMIT: Duration = Duration::millis(1);
// Allow for software overhead and the final observation inside the full 1 ms gate.
const RECHECK_US: u64 = 900;
const TERMINAL: u32 = int::CH_DISABLED | int::DMA_TFR_DONE;
const FIXTURE_SLOT: usize = 544;

#[repr(align(16))]
struct FixtureMemory(UnsafeCell<[[u8; 2 * FIXTURE_SLOT]; 2]>);
// SAFETY: the single proc1 entry takes exclusive ownership.
unsafe impl Sync for FixtureMemory {}
#[unsafe(link_section = ".shared")]
static FIXTURES: FixtureMemory = FixtureMemory(UnsafeCell::new([[0; 2 * FIXTURE_SLOT]; 2]));

#[unsafe(link_section = ".shared")]
static mut FAULT_DESTINATION: [u8; 16] = [0; 16];

type Pair<'a> = (Lease<'a>, Lease<'a>);

#[entry(proc0)]
fn main0(mut core: Proc0, p: rp1::Proc0Resources) -> ! {
    let sys = rp1::platform::init(&mut core, p.platform).unwrap();
    let nonce = status_nonce::publish(&sys.report);
    log!(
        "PARITY_DMA_BOOT nonce={nonce:08x} PLAT_INIT ok=1 {}\n",
        sys.report
    )
    .unwrap();
    p.proc1.launch(&mut core).unwrap();
    park()
}

#[entry(proc1)]
fn main1(core: Proc1, p: rp1::Proc1Resources) -> ! {
    // SAFETY: proc0 published the nonce before launching this core.
    let nonce = unsafe { status_nonce::WORD.read_volatile() };
    match run(core, p, nonce) {
        Ok(()) => log!(
            "PARITY_DMA_SINGLE_END nonce={nonce:08x} channels=8 copies=24000 cancels=100 ok=1\n"
        )
        .unwrap(),
        Err(error) => {
            log!("PARITY_DMA_SINGLE_END nonce={nonce:08x} ok=0 error={error:?}\n").unwrap()
        }
    }
    park()
}

fn run(core: Proc1, p: rp1::Proc1Resources, nonce: u32) -> Result<(), Error> {
    // Proc0 publishes before launch. A zero timeout performs one clock check.
    let clocks =
        Clocks::wait_from_proc0(&core, Duration::micros(0)).map_err(|_| Error::Precondition)?;
    let mut dma = p.dma.init(&clocks)?;
    let mut wait = p.wait_timer;
    let pool = p.dma_pool;
    let [src, dst] = pool.leases(CAPACITY)?;
    let mut buffers = (src, dst);
    let channels: [&mut dyn Exercise; 8] = [
        &mut dma.ch0,
        &mut dma.ch1,
        &mut dma.ch2,
        &mut dma.ch3,
        &mut dma.ch4,
        &mut dma.ch5,
        &mut dma.ch6,
        &mut dma.ch7,
    ];
    let mut partial = 0;
    for (n, channel) in channels.into_iter().enumerate() {
        partial += channel_run(channel, &mut wait, nonce, n, &mut buffers)?;
    }
    if partial == 0 {
        return Err(Error::Invalid);
    }

    prepare(&mut buffers, CAPACITY, CANCEL_OFFSET, CANCEL_BYTES)?;
    buffers = drop_trial(&mut dma.ch0, buffers, &mut wait, nonce, None)?.unwrap();

    // SAFETY: run is entered once on proc1 and these are distinct .shared reservations.
    let [forgotten, drop_fixture] = unsafe { &mut *FIXTURES.0.get() };
    let forgotten = Pool::<2>::new(forgotten)?;
    let [mut src, mut dst] = forgotten.leases(512)?;
    src.set_range(CANCEL_OFFSET, 512)?;
    dst.set_range(CANCEL_OFFSET, 512)?;
    core::mem::forget(dma.ch7.try_start(&mut src, &mut dst)?);
    if !src.as_slice().is_empty() || !dst.as_mut_slice().is_empty() {
        return Err(Error::Invalid);
    }
    drop((src, dst));
    let token = dma
        .free_until(&mut wait, Deadline::after(STOP_LIMIT)?)
        .map_err(|e| *e.error())?;
    let mut dma = token.init(&clocks)?;
    if !matches!(forgotten.lease(0, 4), Err(Error::Busy))
        || !matches!(forgotten.lease(1, 4), Err(Error::Busy))
    {
        return Err(Error::Invalid);
    }
    log!("PARITY_DMA_RECOVERY nonce={nonce:08x} forgotten=1 leases_claimed=2 reinit=1\n").unwrap();

    inject_bus_error(&mut dma.ch6, nonce)?;
    prepare(&mut buffers, CAPACITY, CANCEL_OFFSET, CANCEL_BYTES)?;
    let quarantine = quarantine_cancel(&mut dma.ch7, buffers, &pool, &mut wait, nonce);
    let drop_pool = Pool::<2>::new(drop_fixture)?;
    let [src, dst] = drop_pool.leases(512)?;
    let mut buffers = (src, dst);
    prepare(&mut buffers, FIXTURE_SLOT, CANCEL_OFFSET, 512)?;
    let dropped = drop_trial(&mut dma.ch1, buffers, &mut wait, nonce, Some(&drop_pool));
    quarantine.and(dropped.map(|_| ()))
}

trait Exercise {
    fn copy(&mut self, buffers: &mut Pair<'_>) -> Result<(), Error>;
    fn cancel(
        &mut self,
        buffers: &mut Pair<'_>,
        wait: &mut WaitTimer,
        stimulus_us: u32,
    ) -> Result<(Result<(), Error>, CancelSample), Error>;
}

impl<const N: u8> Exercise for Channel<N> {
    fn copy(&mut self, (src, dst): &mut Pair<'_>) -> Result<(), Error> {
        self.copy_polling(src, dst, TIMEOUT)
    }

    fn cancel(
        &mut self,
        (src, dst): &mut Pair<'_>,
        wait: &mut WaitTimer,
        stimulus_us: u32,
    ) -> Result<(Result<(), Error>, CancelSample), Error> {
        let timer = Timer::new();
        let mut transfer = self.try_start(src, dst)?;
        // Polled delay is an explicit stimulus just before cancellation.
        timer.delay_polling(Duration::micros(stimulus_us))?;
        let (decision, sample) = timed_cancel::<N>(&mut transfer, wait);
        if let Err(error) = decision {
            log!(
                "PARITY_DMA_CANCEL_FAIL ch={N} first_us={} decision_us={} ch_status_before=0x{:08x} ch_status_after=0x{:08x} int_status_after=0x{:08x} error={error:?}\n",
                sample.first_us, sample.decision_us, sample.before, sample.after, sample.after_ack,
            )
            .unwrap();
            return Err(error);
        }
        let result = transfer.try_reclaim().map_err(|_| Error::Recovering)?;
        Ok((result, sample))
    }
}

#[derive(Default)]
struct CancelSample {
    first_us: u32,
    decision_us: u32,
    rechecked: u32,
    before: u32,
    terminal: u32,
    after: u32,
    after_ack: u32,
}

fn timed_cancel<const N: u8>(
    transfer: &mut Transfer<'_>,
    wait: &mut WaitTimer,
) -> (Result<(), Error>, CancelSample) {
    let timer = Timer::new();
    let before = channel_read::<N>(ch::STATUS);
    let start = timer.now().ticks();
    let first = transfer.try_cancel();
    let first_us = timer.now_32().ticks().wrapping_sub(start as u32);
    let rechecked = first == Err(Error::Recovering);
    let decision = if rechecked {
        let end = Deadline::at(Instant::from_ticks(start + RECHECK_US));
        transfer.cancel_until(wait, end)
    } else {
        first
    };
    let decision_us = timer.now_32().ticks().wrapping_sub(start as u32);
    let sample = CancelSample {
        first_us,
        decision_us,
        rechecked: u32::from(rechecked),
        before,
        terminal: transfer.terminal_status(),
        after: channel_read::<N>(ch::STATUS),
        after_ack: channel_read::<N>(ch::INTSTATUS),
    };
    (decision, sample)
}

fn channel_read<const N: u8>(offset: usize) -> u32 {
    let address = BASE + CHANNEL_BASE + N as usize * CHANNEL_STRIDE + offset;
    read_register(address)
}

fn channel_enabled(n: u8) -> bool {
    read_register(BASE + common::CHEN) & (1 << n) != 0
}

fn read_register(address: usize) -> u32 {
    // SAFETY: the caller selects an aligned DMA MMIO register on the owning core.
    unsafe { (address as *const u32).read_volatile() }
}

fn write_register(address: usize, value: u32) {
    // SAFETY: only the sole DMA-owning core runs the fixture, on a reserved channel.
    unsafe { (address as *mut u32).write_volatile(value) };
}

struct Injection {
    mask_before: u32,
    mask_idle: u32,
    int_status_completed: u32,
    int_status_armed: u32,
    mask_after: u32,
}

impl fmt::Display for Injection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            " mask_before=0x{:08x} mask_idle=0x{:08x} int_status_completed=0x{:08x} int_status_armed=0x{:08x} mask_after=0x{:08x}",
            self.mask_before,
            self.mask_idle,
            self.int_status_completed,
            self.int_status_armed,
            self.mask_after,
        )
    }
}

fn start_injected<'t, const N: u8>(
    channel: &'t mut Channel<N>,
    src: &'t mut Lease<'_>,
    dst: &'t mut Lease<'_>,
    nonce: u32,
) -> Result<(Transfer<'t>, Injection), Error> {
    let source = BusAddr::from_proc(src.as_slice().as_ptr() as usize)?.0;
    let destination = BusAddr::from_proc(dst.as_slice().as_ptr() as usize)?.0;
    let bytes = src.as_slice().len() as u32;
    let transfer = channel.try_start(src, dst)?;
    let injection = rearm_masked(N, source, destination, bytes, nonce)?;
    Ok((transfer, injection))
}

#[inline(never)]
fn rearm_masked(
    n: u8,
    source: u64,
    destination: u64,
    bytes: u32,
    nonce: u32,
) -> Result<Injection, Error> {
    // HAL start installs the normal capture mask. Wait for this first block to finish
    // without polling the Transfer, so the HAL remains Active and both leases stay
    // engine-owned. Rearm exactly the same SRAM ranges with the terminal mask cleared
    // while CH_EN=0. The Transfer then issues the real graceful disable as usual.
    let timer = Timer::new();
    let start = timer.now_32().ticks();
    while channel_enabled(n) && timer.now_32().ticks().wrapping_sub(start) < STOP_LIMIT.ticks() {
        core::hint::spin_loop();
    }
    let base = BASE + CHANNEL_BASE + n as usize * CHANNEL_STRIDE;
    let idle = !channel_enabled(n);
    let int_status_completed = read_register(base + ch::INTSTATUS);
    let mask_before = read_register(base + ch::INTSTATUS_ENA);
    let mut injection = Injection {
        mask_before,
        mask_idle: mask_before,
        int_status_completed,
        int_status_armed: int_status_completed,
        mask_after: mask_before,
    };
    if idle && int_status_completed == int::DMA_TFR_DONE && mask_before & TERMINAL == TERMINAL {
        write_register(base + ch::INTCLEAR, int::ALL);
        write_register(base + ch::INTSTATUS_ENA, mask_before & !TERMINAL);
        // Restore transfer addresses and count in case the engine advanced them.
        for (offset, value) in [
            (ch::SAR, source),
            (ch::DAR, destination),
            (
                ch::BLOCK_TS,
                u64::from((bytes >> width_log2(source, destination, bytes)) - 1),
            ),
        ] {
            write_register(base + offset, value as u32);
            write_register(base + offset + 4, (value >> 32) as u32);
        }
        let mask_idle = read_register(base + ch::INTSTATUS_ENA);
        injection.mask_idle = mask_idle;
        injection.mask_after = mask_idle;
        injection.int_status_armed = read_register(base + ch::INTSTATUS);
        if mask_idle == mask_before & !TERMINAL && injection.int_status_armed == 0 {
            write_register(
                BASE + common::CHEN,
                (1 << n) | (1 << (u32::from(n) + common::CHEN_WE_SHIFT)),
            );
            injection.int_status_armed = read_register(base + ch::INTSTATUS);
            return Ok(injection);
        }
    }
    log!(
        "PARITY_DMA_INJECT_FAIL nonce={nonce:08x} ch={n} idle={}{injection}\n",
        u32::from(idle)
    )
    .unwrap();
    Err(Error::Precondition)
}

struct DropInjection(Option<Injection>);

impl fmt::Display for DropInjection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.as_ref().map_or(Ok(()), |snapshot| snapshot.fmt(f))
    }
}

fn drop_trial<'a, const N: u8>(
    channel: &mut Channel<N>,
    (mut src, mut dst): Pair<'a>,
    wait: &mut WaitTimer,
    nonce: u32,
    fixture: Option<&Pool<2>>,
) -> Result<Option<Pair<'a>>, Error> {
    src.as_mut_slice().fill(0x3c);
    let (transfer, mut injection) = if fixture.is_some() {
        let (transfer, injection) = start_injected(channel, &mut src, &mut dst, nonce)?;
        (transfer, Some(injection))
    } else {
        (channel.try_start(&mut src, &mut dst)?, None)
    };
    let injected = injection.is_some();
    let active = channel_enabled(N);
    let timer = Timer::new();
    let start = timer.now_32().ticks();
    drop(transfer);
    let drop_us = timer.now_32().ticks().wrapping_sub(start);
    let held = dst.is_quarantined();
    let valid_claims = src.is_quarantined() == held
        && src.as_slice().is_empty() == held
        && dst.as_mut_slice().is_empty() == held;
    let signature = crc32(dst.as_slice());
    wait.delay(Duration::micros(100))?;
    let late_diff = u32::from(signature != crc32(dst.as_slice()));
    let chen_after = u32::from(channel_enabled(N));
    let int_status_after = channel_read::<N>(ch::INTSTATUS);
    let good = active
        && valid_claims
        && held == injected
        && drop_us <= STOP_LIMIT.ticks()
        && late_diff == 0
        && chen_after == 0
        && int_status_after & (TERMINAL | int::ALL_ERR | int::CH_ABORTED) == 0;
    let mut buffers = Some((src, dst));
    let mut good = good;
    let pair_busy = if let (Some(injection), Some(pool)) = (&mut injection, fixture) {
        // Release the handles before trying to lease the pair again.
        drop(buffers.take());
        let busy = matches!(pool.leases(4), Err(Error::Busy));
        injection.mask_after = channel_read::<N>(ch::INTSTATUS_ENA);
        good &=
            busy && injection.int_status_armed == 0 && injection.mask_after == injection.mask_idle;
        busy
    } else {
        false
    };
    log!(
        "PARITY_DMA_DROP nonce={nonce:08x} ch={N} injected={} active={} drop_us={drop_us} quarantined={} chen_after={chen_after} int_status_after=0x{int_status_after:08x} late_diff={late_diff} pair_busy={}{} ok={}\n",
        u32::from(injected), u32::from(active), u32::from(held), u32::from(pair_busy),
        DropInjection(injection), u32::from(good),
    )
    .unwrap();
    if good {
        Ok(buffers)
    } else {
        Err(Error::Invalid)
    }
}

fn quarantine_cancel<const N: u8>(
    channel: &mut Channel<N>,
    (mut src, mut dst): Pair<'_>,
    pool: &Pool<2>,
    wait: &mut WaitTimer,
    nonce: u32,
) -> Result<(), Error> {
    let (mut transfer, mut injection) = start_injected(channel, &mut src, &mut dst, nonce)?;
    let active = channel_enabled(N);
    let (decision, sample) = timed_cancel::<N>(&mut transfer, wait);
    drop(transfer);
    let quarantined = src.is_quarantined()
        && dst.is_quarantined()
        && src.as_slice().is_empty()
        && dst.as_mut_slice().is_empty();
    drop((src, dst));
    let claimed = matches!(pool.lease(0, 4), Err(Error::Busy))
        && matches!(pool.lease(1, 4), Err(Error::Busy));
    let chen_after = u32::from(channel_enabled(N));
    injection.mask_after = channel_read::<N>(ch::INTSTATUS_ENA);
    let good = active
        && injection.int_status_armed == 0
        && injection.mask_after == injection.mask_idle
        && decision == Err(Error::Recovering)
        && sample.rechecked == 1
        && sample.decision_us <= STOP_LIMIT.ticks()
        && chen_after == 0
        && sample.after_ack == 0
        && quarantined
        && claimed;
    log!(
        "PARITY_DMA_QUARANTINE nonce={nonce:08x} ch={N} active={} first_us={} decision_us={} rechecked={} ch_status_before=0x{:08x} ch_status_after=0x{:08x} int_status_after=0x{:08x} chen_after={chen_after} quarantined={} leases_claimed={}{injection} ok={}\n",
        u32::from(active), sample.first_us, sample.decision_us, sample.rechecked,
        sample.before, sample.after, sample.after_ack, u32::from(quarantined),
        2 * u32::from(claimed), u32::from(good),
    )
    .unwrap();
    if good { Ok(()) } else { Err(Error::Invalid) }
}

fn inject_bus_error(channel: &mut Channel<6>, nonce: u32) -> Result<(), Error> {
    // A read from an unmapped RP1 bus address cannot write outside the dedicated destination.
    // SAFETY: the destination occupies a private shared-SRAM static retired for the epoch.
    let dst = BusAddr::from_proc((&raw mut FAULT_DESTINATION) as usize)?;
    let mut transfer = unsafe { channel.start_raw(BusAddr(0xc0_6000_0000), dst, 16) }?;
    let result = transfer.wait_polling(Deadline::after(STOP_LIMIT)?);
    let latched = channel_read::<6>(ch::INTSTATUS);
    let (status, stopped) = match result {
        Err(Error::Bus(status)) => (status, 1),
        Err(Error::Recovering) => (latched, 0),
        Err(error) => return Err(error),
        Ok(()) => return Err(Error::Invalid),
    };
    drop(transfer);
    let retry = channel.try_recover();
    let valid_recovery = if stopped == 1 {
        retry.is_ok() && latched == 0
    } else {
        // A status bit may arrive between the wait and this bounded recovery check.
        matches!(retry, Ok(()) | Err(Error::Recovering))
    };
    let chen_after = u32::from(channel_enabled(6));
    let good = status & int::SRC_DEC_ERR != 0 && chen_after == 0 && valid_recovery;
    log!(
        "PARITY_DMA_BUS_ERROR nonce={nonce:08x} ch=6 status=0x{status:08x} stopped={stopped} recovered={} chen_after={chen_after} ok={}\n",
        u32::from(retry.is_ok()), u32::from(good),
    )
    .unwrap();
    if good { Ok(()) } else { Err(Error::Invalid) }
}

fn channel_run(
    channel: &mut dyn Exercise,
    wait: &mut WaitTimer,
    nonce: u32,
    n: usize,
    buffers: &mut Pair<'_>,
) -> Result<u32, Error> {
    let timer = Timer::new();
    for bytes in SIZES {
        let (mut crc_diff, mut range_diff, mut total_us, mut max_us) = (0, 0, 0, 0);
        for iteration in 0..REPEATS {
            prepare(buffers, CAPACITY, 16, bytes)?;
            let payload = buffers.0.as_mut_slice();
            for (i, word) in payload.as_chunks_mut::<4>().0.iter_mut().enumerate() {
                let pattern = (i as u32).wrapping_mul(0x9e37_79b9)
                    ^ (iteration.wrapping_mul(0x85eb_ca6b))
                    ^ (n as u32).rotate_left(17);
                word.copy_from_slice(&pattern.to_le_bytes());
            }
            let expected = crc32(payload);
            let start = timer.now_32().ticks();
            channel.copy(buffers)?;
            let elapsed = timer.now_32().ticks().wrapping_sub(start);
            total_us += elapsed;
            max_us = max_us.max(elapsed);
            crc_diff += u32::from(crc32(buffers.0.as_slice()) != expected);
            crc_diff += u32::from(crc32(buffers.1.as_slice()) != expected);
            range_diff += guards(buffers, 16, bytes)?;
        }
        log!(
            "PARITY_DMA_COPY nonce={nonce:08x} ch={n} bytes={bytes} count={REPEATS} total_us={total_us} max_us={max_us} crc_diff={crc_diff} range_diff={range_diff}\n"
        )
        .unwrap();
        if crc_diff != 0 || range_diff != 0 {
            return Err(Error::Invalid);
        }
    }

    // Four channels get 13 trials and four get 12, for exactly 100 actual cancellations.
    let attempts = 12 + u32::from(n < 4);
    let (mut partial, mut early) = (0, 0);
    let (mut range_diff, mut late_diff, mut payload_diff) = (0, 0, 0);
    let mut cancel = CancelSample::default();
    for trial in 0..attempts {
        prepare(buffers, CAPACITY, CANCEL_OFFSET, CANCEL_BYTES)?;
        buffers.0.as_mut_slice().fill(0x3c);
        let (result, sample) = channel.cancel(buffers, wait, 2 + trial % 5)?;
        cancel.first_us = cancel.first_us.max(sample.first_us);
        if sample.decision_us >= cancel.decision_us {
            cancel.decision_us = sample.decision_us;
            cancel.before = sample.before;
            cancel.terminal = sample.terminal;
            cancel.after = sample.after;
        }
        cancel.rechecked += sample.rechecked;
        cancel.after_ack |= sample.after_ack;
        match result {
            Err(Error::Cancelled) => {}
            Ok(()) => early += 1,
            Err(error) => return Err(error),
        }
        let dst = buffers.1.as_slice();
        let prefix = dst.iter().take_while(|&&b| b == 0x3c).count();
        partial += u32::from((1..CANCEL_BYTES).contains(&prefix));
        payload_diff += dst[prefix..].iter().filter(|&&b| b != CANARY).count();
        payload_diff += buffers.0.as_slice().iter().filter(|&&b| b != 0x3c).count();
        let signature = crc32(dst);
        wait.delay(Duration::micros(100))?;
        late_diff += u32::from(crc32(buffers.1.as_slice()) != signature);
        range_diff += guards(buffers, CANCEL_OFFSET, CANCEL_BYTES)?;
    }
    log!(
        "PARITY_DMA_CANCEL nonce={nonce:08x} ch={n} count={attempts} stopped={attempts} quarantined=0 rechecked={} first_us={} decision_us={} ch_status_before=0x{:08x} terminal_status=0x{:08x} ch_status_after=0x{:08x} int_status_after=0x{:08x} partial={partial} early={early} range_diff={range_diff} late_diff={late_diff} payload_diff={payload_diff}\n",
        cancel.rechecked, cancel.first_us, cancel.decision_us, cancel.before,
        cancel.terminal, cancel.after, cancel.after_ack,
    )
    .unwrap();
    if cancel.decision_us > STOP_LIMIT.ticks()
        || cancel.after_ack != 0
        || early != 0
        || (range_diff + late_diff as usize + payload_diff) != 0
    {
        return Err(Error::Invalid);
    }
    Ok(partial)
}

fn prepare(
    buffers: &mut Pair<'_>,
    capacity: usize,
    offset: usize,
    bytes: usize,
) -> Result<(), Error> {
    for lease in [&mut buffers.0, &mut buffers.1] {
        lease.set_len(capacity)?;
        lease.as_mut_slice().fill(CANARY);
        lease.set_range(offset, bytes)?;
    }
    Ok(())
}

fn guards(buffers: &mut Pair<'_>, offset: usize, bytes: usize) -> Result<usize, Error> {
    let mut count = 0;
    for lease in [&mut buffers.0, &mut buffers.1] {
        lease.set_len(CAPACITY)?;
        let slot = lease.as_slice();
        count += slot[..offset]
            .iter()
            .chain(&slot[offset + bytes..])
            .filter(|&&b| b != CANARY)
            .count();
    }
    Ok(count)
}

const fn crc_table() -> [u32; 256] {
    let mut table = [0; 256];
    let mut i = 0;
    while i < 256 {
        let mut crc = i as u32;
        let mut bit = 0;
        while bit < 8 {
            crc = (crc >> 1) ^ ((crc & 1).wrapping_neg() & 0xedb8_8320);
            bit += 1;
        }
        table[i] = crc;
        i += 1;
    }
    table
}

fn crc32(bytes: &[u8]) -> u32 {
    const TABLE: [u32; 256] = crc_table();
    let crc = bytes.iter().fold(u32::MAX, |crc, &b| {
        TABLE[((crc as u8) ^ b) as usize] ^ (crc >> 8)
    });
    !crc
}

fn park() -> ! {
    loop {
        rp1_rt::wait_for_event();
    }
}
