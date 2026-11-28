//! SPI timing, internal loopback and GPIO chip-select parity.
#![no_std]
#![no_main]

use core::{
    mem::offset_of,
    sync::atomic::{AtomicU32, Ordering::Relaxed},
};
#[cfg(not(feature = "isr-trace"))]
use embedded_hal::spi::{Operation, SpiBus, SpiDevice};
use embedded_hal::{digital::OutputPin, spi::MODE_0};
use rp1_hal::{
    fugit::{ExtU32, RateExtU32},
    gpio::{Output, Pin},
    log, pac,
    spi::{Blocking, Config, Device, Error, Try},
    timer::{Alarm, Deadline, Timer, WaitTimer},
};
use rp1_rt::{Proc0, Proc1, entry, interrupt, interrupt::Slot};

rp1_rt::bind!();
include!(concat!(env!("OUT_DIR"), "/instance.rs"));

static RX: AtomicU32 = AtomicU32::new(0);
static TX: AtomicU32 = AtomicU32::new(0);
static FAULT: AtomicU32 = AtomicU32::new(0);
static STRAY: AtomicU32 = AtomicU32::new(0);
static WAIT_ALARM: AtomicU32 = AtomicU32::new(0);
static INJECT: Slot<Option<Alarm<0>>, 0, 4> = Slot::empty();
static INJECTED_AT: AtomicU32 = AtomicU32::new(0);

static CS_EXPECTED: AtomicU32 = AtomicU32::new(0);
static CS_BAD: AtomicU32 = AtomicU32::new(0);
static CS_FIRST_RX: AtomicU32 = AtomicU32::new(0);
static CS_LAST_RX: AtomicU32 = AtomicU32::new(0);

const TRIALS: u32 = 100;
const BATCH: usize = 1_000;
#[cfg(not(feature = "isr-trace"))]
const CS_TRIALS: u16 = 1_000;
#[cfg(not(feature = "isr-trace"))]
const BLOCKED_US: u32 = 500;
#[cfg(not(feature = "isr-trace"))]
const INJECT_US: u32 = 250;
const SLACK_US: u32 = 100;
#[cfg(not(feature = "isr-trace"))]
const CS_DELAY_US: u32 = 4;
const CS0_HIGH: u32 = 1 << 12;
const CS1_HIGH: u32 = 1 << 13;
const CS_IDLE: u32 = CS0_HIGH | CS1_HIGH;

type Bus<'a> = Blocking<'a, N, DriverPins, 0>;

fn now() -> u32 {
    Timer::new().now_32().ticks()
}

#[inline(always)]
fn add(counter: &AtomicU32, flag: bool) {
    if flag {
        counter.store(counter.load(Relaxed) + 1, Relaxed);
    }
}

#[inline(always)]
fn observe_device(dispatch: impl FnOnce() -> u32) {
    use pac::dw_ssi::int;
    let status = dispatch();
    let rx = status & int::RXFI != 0;
    add(&RX, rx);
    add(&TX, status & int::TXEI != 0);
    add(&FAULT, status & int::RXUI != 0);
    add(&STRAY, status == 0);
    let expected = CS_EXPECTED.load(Relaxed);
    if rx && expected != 0 {
        add(&CS_BAD, cs_sense() != expected);
        let tick = now();
        if CS_FIRST_RX.load(Relaxed) == 0 {
            CS_FIRST_RX.store(tick, Relaxed);
        }
        CS_LAST_RX.store(tick, Relaxed);
    }
}

#[inline(always)]
fn observe_wait_alarm() {
    // SAFETY: called from the generated handler for the declared ALARM1.
    unsafe { WaitTimer::__on_interrupt::<1>() };
    add(&WAIT_ALARM, true);
}

#[interrupt(proc0)]
fn TIMER_0() {
    INJECT.with(|alarm| alarm.clear_interrupt());
    add(&CS_BAD, cs_sense() != CS1_HIGH);
    INJECTED_AT.store(now(), Relaxed);
    // SAFETY: this image owns the controller. SER is zero and RX is empty. Reading DR
    // here raises RX underflow without modifying a frame under test.
    unsafe { spi_reg(offset_of!(pac::dw_ssi::RegisterBlock, dr)).read_volatile() };
}

fn spi_reg(offset: usize) -> *mut u32 {
    Registers::PTR
        .cast_mut()
        .cast::<u8>()
        .wrapping_add(offset)
        .cast()
}

fn cs_sense() -> u32 {
    // SAFETY: read-only RIO0 input. The GPIO12/13 pads are configured as inputs
    // and sense the GPIO22/23 select outputs through the two fixed loopbacks.
    (unsafe { (0x400e_0008 as *const u32).read_volatile() }) & CS_IDLE
}

fn select_wire(selected: bool) {
    // SAFETY: only used with the device idle or to start a deliberately
    // primed FIFO. SER zero prevents shifting while the FIFO is primed.
    unsafe {
        spi_reg(offset_of!(pac::dw_ssi::RegisterBlock, ser)).write_volatile(u32::from(selected))
    };
}

fn spi_status() -> u32 {
    // SAFETY: read-only controller status on this core's controller.
    unsafe { spi_reg(offset_of!(pac::dw_ssi::RegisterBlock, sr)).read_volatile() }
}

fn bulk(bus: &mut Bus<'_>, gate: &mut Pin<8, Output>, hz: u32) -> Result<(), Error> {
    assert_eq!(bus.set_frequency(hz.Hz())?.raw(), hz);
    let depth = bus.fifo_depth();
    let rx_before = RX.load(Relaxed);
    let stray_before = STRAY.load(Relaxed);
    let mut tx = [0u8; BATCH];
    let mut rx = [0u8; BATCH];
    for trial in 0..TRIALS {
        for (i, word) in tx.iter_mut().enumerate() {
            *word = (i as u8).wrapping_mul(37) ^ trial as u8;
        }
        let rx_trial = RX.load(Relaxed);
        let end = Deadline::after(100.millis())?;
        gate.set_high().unwrap();
        bus.transfer_until(&mut rx, &tx, end)?;
        bus.flush_until(end)?;
        gate.set_low().unwrap();
        assert_eq!(rx, tx);
        assert!(RX.load(Relaxed) > rx_trial);
    }
    let irq = RX.load(Relaxed) - rx_before;
    let stray = STRAY.load(Relaxed) - stray_before;
    log!(
        "SPI_IRQ_RATE hz={} frames={} trials={} depth={} rx_irq={} stray={} errors=0\n",
        hz,
        BATCH as u32 * TRIALS,
        TRIALS,
        depth,
        irq,
        stray
    )
    .unwrap();
    assert!((TRIALS..=TRIALS * (BATCH.div_ceil(depth.div_ceil(2)) as u32 + 2)).contains(&irq));
    assert_eq!(stray, 0);
    Ok(())
}

mod idle_probe;
#[cfg(feature = "isr-trace")]
mod platform_trace;

#[cfg(feature = "isr-trace")]
fn traced_flushes(
    phase: &str,
    spi: &mut Driver,
    timer: &mut WaitTimer,
    gate: &mut Pin<8, Output>,
) -> Result<(), Error> {
    log!("ATTR_BEGIN instance={N} phase={phase}\n").unwrap();
    platform_trace::inventory();
    tx_flush(spi, timer, gate, 1_000_000, false)?;
    tx_flush(spi, timer, gate, 100_000, true)?;
    platform_trace::inventory();
    platform_trace::platform_totals();
    log!("ATTR_END instance={N} phase={phase}\n").unwrap();
    Ok(())
}

fn tx_flush(
    spi: &mut Driver,
    timer: &mut WaitTimer,
    gate: &mut Pin<8, Output>,
    hz: u32,
    require_tail_alarm: bool,
) -> Result<(), Error> {
    use pac::dw_ssi::sr;
    assert_eq!(spi.blocking(timer).set_frequency(hz.Hz())?.raw(), hz);
    let depth = spi.fifo_depth();
    let burst_us = depth as u32 * (8_000_000 / hz);
    let mut flush_max = 0;
    let mut late_max = 0;
    let mut tail_trials = 0;
    let tx_before = TX.load(Relaxed);
    let stray_before = STRAY.load(Relaxed);
    #[cfg(feature = "isr-trace")]
    let trials = platform_trace::TRIALS as u32;
    #[cfg(not(feature = "isr-trace"))]
    let trials = TRIALS;
    #[cfg(feature = "isr-trace")]
    let mut samples = [0; platform_trace::TRIALS];
    #[cfg(feature = "isr-trace")]
    let mut summary = platform_trace::Summary::default();
    for trial in 0..trials {
        select_wire(false);
        for i in 0..depth {
            assert_eq!(spi.try_write_word(i as u8)?, Try::Ready(()));
        }
        assert_eq!(spi_status() & sr::TFE, 0);
        let tx = TX.load(Relaxed);
        let alarms = WAIT_ALARM.load(Relaxed);
        let end = Deadline::after(100.millis())?;
        let window = idle_probe::arm(burst_us);
        let launch = now();
        select_wire(true);
        gate.set_high().unwrap();
        #[cfg(feature = "isr-trace")]
        let start = platform_trace::begin();
        #[cfg(not(feature = "isr-trace"))]
        let start = now();
        spi.blocking(timer).flush_until(end)?;
        #[cfg(feature = "isr-trace")]
        let returned = platform_trace::end();
        #[cfg(not(feature = "isr-trace"))]
        let returned = now();
        // TFE and !BUSY establish that both the FIFO and shifter are empty at return.
        let status = spi_status();
        let elapsed = returned.wrapping_sub(start);
        let from_launch = returned.wrapping_sub(launch);
        gate.set_low().unwrap();
        assert_eq!(status & (sr::BUSY | sr::TFE), sr::TFE);
        let tx_irq = TX.load(Relaxed) - tx;
        let tail_alarms = WAIT_ALARM.load(Relaxed) - alarms;
        // Allow one SCLK period for the first/last edge convention and one RP1
        // tick for phase. The 100 us slack is measured from the earliest end.
        let earliest = burst_us - (1_000_000 / hz) - 1;
        let late = from_launch.saturating_sub(earliest) + 1;
        #[cfg(not(feature = "isr-trace"))]
        {
            log!(
            "SPI_TX_SAMPLE hz={} trial={} flush_us={} launch_us={} late_us={} tx_irq={} tail_alarms={} idle=1\n",
            hz, trial, elapsed, from_launch, late, tx_irq, tail_alarms
        )
        .unwrap();
            idle_probe::report(window, returned, hz, trial);
        }
        #[cfg(feature = "isr-trace")]
        {
            let (last_busy, first_idle) = idle_probe::bracket(window, returned);
            samples[trial as usize] = platform_trace::assess(
                &mut summary,
                hz,
                trial as usize,
                start,
                returned,
                last_busy,
                first_idle,
            );
        }
        assert!(from_launch >= earliest && late <= SLACK_US && elapsed <= burst_us + SLACK_US);
        assert_eq!(tx_irq, 1);
        if require_tail_alarm {
            assert!(tail_alarms > 0);
        }
        tail_trials += u32::from(tail_alarms > 0);
        assert!(tail_alarms <= 1 + SLACK_US);
        for i in 0..depth {
            assert_eq!(spi.try_read_word::<u8>()?, Try::Ready(i as u8));
        }
        flush_max = flush_max.max(elapsed);
        late_max = late_max.max(late);
    }
    let stray = STRAY.load(Relaxed) - stray_before;
    #[cfg(feature = "isr-trace")]
    platform_trace::report(hz, &samples, summary);
    log!(
        "SPI_TX_TRIALS hz={} tx_trials={} depth={} tx_irq={} tail_trials={} flush_max_us={} flush_late_max_us={} stray={} errors=0\n",
        hz, trials, depth, TX.load(Relaxed) - tx_before, tail_trials, flush_max, late_max, stray
    ).unwrap();
    assert_eq!(stray, 0);
    Ok(())
}

#[cfg(not(feature = "isr-trace"))]
fn timeout_rearm(spi: &mut Driver, timer: &mut WaitTimer) -> Result<(), Error> {
    spi.blocking(timer).set_frequency(1.MHz())?;
    let mut max_timeout = 0;
    let tx_before = TX.load(Relaxed);
    let stray_before = STRAY.load(Relaxed);
    for _ in 0..TRIALS {
        select_wire(false);
        let mut bus = spi.blocking(timer);
        let mut rx = [0; 2];
        let start = Timer::new().now();
        let end = Deadline::at(start + BLOCKED_US.micros());
        assert_eq!(
            bus.transfer_until(&mut rx, &[0x38, 0x7b], end),
            Err(Error::Timeout)
        );
        let elapsed = (Timer::new().now() - start).ticks() as u32;
        assert!((BLOCKED_US..=BLOCKED_US + SLACK_US).contains(&elapsed));
        max_timeout = max_timeout.max(elapsed);
        assert_eq!(bus.transfer(&mut rx, &[1u8, 2]), Err(Error::Busy));
        bus.try_recover()?;
        bus.transfer(&mut rx, &[0x38, 0x7b])?;
        bus.flush_until(Deadline::after(100.millis())?)?;
        assert_eq!(rx, [0x38, 0x7b]);
    }
    let stray = STRAY.load(Relaxed) - stray_before;
    let tx = TX.load(Relaxed) - tx_before;
    log!(
        "SPI_IRQ_TRIALS timeout={} timeout_max_us={} rearm={} tx_irq={} stray={} errors=0\n",
        TRIALS,
        max_timeout,
        TRIALS,
        tx,
        stray
    )
    .unwrap();
    assert_eq!((tx, stray), (0, 0));
    Ok(())
}

#[cfg(not(feature = "isr-trace"))]
fn selected<const N: u8>(
    bus: &mut Bus<'_>,
    cs: Pin<N>,
    id: u8,
    i: u16,
) -> Result<(Pin<N>, u32, u32), Error> {
    let mut dev = Device::new(bus, cs);
    assert_eq!(cs_sense(), CS_IDLE);
    CS_FIRST_RX.store(0, Relaxed);
    CS_LAST_RX.store(0, Relaxed);
    let rx_before = RX.load(Relaxed);
    CS_EXPECTED.store(if id == 0 { CS1_HIGH } else { CS0_HIGH }, Relaxed);

    let [lo, hi] = i.to_le_bytes();
    let tx = [0x5a, id, lo, hi];
    let mut rx = [0xff; 4];
    let start = now();
    dev.transaction(&mut [
        Operation::Write(&tx),
        Operation::DelayNs(CS_DELAY_US * 1_000),
        Operation::Read(&mut rx),
    ])?;
    let elapsed = now().wrapping_sub(start);
    CS_EXPECTED.store(0, Relaxed);
    assert_eq!(rx, [0; 4]);
    assert_eq!(cs_sense(), CS_IDLE);
    assert_eq!(RX.load(Relaxed) - rx_before, 2);
    let rx_gap = CS_LAST_RX
        .load(Relaxed)
        .wrapping_sub(CS_FIRST_RX.load(Relaxed));
    // Each RX IRQ corresponds to the end of four frames. The difference
    // includes the second 32 us burst, the CS delay and ISR phase.
    let delay = rx_gap.saturating_sub(32);
    // Entire transaction minus the earliest two mode-0 bursts, with tick allowance,
    // is a conservative upper bound even if the RP1 is preempted in either burst.
    let upper = elapsed.saturating_sub(62) + 1;
    if !(CS_DELAY_US..=CS_DELAY_US + SLACK_US).contains(&delay) || upper > CS_DELAY_US + SLACK_US {
        log!(
            "SPI_PARITY_FAIL cs={} trial={} elapsed_us={} rx_gap_us={} upper_us={}\n",
            id,
            i,
            elapsed,
            rx_gap,
            upper
        )
        .unwrap();
        return Err(Error::Timeout);
    }
    Ok((dev.free().1, delay, upper))
}

#[cfg(not(feature = "isr-trace"))]
fn devices(bus: &mut Bus<'_>, mut cs0: Pin<22>, mut cs1: Pin<23>) -> Result<(), Error> {
    let mut delay_min = u32::MAX;
    let mut delay_upper = 0;
    for i in 0..CS_TRIALS {
        let (pin0, delay0, upper0) = selected(bus, cs0, 0, i)?;
        let (pin1, delay1, upper1) = selected(bus, cs1, 1, i)?;
        cs0 = pin0;
        cs1 = pin1;
        delay_min = delay_min.min(delay0).min(delay1);
        delay_upper = delay_upper.max(upper0).max(upper1);
    }
    let mut fault_min = u32::MAX;
    let mut fault_max = 0;
    let mut fault_late_max = 0;
    let stray_before = STRAY.load(Relaxed);
    for trial in 0..TRIALS {
        select_wire(false);
        let before = FAULT.load(Relaxed);
        let mut dev = Device::new(&mut *bus, cs0);
        assert_eq!(cs_sense(), CS_IDLE);
        let start = Timer::new().now_32();
        INJECT
            .with(|alarm| alarm.try_arm(start + INJECT_US.micros()))
            .ok_or(Error::Busy)??;
        let result = dev.write(&[0x79u8]);
        let sensed = cs_sense();
        let returned = now();
        let elapsed = returned.wrapping_sub(start.ticks());
        let at = INJECTED_AT.load(Relaxed).wrapping_sub(start.ticks());
        let late = elapsed.saturating_sub(at) + 1;
        let irq = FAULT.load(Relaxed) - before;
        assert_eq!(result, Err(Error::Overrun));
        assert_eq!(sensed, CS_IDLE);
        log!("SPI_FAULT_SAMPLE trial={} inject_us={} release_us={} late_us={} fault_irq={} released=1\n",
            trial, at, elapsed, late, irq).unwrap();
        assert_eq!(irq, 1);
        assert!((INJECT_US - 50..=INJECT_US + SLACK_US).contains(&elapsed));
        assert!((INJECT_US..=INJECT_US + SLACK_US).contains(&at));
        assert!(at <= elapsed && late <= SLACK_US);
        fault_min = fault_min.min(elapsed);
        fault_max = fault_max.max(elapsed);
        fault_late_max = fault_late_max.max(late);
        cs0 = dev.free().1;
        bus.try_recover()?;
        let mut rx = [0u8; 1];
        bus.transfer_until(&mut rx, &[0x79], Deadline::after(100.millis())?)?;
        bus.flush_until(Deadline::after(100.millis())?)?;
        assert_eq!(rx, [0x79]);
    }
    let bad_cs = CS_BAD.load(Relaxed);
    let stray = STRAY.load(Relaxed) - stray_before;
    log!(
        "SPI_BUS_TRIALS cs0={} cs1={} forced_error={} recovered={} cs_delay_min_us={} cs_delay_upper_us={} fault_release_min_us={} fault_release_max_us={} fault_late_max_us={} bad_cs={} stray={} errors=0\n",
        CS_TRIALS, CS_TRIALS, TRIALS, TRIALS, delay_min, delay_upper, fault_min, fault_max, fault_late_max, bad_cs, stray
    ).unwrap();
    assert_eq!((bad_cs, stray), (0, 0));
    Ok(())
}

fn run(core: &mut Proc0, mut p: rp1::Proc0Resources) -> Result<(), Error> {
    let sys = rp1::platform::init(core, p.platform).unwrap();
    p.proc1.launch(core).unwrap();
    #[cfg(feature = "isr-trace")]
    platform_trace::install();
    log!(
        "SPI_INSTANCE instance={}
",
        N
    )
    .unwrap();
    let mut inject = p.TIMER_ALARM0;
    inject.clear_interrupt();
    inject.enable_interrupt();
    INJECT
        .put(inject)
        .unwrap_or_else(|_| panic!("fault injection alarm slot occupied"));
    p.irq.TIMER_0.enable();

    let mut spi = parts!(p)
        .init(&sys.clocks, Config::new(1.MHz(), MODE_0))
        .map_err(|error| *error.error())?;
    let wait = &mut p.wait_timer;
    spi.blocking(wait).set_loopback(true)?;
    let mut gate = p.pins.gpio8.into_output(false);
    let _sense0 = p.pins.gpio12.into_input();
    let _sense1 = p.pins.gpio13.into_input();
    let _cs0 = Device::new((), p.pins.gpio22).free().1;
    let _cs1 = Device::new((), p.pins.gpio23).free().1;
    assert_eq!(cs_sense(), CS_IDLE);
    log!("SPI_PARITY_READY\n").unwrap();

    let mut bus = spi.blocking(wait);
    bulk(&mut bus, &mut gate, 1_000_000)?;
    bulk(&mut bus, &mut gate, 10_000_000)?;
    #[cfg(feature = "isr-trace")]
    {
        traced_flushes("early", &mut spi, wait, &mut gate)?;
        platform_trace::wait_for_linux();
        traced_flushes("linux", &mut spi, wait, &mut gate)?;
    }
    #[cfg(not(feature = "isr-trace"))]
    {
        tx_flush(&mut spi, wait, &mut gate, 1_000_000, false)?;
        // The slower fixed rate leaves time after TXEI to exercise the tail
        // one-shot even if the shifter has already finished at 1 MHz.
        tx_flush(&mut spi, wait, &mut gate, 100_000, true)?;
        timeout_rearm(&mut spi, wait)?;
        devices(&mut spi.blocking(wait), _cs0, _cs1)?;

        let bytes: [u8; 32] = core::array::from_fn(|i| (i as u8).wrapping_mul(37));
        let mut rx = [0; 32];
        let mut compare = |bus: &mut dyn SpiBus<u8, Error = Error>| -> Result<u32, Error> {
            let start = now();
            bus.transfer(&mut rx, &bytes)?;
            bus.flush()?;
            assert_eq!(rx, bytes);
            Ok(now().wrapping_sub(start))
        };
        let irq_us = compare(&mut spi.blocking(wait))?;
        let poll_us = compare(&mut spi.polling())?;
        log!(
            "SPI_POLL_COMPARISON hz=1000000 frames=32 irq_us={} polling_us={} stray={} errors=0\n",
            irq_us,
            poll_us,
            STRAY.load(Relaxed)
        )
        .unwrap();
    }
    assert_eq!(STRAY.load(Relaxed), 0);
    Ok(())
}

#[entry(proc0)]
fn main(mut core: Proc0, p: rp1::Proc0Resources) -> ! {
    match run(&mut core, p) {
        Ok(()) => log!("SPI_PARITY_FIRMWARE_END result=PASS\n").unwrap(),
        Err(e) => log!("SPI_PARITY_FAIL code={}\n", e as u32).unwrap(),
    }
    idle()
}

#[entry(proc1)]
fn park(_core: Proc1, _p: rp1::Proc1Resources) -> ! {
    idle_probe::observe()
}

fn idle() -> ! {
    loop {
        rp1_rt::wait_for_event();
    }
}
