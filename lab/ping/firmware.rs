#[path = "commands.rs"]
mod commands;

macro_rules! probe {
    ($resources:ident) => {
        $crate::ping_test::Probe::new(
            #[cfg(feature = "link-wake")]
            $resources.host_irq,
        )
    };
}
pub(super) use probe;

pub struct Probe {
    delayed: u32,
    #[cfg(feature = "link-wake")]
    host_irq: rp1_link::notify::VectorOwner,
}

impl Probe {
    pub fn new(#[cfg(feature = "link-wake")] host_irq: rp1_link::notify::VectorOwner) -> Self {
        Self {
            delayed: 0,
            #[cfg(feature = "link-wake")]
            host_irq,
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
