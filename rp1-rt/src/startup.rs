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
    b __rp1_initialize
    .global __rp1_reset1
    .type __rp1_reset1,%function
    .thumb_func
__rp1_reset1:
    cpsid i
    ldr r0, =0x10004000
    msr msp, r0
    movs r0, #1
    b __rp1_initialize

    .section .vectors.proc0,"a",%progbits
    .word 0x10004000
    .word __rp1_reset0
    .rept 78
    .word __rp1_default
    .endr
    .section .vectors.proc1,"a",%progbits
    .word 0x10004000
    .word __rp1_reset1
    .rept 78
    .word __rp1_default
    .endr
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
unsafe extern "C" fn __rp1_initialize(core: u32) -> ! {
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
        // IRQs remain masked until the application installs and enables its handlers.
        if core == 0 {
            __rp1_proc0_main()
        } else {
            __rp1_proc1_main()
        }
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn __rp1_default() -> ! {
    loop {
        crate::wait_for_event();
    }
}
