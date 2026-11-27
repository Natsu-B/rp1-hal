//! Proc0 declaration: `timer = ["ALARM0"]`.
//! No wiring. ALARM0 fires TIMER_0 three times, and main checks the ISR timestamps.

use crate::rp1;
use core::sync::atomic::{AtomicU32, Ordering::Relaxed};
use rp1_hal::{
    fugit::ExtU32,
    timer::{Alarm, Timer},
};
use rp1_rt::{interrupt, interrupt::Slot};

// These statics live in core-local DSRAM. Only the handler writes the count and timestamp.
static ALARM: Slot<Option<Alarm<0>>, 0> = Slot::empty();
static TICKS: AtomicU32 = AtomicU32::new(0);
static FIRED_AT: AtomicU32 = AtomicU32::new(0);

#[interrupt(proc0)]
fn TIMER_0() {
    ALARM.with(|alarm| {
        // Acknowledge at the peripheral to deassert the NVIC line.
        alarm.clear_interrupt();
        FIRED_AT.store(Timer::new().now_32().ticks(), Relaxed);
        TICKS.store(TICKS.load(Relaxed) + 1, Relaxed);
    });
}

/// Pass the generated `irq.TIMER_0` line for ALARM0.
pub fn run(mut alarm: Alarm<0>, mut irq: rp1::proc0_irq::TIMER_0) -> Result<(), &'static str> {
    alarm.clear_interrupt();
    irq.unpend();
    alarm.enable_interrupt();
    ALARM.put(alarm).ok().unwrap();
    irq.enable();

    let result = sample_ticks();
    irq.disable();
    let mut alarm = ALARM.take().unwrap();
    alarm.disable_interrupt();
    alarm.cancel();
    alarm.clear_interrupt();
    irq.unpend();
    result
}

fn sample_ticks() -> Result<(), &'static str> {
    let timer = Timer::new();
    let mut deadline = timer.now_32();
    let mut previous = deadline.ticks();
    for count in 1..=3 {
        deadline += 20.millis();
        ALARM
            .with(|alarm| alarm.schedule(deadline))
            .unwrap()
            .map_err(|_| "could not arm ALARM0")?;
        // Bound the wait so the next demo still runs if the IRQ is never delivered.
        while TICKS.load(Relaxed) < count {
            if timer.now_32() >= deadline + 10.millis() {
                return Err("TIMER_0 delivery timeout");
            }
        }
        let fired = FIRED_AT.load(Relaxed);
        // The timer runs at 1 MHz, so 18,000–22,000 ticks is 20 ms ± 2 ms.
        if TICKS.load(Relaxed) != count
            || !(18_000..=22_000).contains(&fired.wrapping_sub(previous))
        {
            return Err("TIMER_0 count or interval out of range");
        }
        previous = fired;
    }
    Ok(())
}
