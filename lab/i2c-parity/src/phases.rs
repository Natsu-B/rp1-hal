//! Sample controller and pad transitions during the IRQ-backed transaction.
use super::{N, Registers, SCL, Timer, levels, log, pac, rp1};
use core::sync::atomic::{
    AtomicU32,
    Ordering::{Acquire, Relaxed, Release},
};
use rp1_rt::{send_event, wait_for_event};

#[unsafe(link_section = ".shared")]
static REQUEST: AtomicU32 = AtomicU32::new(0);
#[unsafe(link_section = ".shared")]
static STATE: AtomicU32 = AtomicU32::new(0);
#[unsafe(link_section = ".shared")]
static STAMPS: [AtomicU32; 8] = [const { AtomicU32::new(0) }; 8];

fn now() -> u32 {
    Timer::new().now_32().ticks()
}

fn wait_for(state: u32) {
    let start = now();
    while STATE.load(Acquire) != state {
        assert!(
            now().wrapping_sub(start) < 10_000,
            "phase sampler state {state}"
        );
    }
}

pub(super) fn arm() {
    REQUEST.store(1, Release);
    send_event();
    wait_for(1);
}

pub(super) fn report(phase: &str, start: u32, irq: u32, returned: u32) {
    wait_for(2);
    let [
        enable,
        enabled,
        first_low,
        ninth_rise,
        disable,
        disabled,
        edges,
        periods,
    ] = STAMPS.each_ref().map(|stamp| stamp.load(Relaxed));
    REQUEST.store(0, Release);
    wait_for(0);
    let from_start = |tick: u32| tick.wrapping_sub(start);
    let (lo, hi) = (periods & 0xffff, periods >> 16);
    log!(
        "I2C_NACK_PHASE instance={N} phase={phase} enable_us={} enabled_us={} first_scl_low_us={} ninth_rise_us={} irq_us={irq} disable_us={} disabled_us={} return_us={returned} rising_edges={edges} scl_period_us={lo}..{hi}\n",
        from_start(enable), from_start(enabled), from_start(first_low), from_start(ninth_rise),
        from_start(disable), from_start(disabled)
    )
    .unwrap();
    assert!((9..=10).contains(&edges) && (8..=20).contains(&lo) && (lo..=20).contains(&hi));
}

#[rp1_rt::isram]
pub(super) fn observe() -> ! {
    // These registers have no read side effects. Proc0 owns all controller writes.
    let block = unsafe { &*Registers::PTR.cast::<pac::dw_i2c::RegisterBlock>() };
    loop {
        if REQUEST.load(Acquire) == 0 {
            wait_for_event();
            continue;
        }
        STATE.store(1, Release);
        while block.enable.read().bits() & 1 == 0 {}
        let enable = now();
        while block.enable_status.read().bits() & 1 == 0 {}
        let enabled = now();
        let (mut first_low, mut nth_rise, mut previous_rise) = (0, 0, 0);
        let (mut edges, mut lo, mut hi) = (0, u32::MAX, 0);
        let mut high = true;
        while block.enable.read().bits() & 1 != 0 {
            let level = levels() & SCL != 0;
            if level && !high {
                let tick = now();
                edges += 1;
                if (2..=9).contains(&edges) {
                    let period = tick.wrapping_sub(previous_rise);
                    lo = lo.min(period);
                    hi = hi.max(period);
                }
                if edges == 9 {
                    nth_rise = tick;
                }
                previous_rise = tick;
            } else if !level && first_low == 0 {
                first_low = now();
            }
            high = level;
        }
        let disable = now();
        while block.enable_status.read().bits() & 1 != 0 {}
        let disabled = now();
        for (slot, value) in STAMPS.iter().zip([
            enable,
            enabled,
            first_low,
            nth_rise,
            disable,
            disabled,
            edges,
            (hi << 16) | (lo & 0xffff),
        ]) {
            slot.store(value, Relaxed);
        }
        STATE.store(2, Release);
        while REQUEST.load(Acquire) != 0 {
            core::hint::spin_loop();
        }
        STATE.store(0, Release);
    }
}
