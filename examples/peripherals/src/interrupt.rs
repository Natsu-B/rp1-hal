//! Proc0 declaration: `timer = ["ALARM0"]`.
//! No wiring. ALARM0 fires TIMER_0 three times, and main checks the ISR timestamps.

use crate::rp1::proc0_irq::TIMER_0;
use Ordering::{Acquire, Relaxed, Release};
use core::sync::atomic::{AtomicU32, Ordering};
use rp1_hal::{
    fugit::ExtU32,
    timer::{Alarm, Timer, WaitTimer},
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
        TICKS.store(TICKS.load(Relaxed) + 1, Release);
        rp1_rt::send_event();
    });
}

/// Pass the generated `irq.TIMER_0` line for ALARM0.
pub fn run(
    mut alarm: Alarm<0>,
    mut irq: TIMER_0,
    wait: &mut WaitTimer,
) -> Result<(), &'static str> {
    alarm.clear_interrupt();
    irq.unpend();
    alarm.enable_interrupt();
    ALARM.put(alarm).ok().unwrap();
    irq.enable();

    let result = sample_ticks(wait, &irq);
    irq.disable();
    let mut alarm = ALARM.take().unwrap();
    alarm.disable_interrupt();
    alarm.cancel();
    alarm.clear_interrupt();
    irq.unpend();
    result
}

fn sample_ticks(wait: &mut WaitTimer, irq: &TIMER_0) -> Result<(), &'static str> {
    let timer = Timer::new();
    let mut previous = timer.now_32().ticks();
    let mut deadline = timer.now();
    for count in 1..=3 {
        deadline += 20.millis();
        ALARM
            .with(|alarm| alarm.try_arm(deadline))
            .unwrap()
            .map_err(|_| "could not arm ALARM0")?;
        // The wait timer bounds the wait so the next demo still runs if TIMER_0 is never delivered.
        let fired = wait
            .wait_irq_until(irq, deadline + 10.millis(), || {
                (TICKS.load(Acquire) >= count).then(|| FIRED_AT.load(Relaxed))
            })
            .map_err(|_| "TIMER_0 wait failed")?;
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
