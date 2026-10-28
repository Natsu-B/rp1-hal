#![no_std]
pub use rp1_pac as pac;
pub mod clocks;
pub mod gpio;
pub mod logging;
pub mod scmi;
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
pub(crate) fn read(address: usize) -> u32 {
    unsafe { (address as *const u32).read_volatile() }
}
pub(crate) fn write(address: usize, value: u32) {
    unsafe {
        (address as *mut u32).write_volatile(value);
    }
    rp1_proto::barrier();
}
pub(crate) fn poll(address: usize, mask: u32, value: u32) -> Result<(), Error> {
    for _ in 0..100_000 {
        if read(address) & mask == value {
            return Ok(());
        }
        core::hint::spin_loop();
    }
    Err(Error::Timeout)
}

/// The calling firmware must declare and initialize the generated `rp1::log` module.
#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => { rp1::log::write(core::format_args!($($arg)*)) };
}
