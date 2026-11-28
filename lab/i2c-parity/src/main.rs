//! I2C controller parity, with an ESP32 peer or an open-drain bus with no target.
#![no_std]
#![no_main]

use core::sync::atomic::{AtomicU32, Ordering::Relaxed};
use embedded_hal::i2c::Operation;
use rp1_hal::{
    fugit::ExtU32,
    i2c::{Address7, Error, Speed},
    log, pac,
    timer::{Deadline, Timer, WaitTimer},
};
use rp1_rt::{Proc0, Proc1, entry};

rp1_rt::bind!();
include!(concat!(env!("OUT_DIR"), "/instance.rs"));

const FAULT_TRIALS: u32 = 100;
const DEADLINE_US: u32 = 2_000;

static RX: AtomicU32 = AtomicU32::new(0);
static TX: AtomicU32 = AtomicU32::new(0);
static STOP: AtomicU32 = AtomicU32::new(0);
static ABORT: AtomicU32 = AtomicU32::new(0);
static OTHER: AtomicU32 = AtomicU32::new(0);
static STRAY: AtomicU32 = AtomicU32::new(0);
static ALARM: AtomicU32 = AtomicU32::new(0);
static FIRST_ALARM: AtomicU32 = AtomicU32::new(0);
#[cfg(any(feature = "i2c0", feature = "i2c2", feature = "i2c3"))]
static ABORT_AT: AtomicU32 = AtomicU32::new(0);
static TOTAL_STRAY: AtomicU32 = AtomicU32::new(0);
static TOTAL_OTHER: AtomicU32 = AtomicU32::new(0);

#[inline(always)]
fn observe_device(dispatch: impl FnOnce() -> u32) {
    use pac::dw_i2c::intr::{RX_FULL, STOP_DET, TX_ABRT, TX_EMPTY};
    #[cfg(any(feature = "i2c0", feature = "i2c2", feature = "i2c3"))]
    let entered = Timer::new().now_32().ticks();
    let bits = dispatch();
    #[cfg(any(feature = "i2c0", feature = "i2c2", feature = "i2c3"))]
    if bits & TX_ABRT != 0 {
        ABORT_AT.store(entered, Relaxed);
    }
    for (counter, seen) in [
        (&RX, bits & RX_FULL != 0),
        (&TX, bits & TX_EMPTY != 0),
        (&STOP, bits & STOP_DET != 0),
        (&ABORT, bits & TX_ABRT != 0),
        (
            &OTHER,
            bits & !(RX_FULL | TX_EMPTY | STOP_DET | TX_ABRT) != 0,
        ),
        (&STRAY, bits == 0),
        (&TOTAL_STRAY, bits == 0),
        (
            &TOTAL_OTHER,
            bits & !(RX_FULL | TX_EMPTY | STOP_DET | TX_ABRT) != 0,
        ),
    ] {
        if seen {
            counter.store(counter.load(Relaxed) + 1, Relaxed);
        }
    }
}

#[inline(always)]
fn observe_wait_alarm() {
    // SAFETY: called on proc0 by the generated ALARM1 dispatcher.
    unsafe { WaitTimer::__on_interrupt::<1>() };
    let alarms = ALARM.load(Relaxed);
    if alarms == 0 {
        FIRST_ALARM.store(Timer::new().now_32().ticks(), Relaxed);
    }
    ALARM.store(alarms + 1, Relaxed);
}

fn idle() -> bool {
    // SAFETY: this fixture owns the block and only reads non-acknowledging status.
    let block = unsafe { &*Registers::PTR.cast::<pac::dw_i2c::RegisterBlock>() };
    block.enable_status.read().bits() & 1 == 0
        && block.status.read().bits() & (1 << 5) == 0
        && block.intr_mask.read().bits() == 0
        && block.irq_wrap.read().bits() & 1 == 0
}

fn levels() -> u32 {
    // SAFETY: RIO0 input is read-only and monitors the pads with the I2C mux.
    unsafe { (0x400e_0008 as *const u32).read_volatile() }
}

#[entry(proc0)]
fn main(mut core: Proc0, mut p: rp1::Proc0Resources) -> ! {
    let sys = rp1::platform::init(&mut core, p.platform).unwrap();
    let mut bus = parts!(p).init(&sys.clocks, Speed::Standard).unwrap();
    #[cfg(not(any(feature = "i2c0", feature = "i2c2", feature = "i2c3")))]
    let result = esp32::run(&mut bus, &mut p.wait_timer);
    #[cfg(any(feature = "i2c0", feature = "i2c2", feature = "i2c3"))]
    let result = {
        #[cfg(any(feature = "i2c2", feature = "i2c3"))]
        let pull_up = rp1_hal::gpio::PadConfig::INPUT_DEFAULT.with_pull(rp1_hal::gpio::Pull::Up);
        #[cfg(feature = "i2c2")]
        let _peer = (
            p.pins.gpio22.into_input_with_pad(pull_up),
            p.pins.gpio23.into_input_with_pad(pull_up),
        );
        #[cfg(feature = "i2c3")]
        let _peer = (
            p.pins.gpio12.into_input_with_pad(pull_up),
            p.pins.gpio13.into_input_with_pad(pull_up),
        );
        p.proc1.launch(&mut core).unwrap();
        absent_target(&mut bus, &mut p.wait_timer)
    };
    let _ = log!(
        "I2C_FIXTURE_END stray={} other={} result={:?}\n",
        TOTAL_STRAY.load(Relaxed),
        TOTAL_OTHER.load(Relaxed),
        result
    );
    loop {
        rp1_rt::wait_for_event();
    }
}

#[cfg(not(any(feature = "i2c0", feature = "i2c2", feature = "i2c3")))]
mod esp32;

fn clear_observations() {
    for counter in [&RX, &TX, &STOP, &ABORT, &STRAY, &OTHER, &ALARM] {
        counter.store(0, Relaxed);
    }
}

#[cfg(any(feature = "i2c0", feature = "i2c2", feature = "i2c3"))]
mod phases;

#[cfg(any(feature = "i2c0", feature = "i2c2", feature = "i2c3"))]
const SCL: u32 = match N {
    0 => 1 << 9,
    2 => 1 << 13,
    _ => 1 << 23,
};
#[cfg(any(feature = "i2c0", feature = "i2c2", feature = "i2c3"))]
const LINES: u32 = if N == 0 {
    3 << 8
} else {
    (3 << 12) | (3 << 22)
};
#[cfg(any(feature = "i2c0", feature = "i2c2", feature = "i2c3"))]
const ABSENT: Address7 = Address7::new_const::<0x55>();

#[cfg(any(feature = "i2c0", feature = "i2c2", feature = "i2c3"))]
fn nack(
    bus: &mut Driver,
    timer: &mut WaitTimer,
    phase: &str,
    bounds: core::ops::RangeInclusive<u32>,
    sample: bool,
) -> (u32, u32) {
    clear_observations();
    if sample {
        phases::arm();
    }
    let start = Timer::new().now();
    let result = bus.blocking(timer).transaction_until(
        ABSENT,
        &mut [Operation::Write(&[0xa5])],
        Deadline::at(start + DEADLINE_US.micros()),
    );
    let us = (Timer::new().now() - start).ticks() as u32;
    let abrt = ABORT.load(Relaxed);
    let irq_us = ABORT_AT.load(Relaxed).wrapping_sub(start.ticks() as u32);
    let rx = RX.load(Relaxed);
    let tx = TX.load(Relaxed);
    let other = OTHER.load(Relaxed);
    let stray = STRAY.load(Relaxed);
    let idle = idle();
    let pads = levels() & LINES;
    if sample {
        phases::report(phase, start.ticks() as u32, irq_us, us);
    }
    assert!(
        result == Err(Error::AddressNack)
            && abrt == 1
            && rx + tx + other + stray == 0
            && (70..=us).contains(&irq_us)
            && bounds.contains(&us)
            && idle
            && pads == LINES,
        "I2C{N} {phase} NACK {result:?} {us} us (bound {bounds:?}), IRQ {irq_us} us, TX_ABRT={abrt} rx={rx} tx={tx} other={other} stray={stray} idle={idle} pads={pads:#x}"
    );
    (us, irq_us)
}

#[cfg(any(feature = "i2c0", feature = "i2c2", feature = "i2c3"))]
fn absent_target(bus: &mut Driver, timer: &mut WaitTimer) -> Result<(), Error> {
    timer.delay(10.micros())?;
    let idle_levels = levels() & LINES;
    assert_eq!(idle_levels, LINES);
    log!("I2C_NACK_IDLE instance={N} pad_levels={idle_levels:#x} pull_up=1\n").unwrap();
    let (first, _) = nack(bus, timer, "first", 100..=550, true);
    let mut lo = u32::MAX;
    let mut hi = 0;
    let mut irq_lo = u32::MAX;
    let mut irq_hi = 0;
    for trial in 1..FAULT_TRIALS {
        let (us, irq_us) = nack(bus, timer, "steady", 100..=220, trial == 1);
        lo = lo.min(us);
        hi = hi.max(us);
        irq_lo = irq_lo.min(irq_us);
        irq_hi = irq_hi.max(irq_us);
    }
    log!(
        "I2C_NACK_IRQ instance={N} hz=100000 trials={FAULT_TRIALS} tx_abrt={FAULT_TRIALS} first_us={first} steady_us={lo}..{hi} steady_irq_us={irq_lo}..{irq_hi}\n"
    )
    .unwrap();

    // Expire before the ninth SCL edge, then verify the IRQ can arm again.
    const SHORT_US: u32 = 40;
    clear_observations();
    let start = Timer::new().now();
    let result = bus.blocking(timer).transaction_until(
        ABSENT,
        &mut [Operation::Write(&[0xa5])],
        Deadline::at(start + SHORT_US.micros()),
    );
    let alarm_us = FIRST_ALARM.load(Relaxed).wrapping_sub(start.ticks() as u32);
    let device_irqs = ABORT.load(Relaxed) + RX.load(Relaxed) + TX.load(Relaxed);
    assert!(
        result == Err(Error::Timeout)
            && device_irqs == 0
            && (SHORT_US..=SHORT_US + 4).contains(&alarm_us),
        "I2C{N} timeout {result:?} alarm_us={alarm_us} device_irqs={device_irqs}"
    );
    bus.try_recover()?;
    assert!(idle() && levels() & LINES == LINES);
    nack(bus, timer, "rearm", 100..=550, true);
    log!("I2C_NACK_REARM instance={N} alarm_us={alarm_us} tx_abrt=1 result=PASS\n").unwrap();
    Ok(())
}

#[entry(proc1)]
fn parked(_core: Proc1, _p: rp1::Proc1Resources) -> ! {
    #[cfg(any(feature = "i2c0", feature = "i2c2", feature = "i2c3"))]
    phases::observe();
    #[cfg(not(any(feature = "i2c0", feature = "i2c2", feature = "i2c3")))]
    loop {
        rp1_rt::wait_for_event();
    }
}
