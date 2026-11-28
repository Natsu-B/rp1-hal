//! Stamped control IRQs and proc0 device waits during a Linux clock sweep.
#![no_std]
#![no_main]

use core::{
    mem::{MaybeUninit, offset_of},
    sync::atomic::{AtomicU32, Ordering},
};
use embedded_hal::{digital::OutputPin, spi::MODE_0};
use rp1_hal::{
    fugit::{ExtU32, RateExtU32, TimerInstantU32},
    gpio::{Output, Pin},
    log, pac, platform,
    spi::{Config, Try},
    timer::{Alarm, Deadline, Timer},
    uart,
};
use rp1_rt::{Proc0, Proc1, entry, interrupt, interrupt::Priority};

rp1_rt::bind!();

#[path = "../../spi-parity/src/idle_probe.rs"]
mod idle_probe;
#[allow(dead_code)]
#[path = "../../spi-parity/src/platform_trace.rs"]
mod platform_trace;

const TICKS: u32 = 12_000;
const PERIOD: u32 = 1_000;
const OBSERVED_PULSES: u32 = 256;
const START: u32 = u32::from_le_bytes(*b"PRIO");
const BINS: usize = 65;

#[unsafe(link_section = ".shared")]
static FIRST: AtomicU32 = AtomicU32::new(0);
#[unsafe(link_section = ".shared")]
static PROC1_COUNT: AtomicU32 = AtomicU32::new(0);
#[unsafe(link_section = ".shared")]
static PROC1_P99: AtomicU32 = AtomicU32::new(0);
#[unsafe(link_section = ".shared")]
static PROC1_MAX: AtomicU32 = AtomicU32::new(0);
#[unsafe(link_section = ".shared")]
static PROC1_IRQ_MAX: AtomicU32 = AtomicU32::new(0);

static mut ALARM0: MaybeUninit<Alarm<0>> = MaybeUninit::uninit();
static mut ALARM1: MaybeUninit<Alarm<1>> = MaybeUninit::uninit();
static mut OUTPUT: MaybeUninit<Pin<13, Output>> = MaybeUninit::uninit();
// Each core has its own DSRAM copy. Only the control IRQ updates its copy.
static mut CONTROL: Samples = Samples::new();
static mut NEXT: u32 = 0;
static IRQ_COUNT: [AtomicU32; 64] = [const { AtomicU32::new(0) }; 64];
static IRQ_MAX: [AtomicU32; 64] = [const { AtomicU32::new(0) }; 64];

#[inline(always)]
fn now() -> u32 {
    Timer::new().now_32().ticks()
}

struct Samples {
    count: u32,
    max: u32,
    bins: [u32; BINS],
}

impl Samples {
    const fn new() -> Self {
        Self {
            count: 0,
            max: 0,
            bins: [0; BINS],
        }
    }

    fn record(&mut self, value: u32) {
        self.count += 1;
        self.max = self.max.max(value);
        self.bins[(value as usize).min(BINS - 1)] += 1;
    }

    fn p99(&self) -> u32 {
        let rank = (99 * self.count).div_ceil(100);
        let mut total = 0;
        for (bucket, count) in self.bins.iter().enumerate() {
            total += count;
            if total >= rank {
                return bucket as u32;
            }
        }
        0
    }

    fn report(&self, metric: &str) {
        log!(
            "PRIO_SAMPLE metric={metric} n={} p99_us={} max_us={}\n",
            self.count,
            self.p99(),
            self.max
        )
        .unwrap();
    }
}

#[inline(always)]
fn stamp<const IRQ: usize>(handler: impl FnOnce()) {
    let entered = now();
    handler();
    let elapsed = now().wrapping_sub(entered);
    let count = &IRQ_COUNT[IRQ];
    let max = &IRQ_MAX[IRQ];
    count.store(count.load(Ordering::Relaxed) + 1, Ordering::Relaxed);
    max.store(max.load(Ordering::Relaxed).max(elapsed), Ordering::Relaxed);
}

#[inline(always)]
fn tick<const N: u8>(alarm: &mut Alarm<N>, entered: u32) -> bool {
    alarm.clear_interrupt();
    // The handler owns these DSRAM variables after the owning core enables it.
    let next = unsafe { (&raw mut NEXT).as_mut().unwrap() };
    let stats = unsafe { (&raw mut CONTROL).as_mut().unwrap() };
    let lateness = entered.wrapping_sub(*next);
    stats.record(lateness);
    *next = next.wrapping_add(PERIOD);
    if stats.count < TICKS {
        alarm.try_arm(TimerInstantU32::from_ticks(*next)).unwrap();
    }
    stats.count == TICKS
}

#[interrupt(proc0)]
fn TIMER_0() {
    let entered = now();
    stamp::<26>(|| {
        let alarm = unsafe { (*(&raw mut ALARM0)).assume_init_mut() };
        tick(alarm, entered);
        if unsafe { (*(&raw const CONTROL)).count } <= OBSERVED_PULSES {
            let out = unsafe { (*(&raw mut OUTPUT)).assume_init_mut() };
            out.set_high().unwrap();
            let high_at = now();
            while now().wrapping_sub(high_at) < 3 {}
            out.set_low().unwrap();
        }
    });
}

#[interrupt(proc1)]
fn TIMER_1() {
    let entered = now();
    let mut done = false;
    stamp::<27>(|| {
        let alarm = unsafe { (*(&raw mut ALARM1)).assume_init_mut() };
        done = tick(alarm, entered);
    });
    if done {
        // The last comparison has been consumed. The histogram is now immutable.
        let stats = unsafe { &*(&raw const CONTROL) };
        PROC1_P99.store(stats.p99(), Ordering::Relaxed);
        PROC1_MAX.store(stats.max, Ordering::Relaxed);
        PROC1_IRQ_MAX.store(IRQ_MAX[27].load(Ordering::Relaxed), Ordering::Relaxed);
        PROC1_COUNT.store(stats.count, Ordering::Release);
    }
}

fn arm<const N: u8>(alarm: &mut Alarm<N>, first: u32) {
    unsafe { (&raw mut NEXT).write(first) };
    alarm.clear_interrupt();
    alarm.enable_interrupt();
    alarm.try_arm(TimerInstantU32::from_ticks(first)).unwrap();
}

fn inventory(core: u8, allowed: &[u32]) {
    let get = |addr: usize| unsafe { (addr as *const u32).read_volatile() };
    let enabled = u64::from(get(0xe000_e100)) | (u64::from(get(0xe000_e104)) << 32);
    assert_eq!(
        enabled & !allowed.iter().fold(0, |bits, irq| bits | 1 << irq),
        0
    );
    for &irq in allowed {
        let encoded = unsafe { (0xe000_e400 as *const u8).add(irq as usize).read_volatile() };
        log!(
            "PRIO_IRQ core={core} irq={irq} enabled={} priority={}\n",
            (enabled >> irq) & 1,
            encoded >> 5
        )
        .unwrap();
    }
}

fn observe_reset(pcie: &platform::Pcie) {
    if !matches!(platform::pcie_state(), platform::PcieState::Lost { .. }) {
        return;
    }
    if let Some(lost) = pcie.lost_detail() {
        let address = rp1_proto::LINK_START as usize + rp1_proto::link::SHM_STATUS;
        let status = unsafe { (address as *const u32).read_volatile() };
        log!("PRIO_PERST lost={lost:?} shm={status:#x}\n").unwrap();
        platform_trace::platform_totals();
        loop {
            rp1_rt::wait_for_event();
        }
    }
}

fn scmi_pending() -> u32 {
    // NVIC retains pending state for a disabled line.
    (unsafe { (0xe000_e204 as *const u32).read_volatile() } >> (57 - 32)) & 1
}

#[entry(proc0)]
fn run(mut core: Proc0, mut p: rp1::Proc0Resources) -> ! {
    let sys = rp1::platform::init(&mut core, p.platform).unwrap();
    platform_trace::install();
    let _sense = p.pins.gpio23.into_input();
    unsafe { (&raw mut OUTPUT).write(MaybeUninit::new(p.pins.gpio13.into_output(false))) };
    let mut spi = p
        .spi0
        .init(&sys.clocks, Config::new(1.MHz(), MODE_0))
        .map_err(|error| *error.error())
        .unwrap();
    let mut uart = p
        .uart1
        .init_loopback(&sys.clocks, 115_200.Hz())
        .map_err(|error| *error.error())
        .unwrap();
    let timer = &mut p.wait_timer;
    spi.blocking(timer).set_loopback(true).unwrap();
    p.proc1.launch(&mut core).unwrap();
    log!("PRIO_AWAIT_LINUX\n").unwrap();
    while !platform_trace::host_requested(START) {
        observe_reset(&sys.pcie);
    }
    let first = now().wrapping_add(20_000);
    arm(&mut p.TIMER_ALARM0, first);
    unsafe { (&raw mut ALARM0).write(MaybeUninit::new(p.TIMER_ALARM0)) };
    p.irq.TIMER_0.set_priority(Priority::control::<2>());
    p.irq.TIMER_0.enable();
    FIRST.store(first, Ordering::Release);
    rp1_rt::send_event();

    let reg = |offset| pac::SPI0::PTR.wrapping_add(offset).cast::<u32>().cast_mut();
    let select = reg(offset_of!(pac::dw_ssi::RegisterBlock, ser));
    let status = reg(offset_of!(pac::dw_ssi::RegisterBlock, sr));
    let depth = spi.fifo_depth();
    let mut spi_tail = Samples::new();
    let mut uart_tail = Samples::new();
    let mut scope_us = Samples::new();
    let mut pending_spi = 0;
    let mut pending_uart = 0;
    let before = platform::scmi_stats(&sys.pcie).requests;
    for trial in 0..1_000 {
        // Vary the phase relative to both alarms and spread the scopes across the clock load.
        let trial_at = first.wrapping_add(trial * 10_000 + trial % PERIOD);
        while (now().wrapping_sub(trial_at) as i32) < 0 {}
        unsafe { select.write_volatile(0) };
        for value in 0..depth as u8 {
            assert_eq!(spi.try_write_word(value).unwrap(), Try::Ready(()));
        }
        let (window, returned) = platform::with_scmi_deferred(&sys.pcie, || {
            let entries = platform_trace::scmi_entries();
            let end = Deadline::after(2_000.micros()).unwrap();
            let window = idle_probe::arm(status, depth as u32 * 8);
            unsafe { select.write_volatile(1) };
            spi.blocking(timer).flush_until(end).unwrap();
            let returned = now();
            pending_spi += scmi_pending();
            assert_eq!(platform_trace::scmi_entries(), entries);
            (window, returned)
        })
        .unwrap();
        let (last_busy, first_idle) = idle_probe::bracket(window, returned);
        assert_ne!(last_busy, 0);
        assert!(first_idle.wrapping_sub(last_busy) <= 12);
        spi_tail.record(returned.wrapping_sub(last_busy) + 1);
        for value in 0..depth as u8 {
            assert_eq!(spi.try_read_word::<u8>().unwrap(), Try::Ready(value));
        }

        let (started, end, returned) = platform::with_scmi_deferred(&sys.pcie, || {
            let entries = platform_trace::scmi_entries();
            let started = now();
            let end = Deadline::after((230 + trial % 71).micros()).unwrap();
            assert_eq!(
                uart.blocking(timer).read_until(&mut [0], end),
                Err(uart::Error::Timeout)
            );
            let returned = now();
            pending_uart += scmi_pending();
            assert_eq!(platform_trace::scmi_entries(), entries);
            (started, end, returned)
        })
        .unwrap();
        let deadline = end.instant().ticks() as u32;
        uart_tail.record(returned.wrapping_sub(deadline));
        scope_us.record(returned.wrapping_sub(started));
        let reply = [trial as u8];
        let end = Deadline::after(3_000.micros()).unwrap();
        let mut stream = uart.blocking(timer);
        stream.write_all_until(&reply, end).unwrap();
        let mut rx = [0];
        stream.read_exact_until(&mut rx, end).unwrap();
        assert_eq!(rx, reply);
    }

    while (now().wrapping_sub(first) as i32) < (TICKS * PERIOD) as i32 {}
    p.irq.TIMER_0.disable();
    let proc1 = PROC1_COUNT.load(Ordering::Acquire);
    assert_eq!(proc1, TICKS);
    let control = unsafe { &*(&raw const CONTROL) };
    control.report("proc0_start");
    log!(
        "PRIO_SAMPLE metric=proc1_start n={proc1} p99_us={} max_us={}\n",
        PROC1_P99.load(Ordering::Relaxed),
        PROC1_MAX.load(Ordering::Relaxed)
    )
    .unwrap();
    spi_tail.report("spi_tail");
    uart_tail.report("uart_overshoot");
    scope_us.report("uart_scope");
    inventory(0, &[19, 26, 28, 42, 53, 57]);
    for irq in [19, 26, 28, 42] {
        log!(
            "PRIO_ISR core=0 irq={irq} n={} max_us={}\n",
            IRQ_COUNT[irq].load(Ordering::Relaxed),
            IRQ_MAX[irq].load(Ordering::Relaxed)
        )
        .unwrap();
    }
    log!(
        "PRIO_ISR core=1 irq=27 n={proc1} max_us={}\n",
        PROC1_IRQ_MAX.load(Ordering::Relaxed)
    )
    .unwrap();
    platform_trace::platform_totals();
    let after = platform::scmi_stats(&sys.pcie).requests;
    log!("PRIO_DONE requests_before={before} requests_after={after} pending_spi={pending_spi} pending_uart={pending_uart}\n").unwrap();
    loop {
        rp1_rt::wait_for_event();
    }
}

#[entry(proc1)]
fn observe(_core: Proc1, mut p: rp1::Proc1Resources) -> ! {
    let first = loop {
        let first = FIRST.load(Ordering::Acquire);
        if first != 0 {
            break first;
        }
        rp1_rt::wait_for_event();
    };
    arm(&mut p.TIMER_ALARM1, first.wrapping_add(PERIOD / 2));
    unsafe { (&raw mut ALARM1).write(MaybeUninit::new(p.TIMER_ALARM1)) };
    p.irq.TIMER_1.set_priority(Priority::control::<2>());
    p.irq.TIMER_1.enable();
    inventory(1, &[27]);
    idle_probe::observe()
}
