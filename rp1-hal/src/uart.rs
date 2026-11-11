//! PL011 UARTs 0..4 on their bank-0 pins (DOC: RP1 peripherals §3.5). The baud divisors come
//! from clk_uart as reported by [`Clocks`]. UART5 has no bank-0 pins. The `log!` sink (UART0 on
//! GPIO14/15) is configured by [`crate::logging::Logger`], not through a [`Token`].
use crate::{
    Error,
    clocks::Clocks,
    gpio::{Alternate, Pin},
    pac::pl011::int,
    poll, read, write,
};
use core::fmt;

mod sealed {
    pub trait Pin {}
    impl<const N: u8> Pin for crate::gpio::Pin<N> {}
}
/// TX route of UART N; `FUNC` is the GPIO function select (`crate::pinmux`).
pub trait Tx<const N: u8>: sealed::Pin {
    const FUNC: u8;
}
/// RX route of UART N.
pub trait Rx<const N: u8>: sealed::Pin {
    const FUNC: u8;
}
/// Pad while the mux switches (output disabled) and for RX: input enable, pull-up (line idle),
/// Schmitt. TX: input enable, 8 mA, Schmitt.
const IDLE_PAD: u32 = 0x4a;
const TX_PAD: u32 = 0x62;

const fn base(n: u8) -> usize {
    0x4003_0000 + n as usize * 0x4000
}

pub struct Token<const N: u8> {
    _private: (),
}
/// A configured UART that owns its TX and RX pins.
pub struct Uart<const N: u8, const TX: u8, const RX: u8> {
    _tx: Pin<TX, Alternate>,
    _rx: Pin<RX, Alternate>,
}
/// [`Token::init`] failed: the error, and the token and pins back (unchanged) for a retry.
pub struct InitError<const N: u8, const TX: u8, const RX: u8> {
    pub error: Error,
    pub token: Token<N>,
    pub tx: Pin<TX>,
    pub rx: Pin<RX>,
}
impl<const N: u8, const TX: u8, const RX: u8> fmt::Debug for InitError<N, TX, RX> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.error.fmt(f)
    }
}
impl<const N: u8, const TX: u8, const RX: u8> From<InitError<N, TX, RX>> for Error {
    fn from(e: InitError<N, TX, RX>) -> Self {
        e.error
    }
}
impl<const N: u8> Token<N> {
    /// # Safety
    /// Generated resources only: sole owner of UART N, with Linux's node disabled.
    ///
    /// UART5 has no bank-0 pins and is a compile-time error:
    ///
    /// ```compile_fail,E0080
    /// let _ = unsafe { rp1_hal::uart::Token::<5>::new() };
    /// ```
    pub const unsafe fn new() -> Self {
        const { assert!(N < 5) };
        Self { _private: () }
    }
    /// Checks clk_uart and the baud rate, releases only this UART's reset, configures 8N1 with
    /// FIFOs, then routes the typed pins. On error the token and the pins come back in
    /// [`InitError`]; the pins are not touched before success.
    ///
    /// ```no_run
    /// # use rp1_hal::{Error, clocks::Clocks, gpio::Pin, uart::{Token, Uart}};
    /// fn bring_up(token: Token<2>, tx: Pin<4>, rx: Pin<5>, clocks: &Clocks) -> Result<Uart<2, 4, 5>, Error> {
    ///     Ok(token.init(tx, rx, clocks, 115_200)?)
    /// }
    /// ```
    pub fn init<const TX: u8, const RX: u8>(
        self,
        tx: Pin<TX>,
        rx: Pin<RX>,
        clocks: &Clocks,
        baud: u32,
    ) -> Result<Uart<N, TX, RX>, InitError<N, TX, RX>>
    where
        Pin<TX>: Tx<N>,
        Pin<RX>: Rx<N>,
    {
        let started = if clocks.still_current() {
            start(N, clocks.uart_hz(), baud)
        } else {
            Err(Error::Precondition)
        };
        match started {
            Ok(()) => Ok(Uart {
                _tx: tx.into_alternate(<Pin<TX> as Tx<N>>::FUNC, IDLE_PAD, TX_PAD),
                _rx: rx.into_alternate(<Pin<RX> as Rx<N>>::FUNC, IDLE_PAD, IDLE_PAD),
            }),
            Err(error) => Err(InitError {
                error,
                token: self,
                tx,
                rx,
            }),
        }
    }
}
/// A declared UART N with its pins, as the generated resources hand it out
/// (`rp1-build`); only routes from the bank-0 pin table can be named.
///
/// ```no_run
/// # use rp1_hal::{Error, clocks::Clocks, gpio::Pin, uart::{Parts, Token, Uart}};
/// // The resource generated for `uart2 = { tx = 4, rx = 5 }`:
/// let uart2: Parts<2, 4, 5> = Parts {
///     token: unsafe { Token::new() },
///     tx: unsafe { Pin::new() },
///     rx: unsafe { Pin::new() },
/// };
/// fn bring_up(uart2: Parts<2, 4, 5>, clocks: &Clocks) -> Result<Uart<2, 4, 5>, Error> {
///     Ok(uart2.init(clocks, 115_200)?)
/// }
/// ```
pub struct Parts<const N: u8, const TX: u8, const RX: u8>
where
    Pin<TX>: Tx<N>,
    Pin<RX>: Rx<N>,
{
    pub token: Token<N>,
    pub tx: Pin<TX>,
    pub rx: Pin<RX>,
}
impl<const N: u8, const TX: u8, const RX: u8> Parts<N, TX, RX>
where
    Pin<TX>: Tx<N>,
    Pin<RX>: Rx<N>,
{
    /// [`Token::init`] with the declared pins.
    pub fn init(self, clocks: &Clocks, baud: u32) -> Result<Uart<N, TX, RX>, InitError<N, TX, RX>> {
        self.token.init(self.tx, self.rx, clocks, baud)
    }
}
/// Releases UART `n`'s reset and configures it for 8N1 at `baud`. Nothing is written unless
/// the divisors are valid.
pub(crate) fn start(n: u8, clock_hz: u32, baud: u32) -> Result<(), Error> {
    let (integer, fraction) = divisors(clock_hz, baud)?;
    write(0x4001_7004, 1 << (26 + n));
    poll(0x4001_401c, 1 << (26 + n), 1 << (26 + n))?;
    let base = base(n);
    poll(base + 0x18, 1 << 3, 0)?;
    write(base + 0x30, 0);
    write(base + 0x38, 0);
    write(base + 0x44, 0x7ff);
    write(base + 4, 0);
    write(base + 0x24, integer);
    write(base + 0x28, fraction);
    write(base + 0x2c, 0x70);
    write(base + 0x30, 0x301);
    Ok(())
}
pub fn divisors(clock_hz: u32, baud: u32) -> Result<(u32, u32), Error> {
    if baud == 0 {
        return Err(Error::Invalid);
    }
    let scaled = (u64::from(clock_hz) * 4 + u64::from(baud) / 2) / u64::from(baud);
    if !(64..=(65535 * 64 + 63)).contains(&scaled) {
        return Err(Error::Invalid);
    }
    Ok(((scaled / 64) as u32, (scaled % 64) as u32))
}
impl<const N: u8, const TX: u8, const RX: u8> Uart<N, TX, RX> {
    pub fn write_byte(&mut self, byte: u8) -> Result<(), Error> {
        let base = base(N);
        poll(base + 0x18, 1 << 5, 0)?;
        write(base, u32::from(byte));
        Ok(())
    }
    pub fn read_byte(&mut self) -> Result<Option<u8>, Error> {
        let base = base(N);
        if read(base + 0x18) & (1 << 4) != 0 {
            return Ok(None);
        }
        let data = read(base);
        if data & 0xf00 != 0 {
            write(base + 4, 0);
            return Err(Error::Hardware);
        }
        Ok(Some(data as u8))
    }
    pub fn flush(&mut self) -> Result<(), Error> {
        poll(base(N) + 0x18, 1 << 3, 0)
    }
    /// Unmasks the receive interrupts: RX FIFO at its trigger level, and receive timeout so a
    /// short message is not left below the level. The NVIC line is `irq.UARTn` in the generated
    /// resources.
    pub fn enable_rx_interrupt(&mut self) {
        set_mask(N, int::RX | int::RT, true);
    }
    pub fn disable_rx_interrupt(&mut self) {
        set_mask(N, int::RX | int::RT, false);
    }
    /// Unmasks the interrupt for the TX FIFO at or below its trigger level.
    pub fn enable_tx_interrupt(&mut self) {
        set_mask(N, int::TX, true);
    }
    pub fn disable_tx_interrupt(&mut self) {
        set_mask(N, int::TX, false);
    }
    /// Pending unmasked interrupts (MIS), as [`int`](crate::pac::pl011::int) bits.
    pub fn interrupt_status(&self) -> u32 {
        read(base(N) + 0x40)
    }
    /// Clears the [`int`](crate::pac::pl011::int) bits in `mask`. RX and RT also clear by
    /// draining the FIFO, TX by refilling it.
    pub fn clear_interrupt(&mut self, mask: u32) {
        write(base(N) + 0x44, mask);
    }
    /// Waits up to 10 ms for the transmitter to drain, masks the interrupts, disables the UART
    /// and gives back the token and the (still muxed) pins.
    pub fn free(self) -> (Token<N>, Pin<TX>, Pin<RX>) {
        let _ = poll(base(N) + 0x18, 1 << 3, 0);
        write(base(N) + 0x38, 0);
        write(base(N) + 0x30, 0);
        (
            Token { _private: () },
            self._tx.release(),
            self._rx.release(),
        )
    }
}
/// Sets or clears `bits` in UART `n`'s interrupt mask (IMSC).
fn set_mask(n: u8, bits: u32, on: bool) {
    let imsc = read(base(n) + 0x38);
    write(base(n) + 0x38, if on { imsc | bits } else { imsc & !bits });
}
impl<const N: u8, const TX: u8, const RX: u8> fmt::Write for Uart<N, TX, RX> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.bytes() {
            self.write_byte(b).map_err(|_| fmt::Error)?;
        }
        Ok(())
    }
}
impl embedded_io::Error for Error {
    fn kind(&self) -> embedded_io::ErrorKind {
        embedded_io::ErrorKind::Other
    }
}
impl<const N: u8, const TX: u8, const RX: u8> embedded_io::ErrorType for Uart<N, TX, RX> {
    type Error = Error;
}
impl<const N: u8, const TX: u8, const RX: u8> embedded_io::Write for Uart<N, TX, RX> {
    fn write(&mut self, b: &[u8]) -> Result<usize, Error> {
        for byte in b {
            self.write_byte(*byte)?;
        }
        Ok(b.len())
    }
    fn flush(&mut self) -> Result<(), Error> {
        Uart::flush(self)
    }
}
impl<const N: u8, const TX: u8, const RX: u8> embedded_io::Read for Uart<N, TX, RX> {
    fn read(&mut self, b: &mut [u8]) -> Result<usize, Error> {
        if b.is_empty() {
            return Ok(0);
        }
        for _ in 0..100_000 {
            if let Some(byte) = self.read_byte()? {
                b[0] = byte;
                return Ok(1);
            }
        }
        Err(Error::Timeout)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn baud_divisor_and_invalid_inputs() {
        assert_eq!(divisors(50_000_000, 115200), Ok((27, 8)));
        assert_eq!(divisors(0, 115200), Err(Error::Invalid));
        assert_eq!(divisors(50_000_000, 0), Err(Error::Invalid));
        assert_eq!(divisors(50_000_000, 1), Err(Error::Invalid));
    }
}
