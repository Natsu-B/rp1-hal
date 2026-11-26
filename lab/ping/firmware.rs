#[path = "commands.rs"]
mod commands;

#[derive(Default)]
pub struct Probe {
    delayed: u32,
}

impl Probe {
    pub fn reply(&mut self, value: u32) -> u32 {
        match value {
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
    let start = timer.now() as u32;
    while (timer.now() as u32).wrapping_sub(start) < 2_000 {
        core::hint::spin_loop();
    }
}
