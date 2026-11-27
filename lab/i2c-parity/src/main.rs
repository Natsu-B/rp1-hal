//! I2C1 FIFO, STOP, abort and held-SCL deadline parity.
#![no_std]
#![no_main]

use core::sync::atomic::{AtomicU32, Ordering::Relaxed};
use embedded_hal::i2c::{I2c as _, Operation};
use rp1_hal::{
    fugit::ExtU32,
    i2c::{Address7, Blocking, Error, Speed},
    log, pac,
    timer::{Deadline, Timer, WaitTimer},
};
use rp1_rt::{Proc0, Proc1, entry};

rp1_rt::bind!();

const PEER: u8 = 0x2d;
const TRIALS: u32 = 1_000;
const FAULT_TRIALS: u32 = 100;
const FRAME: [u8; 16] = *b"D1DR\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f";
const LONG_FRAME: [u8; 96] = {
    let mut frame = [0; 96];
    let mut i = 0;
    while i < frame.len() {
        frame[i] = if i < 16 { FRAME[i] } else { i as u8 };
        i += 1;
    }
    frame
};
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

type Bus<'a> = Blocking<'a, rp1::I2c1Pins, 0>;

#[inline(always)]
fn observe_i2c1(dispatch: impl FnOnce() -> u32) {
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
    let block = unsafe { &*pac::I2C1::PTR.cast::<pac::dw_i2c::RegisterBlock>() };
    block.enable_status.read().bits() & 1 == 0
        && block.status.read().bits() & (1 << 5) == 0
        && block.intr_mask.read().bits() == 0
        && block.irq_wrap.read().bits() & 1 == 0
}

fn scl_high() -> bool {
    // GPIO3 input still monitors the pad while I2C owns the mux.
    unsafe { (0x400e_0008 as *const u32).read_volatile() & (1 << 3) != 0 }
}

struct HoldScl(u32);

impl HoldScl {
    // SAFETY: only this image owns GPIO3. Override the I2C output with a
    // constant LOW and force OE on. Keep the input path and SDA untouched.
    const CTRL: *mut u32 = (0x400d_0004 + 3 * 8) as *mut u32;
    fn low() -> Self {
        unsafe {
            let saved = Self::CTRL.read_volatile();
            let force_low_and_enable = (2 << 12) | (3 << 14);
            Self::CTRL.write_volatile((saved & !(0xf << 12)) | force_low_and_enable);
            Self(saved)
        }
    }
}

impl Drop for HoldScl {
    fn drop(&mut self) {
        // The saved I2C configuration restores open-drain control.
        unsafe { Self::CTRL.write_volatile(self.0) };
    }
}

fn check(reply: &[u8; 64]) -> bool {
    *reply == core::array::from_fn(|i| (0x31 + 29 * i) as u8)
}

fn refill(bus: &mut Bus<'_>) -> Result<(), Error> {
    bus.delay(2.millis())
}

struct Stats {
    count: u32,
    ok: u32,
    idle: u32,
    min: u64,
    max: u64,
    irq_bad: u32,
    rx: u32,
    tx: u32,
    stop: u32,
    abort: u32,
    stray: u32,
    other: u32,
}

impl Stats {
    fn new() -> Self {
        Self {
            count: 0,
            ok: 0,
            idle: 0,
            min: u64::MAX,
            max: 0,
            irq_bad: 0,
            rx: 0,
            tx: 0,
            stop: 0,
            abort: 0,
            stray: 0,
            other: 0,
        }
    }

    fn record(&mut self, start: u64, ok: bool, irq_ok: bool) {
        let elapsed = Timer::new().now().ticks() - start;
        self.count += 1;
        self.ok += u32::from(ok);
        self.idle += u32::from(idle());
        self.min = self.min.min(elapsed);
        self.max = self.max.max(elapsed);
        self.irq_bad += u32::from(!irq_ok || STRAY.load(Relaxed) != 0 || OTHER.load(Relaxed) != 0);
        self.rx += RX.load(Relaxed);
        self.tx += TX.load(Relaxed);
        self.stop += STOP.load(Relaxed);
        self.abort += ABORT.load(Relaxed);
        self.stray += STRAY.load(Relaxed);
        self.other += OTHER.load(Relaxed);
    }

    fn report(&self, kind: &str, hz: u32) {
        let _ = log!(
            "I2C_PHASE kind={} hz={} count={} ok={} idle={} min_us={} max_us={} irq_bad={} rx={} tx={} stop={} abort={} stray={} other={}\n",
            kind,
            hz,
            self.count,
            self.ok,
            self.idle,
            self.min,
            self.max,
            self.irq_bad,
            self.rx,
            self.tx,
            self.stop,
            self.abort,
            self.stray,
            self.other,
        );
    }
}

#[entry(proc0)]
fn main(mut core: Proc0, mut p: rp1::Proc0Resources) -> ! {
    let sys = rp1::platform::init(&mut core, p.platform).unwrap();
    let mut bus = p.i2c1.init(&sys.clocks, Speed::Standard).unwrap();
    let result = run(&mut bus, &mut p.wait_timer);
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

fn run(bus: &mut rp1::I2c1, timer: &mut WaitTimer) -> Result<(), Error> {
    let mut reply = [0; 64];
    let mut blocking = bus.blocking(timer);
    blocking.read(PEER, &mut reply)?;
    refill(&mut blocking)?;
    blocking.read(PEER, &mut reply)?;
    if !check(&reply) || !idle() || !scl_high() {
        return Err(Error::Precondition);
    }
    refill(&mut blocking)?;
    let _ = log!("I2C_PEER_READY addr=0x2d bytes=64\n");
    // SAFETY: the fixture owns I2C1; COMP_PARAM_1 is read-only.
    let block = unsafe { &*pac::I2C1::PTR.cast::<pac::dw_i2c::RegisterBlock>() };
    let tx_depth = ((block.comp_param_1.read().bits() >> 16) & 0xff) + 1;
    if tx_depth as usize >= LONG_FRAME.len() {
        return Err(Error::Precondition);
    }
    let _ = log!(
        "I2C_TX_FIFO depth={} frame={}\n",
        tx_depth,
        LONG_FRAME.len()
    );
    let counts = [
        combined(bus, timer, Speed::Standard, 100_000)?,
        combined(bus, timer, Speed::Fast, 400_000)?,
    ];
    let tx_counts = [
        tx_refill(bus, timer, Speed::Standard, 100_000)?,
        tx_refill(bus, timer, Speed::Fast, 400_000)?,
    ];
    let mut blocking = bus.blocking(timer);
    blocking.set_speed(Speed::Standard)?;
    let mut stats = Stats::new();
    for _ in 0..FAULT_TRIALS {
        clear_observations();
        let start = Timer::new().now();
        let result = blocking.transaction_until(
            Address7::new_const::<PEER>(),
            &mut [Operation::Write(&FRAME)],
            Deadline::at(start + 10_000.micros()),
        );
        let irq_ok = RX.load(Relaxed) == 0
            && TX.load(Relaxed) == 0
            && (result.is_err() || (STOP.load(Relaxed) == 1 && ABORT.load(Relaxed) == 0));
        stats.record(start.ticks(), result.is_ok(), irq_ok);
        refill(&mut blocking)?;
    }
    stats.report("stop", 100_000);

    let mut stats = Stats::new();
    let mut rearm = 0;
    for _ in 0..FAULT_TRIALS {
        clear_observations();
        let start = Timer::new().now();
        let result = blocking.transaction_until(
            Address7::new_const::<0x2e>(),
            &mut [Operation::Read(&mut [0])],
            Deadline::at(start + 10_000.micros()),
        );
        let irq_ok = ABORT.load(Relaxed) == 1
            && RX.load(Relaxed) == 0
            && TX.load(Relaxed) == 0
            && STOP.load(Relaxed) == 0;
        let ok = result == Err(Error::AddressNack);
        stats.record(start.ticks(), ok, !ok || irq_ok);
        reply.fill(0);
        rearm += u32::from(blocking.read(PEER, &mut reply).is_ok() && check(&reply) && idle());
        refill(&mut blocking)?;
    }
    stats.report("nack", 100_000);
    let _ = log!("I2C_REARM kind=nack count={}\n", rearm);

    let mut stats = Stats::new();
    let mut held_low = 0;
    let mut rejected = 0;
    let mut recovered = 0;
    let mut rearm = 0;
    for _ in 0..FAULT_TRIALS {
        let hold = HoldScl::low();
        bus.blocking(timer).delay(10.micros())?;
        clear_observations();
        let start = Timer::new().now();
        let result = bus.blocking(timer).transaction_until(
            Address7::new_const::<PEER>(),
            &mut [Operation::Read(&mut [0])],
            Deadline::at(start + DEADLINE_US.micros()),
        );
        let irq_ok = RX.load(Relaxed) == 0
            && TX.load(Relaxed) == 0
            && STOP.load(Relaxed) == 0
            && ABORT.load(Relaxed) == 0
            && ALARM.load(Relaxed) != 0
            && (DEADLINE_US..=DEADLINE_US + 100)
                .contains(&FIRST_ALARM.load(Relaxed).wrapping_sub(start.ticks() as u32));
        let ok = result == Err(Error::Timeout);
        stats.record(start.ticks(), ok, !ok || irq_ok);
        held_low += u32::from(!scl_high());
        rejected += u32::from(matches!(
            bus.blocking(timer).read(PEER, &mut [0]),
            Err(Error::Precondition | Error::Busy)
        ));
        drop(hold);
        refill(&mut bus.blocking(timer))?;
        recovered += u32::from(bus.try_recover().is_ok() && idle() && scl_high());
        // The aborted read may consume a prefix at release. A full read with STOP
        // resynchronizes the ESP32 before the checked re-arm.
        let _ = bus.blocking(timer).read(PEER, &mut reply);
        refill(&mut bus.blocking(timer))?;
        reply.fill(0);
        rearm += u32::from(
            bus.blocking(timer).read(PEER, &mut reply).is_ok() && check(&reply) && idle(),
        );
        refill(&mut bus.blocking(timer))?;
    }
    stats.report("timeout", 100_000);
    let _ = log!(
        "I2C_REARM kind=timeout count={} held_low={} rejected={} recovered={}\n",
        rearm,
        held_low,
        rejected,
        recovered
    );
    let _ = log!("I2C_FMP_BEGIN\n");
    // Let the host snapshot peer counters before Fm+ traffic starts.
    bus.blocking(timer).delay(3_000.millis())?;
    combined(bus, timer, Speed::FastPlus, 1_000_000)?;
    if counts != [TRIALS; 2] || tx_counts != [TRIALS; 2] || rearm != FAULT_TRIALS {
        return Err(Error::Precondition);
    }
    Ok(())
}

fn combined(
    bus: &mut rp1::I2c1,
    timer: &mut WaitTimer,
    speed: Speed,
    hz: u32,
) -> Result<u32, Error> {
    bus.blocking(timer).set_speed(speed)?;
    let mut reply = [0; 64];
    let mut stats = Stats::new();
    for _ in 0..TRIALS {
        clear_observations();
        reply.fill(0);
        let start = Timer::new().now();
        let result = bus.blocking(timer).transaction_until(
            Address7::new_const::<PEER>(),
            &mut [Operation::Write(&FRAME), Operation::Read(&mut reply)],
            Deadline::at(start + 10_000.micros()),
        );
        let ok = result.is_ok() && check(&reply);
        let irq_ok = RX.load(Relaxed) <= 64
            && TX.load(Relaxed) <= 17
            && STOP.load(Relaxed) <= 1
            && ABORT.load(Relaxed) <= 1
            && (result.is_err() || (RX.load(Relaxed) != 0 && ABORT.load(Relaxed) == 0));
        stats.record(start.ticks(), ok, irq_ok);
        if !ok && stats.ok + 1 == stats.count {
            let _ = log!(
                "I2C_COMBINED_ERROR hz={} trial={} result={:?} first_byte={}\n",
                hz,
                stats.count,
                result,
                reply[0]
            );
        }
        if matches!(result, Err(e) if e != Error::AddressNack && e != Error::DataNack) {
            bus.try_recover()?;
        }
        refill(&mut bus.blocking(timer))?;
    }
    stats.report("combined", hz);
    Ok(stats.ok)
}

fn tx_refill(
    bus: &mut rp1::I2c1,
    timer: &mut WaitTimer,
    speed: Speed,
    hz: u32,
) -> Result<u32, Error> {
    bus.blocking(timer).set_speed(speed)?;
    let mut stats = Stats::new();
    for _ in 0..TRIALS {
        clear_observations();
        let start = Timer::new().now();
        let result = bus.blocking(timer).transaction_until(
            Address7::new_const::<PEER>(),
            &mut [Operation::Write(&LONG_FRAME)],
            Deadline::at(start + 10_000.micros()),
        );
        let tx = TX.load(Relaxed);
        let irq_ok = tx < LONG_FRAME.len() as u32
            && RX.load(Relaxed) == 0
            && STOP.load(Relaxed) <= 1
            && ABORT.load(Relaxed) <= 1
            && (result.is_err() || (tx != 0 && ABORT.load(Relaxed) == 0));
        stats.record(start.ticks(), result.is_ok(), irq_ok);
        if let Err(error) = result {
            stats.report("tx_refill", hz);
            return Err(error);
        }
        // Give the ESP32's receive callback time to validate this frame.
        refill(&mut bus.blocking(timer))?;
    }
    stats.report("tx_refill", hz);
    Ok(stats.ok)
}

fn clear_observations() {
    for counter in [&RX, &TX, &STOP, &ABORT, &STRAY, &OTHER, &ALARM] {
        counter.store(0, Relaxed);
    }
}

#[entry(proc1)]
fn parked(_core: Proc1, _p: rp1::Proc1Resources) -> ! {
    loop {
        rp1_rt::wait_for_event();
    }
}
