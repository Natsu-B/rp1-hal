//! I2C controller parity, with an ESP32 peer or an open-drain jumper bus.
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
static TOTAL_STRAY: AtomicU32 = AtomicU32::new(0);
static TOTAL_OTHER: AtomicU32 = AtomicU32::new(0);

#[inline(always)]
fn observe_device(dispatch: impl FnOnce() -> u32) {
    use pac::dw_i2c::intr::{RX_FULL, STOP_DET, TX_ABRT, TX_EMPTY};
    let bits = dispatch();
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
    #[cfg(not(any(feature = "i2c2", feature = "i2c3")))]
    let result = esp32::run(&mut bus, &mut p.wait_timer);
    #[cfg(any(feature = "i2c2", feature = "i2c3"))]
    let result = {
        use rp1_hal::gpio::{PadConfig, Pull};
        let pull_up = PadConfig::INPUT_DEFAULT.with_pull(Pull::Up);
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
        jumpers(&mut bus, &mut p.wait_timer)
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

#[cfg(not(any(feature = "i2c2", feature = "i2c3")))]
mod esp32;

fn clear_observations() {
    for counter in [&RX, &TX, &STOP, &ABORT, &STRAY, &OTHER, &ALARM] {
        counter.store(0, Relaxed);
    }
}

#[cfg(any(feature = "i2c2", feature = "i2c3"))]
fn jumpers(bus: &mut Driver, timer: &mut WaitTimer) -> Result<(), Error> {
    const LINES: u32 = (3 << 12) | (3 << 22);
    const ABSENT: Address7 = Address7::new_const::<0x55>();
    const MIN_US: u32 = 80;
    const MAX_US: u32 = 250;
    let mut latency = (u32::MAX, 0);
    let mut nack = |bus: &mut Driver, timer: &mut WaitTimer| {
        clear_observations();
        let start = Timer::new().now();
        let result = bus.blocking(timer).transaction_until(
            ABSENT,
            &mut [Operation::Write(&[0xa5])],
            Deadline::at(start + DEADLINE_US.micros()),
        );
        let us = (Timer::new().now() - start).ticks() as u32;
        assert_eq!(result, Err(Error::AddressNack));
        assert_eq!(ABORT.load(Relaxed), 1);
        assert_eq!(
            RX.load(Relaxed) + TX.load(Relaxed) + OTHER.load(Relaxed) + STRAY.load(Relaxed),
            0
        );
        assert!((MIN_US..=MAX_US).contains(&us));
        assert!(idle() && levels() & LINES == LINES);
        latency = (latency.0.min(us), latency.1.max(us));
    };

    timer.delay(10.micros())?;
    let idle_levels = levels() & LINES;
    assert_eq!(idle_levels, LINES);
    log!(
        "I2C_JUMPER_IDLE instance={N} pad_levels={idle_levels:#x} pull_up=1
"
    )
    .unwrap();
    for _ in 0..FAULT_TRIALS {
        nack(bus, timer);
    }
    let (lo, hi) = latency;
    log!("I2C_JUMPER_NACK instance={N} hz=100000 trials={FAULT_TRIALS} tx_abrt={FAULT_TRIALS} min_us={lo} max_us={hi}
").unwrap();

    // Expire before the ninth SCL edge while leaving both controllers open drain.
    const SHORT_US: u32 = 40;
    clear_observations();
    let start = Timer::new().now();
    let result = bus.blocking(timer).transaction_until(
        ABSENT,
        &mut [Operation::Write(&[0xa5])],
        Deadline::at(start + SHORT_US.micros()),
    );
    let alarm_us = FIRST_ALARM.load(Relaxed).wrapping_sub(start.ticks() as u32);
    assert_eq!(result, Err(Error::Timeout));
    assert_eq!(ABORT.load(Relaxed) + RX.load(Relaxed) + TX.load(Relaxed), 0);
    assert!((SHORT_US..=SHORT_US + 4).contains(&alarm_us));
    bus.try_recover()?;
    assert!(idle() && levels() & LINES == LINES);
    nack_after_timeout(bus, timer, ABSENT)?;
    log!(
        "I2C_JUMPER_REARM instance={N} alarm_us={alarm_us} tx_abrt=1 result=PASS
"
    )
    .unwrap();
    Ok(())
}

#[cfg(any(feature = "i2c2", feature = "i2c3"))]
fn nack_after_timeout(
    bus: &mut Driver,
    timer: &mut WaitTimer,
    address: Address7,
) -> Result<(), Error> {
    clear_observations();
    let end = Deadline::after(DEADLINE_US.micros())?;
    assert_eq!(
        bus.blocking(timer)
            .transaction_until(address, &mut [Operation::Write(&[0xa5])], end),
        Err(Error::AddressNack)
    );
    assert_eq!(ABORT.load(Relaxed), 1);
    assert!(idle());
    Ok(())
}

#[entry(proc1)]
fn parked(_core: Proc1, _p: rp1::Proc1Resources) -> ! {
    loop {
        rp1_rt::wait_for_event();
    }
}
