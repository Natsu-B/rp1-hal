//! All-channel single-block DMA, bounded cancellation and controller recovery.
#![no_std]
#![no_main]

#[path = "../../status-nonce.rs"]
mod status_nonce;

use core::cell::UnsafeCell;
use rp1_hal::{
    clocks::Clocks,
    dma::{Channel, Completed, Error, Lease, PollingTimeout, Pool},
    log,
    timer::{Deadline, Duration, Timer, WaitTimer},
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

#[repr(align(16))]
struct Memory(UnsafeCell<[u8; 2 * CAPACITY]>);
// SAFETY: the single proc1 entry takes exclusive ownership.
unsafe impl Sync for Memory {}
#[unsafe(link_section = ".shared")]
static MEMORY: Memory = Memory(UnsafeCell::new([0; 2 * CAPACITY]));

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
    let pool = Pool::<2>::new(unsafe { &mut *MEMORY.0.get() })?;
    let mut buffers = (pool.lease(0, CAPACITY)?, pool.lease(1, CAPACITY)?);
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
        let (next, count) = channel_run(channel, &mut wait, nonce, n, buffers)?;
        buffers = next;
        partial += count;
    }
    if partial == 0 {
        return Err(Error::Invalid);
    }

    // A forgotten handle keeps both leases claimed even after the controller is recovered and
    // reinitialized. The fake also forces the case where the leaked transfer refuses to stop.
    buffers.0.set_range(CANCEL_OFFSET, CANCEL_BYTES)?;
    buffers.1.set_range(CANCEL_OFFSET, CANCEL_BYTES)?;
    core::mem::forget(dma.ch7.try_start(buffers.0, buffers.1)?);
    let token = dma
        .free_until(&mut wait, Deadline::after(STOP_LIMIT)?)
        .map_err(|e| *e.error())?;
    token.init(&clocks)?.try_free().map_err(|e| *e.error())?;
    if !matches!(pool.lease(0, 4), Err(Error::Busy))
        || !matches!(pool.lease(1, 4), Err(Error::Busy))
    {
        return Err(Error::Invalid);
    }
    log!("PARITY_DMA_RECOVERY nonce={nonce:08x} forgotten=1 leases_claimed=2 reinit=1\n").unwrap();
    Ok(())
}

trait Exercise {
    fn copy<'p>(&mut self, buffers: Pair<'p>) -> Result<Pair<'p>, Error>;
    fn cancel<'p>(
        &mut self,
        buffers: Pair<'p>,
        wait: &mut WaitTimer,
        stimulus_us: u32,
    ) -> Result<(Completed<'p>, bool, u32), Error>;
}

impl<const N: u8> Exercise for Channel<N> {
    fn copy<'p>(&mut self, (src, dst): Pair<'p>) -> Result<Pair<'p>, Error> {
        self.copy(src, dst, PollingTimeout::new(TIMEOUT))
    }

    fn cancel<'p>(
        &mut self,
        (src, dst): Pair<'p>,
        wait: &mut WaitTimer,
        stimulus_us: u32,
    ) -> Result<(Completed<'p>, bool, u32), Error> {
        let timer = Timer::new();
        let mut transfer = self.try_start(src, dst)?;
        // Polled delay is an explicit stimulus just before the cancellation, not part of its time.
        timer.delay_polling(Duration::micros(stimulus_us))?;
        let start = timer.now_32().ticks();
        let decision = transfer.try_cancel();
        let elapsed = timer.now_32().ticks().wrapping_sub(start);
        let quarantined = match decision {
            Ok(()) => false,
            Err(Error::Recovering) => {
                transfer.cancel_until(wait, Deadline::after(STOP_LIMIT)?)?;
                true
            }
            Err(error) => return Err(error),
        };
        let done = transfer.try_reclaim().map_err(|_| Error::Recovering)?;
        Ok((done, quarantined, elapsed))
    }
}

fn channel_run<'p>(
    channel: &mut dyn Exercise,
    wait: &mut WaitTimer,
    nonce: u32,
    n: usize,
    mut buffers: Pair<'p>,
) -> Result<(Pair<'p>, u32), Error> {
    for bytes in SIZES {
        let mut crc_diff = 0;
        let mut range_diff = 0;
        for iteration in 0..REPEATS {
            prepare(&mut buffers, 16, bytes)?;
            let payload = buffers.0.as_mut_slice();
            for (i, word) in payload.as_chunks_mut::<4>().0.iter_mut().enumerate() {
                let pattern = (i as u32).wrapping_mul(0x9e37_79b9)
                    ^ (iteration.wrapping_mul(0x85eb_ca6b))
                    ^ (n as u32).rotate_left(17);
                word.copy_from_slice(&pattern.to_le_bytes());
            }
            let expected = crc32(payload);
            buffers = channel.copy(buffers)?;
            crc_diff += u32::from(crc32(buffers.0.as_slice()) != expected);
            crc_diff += u32::from(crc32(buffers.1.as_slice()) != expected);
            range_diff += guards(&mut buffers, 16, bytes)?;
        }
        log!(
            "PARITY_DMA_COPY nonce={nonce:08x} ch={n} bytes={bytes} count={REPEATS} crc_diff={crc_diff} range_diff={range_diff}\n"
        )
        .unwrap();
        if crc_diff != 0 || range_diff != 0 {
            return Err(Error::Invalid);
        }
    }

    // Four channels get 13 trials and four get 12, for exactly 100 actual cancellations.
    let attempts = 12 + u32::from(n < 4);
    let (mut quarantined, mut max_us, mut partial, mut early) = (0, 0, 0, 0);
    let (mut range_diff, mut late_diff, mut payload_diff) = (0, 0, 0);
    for trial in 0..attempts {
        prepare(&mut buffers, CANCEL_OFFSET, CANCEL_BYTES)?;
        buffers.0.as_mut_slice().fill(0x3c);
        let (done, held, elapsed) = channel.cancel(buffers, wait, 2 + trial % 5)?;
        max_us = max_us.max(elapsed);
        quarantined += u32::from(held);
        match done.result {
            Err(Error::Cancelled) => {}
            Ok(()) => early += 1,
            Err(error) => return Err(error),
        }
        buffers = (done.src, done.dst);
        let dst = buffers.1.as_slice();
        let prefix = dst.iter().take_while(|&&b| b == 0x3c).count();
        partial += u32::from((1..CANCEL_BYTES).contains(&prefix));
        payload_diff += dst[prefix..].iter().filter(|&&b| b != CANARY).count();
        payload_diff += buffers.0.as_slice().iter().filter(|&&b| b != 0x3c).count();
        let signature = crc32(dst);
        wait.delay(Duration::micros(100))?;
        late_diff += u32::from(crc32(buffers.1.as_slice()) != signature);
        range_diff += guards(&mut buffers, CANCEL_OFFSET, CANCEL_BYTES)?;
    }
    let stopped = attempts - quarantined;
    log!(
        "PARITY_DMA_CANCEL nonce={nonce:08x} ch={n} count={attempts} stopped={stopped} quarantined={quarantined} max_us={max_us} partial={partial} early={early} range_diff={range_diff} late_diff={late_diff} payload_diff={payload_diff}\n"
    )
    .unwrap();
    if max_us > STOP_LIMIT.ticks()
        || early != 0
        || (range_diff + late_diff as usize + payload_diff) != 0
    {
        return Err(Error::Invalid);
    }
    Ok((buffers, partial))
}

fn prepare(buffers: &mut Pair<'_>, offset: usize, bytes: usize) -> Result<(), Error> {
    for lease in [&mut buffers.0, &mut buffers.1] {
        lease.set_len(CAPACITY)?;
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
