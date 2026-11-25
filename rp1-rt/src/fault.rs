const STACK_ERRORS: u32 = (1 << 4) | (1 << 3) | (1 << 12) | (1 << 11);
const INVPC: u32 = 1 << 18;

trait Registers {
    fn cfsr(&self) -> u32;
    fn ipsr(&self) -> u32;
    fn read_frame(&self, index: usize) -> u32;
    fn write_frame(&self, index: usize, value: u32);
    fn record(&self, exception: u32, pc: u32, cfsr: u32);
    fn clear_cfsr(&self, bits: u32);
    fn write_basepri(&self, mask: u32);
    fn sync(&self);
    fn stop(&self) -> !;
}

fn configurable_fault(regs: &impl Registers, park: u32) {
    let cfsr = regs.cfsr();
    if cfsr & (STACK_ERRORS | INVPC) != 0 {
        terminal(regs, cfsr);
    }
    let pc = regs.read_frame(6);
    regs.record(regs.ipsr(), pc, cfsr);
    regs.clear_cfsr(cfsr);
    regs.write_frame(6, park & !1);
    regs.write_frame(7, (regs.read_frame(7) & 0x3ff) | (1 << 24));
    regs.write_basepri(2 << 5);
    regs.sync();
}

fn psp_fault(regs: &impl Registers) -> ! {
    terminal(regs, regs.cfsr())
}

fn terminal(regs: &impl Registers, cfsr: u32) -> ! {
    regs.record(regs.ipsr(), 0, cfsr);
    regs.stop()
}

#[inline(always)]
fn hardfault(regs: &impl Registers, exc_return: u32) -> ! {
    let cfsr = regs.cfsr();
    let pc = if cfsr & STACK_ERRORS == 0 && exc_return & 4 == 0 {
        regs.read_frame(6)
    } else {
        0
    };
    regs.record(3, pc, cfsr);
    regs.stop()
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
mod hardware {
    use super::Registers;
    use core::arch::{asm, global_asm, naked_asm};

    global_asm!(
        r#"
    .syntax unified
    .thumb
    .section .text.__rp1_fault,"ax",%progbits
    .global __rp1_fault
    .type __rp1_fault,%function
    .thumb_func
__rp1_fault:
    tst lr, #4
    bne __rp1_fault_terminal
    mov r0, sp
    push {{r0, lr}}
    bl rp1_fault_rust
    pop {{r0, lr}}
    bx lr
    .size __rp1_fault, . - __rp1_fault

    .type __rp1_fault_terminal,%function
    .thumb_func
__rp1_fault_terminal:
    b rp1_fault_terminal
    .size __rp1_fault_terminal, . - __rp1_fault_terminal

    .section .text.__rp1_hardfault,"ax",%progbits
    .global __rp1_hardfault
    .type __rp1_hardfault,%function
    .thumb_func
__rp1_hardfault:
    mov r0, sp
    mov r1, lr
    b rp1_hardfault
    .size __rp1_hardfault, . - __rp1_hardfault
"#
    );

    const CFSR: *mut u32 = 0xe000_ed28 as *mut u32;

    // The frame pointer may be invalid until CFSR and EXC_RETURN have been checked.
    struct Fault(*mut [u32; 8]);

    impl Registers for Fault {
        fn cfsr(&self) -> u32 {
            unsafe { CFSR.read_volatile() }
        }

        fn ipsr(&self) -> u32 {
            let ipsr;
            unsafe { asm!("mrs {}, IPSR", out(reg) ipsr, options(nostack, preserves_flags)) };
            ipsr
        }

        fn read_frame(&self, index: usize) -> u32 {
            unsafe { self.0.cast::<u32>().add(index).read_volatile() }
        }

        fn write_frame(&self, index: usize, value: u32) {
            unsafe { self.0.cast::<u32>().add(index).write_volatile(value) };
        }

        fn record(&self, exception: u32, pc: u32, cfsr: u32) {
            crate::status::record_fault(exception, pc, cfsr);
        }

        fn clear_cfsr(&self, bits: u32) {
            unsafe { CFSR.write_volatile(bits) };
        }

        fn write_basepri(&self, mask: u32) {
            unsafe { asm!("msr BASEPRI, {}", in(reg) mask, options(nostack, preserves_flags)) };
        }

        fn sync(&self) {
            unsafe { asm!("dsb sy", "isb sy", options(nostack, preserves_flags)) };
        }

        fn stop(&self) -> ! {
            loop {
                crate::wait_for_event();
            }
        }
    }

    #[unsafe(no_mangle)]
    extern "C" fn rp1_fault_rust(frame: *mut [u32; 8]) {
        super::configurable_fault(&Fault(frame), park as *const () as u32);
    }

    #[unsafe(no_mangle)]
    extern "C" fn rp1_fault_terminal() -> ! {
        super::psp_fault(&Fault(core::ptr::null_mut()))
    }

    #[unsafe(no_mangle)]
    extern "C" fn rp1_hardfault(frame: *mut [u32; 8], exc_return: u32) -> ! {
        super::hardfault(&Fault(frame), exc_return)
    }

    // Exception return restores the interrupted context's SP, so park uses no stack.
    #[unsafe(naked)]
    extern "C" fn park() -> ! {
        naked_asm!("2:", "wfi", "b 2b");
    }
}

#[cfg(test)]
mod tests;
