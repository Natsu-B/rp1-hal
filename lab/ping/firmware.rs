#[path = "commands.rs"]
mod commands;

pub struct Probe {
    delayed: u32,
    #[cfg(feature = "link-wake")]
    pub host_irq: rp1_link::notify::VectorOwner,
    #[cfg(feature = "link-wake")]
    idle: Option<Idle>,
}

#[cfg(feature = "link-wake")]
struct Idle {
    started: u32,
    slept: u32,
    returns: u32,
    longest: u32,
}

#[cfg(feature = "link-wake")]
use {
    crate::rp1,
    core::sync::atomic::{AtomicBool, Ordering},
    rp1_hal::{
        platform::Pcie,
        timer::{Deadline, WaitTimer},
    },
    rp1_link::notify::VectorOwner,
    rp1_rt::interrupt::{Bank0Irq, Binding},
};
#[cfg(feature = "link-wake")]
struct PanicFlag(AtomicBool);
// Proc1 stores and proc0 loads; neither uses an exclusive access to shared SRAM.
#[cfg(feature = "link-wake")]
unsafe impl rp1_proto::SharedSafe for PanicFlag {}
#[cfg(feature = "link-wake")]
#[rp1_rt::shared]
static PROC0_PANIC: PanicFlag = PanicFlag(AtomicBool::new(false));

#[cfg(feature = "link-wake")]
pub fn check_proc0_panic() {
    if PROC0_PANIC.0.load(Ordering::Acquire) {
        panic!("ping test proc0");
    }
}

impl Probe {
    #[cfg(not(feature = "link-wake"))]
    pub fn new() -> Self {
        Self { delayed: 0 }
    }

    #[cfg(feature = "link-wake")]
    pub fn new(host_irq: rp1_link::notify::VectorOwner) -> Self {
        Self {
            delayed: 0,
            host_irq,
            idle: None,
        }
    }

    #[cfg(feature = "link-wake")]
    pub fn sleep(&mut self) {
        if let Some(idle) = &mut self.idle {
            let before = now();
            rp1_rt::wait_for_event();
            let elapsed = now().wrapping_sub(before);
            idle.slept += elapsed;
            idle.longest = idle.longest.max(elapsed);
            idle.returns += 1;
        } else {
            rp1_rt::wait_for_event();
        }
    }

    pub fn reply(&mut self, value: u32) -> u32 {
        match value {
            #[cfg(feature = "link-wake")]
            commands::STOP => {
                rp1_rt::status::__record_stop_before_park(1);
                self.host_irq.pulse_terminals().expect("stop wake failed");
                rp1_rt::status::stop(1);
            }
            #[cfg(feature = "link-wake")]
            commands::FAULT => {
                pause();
                // A local configurable UsageFault in proc1 thread mode.
                unsafe { core::arch::asm!("udf #0", options(nomem, nostack, noreturn)) }
            }
            #[cfg(feature = "link-wake")]
            commands::PROC0_PANIC => {
                PROC0_PANIC.0.store(true, Ordering::Release);
                rp1_rt::send_event();
                // Leave the request without a reply so Linux observes terminal delivery.
                value
            }
            #[cfg(feature = "link-wake")]
            commands::IDLE_BEGIN => {
                self.idle = Some(Idle {
                    started: now(),
                    slept: 0,
                    returns: 0,
                    longest: 0,
                });
                value
            }
            #[cfg(feature = "link-wake")]
            commands::IDLE_END => {
                if let Some(idle) = self.idle.take() {
                    let _ = rp1_hal::log!(
                        "RP1_IDLE wall_us={} wfe_us={} returns={} longest_us={}\n",
                        now().wrapping_sub(idle.started),
                        idle.slept,
                        idle.returns,
                        idle.longest
                    );
                }
                value
            }
            commands::COUNT => self.delayed,
            commands::PANIC => {
                pause();
                panic!("ping test");
            }
            value if value & commands::DELAY != 0 => {
                pause();
                self.delayed += 1;
                value
            }
            value => value,
        }
    }
}

fn pause() {
    let timer = rp1_hal::timer::Timer::new();
    let start = timer.now_32().ticks();
    while timer.now_32().ticks().wrapping_sub(start) < 2_000 {
        core::hint::spin_loop();
    }
}

#[cfg(feature = "link-wake")]
fn now() -> u32 {
    rp1_hal::timer::Timer::new().now_32().ticks()
}

#[cfg(feature = "link-wake")]
pub fn enable_startup(
    pcie: &mut Pcie,
    mut irq: Binding<Bank0Irq, 0, 0>,
    mut timer: WaitTimer,
    end: Deadline,
) {
    let mut context = timer.thread_context().unwrap();
    if rp1::link::enable(pcie, &mut irq, &mut context, end).is_err() {
        park_after_startup_error();
    }
}

#[cfg(feature = "link-wake")]
pub fn open_and_run(
    link: rp1::link::Proc1LinkInit,
    mut irq: Binding<Bank0Irq, 0, 1>,
    mut timer: WaitTimer,
    end: Deadline,
    owner: VectorOwner,
) -> ! {
    let mut context = timer.thread_context().unwrap();
    match link.open_until(&mut irq, &mut context, end) {
        Ok(link) => run_wake(link, owner),
        Err(_) => park_after_startup_error(),
    }
}

#[cfg(feature = "link-wake")]
fn park_after_startup_error() -> ! {
    // Leave the committed startup diagnosis visible to Linux.
    loop {
        rp1_rt::wait_for_event();
    }
}

#[cfg(feature = "link-wake")]
#[rp1_rt::isram]
fn run_wake(link: rp1::link::Proc1Link, owner: VectorOwner) -> ! {
    use rp1_link::Progress;

    let mut ping = link.ping;
    let mut channel = Some(link.channel);
    let (mut dropped_tx, mut dropped_rx) = link.dropped.split();
    let mut probe = Probe::new(owner);
    let mut pulse_failures = 0;
    loop {
        let progress = ping
            .serve(8, |value| probe.reply(value))
            .expect("ping serve failed");
        let _ = probe.host_irq.pulse_terminals();
        let drop_progress = dropped_rx
            .drain(8, |value| {
                if value == commands::PROC0_PANIC {
                    probe.reply(value);
                } else {
                    dropped_tx.push(&value).expect("drop probe reply");
                    rp1_link::flush_window();
                }
            })
            .expect("drop probe drain");
        if drop_progress == Progress::More {
            continue;
        }
        if let Some(endpoint) = channel.as_mut() {
            match endpoint.respond(8, |nonce| nonce) {
                Ok(Progress::More) => continue,
                Ok(Progress::Idle) => {}
                Err(error) => {
                    let _ = rp1_hal::log!("channel stopped: {error}\n");
                    channel = None;
                }
            }
        }
        let failures = ping.pulse_failures();
        if pulse_failures != failures {
            pulse_failures = failures;
            let _ = rp1_hal::log!("PING_PULSE_FAILURES count={}\n", failures);
            let _ = ping.retry_notification();
        }
        // Endpoints share the doorbell. Check each one before sleeping.
        if progress == Progress::Idle {
            probe.sleep();
        }
    }
}
