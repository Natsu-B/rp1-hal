//! Gate D: one-beat PIO DMA and graceful disable, with no pin stimuli.
#![no_std]
#![no_main]

#[path = "../../shm-firmware/src/stack_watermark.rs"]
mod stack_watermark;

macro_rules! log {
    ($($arg:tt)*) => { write_log(format_args!($($arg)*)) };
}

use rp1_hal::{
    clocks::Clocks,
    dma::{Channel, Error, Lease, PioFifo, PioRequests, PioTransfer},
    pac::dw_axi_dmac::{BASE, CHANNEL_BASE, CHANNEL_STRIDE, ch, common, int},
    pio::{
        InstalledProgram, Irq, PIOBuilder, PIOExt, Rx, StateMachine, Stopped, Tx,
        UninitStateMachine, pio_asm,
    },
    timer::{Deadline, Duration, Instant, Timer, WaitTimer},
};
use rp1_rt::{Proc0, Proc1, entry};

rp1_rt::bind!();
type PingWord = u32;
type Checked<T> = Result<T, &'static str>;
type Dma<const N: u8> = Channel<N, PioRequests>;
type FifoIrq = Irq<1, 3>;
type TxDma<const N: u8> = PioFifo<Tx<N, FifoIrq>>;
type RxDma<const N: u8> = PioFifo<Rx<N, FifoIrq>>;
type Sm<const N: u8> = UninitStateMachine<N, FifoIrq>;

const WORDS: [usize; 5] = [1, 7, 8, 9, 257];
const TRIALS: u32 = 100;
const ABORTS_PER_SM: u32 = 25;
const SLOT: usize = 1040;
const DREQ_EN: u32 = 1 << 31;
const FDEBUG: usize = 0xf000_0008;
const FLEVEL: usize = 0xf000_000c;
const CANARY: u32 = 0;
const GUARD: [u8; 4] = [0x5a; 4];
const PREFILL: u32 = 0xffff_ffff;
const WINDOW: u32 = 1_000;
const COMPLETION: u32 = 10_000;
const PHASE_LIMIT: u32 = 30_000_000;

#[entry(proc0)]
fn main0(mut core: Proc0, p: rp1::Proc0Resources) -> ! {
    let stack = unsafe { stack_watermark::paint_core::<0>() };
    let _ = rp1::platform::init(&mut core, p.platform).unwrap();
    log!("PIO_D_PLAT ok=1 stack_free={}\n", stack.free_bytes()).unwrap();
    p.proc1.launch(&mut core).unwrap();
    park()
}

#[entry(proc1)]
fn main1(core: Proc1, p: rp1::Proc1Resources) -> ! {
    let stack = unsafe { stack_watermark::paint() };
    let clocks = Clocks::wait_from_proc0(&core, Duration::millis(10)).unwrap();
    let (pio, sms) = p.pio.split_wait(&clocks).unwrap();
    let mut dma = p.dma.init(&clocks).unwrap();
    let pool = p.dma_pool;
    let wait = &mut { p.wait_timer };
    let mut lease = pool.lease(0, SLOT).unwrap();
    // Pull NOBLOCK substitutes X=0 on an empty TX FIFO. Zero only means idle.
    let echo = pio_asm!(
        ".wrap_target",
        "again:",
        "pull noblock",
        "mov y, osr",
        "jmp !y again",
        "mov isr, y",
        "push noblock",
        "nop [7]",
        ".wrap",
    );
    let program = pio.install(&echo.program).unwrap();
    let mut ping = p.link.open().unwrap().ping;
    let mut tokens = (sms.sm0, sms.sm1, sms.sm2, sms.sm3);
    log!(
        "PIO_D_READY core=1 clk_hz={} divider=200:0 quantization_us=1 irq40=masked\n",
        clocks.sys().raw()
    )
    .unwrap();
    for run in 1..=2 {
        loop {
            let mut triggered = false;
            let _ = ping
                .serve(1, |word| {
                    triggered = true;
                    word
                })
                .unwrap();
            if triggered {
                break;
            }
        }
        let start = tick();
        let result = (|| {
            let (a, b, c, d) = tokens;
            let a = run_sm(a, &program, run, &mut dma.ch0, &mut lease, wait)?;
            let b = run_sm(b, &program, run, &mut dma.ch0, &mut lease, wait)?;
            let c = run_sm(c, &program, run, &mut dma.ch0, &mut lease, wait)?;
            let d = run_sm(d, &program, run, &mut dma.ch0, &mut lease, wait)?;
            Ok::<_, &'static str>((a, b, c, d))
        })();
        let end = tick();
        let pass = result.is_ok() && elapsed(start, end) < PHASE_LIMIT;
        log!(
            "PIO_D_END run={run} pass={} start={start} end={end} stack_free={} error={}\n",
            u8::from(pass),
            stack.free_bytes(),
            result.as_ref().err().copied().unwrap_or("None")
        )
        .unwrap();
        if !pass {
            park();
        }
        tokens = result.unwrap();
    }
    let (_, _, _, last) = tokens;
    let result = quarantine(last, &program, &mut dma.ch7, &pool, lease, wait);
    log!(
        "PIO_D_QUARANTINE_END pass={} error={}\n",
        u8::from(result.is_ok()),
        result.err().unwrap_or("None")
    )
    .unwrap();
    park()
}

fn park() -> ! {
    loop {
        rp1_rt::wait_for_event();
    }
}

fn read(address: usize) -> u32 {
    // This probe owns the selected PIO and DMA registers on proc1.
    unsafe { (address as *const u32).read_volatile() }
}

fn write(address: usize, value: u32) {
    unsafe { (address as *mut u32).write_volatile(value) };
    rp1_proto::barrier();
}

fn dma_reg<const C: u8>(offset: usize) -> usize {
    BASE + CHANNEL_BASE + usize::from(C) * CHANNEL_STRIDE + offset
}

fn ctrl(sm: u8, rx: bool) -> usize {
    0xf000_00e4 + usize::from(sm) * 0x20 + usize::from(rx) * 4
}

fn level(sm: u8, rx: bool) -> u32 {
    (read(FLEVEL) >> (sm * 8 + u8::from(rx) * 4)) & 15
}

fn tick() -> u32 {
    Timer::new().now_32().ticks()
}

fn elapsed(start: u32, end: u32) -> u32 {
    end.wrapping_sub(start)
}

fn deadline(us: u32) -> Deadline {
    Deadline::at(Instant::from_ticks(
        Timer::new().now().ticks() + u64::from(us),
    ))
}

fn check(condition: bool, stage: &'static str) -> Checked<()> {
    condition.then_some(()).ok_or(stage)
}

fn dma_error<T>(result: Result<T, Error>, stage: &'static str) -> Checked<T> {
    result.map_err(|error| log_dma_error(error, stage))
}

#[cold]
#[inline(never)]
fn log_dma_error(error: Error, stage: &'static str) -> &'static str {
    let _ = log!("PIO_D_DMA_ERROR stage={stage} error={error:?}\n");
    stage
}

fn until(mut probe: impl FnMut() -> bool, us: u32) -> Checked<()> {
    let start = tick();
    while !probe() {
        check(elapsed(start, tick()) < us, "fifo deadline")?;
    }
    Ok(())
}

fn seed<const SM: u8>(run: u32, trial: u32, n: usize, rx: bool) -> u32 {
    (run << 24) | (u32::from(SM) << 22) | (u32::from(rx) << 21) | ((n as u32) << 8) | trial
}

#[inline]
fn word(seed: u32, index: usize) -> u32 {
    let key = seed.rotate_left(13) ^ (index as u32).wrapping_mul(0x9e37_79b9);
    // A nonzero payload distinct from prefill; odd multiplication preserves index uniqueness.
    0x8000_0000 | (key & 0x3fff_ffff)
}

#[inline(never)]
fn prepare(lease: &mut Lease<'_>, n: usize, seed: u32, rx: bool) -> Checked<()> {
    dma_error(lease.set_len(SLOT), "whole slot")?;
    let words = lease.as_mut_slice().as_chunks_mut::<4>().0;
    words.fill(GUARD);
    let payload = &mut words[1..=n];
    if rx {
        payload.fill(CANARY.to_le_bytes());
    } else {
        for (i, slot) in payload.iter_mut().enumerate() {
            *slot = word(seed, i).to_le_bytes();
        }
    }
    dma_error(lease.set_range(4, n * 4), "payload range")
}

#[inline(never)]
fn verify_memory(lease: &mut Lease<'_>, n: usize, seed: u32, partial: bool) -> Checked<usize> {
    dma_error(lease.set_len(SLOT), "inspect slot")?;
    let slots = lease.as_slice().as_chunks::<4>().0;
    check(
        slots[0] == GUARD && slots[n + 1..].iter().all(|w| *w == GUARD),
        "guard",
    )?;
    let prefix = slots[1..=n]
        .iter()
        .take_while(|w| **w != CANARY.to_le_bytes())
        .count();
    check(partial || prefix == n, "word count")?;
    check(
        slots[1..=prefix]
            .iter()
            .enumerate()
            .all(|(i, w)| *w == word(seed, i).to_le_bytes())
            && slots[prefix + 1..=n]
                .iter()
                .all(|w| *w == CANARY.to_le_bytes()),
        "contents or prefix",
    )?;
    Ok(prefix)
}

#[inline(never)]
fn reclaim(transfer: PioTransfer<'_>, expected: Result<(), Error>) -> Checked<()> {
    match transfer.try_reclaim() {
        Ok(result) => check(result == expected, "transfer outcome"),
        Err(_transfer) => Err("transfer still quarantined"),
    }
}

// Measure strictly more than a thousand timer ticks, including quantisation.
#[inline(never)]
fn no_progress(sm: u8) -> Checked<Sample> {
    let progress = read(dma_reg::<0>(ch::STATUS));
    let mask = 0xff << (sm * 8);
    let fifo = read(FLEVEL) & mask;
    let start = tick();
    Timer::new()
        .delay_polling(Duration::micros(WINDOW + 1))
        .map_err(|_| "off timer")?;
    let stop = tick();
    check(elapsed(start, stop) > WINDOW, "off time")?;
    check(
        read(dma_reg::<0>(ch::STATUS)) == progress
            && read(FLEVEL) & mask == fifo
            && read(BASE + common::CHEN) & 1 != 0
            && read(dma_reg::<0>(ch::INTSTATUS)) & (int::DMA_TFR_DONE | int::ALL_ERR) == 0,
        "off progress",
    )?;
    Ok(Sample { start, stop })
}

#[derive(Clone, Copy)]
struct Sample {
    start: u32,
    stop: u32,
}
impl Sample {
    fn upper(self) -> u32 {
        elapsed(self.start, self.stop) + 1
    }

    fn lower(self) -> u32 {
        self.upper().saturating_sub(2)
    }
}

#[derive(Clone, Copy)]
struct Range {
    min: Sample,
    max: Sample,
}

impl Range {
    fn new() -> Self {
        Self {
            min: Sample {
                start: 0,
                stop: PHASE_LIMIT,
            },
            max: Sample { start: 0, stop: 0 },
        }
    }

    fn add(&mut self, sample: Sample) {
        if sample.lower() < self.min.lower() {
            self.min = sample;
        }
        if sample.upper() >= self.max.upper() {
            self.max = sample;
        }
    }
}

type Machine<'a, 'b, const SM: u8> = StateMachine<'a, 'b, SM, Stopped>;

trait CpuFifo {
    fn write(&mut self, _word: u32) -> bool {
        false
    }
    fn read(&mut self) -> Option<u32> {
        None
    }
}
impl<const SM: u8> CpuFifo for Tx<SM, FifoIrq> {
    fn write(&mut self, word: u32) -> bool {
        self.write(word)
    }
}
impl<const SM: u8> CpuFifo for Rx<SM, FifoIrq> {
    fn read(&mut self) -> Option<u32> {
        self.read()
    }
}

struct Trial {
    done: Sample,
    blocked: Sample,
    masked: Sample,
}
#[inline(never)]
fn trial<'a, 'b, const SM: u8>(
    sm: Machine<'a, 'b, SM>,
    tx: &mut TxDma<SM>,
    rx: &mut RxDma<SM>,
    dma: &mut Dma<0>,
    lease: &mut Lease<'_>,
    n: usize,
    seed: u32,
    inbound: bool,
) -> Checked<(Machine<'a, 'b, SM>, Trial)> {
    prepare(lease, n, seed, inbound)?;
    let address = ctrl(SM, inbound);
    if !inbound {
        let tx = tx.try_cpu().ok_or("TX held")?;
        for _ in 0..8 {
            check(tx.write(PREFILL), "TX prefill")?;
        }
    }
    let expected_level = if inbound { 0 } else { 8 };
    check(level(SM, inbound) == expected_level, "blocked FIFO")?;
    let (transfer, cpu): (PioTransfer, &mut dyn CpuFifo) = if inbound {
        (
            dma_error(dma.try_start_from_pio(rx, lease), "start rx")?,
            tx.try_cpu().ok_or("TX held")?,
        )
    } else {
        (
            dma_error(dma.try_start_to_pio(lease, tx), "start tx")?,
            rx.try_cpu().ok_or("RX held")?,
        )
    };
    let config = read(address);
    let threshold = if inbound { 1 } else { 7 };
    check(
        config & (DREQ_EN | 31) == DREQ_EN | threshold,
        "DREQ enable/threshold",
    )?;
    let blocked = no_progress(SM)?;

    // The laboratory masks the live request while there is no FIFO condition, then
    // establishes the condition using the same echo and CPU peer as the on phase.
    write(address, config & 31);
    check(read(address) & DREQ_EN == 0, "DREQ mask injection")?;
    let running = sm.start().map_err(|_| "SM start")?;
    let (done, masked) = exchange(transfer, cpu, SM, inbound, n, seed, address, config)?;
    verify_memory(lease, n, seed, false)?;
    Ok((
        running.stop(),
        Trial {
            done,
            blocked,
            masked,
        },
    ))
}

#[inline(never)]
fn exchange(
    mut transfer: PioTransfer<'_>,
    cpu: &mut dyn CpuFifo,
    sm: u8,
    inbound: bool,
    n: usize,
    seed: u32,
    address: usize,
    config: u32,
) -> Checked<(Sample, Sample)> {
    if inbound {
        check(cpu.write(word(seed, 0)), "RX prime")?;
        until(|| level(sm, true) == 1, WINDOW)?;
    } else {
        let mut drained = 0;
        until(
            || {
                if let Some(value) = cpu.read() {
                    if value != PREFILL {
                        return true;
                    }
                    drained += 1;
                }
                drained == 8
            },
            WINDOW,
        )?;
        check(
            drained == 8 && level(sm, false) == 0,
            "TX prefill count/content",
        )?;
    }
    let masked = no_progress(sm)?;

    let end = deadline(COMPLETION - 1);
    let start = tick();
    write(address, config & (DREQ_EN | 31));
    if inbound {
        let mut sent = 1;
        until(
            || {
                if sent < n && cpu.write(word(seed, sent)) {
                    sent += 1;
                }
                sent == n
            },
            COMPLETION - 1,
        )?;
    } else {
        let mut received = 0;
        until(
            || {
                if let Some(value) = cpu.read() {
                    if value != word(seed, received) {
                        return true;
                    }
                    received += 1;
                }
                received == n
            },
            COMPLETION - 1,
        )?;
        check(received == n, "TX contents or count")?;
    }
    dma_error(transfer.wait_polling(end), "wait complete")?;
    let done = Sample {
        start,
        stop: tick(),
    };
    check(done.upper() <= COMPLETION, "completion time")?;
    check(
        transfer.terminal_status() & int::DMA_TFR_DONE != 0,
        "completion proof",
    )?;
    reclaim(transfer, Ok(()))?;
    check(read(address) & DREQ_EN == 0, "DREQ retired")?;
    check(
        level(sm, false) == 0 && level(sm, true) == 0,
        "surplus FIFO data",
    )?;
    Timer::new()
        .delay_polling(Duration::micros(40))
        .map_err(|_| "extra guard timer")?;
    check(
        level(sm, false) == 0 && level(sm, true) == 0,
        "late FIFO data",
    )?;
    check(read(FDEBUG) & (0x0101_0101 << sm) == 0, "FDEBUG")?;
    Ok((done, masked))
}

#[inline(never)]
fn run_sm<const SM: u8>(
    token: Sm<SM>,
    program: &InstalledProgram<'_>,
    run: u32,
    dma: &mut Dma<0>,
    lease: &mut Lease<'_>,
    wait: &mut WaitTimer,
) -> Checked<Sm<SM>> {
    let (mut sm, rx, tx) = PIOBuilder::from_installed_program(program)
        .clock_divisor_fixed_point(200, 0)
        .build(token)
        .map_err(|_| "SM build")?;
    let (mut rx, mut tx) = (rx.into_dma(), tx.into_dma());
    for inbound in [false, true] {
        for n in WORDS {
            let mut timings = [Range::new(); 3];
            for i in 0..TRIALS {
                let result = trial(
                    sm,
                    &mut tx,
                    &mut rx,
                    dma,
                    lease,
                    n,
                    seed::<SM>(run, i, n, inbound),
                    inbound,
                );
                let (next, sample) =
                    result.map_err(|stage| trial_fail(run, SM, inbound, n, i, stage))?;
                sm = next;
                for (range, observation) in
                    timings
                        .iter_mut()
                        .zip([sample.done, sample.blocked, sample.masked])
                {
                    range.add(observation);
                }
            }
            log_fifo(run, SM, inbound, n, timings);
        }
    }
    let mut min = usize::MAX;
    let (mut max, mut worst, mut partials) = (0, 0, 0);
    for i in 0..ABORTS_PER_SM {
        let (next, prefix, stop_us) = abort(
            sm,
            &mut rx,
            &mut tx,
            dma,
            lease,
            wait,
            seed::<SM>(run, i, 257, true),
            i,
        )?;
        sm = next;
        min = min.min(prefix);
        max = max.max(prefix);
        worst = worst.max(stop_us);
        partials += u32::from((1..257).contains(&prefix));
    }
    log_abort(run, SM, min, max, worst, partials);
    let (token, _) = sm.uninit(
        rx.try_into_inner().ok().unwrap(),
        tx.try_into_inner().ok().unwrap(),
    );
    Ok(token)
}

#[cold]
#[inline(never)]
fn trial_fail(
    run: u32,
    sm: u8,
    inbound: bool,
    n: usize,
    i: u32,
    stage: &'static str,
) -> &'static str {
    let _ = log!(
        "PIO_D_TRIAL_FAIL run={run} sm={sm} rx={} n={n} i={i} stage={stage} tx={} rxf={} fdebug={:#x} progress={:#x} status={:#x} control={:#x}\n",
        u8::from(inbound),
        level(sm, false),
        level(sm, true),
        read(FDEBUG),
        read(dma_reg::<0>(ch::STATUS)),
        read(dma_reg::<0>(ch::INTSTATUS)),
        read(ctrl(sm, inbound))
    );
    stage
}

#[inline(never)]
fn log_fifo(run: u32, sm: u8, inbound: bool, n: usize, [done, blocked, masked]: [Range; 3]) {
    log!("PIO_D_FIFO run={run} sm={sm} rx={} words={n} count={TRIALS} total_words={} blocked_min={} blocked_max={} b_start={} b_stop={} masked_min={} masked_max={} m_start={} m_stop={} min={} max={} start={} stop={} progress=0 content=0 guard=0 fdebug=0\n",
        u8::from(inbound), n * TRIALS as usize,
        blocked.min.lower(), blocked.max.upper(), blocked.min.start, blocked.min.stop,
        masked.min.lower(), masked.max.upper(), masked.min.start, masked.min.stop,
        done.min.lower(), done.max.upper(), done.max.start, done.max.stop).unwrap();
}

#[inline(never)]
fn log_abort(run: u32, sm: u8, min: usize, max: usize, worst: u32, partials: u32) {
    log!("PIO_D_ABORT run={run} sm={sm} count={ABORTS_PER_SM} words=257 min={min} max={max} partials={partials} max_us={worst} reuse={ABORTS_PER_SM} guard=0 fdebug=0\n").unwrap();
}

#[inline(never)]
fn abort<'a, 'b, const SM: u8>(
    sm: Machine<'a, 'b, SM>,
    rx: &mut RxDma<SM>,
    tx: &mut TxDma<SM>,
    dma: &mut Dma<0>,
    lease: &mut Lease<'_>,
    wait: &mut WaitTimer,
    seed: u32,
    trial: u32,
) -> Checked<(Machine<'a, 'b, SM>, usize, u32)> {
    prepare(lease, 257, seed, true)?;
    let running = sm.start().map_err(|_| "abort SM start")?;
    let transfer = dma_error(dma.try_start_from_pio(rx, lease), "abort start")?;
    let tx = tx.try_cpu().ok_or("TX held")?;
    let stop_us = cancel_probe(transfer, tx, SM, wait, seed, trial)?;
    check(read(ctrl(SM, true)) & DREQ_EN == 0, "abort DREQ retired")?;
    let prefix = verify_memory(lease, 257, seed, true)?;
    // Give pending FIFO work an opportunity to surface before checking the stopped buffer again.
    Timer::new()
        .delay_polling(Duration::micros(100))
        .map_err(|_| "abort guard timer")?;
    check(
        verify_memory(lease, 257, seed, true)? == prefix,
        "late writes after abort",
    )?;
    check(read(FDEBUG) & (0x0101_0101 << SM) == 0, "abort FDEBUG")?;
    let mut sm = running.stop();
    sm.clear_fifos(rx.try_cpu().ok_or("abort RX held")?, tx);
    sm.restart();
    // Confirmed retirements admit a fresh transfer, including the final cancellation.
    prepare(lease, 1, seed, true)?;
    let running = sm.start().map_err(|_| "reuse SM start")?;
    let transfer = dma_error(dma.try_start_from_pio(rx, lease), "reused channel start")?;
    complete_reuse(transfer, tx, seed)?;
    verify_memory(lease, 1, seed, false)?;
    Ok((running.stop(), prefix, stop_us))
}

#[inline(never)]
fn complete_reuse(mut transfer: PioTransfer<'_>, tx: &mut dyn CpuFifo, seed: u32) -> Checked<()> {
    check(tx.write(word(seed, 0)), "reuse write")?;
    dma_error(
        transfer.wait_polling(deadline(COMPLETION)),
        "reused channel completion",
    )?;
    reclaim(transfer, Ok(()))
}

#[inline(never)]
fn cancel_probe(
    mut transfer: PioTransfer<'_>,
    tx: &mut dyn CpuFifo,
    sm: u8,
    wait: &mut WaitTimer,
    seed: u32,
    trial: u32,
) -> Checked<u32> {
    let mut sent = 0;
    let stimulus = (trial * 7 % 23) as usize;
    until(
        || {
            if sent < stimulus && tx.write(word(seed, sent)) {
                sent += 1;
            }
            sent == stimulus
        },
        WINDOW,
    )?;
    until(|| level(sm, false) <= 4, WINDOW)?;
    let end = deadline(900);
    let start = tick();
    dma_error(transfer.cancel_until(wait, end), "abort stop")?;
    let stop_us = elapsed(start, tick()) + 1;
    check(
        stop_us <= WINDOW && (transfer.terminal_status() & int::CH_DISABLED != 0),
        "abort proof",
    )?;
    reclaim(transfer, Err(Error::Cancelled))?;
    Ok(stop_us)
}

fn quarantine<const SM: u8>(
    token: Sm<SM>,
    program: &InstalledProgram<'_>,
    dma: &mut Dma<7>,
    pool: &rp1_hal::dma::Pool<1>,
    mut lease: Lease<'_>,
    wait: &mut WaitTimer,
) -> Checked<()> {
    let (_sm, rx, _tx) = PIOBuilder::from_installed_program(program)
        .clock_divisor_fixed_point(200, 0)
        .build(token)
        .map_err(|_| "quarantine SM build")?;
    let mut rx = rx.into_dma();
    dma_error(lease.set_len(36), "quarantine lease")?;
    let mut transfer = dma_error(
        dma.try_start_from_pio(&mut rx, &mut lease),
        "quarantine start",
    )?;
    let arm = dma_reg::<7>(ch::INTSTATUS_ENA);
    let old = read(arm);
    // Suppress both terminal status sources before disable, leaving hardware EN readable.
    write(arm, old & !(int::CH_DISABLED | int::DMA_TFR_DONE));
    check(
        read(arm) & (int::CH_DISABLED | int::DMA_TFR_DONE) == 0,
        "quarantine inject",
    )?;
    let active = read(BASE + common::CHEN) & (1 << 7) != 0;
    let end = deadline(900);
    let start = tick();
    let decision = transfer.cancel_until(wait, end);
    let stop = tick();
    until(|| elapsed(start, tick()) > WINDOW, WINDOW + 2)?;
    let transfer = match transfer.try_reclaim() {
        Err(transfer) => transfer,
        Ok(_) => return Err("reclaimed without proof"),
    };
    drop(transfer);
    write(arm, old);
    let disabled = read(BASE + common::CHEN) & (1 << 7) == 0;
    let masked = read(ctrl(SM, true)) & DREQ_EN == 0;
    let terminal = read(dma_reg::<7>(ch::INTSTATUS)) & (int::CH_DISABLED | int::DMA_TFR_DONE);
    let quarantined = rx.is_quarantined()
        && rx.try_cpu().is_none()
        && lease.is_quarantined()
        && lease.as_mut_slice().is_empty();
    let retry = dma.try_recover();
    drop(lease);
    let claimed = matches!(pool.lease(0, 36), Err(Error::Busy));
    let fifo_held = rx.try_into_inner().is_err();
    log!("PIO_D_QUARANTINE sm={SM} active={} disabled={} masked={} terminal={terminal} start={start} stop={stop} decision={decision:?} retry={retry:?} held={} claimed={} fifo={}\n",
        u8::from(active), u8::from(disabled), u8::from(masked), u8::from(quarantined), u8::from(claimed), u8::from(fifo_held)).unwrap();
    check(
        active
            && disabled
            && masked
            && terminal == 0
            && elapsed(start, stop) + 1 <= WINDOW
            && decision == Err(Error::Recovering)
            && retry == Err(Error::Recovering)
            && quarantined
            && claimed
            && fifo_held,
        "quarantine proof",
    )
}

// Proc0 logs before launch, then proc1 is the sole UART0 writer.
fn write_log(args: core::fmt::Arguments<'_>) -> core::fmt::Result {
    struct Uart;
    impl core::fmt::Write for Uart {
        fn write_str(&mut self, text: &str) -> core::fmt::Result {
            const UART: usize = 0x4003_0000;
            for byte in text.bytes() {
                until(|| read(UART + 0x18) & (1 << 5) == 0, 10_000)
                    .map_err(|_| core::fmt::Error)?;
                write(UART, u32::from(byte));
            }
            Ok(())
        }
    }
    core::fmt::write(&mut Uart, args)
}
