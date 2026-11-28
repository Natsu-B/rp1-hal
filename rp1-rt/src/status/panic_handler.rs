trait PanicCpu {
    fn mask_for_park(&mut self);
    fn enable_interrupts(&mut self);
    fn wait(&mut self) -> !;
}

// Avoid an extra frame on the interrupt stack when a handler panics.
#[inline(always)]
fn handle(regs: &mut impl PanicCpu, record: impl FnOnce()) -> ! {
    record();
    regs.mask_for_park();
    regs.enable_interrupts();
    regs.wait()
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
mod hardware {
    use super::PanicCpu;
    use crate::cpu;
    use core::panic::PanicInfo;

    struct CortexM;

    impl PanicCpu for CortexM {
        fn mask_for_park(&mut self) {
            crate::interrupt::mask_for_park();
        }

        fn enable_interrupts(&mut self) {
            cpu::enable_interrupts();
        }

        fn wait(&mut self) -> ! {
            loop {
                cpu::wfi();
            }
        }
    }

    #[panic_handler]
    fn panic(info: &PanicInfo<'_>) -> ! {
        super::handle(&mut CortexM, || crate::status::record_panic(info))
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::PanicCpu;
    use crate::status::{
        CoreState, finish, record_panic_details, s, start,
        tests::{PAGE_LOCK, file, read},
        write_file,
    };
    use std::panic::{AssertUnwindSafe, catch_unwind};

    struct Fake {
        expected_words: [u32; 4],
        basepri: u8,
        primask: bool,
    }

    fn words() -> [u32; 4] {
        [s::STATE, s::REASON, s::LINE, s::AUX].map(|offset| read(0, offset))
    }

    impl PanicCpu for Fake {
        fn mask_for_park(&mut self) {
            assert_eq!(words(), self.expected_words);
            assert_eq!(self.basepri, 0x20);
            assert!(self.primask);
            self.basepri = 0x40;
        }

        fn enable_interrupts(&mut self) {
            assert_eq!(self.basepri, 0x40);
            assert!(self.primask);
            self.primask = false;
        }

        fn wait(&mut self) -> ! {
            assert!(!self.primask);
            panic!("parked");
        }
    }

    #[test]
    fn records_before_unmasking_and_preserves_a_previous_terminal_record() {
        let _page = PAGE_LOCK.lock().unwrap();
        for (expected_words, expected_file) in [
            ([CoreState::Panicked.word(), 0, 17, 0], "panic.rs"),
            ([CoreState::Fault.word(), 6, 42, 7], "first.rs"),
        ] {
            start(0);
            if expected_words[0] == CoreState::Fault.word() {
                finish(0, CoreState::Fault, 6, 42, 7, |core| {
                    write_file(core, "first.rs")
                });
            }
            let mut regs = Fake {
                expected_words,
                basepri: 0x20,
                primask: true,
            };
            let parked = catch_unwind(AssertUnwindSafe(|| {
                super::handle(&mut regs, || record_panic_details("panic.rs", 17))
            }))
            .unwrap_err();
            assert_eq!(parked.downcast_ref::<&str>(), Some(&"parked"));
            assert_eq!(words(), expected_words);
            assert_eq!(file(0), expected_file);
        }
    }
}
