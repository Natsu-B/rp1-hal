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

// Keep the set of required routes sealed even when a driver is named by its pin tuple.
macro_rules! complete_pins {
    ($($num:ident: $route:ident),+; $active:ty) => {
        /// The required pin routes for this peripheral.
        pub trait CompletePins<const N: u8>: sealed::CompletePins<N> {
            #[doc(hidden)]
            type Active;
        }

        impl<const N: u8, $(const $num: u8),+> sealed::CompletePins<N> for ($(Pin<$num>,)+)
        where
            $(Pin<$num>: $route<N>,)+
        {}

        impl<const N: u8, $(const $num: u8),+> CompletePins<N> for ($(Pin<$num>,)+)
        where
            $(Pin<$num>: $route<N>,)+
        {
            type Active = $active;
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

/// Failed initialization. Parts are available once the peripheral can be initialized again.
/// If recovery is pending, this error retains ownership of the peripheral and its pins.
pub struct InitError<R, E, H = core::convert::Infallible> {
    error: E,
    state: Result<R, H>,
}

impl<R, E, H> InitError<R, E, H> {
    pub(crate) fn new(error: E, parts: R) -> Self {
        Self {
            error,
            state: Ok(parts),
        }
    }

    pub(crate) fn recovering(error: E, owner: H) -> Self {
        Self {
            error,
            state: Err(owner),
        }
    }

    /// The error that stopped initialization.
    pub fn error(&self) -> &E {
        &self.error
    }

    /// Return the parts if ready for reuse. Otherwise return the recovery owner.
    pub fn try_into_parts(self) -> Result<R, Self> {
        match self.state {
            Ok(parts) => Ok(parts),
            Err(_) => Err(self),
        }
    }
}

impl<R, E> InitError<R, E, R> {
    pub(crate) fn recover(self, stopped: impl FnOnce() -> bool) -> Result<R, Self> {
        match self.state {
            Ok(parts) => Ok(parts),
            Err(parts) if stopped() => Ok(parts),
            Err(_) => Err(self),
        }
    }
}

impl<R, E> InitError<R, E> {
    /// Return resources from an initializer whose failures always permit a retry.
    pub fn into_parts(self) -> R {
        match self.state {
            Ok(parts) => parts,
            Err(never) => match never {},
        }
    }

    /// Wrap the returned resources, for example in a generated ownership type.
    #[doc(hidden)]
    pub fn map_parts<S>(self, f: impl FnOnce(R) -> S) -> InitError<S, E> {
        InitError {
            error: self.error,
            state: self.state.map(f),
        }
    }
}

impl<R, E: core::fmt::Debug, H> core::fmt::Debug for InitError<R, E, H> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.error.fmt(f)
    }
}

/// Failed release. The driver still owns the peripheral. `Debug` prints only the error.
pub struct FreeError<D, E> {
    driver: D,
    error: E,
}

impl<D, E> FreeError<D, E> {
    /// The error that prevented release.
    pub fn error(&self) -> &E {
        &self.error
    }

    /// Recover the driver and the error to continue using or releasing the peripheral.
    pub fn into_parts(self) -> (D, E) {
        (self.driver, self.error)
    }
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
