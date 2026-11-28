use crate::rp1;
use core::sync::atomic::{AtomicU32, Ordering};
use rp1_hal::timer::{Alarm, Instant32, Timer};
use rp1_rt::interrupt::Slot;

const PERIOD_US: u32 = 1_000;
const PERIODS: u32 = 60_000;
const MAX_BOUND_US: usize = 100;

struct Request(AtomicU32);
unsafe impl rp1_proto::SharedSafe for Request {}
#[rp1_rt::shared]
static REQUEST: Request = Request(AtomicU32::new(0));

struct Samples {
    next: u32,
    mode: u32,
    count: u32,
    miss: u32,
    max: u32,
    histogram: [u32; MAX_BOUND_US + 2],
    done: bool,
}

static SAMPLES: Slot<Option<Samples>, 0> = Slot::empty();

pub fn request(mode: u32) {
    REQUEST.0.store(mode, Ordering::Release);
    rp1_rt::send_event();
}

#[rp1_rt::cold]
pub fn tick(alarm: &mut Alarm<0>) -> bool {
    SAMPLES
        .with(|samples| {
            if samples.done {
                return false;
            }
            let now = Timer::new().now_32().ticks();
            let late = now.wrapping_sub(samples.next);
            let skipped = late / PERIOD_US;
            samples.max = samples.max.max(late);
            samples.count += 1;
            samples.miss += skipped;
            samples.histogram[(late as usize).min(MAX_BOUND_US + 1)] += 1;

            if samples.count + samples.miss >= PERIODS {
                samples.done = true;
            } else {
                samples.next = samples.next.wrapping_add((skipped + 1) * PERIOD_US);
                if alarm.try_arm(Instant32::from_ticks(samples.next)).is_err() {
                    samples.miss += 1;
                    samples.done = true;
                }
            }
            if samples.done {
                // The thread reports after the sample window.
                rp1_rt::send_event();
                false
            } else {
                true
            }
        })
        .unwrap_or(false)
}

#[rp1_rt::cold]
pub fn service(last: &mut u32) {
    let report = SAMPLES
        .try_with(|slot| {
            let samples = slot.as_ref().filter(|samples| samples.done)?;
            let rank = (samples.count * 99).div_ceil(100);
            let mut hits = 0;
            let p99 = samples
                .histogram
                .iter()
                .position(|n| {
                    hits += n;
                    hits >= rank
                })
                .unwrap();
            let report = (samples.mode, samples.count, samples.miss, p99, samples.max);
            *slot = None;
            Some(report)
        })
        .flatten();
    if let Some((mode, count, miss, p99, max)) = report {
        let _ = rp1_hal::log!(
            "LWC_CONTROL mode={mode} n={count} miss={miss} p99_us={p99} max_us={max}\n"
        );
    }
    let mode = REQUEST.0.load(Ordering::Acquire);
    if mode == *last {
        return;
    }
    *last = mode;
    crate::ALARM0.with(|alarm| {
        alarm.cancel();
        alarm.clear_interrupt();
        let next = Timer::new().now_32().ticks().wrapping_add(PERIOD_US);
        SAMPLES
            .try_with(|slot| {
                assert!(slot.is_none());
                *slot = Some(Samples {
                    next,
                    mode,
                    count: 0,
                    miss: 0,
                    max: 0,
                    histogram: [0; MAX_BOUND_US + 2],
                    done: false,
                });
            })
            .unwrap();
        alarm.try_arm(Instant32::from_ticks(next)).unwrap();
    });
}
