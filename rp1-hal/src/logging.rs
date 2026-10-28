use crate::{Error, poll, uart::Uart, write};
use core::{
    fmt::{self, Write},
    sync::atomic::{AtomicU32, Ordering},
};

/// Single-writer-per-word Peterson lock: all accesses compile to loads/stores, never RMW.
#[repr(C)]
pub struct Logger {
    want: [AtomicU32; 2],
    turn: AtomicU32,
    ready: AtomicU32,
}
unsafe impl rp1_proto::SharedSafe for Logger {}
impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}
impl Logger {
    pub const fn new() -> Self {
        Self {
            want: [const { AtomicU32::new(0) }; 2],
            turn: AtomicU32::new(0),
            ready: AtomicU32::new(0),
        }
    }
    /// Called by proc0 before launching proc1. Owns UART0 and pins 14/15 for the image lifetime.
    pub fn init(&'static self, _core: &mut rp1_rt::Proc0, baud: u32) -> Result<(), Error> {
        if self.ready.load(Ordering::Acquire) != 0 {
            return Err(Error::Precondition);
        }
        // The 115200-class divisors assume clk_uart = 50 MHz XOSC; refuse anything else.
        if !crate::clocks::uart_clock_is_50mhz() {
            return Err(Error::Precondition);
        }
        write(0x4001_7004, 1 << 19);
        for pin in [14, 15] {
            write(0x400d_0004 + pin * 8, 4);
            write(0x400f_0004 + pin * 4, 0x46);
        }
        write(0x4001_7004, 1 << 26);
        poll(0x4001_401c, 1 << 26, 1 << 26)?;
        let _uart = unsafe { Uart::<0>::new(50_000_000, baud)? };
        self.ready.store(1, Ordering::Release);
        Ok(())
    }
    pub fn write(&self, args: fmt::Arguments<'_>) -> fmt::Result {
        if self.ready.load(Ordering::Acquire) == 0 {
            return Err(fmt::Error);
        }
        #[cfg(target_arch = "arm")]
        let core = (crate::read(0xe00f_f01c) & 1) as usize;
        #[cfg(not(target_arch = "arm"))]
        let core = 0usize;
        #[cfg(target_arch = "arm")]
        let primask: u32;
        #[cfg(target_arch = "arm")]
        unsafe {
            core::arch::asm!("mrs {}, primask", "cpsid i", out(reg) primask, options(nostack, preserves_flags));
        }
        self.want[core].store(1, Ordering::Release);
        self.turn.store((1 - core) as u32, Ordering::Release);
        rp1_proto::barrier();
        while self.want[1 - core].load(Ordering::Acquire) != 0
            && self.turn.load(Ordering::Acquire) == (1 - core) as u32
        {
            core::hint::spin_loop();
        }
        struct Sink;
        impl Write for Sink {
            fn write_str(&mut self, s: &str) -> fmt::Result {
                for b in s.bytes() {
                    poll(0x4003_0018, 1 << 5, 0).map_err(|_| fmt::Error)?;
                    write(0x4003_0000, u32::from(b));
                }
                Ok(())
            }
        }
        let result = Sink.write_fmt(args);
        self.want[core].store(0, Ordering::Release);
        #[cfg(target_arch = "arm")]
        unsafe {
            core::arch::asm!("msr primask, {}", in(reg) primask, options(nostack, preserves_flags));
        }
        result
    }
}
