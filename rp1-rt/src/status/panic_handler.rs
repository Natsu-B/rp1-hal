trait Registers {
    fn write_basepri(&mut self, mask: u32);
    fn enable_interrupts(&mut self);
    fn wait(&mut self) -> !;
}

// Avoid an extra frame on the interrupt stack when a handler panics.
#[inline(always)]
fn handle(regs: &mut impl Registers, record: impl FnOnce()) -> ! {
    record();
    // Open priority 1 even if the caller had set BASEPRI to 0x20.
    regs.write_basepri(2 << 5);
    regs.enable_interrupts();
    regs.wait()
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
mod hardware {
    use super::Registers;
    use core::{arch::asm, panic::PanicInfo};

    struct CortexM;

    impl Registers for CortexM {
        fn write_basepri(&mut self, mask: u32) {
            unsafe { asm!("msr BASEPRI, {}", in(reg) mask, options(nostack, preserves_flags)) };
        }

        fn enable_interrupts(&mut self) {
            unsafe { asm!("cpsie i", options(nostack, preserves_flags)) };
        }

        fn wait(&mut self) -> ! {
            loop {
                unsafe { asm!("wfi", options(nomem, nostack, preserves_flags)) };
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

    use super::Registers;
    use crate::status::{
        CoreState, finish, record_panic_details, s, start,
        tests::{PAGE_LOCK, file, read},
    };
    use std::panic::{AssertUnwindSafe, catch_unwind};

    struct Fake {
        expected_words: [u32; 4],
        basepri: u32,
        primask: bool,
    }

    fn words() -> [u32; 4] {
        [s::STATE, s::REASON, s::LINE, s::AUX].map(|offset| read(0, offset))
    }

    impl Registers for Fake {
        fn write_basepri(&mut self, mask: u32) {
            assert_eq!(words(), self.expected_words);
            assert_eq!(self.basepri, 0x20);
            assert!(self.primask);
            self.basepri = mask;
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
                finish(0, CoreState::Fault, 6, 42, 7, "first.rs");
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
