#![no_std]
pub use rp1_pac as pac;
pub mod adc;
pub mod clocks;
pub mod cold;
pub mod gpio;
pub mod i2c;
pub mod linux_blocks;
pub mod logging;
pub mod scmi;
pub mod shm;
pub mod spi;
pub mod timer;
pub mod uart;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Timeout,
    Invalid,
    Precondition,
    Hardware,
    NotReady,
}
#[inline(always)]
pub(crate) fn read(address: usize) -> u32 {
    unsafe { (address as *const u32).read_volatile() }
}
#[inline(always)]
pub(crate) fn write(address: usize, value: u32) {
    unsafe {
        (address as *mut u32).write_volatile(value);
    }
    rp1_proto::barrier();
}
/// Waits up to 10 ms (raw 1 MHz timer, independent of clk_sys) for `address & mask == value`.
pub(crate) fn poll(address: usize, mask: u32, value: u32) -> Result<(), Error> {
    let start = read(TIMER_RAWL);
    loop {
        if read(address) & mask == value {
            return Ok(());
        }
        if read(TIMER_RAWL).wrapping_sub(start) > POLL_TIMEOUT_US {
            // One more look: the deadline may have passed while this core was stalled.
            return if read(address) & mask == value {
                Ok(())
            } else {
                Err(Error::Timeout)
            };
        }
        core::hint::spin_loop();
    }
}
pub(crate) const TIMER_RAWL: usize = 0x400a_c028;
const POLL_TIMEOUT_US: u32 = 10_000;

/// The calling firmware must declare and initialize the generated `rp1::log` module.
#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => { rp1::log::write(core::format_args!($($arg)*)) };
}
