//! Independent proc1 bracket of the controller's wire-idle bit.
#[cfg(not(feature = "isr-trace"))]
use super::{log, rp1};
use super::{now, spi_status};
use core::sync::atomic::{
    AtomicU32,
    Ordering::{Acquire, Relaxed, Release},
};
use rp1_hal::pac::dw_ssi::sr;
use rp1_rt::{send_event, wait_for_event};

#[unsafe(link_section = ".shared")]
static WINDOW: AtomicU32 = AtomicU32::new(0);
#[unsafe(link_section = ".shared")]
static OBSERVED: AtomicU32 = AtomicU32::new(0);
#[unsafe(link_section = ".shared")]
static LAST_BUSY: AtomicU32 = AtomicU32::new(0);
#[unsafe(link_section = ".shared")]
static FIRST_IDLE: AtomicU32 = AtomicU32::new(0);

pub(super) fn arm(burst_us: u32) -> u32 {
    let window = now().wrapping_add(burst_us - 24);
    WINDOW.store(window, Release);
    send_event();
    window
}

#[rp1_rt::isram]
pub(super) fn observe() -> ! {
    let mut previous = 0;
    loop {
        let window = WINDOW.load(Acquire);
        if window == previous {
            wait_for_event();
            continue;
        }
        previous = window;
        while (now().wrapping_sub(window) as i32) < 0 {
            // Limit APB contention before the sampling window.
            unsafe { core::arch::asm!(".rept 48", "nop", ".endr", options(nostack, nomem)) };
        }
        let mut last_busy = 0;
        loop {
            let stamp = now();
            if spi_status() & (sr::BUSY | sr::TFE) == sr::TFE {
                LAST_BUSY.store(last_busy, Relaxed);
                FIRST_IDLE.store(now(), Relaxed);
                OBSERVED.store(window, Release);
                break;
            }
            last_busy = stamp;
        }
    }
}

pub(super) fn bracket(window: u32, returned: u32) -> (u32, u32) {
    while OBSERVED.load(Acquire) != window {
        assert!(now().wrapping_sub(returned) < 1_000);
    }
    (LAST_BUSY.load(Relaxed), FIRST_IDLE.load(Relaxed))
}

#[cfg(not(feature = "isr-trace"))]
pub(super) fn report(window: u32, returned: u32, hz: u32, trial: u32) {
    let (last_busy, first_idle) = bracket(window, returned);
    assert!(last_busy != 0 && first_idle.wrapping_sub(last_busy) <= 6);
    let upper = returned.wrapping_sub(last_busy) + 1;
    let lower = (returned.wrapping_sub(first_idle) as i32 - 1).max(0);
    log!(
        "OBS_SPI_IDLE hz={} trial={} last_busy={} first_idle={} returned={} lower_us={} upper_us={}\n",
        hz, trial, last_busy, first_idle, returned, lower, upper
    ).unwrap();
}
