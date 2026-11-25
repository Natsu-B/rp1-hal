//! One SCMI load window, triggered from Linux through two lab words in the link page.
#![no_std]
#![no_main]

use core::{
    mem::MaybeUninit,
    sync::atomic::{AtomicU32, Ordering::Relaxed},
};
use rp1_hal::{
    log,
    platform::{ScmiState, scmi_state, scmi_stats},
    timer::{Alarm, Timer},
};
use rp1_rt::{Proc0, Proc1, entry, interrupt, interrupt::Priority};

rp1_rt::bind!();

const PERIOD_US: u32 = 10_000;
const HOLD_US: u32 = 2_000;
const WINDOW_US: u32 = 10_000_000;
const START: u32 = u32::from_le_bytes(*b"SCMI");
const TRIGGER: *mut u32 = (rp1_proto::LINK_START + 0x28) as *mut u32;
const ACK: *mut u32 = (rp1_proto::LINK_START + 0x2c) as *mut u32;

static mut ALARM: MaybeUninit<Alarm<0>> = MaybeUninit::uninit();
static NEXT: AtomicU32 = AtomicU32::new(0);
static TICKS: AtomicU32 = AtomicU32::new(0);
static PREEMPTED_HOLDS: AtomicU32 = AtomicU32::new(0);

const IRQ: usize = rp1_rt::pac::Interrupt::PROC_MISC as usize;
const IPR: *mut u8 = (0xe000_e400 + IRQ) as *mut u8;

unsafe extern "C" {
    fn __rp1_p0_PROC_MISC();
}

fn priority_bits() -> u8 {
    // Reset leaves IRQ57 disabled. Restore its priority before platform init.
    unsafe {
        let saved = IPR.read_volatile();
        IPR.write_volatile(u8::MAX);
        let implemented = IPR.read_volatile();
        IPR.write_volatile(saved);
        implemented
    }
}

fn log_init(report: &rp1_hal::platform::InitReport, prio_bits: u8) {
    let (prigroup, vec57_ok, nvic57) = unsafe {
        let prigroup = ((0xe000_ed0c as *const u32).read_volatile() >> 8) & 7;
        let vectors = (0xe000_ed08 as *const u32).read_volatile() as *const usize;
        let handler = __rp1_p0_PROC_MISC as *const () as usize;
        let vec57_ok = vectors.add(16 + IRQ).read_volatile() == handler;
        let iser = (0xe000_e100 + 4 * (IRQ / 32)) as *const u32;
        let nvic57 = iser.read_volatile() & (1 << (IRQ % 32)) != 0;
        (prigroup, vec57_ok, nvic57)
    };
    log!(
        "PLAT_INIT ok=1 {} prigroup={} prio_bits=0x{:x} vec57_ok={} nvic57={}\n",
        report,
        prigroup,
        prio_bits,
        u8::from(vec57_ok),
        u8::from(nvic57)
    )
    .unwrap();
}

fn now() -> u32 {
    Timer::new().now() as u32
}

#[interrupt(proc0)]
fn TIMER_0() {
    // Proc0 installs the alarm before enabling this line and hands it to this ISR.
    let alarm = &raw mut ALARM;
    let alarm = unsafe { (*alarm).assume_init_mut() };
    alarm.clear_interrupt();
    let next = NEXT.load(Relaxed).wrapping_add(PERIOD_US);
    NEXT.store(next, Relaxed);
    alarm.schedule(next).unwrap();
    let started = now();
    let before = scmi_state();
    while now().wrapping_sub(started) < HOLD_US {
        core::hint::spin_loop();
    }
    if let (ScmiState::Serving { requests: before }, ScmiState::Serving { requests: after }) =
        (before, scmi_state())
        && after > before
    {
        PREEMPTED_HOLDS.store(PREEMPTED_HOLDS.load(Relaxed) + 1, Relaxed);
    }
    TICKS.store(TICKS.load(Relaxed) + 1, Relaxed);
}

#[entry(proc0)]
fn main0(mut core: Proc0, p: rp1::Proc0Resources) -> ! {
    let prio_bits = priority_bits();
    let sys = rp1::platform::init(&mut core, p.platform).unwrap_or_else(|_| {
        let _ = log!("PLAT_INIT ok=0\n");
        panic!("platform init failed")
    });
    log_init(&sys.report, prio_bits);
    unsafe {
        TRIGGER.write_volatile(0);
        ACK.write_volatile(0);
    }
    rp1_proto::barrier();
    log!("SCMI_STARVE_READY\n").unwrap();
    while unsafe { TRIGGER.read_volatile() } != START {
        core::hint::spin_loop();
    }

    log!(
        "SCMI_STARVE_BEGIN window_us={} period_us={} hold_us={} priority=2\n",
        WINDOW_US,
        PERIOD_US,
        HOLD_US
    )
    .unwrap();
    let before = scmi_stats(&sys.pcie);
    let mut line = p.irq.TIMER_0;
    let mut alarm = p.TIMER_ALARM0;
    alarm.clear_interrupt();
    alarm.enable_interrupt();
    line.set_priority(Priority::new(2).unwrap());
    let start = now();
    // Half a period at each end leaves time for the first and last full ISR holds.
    let first = start.wrapping_add(PERIOD_US / 2);
    NEXT.store(first, Relaxed);
    alarm.schedule(first).unwrap();
    unsafe { (&raw mut ALARM).write(MaybeUninit::new(alarm)) };
    line.enable();
    unsafe { ACK.write_volatile(START) };
    rp1_proto::barrier();

    let elapsed = loop {
        let elapsed = now().wrapping_sub(start);
        if elapsed >= WINDOW_US {
            break elapsed;
        }
        core::hint::spin_loop();
    };
    line.disable();
    let after = scmi_stats(&sys.pcie);
    let serving = matches!(scmi_state(), ScmiState::Serving { .. });
    log!(
        "SCMI_STARVE_END elapsed_us={} ticks={} preempted_holds={} requests_before={} requests_after={} duplicates={} spurious57={} max_serve_all_us={} clean_n={} max_serve_clean_us={} serving={}\n",
        elapsed,
        TICKS.load(Relaxed),
        PREEMPTED_HOLDS.load(Relaxed),
        before.requests,
        after.requests,
        after.duplicates,
        after.spurious57,
        after.max_serve_all_us,
        after.clean_n,
        after.max_serve_clean_us,
        u8::from(serving)
    )
    .unwrap();
    loop {
        rp1_rt::wait_for_event();
    }
}

#[entry(proc1)]
fn main1(_core: Proc1, _p: rp1::Proc1Resources) -> ! {
    loop {
        rp1_rt::wait_for_event();
    }
}
