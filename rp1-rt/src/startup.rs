use core::ptr::{addr_of, addr_of_mut};
core::arch::global_asm!(
    r#"
    .syntax unified
    .thumb
    .section .text.rp1_reset,"ax",%progbits
    .global __rp1_reset0
    .type __rp1_reset0,%function
    .thumb_func
__rp1_reset0:
    cpsid i
    ldr r0, =0x10004000
    msr msp, r0
    movs r0, #0
    b __rp1_setup_exceptions
    .size __rp1_reset0, . - __rp1_reset0
    .global __rp1_reset1
    .type __rp1_reset1,%function
    .thumb_func
__rp1_reset1:
    cpsid i
    ldr r0, =0x10004000
    msr msp, r0
    movs r0, #1
    b __rp1_setup_exceptions
    .size __rp1_reset1, . - __rp1_reset1

    .type __rp1_setup_exceptions,%function
    .thumb_func
__rp1_setup_exceptions:
    bl __rp1_initialize
    // r0 is the selected main; keep it intact through exception setup.
    ldr r1, =0xe000ed0c
    ldr r2, =0x05fa0000
    str r2, [r1]       // AIRCR: group all implemented priority bits for preemption.
    ldr r2, [r1, #8]
    orr r2, r2, #(1 << 9)
    str r2, [r1, #8]   // CCR.STKALIGN: preserve the Rust stack alignment at exception entry.
    movs r2, #0
    msr BASEPRI, r2
    cpsie f
    dsb sy
    isb sy
    movs r2, #32
    strb r2, [r1, #12]  // SHPR1: MemManage, BusFault and UsageFault at priority 1.
    strb r2, [r1, #13]
    strb r2, [r1, #14]
    ldr r2, [r1, #24]
    orr r2, r2, #(7 << 16)
    str r2, [r1, #24]   // SHCSR: enable the three configurable faults.
    dsb sy
    isb sy
    cpsie i
    bx r0
    .size __rp1_setup_exceptions, . - __rp1_setup_exceptions
    .ltorg

    .macro system_exceptions
    .word __rp1_default
    .word __rp1_hardfault
    .rept 3
    .word __rp1_fault
    .endr
    .rept 9
    .word __rp1_default
    .endr
    .endm

    .section .vectors.proc0,"a",%progbits
    .word 0x10004000
    .word __rp1_reset0
    system_exceptions
    .section .vectors.proc1,"a",%progbits
    .word 0x10004000
    .word __rp1_reset1
    system_exceptions
"#
);

unsafe extern "C" {
    static __siisram: u32;
    static mut __sisram: u32;
    static mut __eisram: u32;
    static __sidata: u32;
    static mut __sdata: u32;
    static mut __edata: u32;
    static mut __sbss: u32;
    static mut __ebss: u32;
    static __vectors0: u32;
    static __vectors1: u32;
    fn __rp1_proc0_main() -> !;
    fn __rp1_proc1_main() -> !;
}

#[unsafe(no_mangle)]
unsafe extern "C" fn __rp1_initialize(core: u32) -> unsafe extern "C" fn() -> ! {
    unsafe {
        let start = addr_of_mut!(__sisram);
        let len = (addr_of!(__eisram) as usize - start as usize) / 4;
        core::ptr::copy_nonoverlapping(addr_of!(__siisram), start, len);
        let start = addr_of_mut!(__sdata);
        let len = (addr_of!(__edata) as usize - start as usize) / 4;
        core::ptr::copy_nonoverlapping(addr_of!(__sidata), start, len);
        let start = addr_of_mut!(__sbss);
        let len = (addr_of!(__ebss) as usize - start as usize) / 4;
        core::ptr::write_bytes(start, 0, len);
        let vectors = if core == 0 {
            addr_of!(__vectors0)
        } else {
            addr_of!(__vectors1)
        };
        (0xe000_ed08 as *mut u32).write_volatile(vectors as u32);
        core::arch::asm!("dsb sy", "isb sy", options(nostack, preserves_flags));
        crate::interrupt::reset_all_lines();
        crate::status::start(core);
        if core == 0 {
            __rp1_proc0_main
        } else {
            __rp1_proc1_main
        }
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn __rp1_default() -> ! {
    loop {
        crate::wait_for_event();
    }
}
