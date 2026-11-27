#![no_std]
pub use fugit;
pub use rp1_pac as pac;

/// Offsets of registers in a generated PAC block, for tests that tie a driver's own address
/// constants to the PAC. The block is a zeroed one on the host: a reference to the device
/// address would point at unmapped memory, which is undefined even if it is never read.
#[cfg(test)]
macro_rules! pac_offsets {
    ($module:ident: $($register:ident),+ $(,)?) => {{
        // A register block is integers only, so all zeroes is a valid value.
        let block =
            unsafe { core::mem::zeroed::<crate::pac::generated::$module::RegisterBlock>() };
        let base = &raw const block as usize;
        [$(block.$register() as *const _ as usize - base),+]
    }};
}

// Keep the pin tuple private so adding an optional signal does not change the required type.
macro_rules! pin_builder {
    ($pin:ident, []) => {
        /// The selected routes for one peripheral instance.
        pub struct Pins<const N: u8, P = ()>(P);

        impl<const N: u8, P> Pins<N, P> {
            /// Return the selected pins in signal order.
            pub fn into_pins(self) -> P {
                self.0
            }
        }

        impl<const N: u8> Pins<N> {
            /// Start an empty route set. Add the signals in order (UART: tx, then rx;
            /// I²C: sda, then scl; SPI: sck, then mosi, then miso).
            /// Each call accepts only a pin routed to that signal of instance N.
            pub const fn builder() -> Self {
                Self(())
            }
        }
    };
    ($pin:ident, [$($arg:ident: $num:ident),*], $next:ident: $new:ident => $route:ident) => {
        impl<const N: u8, $(const $num: u8,)*> Pins<N, ($($pin<$num>,)*)> {
            /// Add the pin routed to this signal.
            pub fn $next<const $new: u8>(self, $next: $pin<$new>) -> Pins<N, ($($pin<$num>,)* $pin<$new>,)>
            where
                $pin<$new>: $route<N>,
            {
                let ($($arg,)*) = self.0;
                Pins(($($arg,)* $next,))
            }
        }
    };
}

pub mod adc;
pub mod clocks;
pub mod cold;
pub mod dma;
pub mod gpio;
pub mod i2c;
pub mod linux_blocks;
pub mod logging;
mod pinmux;
pub mod platform;
pub mod pwm;
pub mod scmi;
pub mod spi;
pub mod timer;
pub mod uart;

/// A driver's `init` that failed: the driver's error, and what `init` consumed, unchanged,
/// for a retry. uart, i2c and spi return the [`Parts`](uart::Parts) bundle their `free` returns
/// (token and pins), pwm, adc and dma their token. `Debug` prints only the error; `?` converts
/// it to the driver's error, dropping the resources (nothing is running yet).
///
/// ```
/// struct Token;
/// let e = rp1_hal::InitError { error: rp1_hal::uart::Error::Invalid, resources: Token };
/// assert_eq!(format!("{e:?}"), "Invalid");
/// let error: rp1_hal::uart::Error = e.into();
/// assert_eq!(error, rp1_hal::uart::Error::Invalid);
/// ```
pub struct InitError<R, E> {
    pub error: E,
    pub resources: R,
}
impl<R, E: core::fmt::Debug> core::fmt::Debug for InitError<R, E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.error.fmt(f)
    }
}

/// A `free` that did not release: the driver, unchanged and still configured, and the
/// driver's error. `Debug` prints only the error, so `.unwrap()` and `.expect()` work although
/// drivers are not `Debug`. There is deliberately no `From` for the error: `?` would drop a
/// driver that still owns running hardware.
///
/// ```
/// struct Driver;
/// let e = rp1_hal::FreeError { driver: Driver, error: rp1_hal::uart::Error::Timeout };
/// assert_eq!(format!("{e:?}"), "Timeout");
/// ```
pub struct FreeError<D, E> {
    pub driver: D,
    pub error: E,
}
impl<D, E: core::fmt::Debug> core::fmt::Debug for FreeError<D, E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.error.fmt(f)
    }
}

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
    write_relaxed(address, value);
    rp1_proto::barrier();
}
/// A volatile store for callers that order a sequence of writes themselves.
#[inline(always)]
pub(crate) fn write_relaxed(address: usize, value: u32) {
    unsafe {
        (address as *mut u32).write_volatile(value);
    }
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

/// The calling firmware must declare and initialize the generated `rp1::log` module. Interrupts
/// stay enabled while a message is written; in a handler a message may be dropped instead of
/// waited for ([`logging::Logger::write`]).
#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => { rp1::log::write(core::format_args!($($arg)*)) };
}
