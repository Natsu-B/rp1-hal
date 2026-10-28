//! UART10 (PL011 at 0x10_7d00_1000), already configured by the firmware (`enable_uart=1`).
use core::fmt::{self, Write};

const BASE: usize = 0x10_7d00_1000;
const DR: usize = 0x00;
const FR: usize = 0x18;
const FR_TXFF: u32 = 1 << 5;

fn putc(b: u8) {
    for _ in 0..1_000_000 {
        // SAFETY: fixed PL011 MMIO.
        if unsafe { core::ptr::read_volatile((BASE + FR) as *const u32) } & FR_TXFF == 0 {
            break;
        }
    }
    // SAFETY: as above.
    unsafe { core::ptr::write_volatile((BASE + DR) as *mut u32, u32::from(b)) };
}

struct Uart;
impl Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.bytes() {
            if b == b'\n' {
                putc(b'\r');
            }
            putc(b);
        }
        Ok(())
    }
}

pub fn print(args: fmt::Arguments<'_>) {
    let _ = Uart.write_fmt(args);
}
