//! PL011 UARTs 0..4 on their bank-0 pins (DOC: RP1 peripherals §3.5). The baud divisors come
//! from clk_uart as reported by [`Clocks`]. UART5 has no bank-0 pins. The `log!` sink (UART0 on
//! GPIO14/15) is configured by [`crate::logging::Logger`], not through a [`Token`].
use crate::{
    clocks::Clocks,
    gpio::{Alternate, Pin},
    linux_blocks::{Hardware, Mmio},
    pac::pl011::{RegisterBlock, int},
    scmi::Registers,
};
use core::fmt;

macro_rules! offsets {
    ($($name:ident = $field:ident),* $(,)?) => {$(const $name: usize = core::mem::offset_of!(RegisterBlock, $field);)*};
}
offsets! {
    DR = dr, ECR = rsr_ecr, FR = fr, IBRD = ibrd, FBRD = fbrd, LCR_H = lcr_h, CR = cr,
    IMSC = imsc, MIS = mis, ICR = icr,
}
/// DR bits 8..=11: the receive status of the character read with them.
const DR_FE: u32 = 1 << 8;
const DR_PE: u32 = 1 << 9;
const DR_BE: u32 = 1 << 10;
const DR_OE: u32 = 1 << 11;
const FR_BUSY: u32 = 1 << 3;
const FR_RXFE: u32 = 1 << 4;
const FR_TXFF: u32 = 1 << 5;
/// 8 data bits, FIFOs enabled, no parity, one stop bit.
const LCR_H_8N1_FIFO: u32 = 0x70;
/// UARTEN, TXE, RXE.
const CR_ENABLE: u32 = 0x301;
const RESETS_CLR1: usize = 0x4001_7004;
const RESETS_DONE1: usize = 0x4001_401c;
const fn reset_bit(n: u8) -> u32 {
    1 << (26 + n)
}
const TIMEOUT_US: u32 = 10_000;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    /// The RX FIFO was full and at least one later character was lost.
    Overrun,
    /// The line was held low for longer than a character.
    Break,
    /// The parity bit did not match (never with the 8N1 this driver configures).
    Parity,
    /// No valid stop bit.
    Framing,
    /// The reset, the transmitter or the peer kept the driver waiting too long.
    Timeout,
    /// The baud rate has no divisor at clk_uart.
    Invalid,
    /// The clock tree is not the one the [`Clocks`] value describes.
    Precondition,
}
/// `Overrun` has no kind of its own in embedded-io and is `Other`: the data that was read is
/// good, later data is missing.
impl embedded_io::Error for Error {
    fn kind(&self) -> embedded_io::ErrorKind {
        use embedded_io::ErrorKind;
        match self {
            Self::Break | Self::Parity | Self::Framing => ErrorKind::InvalidData,
            Self::Timeout => ErrorKind::TimedOut,
            Self::Invalid => ErrorKind::InvalidInput,
            Self::Overrun | Self::Precondition => ErrorKind::Other,
        }
    }
}
/// The receive status in a DR value. A break also fails the stop-bit check, so it is looked
/// at before framing; overrun first, as the one that means lost data.
fn receive_error(data: u32) -> Option<Error> {
    [
        (DR_OE, Error::Overrun),
        (DR_BE, Error::Break),
        (DR_PE, Error::Parity),
        (DR_FE, Error::Framing),
    ]
    .into_iter()
    .find_map(|(bit, error)| (data & bit != 0).then_some(error))
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
    /// # use rp1_hal::{clocks::Clocks, gpio::Pin, uart::{Error, Token, Uart}};
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
            start(&mut Mmio, N, clocks.uart_hz(), baud)
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
/// # use rp1_hal::{clocks::Clocks, gpio::Pin, uart::{Error, Parts, Token, Uart}};
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
pub(crate) fn start(io: &mut impl Hardware, n: u8, clock_hz: u32, baud: u32) -> Result<(), Error> {
    let (integer, fraction) = divisors(clock_hz, baud)?;
    io.write(RESETS_CLR1, reset_bit(n));
    wait(io, RESETS_DONE1, reset_bit(n), reset_bit(n))?;
    let base = base(n);
    wait(io, base + FR, FR_BUSY, 0)?;
    io.write(base + CR, 0);
    io.write(base + IMSC, 0);
    io.write(base + ICR, 0x7ff);
    io.write(base + ECR, 0);
    io.write(base + IBRD, integer);
    io.write(base + FBRD, fraction);
    io.write(base + LCR_H, LCR_H_8N1_FIFO);
    io.write(base + CR, CR_ENABLE);
    Ok(())
}
/// Waits up to 10 ms for `address & mask == value`.
fn wait(io: &mut impl Hardware, address: usize, mask: u32, value: u32) -> Result<(), Error> {
    let start = io.now_us();
    loop {
        if io.read(address) & mask == value {
            return Ok(());
        }
        if io.now_us().wrapping_sub(start) > TIMEOUT_US {
            // One more look: the deadline may have passed while this core was stalled.
            return if io.read(address) & mask == value {
                Ok(())
            } else {
                Err(Error::Timeout)
            };
        }
    }
}
fn write_byte(io: &mut impl Hardware, n: u8, byte: u8) -> Result<(), Error> {
    wait(io, base(n) + FR, FR_TXFF, 0)?;
    io.write(base(n) + DR, u32::from(byte));
    Ok(())
}
fn read_byte(io: &mut impl Hardware, n: u8) -> Result<Option<u8>, Error> {
    let base = base(n);
    if io.read(base + FR) & FR_RXFE != 0 {
        return Ok(None);
    }
    let data = io.read(base + DR);
    match receive_error(data) {
        Some(error) => {
            io.write(base + ECR, 0);
            Err(error)
        }
        None => Ok(Some(data as u8)),
    }
}
/// Sets or clears `bits` in UART `n`'s interrupt mask (IMSC).
fn set_mask(io: &mut impl Hardware, n: u8, bits: u32, on: bool) {
    let imsc = io.read(base(n) + IMSC);
    io.write(base(n) + IMSC, if on { imsc | bits } else { imsc & !bits });
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
        write_byte(&mut Mmio, N, byte)
    }
    /// The next received character, `None` if the RX FIFO is empty. A character received with
    /// an error is dropped and the error returned in its place.
    pub fn read_byte(&mut self) -> Result<Option<u8>, Error> {
        read_byte(&mut Mmio, N)
    }
    pub fn flush(&mut self) -> Result<(), Error> {
        wait(&mut Mmio, base(N) + FR, FR_BUSY, 0)
    }
    /// Unmasks the receive interrupts: RX FIFO at its trigger level, and receive timeout so a
    /// short message is not left below the level. The NVIC line is `irq.UARTn` in the generated
    /// resources.
    pub fn enable_rx_interrupt(&mut self) {
        set_mask(&mut Mmio, N, int::RX | int::RT, true);
    }
    pub fn disable_rx_interrupt(&mut self) {
        set_mask(&mut Mmio, N, int::RX | int::RT, false);
    }
    /// Unmasks the interrupt raised when the TX FIFO drops to its trigger level.
    pub fn enable_tx_interrupt(&mut self) {
        set_mask(&mut Mmio, N, int::TX, true);
    }
    pub fn disable_tx_interrupt(&mut self) {
        set_mask(&mut Mmio, N, int::TX, false);
    }
    /// Pending unmasked interrupts (MIS), as [`int`](crate::pac::pl011::int) bits.
    pub fn interrupt_status(&self) -> u32 {
        Mmio.read(base(N) + MIS)
    }
    /// Clears the [`int`](crate::pac::pl011::int) bits in `mask`. RX and RT also clear by
    /// draining the FIFO, TX by refilling it.
    pub fn clear_interrupt(&mut self, mask: u32) {
        Mmio.write(base(N) + ICR, mask);
    }
    /// Waits up to 10 ms for the transmitter to drain, masks the interrupts, disables the UART
    /// and gives back the token and the (still muxed) pins.
    pub fn free(self) -> (Token<N>, Pin<TX>, Pin<RX>) {
        let _ = wait(&mut Mmio, base(N) + FR, FR_BUSY, 0);
        Mmio.write(base(N) + IMSC, 0);
        Mmio.write(base(N) + CR, 0);
        (
            Token { _private: () },
            self._tx.release(),
            self._rx.release(),
        )
    }
}
impl<const N: u8, const TX: u8, const RX: u8> fmt::Write for Uart<N, TX, RX> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.bytes() {
            self.write_byte(b).map_err(|_| fmt::Error)?;
        }
        Ok(())
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
    extern crate std;
    use super::*;
    use std::{collections::BTreeMap, collections::VecDeque, vec::Vec};

    /// UART `N`'s registers. FR is derived: BUSY and TXFF from the test's switches, RXFE from
    /// the queue of received DR values.
    const N: u8 = 2;
    struct Fake {
        regs: BTreeMap<usize, u32>,
        writes: Vec<(usize, u32)>,
        rx: VecDeque<u32>,
        busy: bool,
        tx_full: bool,
        now: u32,
    }
    impl Fake {
        fn new() -> Self {
            Self {
                regs: [(RESETS_DONE1, reset_bit(N))].into(),
                writes: Vec::new(),
                rx: VecDeque::new(),
                busy: false,
                tx_full: false,
                // The 32-bit timer wraps during every bounded wait.
                now: u32::MAX - 500,
            }
        }
    }
    impl Registers for Fake {
        fn read(&mut self, address: usize) -> u32 {
            if address == base(N) + FR {
                return if self.busy { FR_BUSY } else { 0 }
                    | if self.tx_full { FR_TXFF } else { 0 }
                    | if self.rx.is_empty() { FR_RXFE } else { 0 };
            }
            if address == base(N) + DR {
                return self.rx.pop_front().expect("RX underflow");
            }
            self.regs.get(&address).copied().unwrap_or(0)
        }
        fn write(&mut self, address: usize, value: u32) {
            self.writes.push((address, value));
            self.regs.insert(address, value);
        }
    }
    impl Hardware for Fake {
        fn now_us(&mut self) -> u32 {
            self.now = self.now.wrapping_add(100);
            self.now
        }
    }

    #[test]
    fn register_offsets_and_instances_match_the_pac() {
        assert_eq!([DR, ECR, FR, IBRD, FBRD], [0, 4, 0x18, 0x24, 0x28]);
        assert_eq!([LCR_H, CR, IMSC, MIS, ICR], [0x2c, 0x30, 0x38, 0x40, 0x44]);
        assert_eq!(crate::pac::UART0::PTR as usize, base(0));
        assert_eq!(crate::pac::UART4::PTR as usize, base(4));
    }
    #[test]
    fn start_releases_the_reset_then_programs_8n1_and_enables() {
        let mut f = Fake::new();
        start(&mut f, N, 50_000_000, 115_200).unwrap();
        let b = base(N);
        assert_eq!(
            f.writes,
            [
                (RESETS_CLR1, 1 << 28),
                (b + CR, 0),
                (b + IMSC, 0),
                (b + ICR, 0x7ff),
                (b + ECR, 0),
                (b + IBRD, 27),
                (b + FBRD, 8),
                (b + LCR_H, 0x70),
                (b + CR, 0x301),
            ]
        );
        let mut f = Fake::new();
        assert_eq!(start(&mut f, N, 50_000_000, 0), Err(Error::Invalid));
        assert!(f.writes.is_empty());
    }
    #[test]
    fn a_reset_or_transmitter_that_stays_busy_times_out_before_configuration() {
        let mut f = Fake::new();
        f.regs.insert(RESETS_DONE1, 0);
        assert_eq!(start(&mut f, N, 50_000_000, 115_200), Err(Error::Timeout));
        assert_eq!(f.writes, [(RESETS_CLR1, 1 << 28)]);
        let mut f = Fake::new();
        f.busy = true;
        assert_eq!(start(&mut f, N, 50_000_000, 115_200), Err(Error::Timeout));
        assert_eq!(f.writes, [(RESETS_CLR1, 1 << 28)]);
    }
    #[test]
    fn receive_errors_are_told_apart_and_replace_their_character() {
        let mut f = Fake::new();
        assert_eq!(read_byte(&mut f, N), Ok(None));
        f.rx.extend([
            0x41,
            DR_FE | 0x42,
            DR_PE | 0x43,
            DR_BE | DR_FE,
            DR_OE | 0x44,
            0x45,
        ]);
        let mut got = Vec::new();
        while !f.rx.is_empty() {
            got.push(read_byte(&mut f, N));
        }
        assert_eq!(
            got,
            [
                Ok(Some(0x41)),
                Err(Error::Framing),
                Err(Error::Parity),
                Err(Error::Break),
                Err(Error::Overrun),
                Ok(Some(0x45)),
            ]
        );
        // Each error also clears the sticky copy in RSR.
        assert_eq!(f.writes, [(base(N) + ECR, 0); 4]);
    }
    #[test]
    fn error_kinds_for_embedded_io() {
        use embedded_io::{Error as _, ErrorKind};
        for error in [Error::Break, Error::Parity, Error::Framing] {
            assert_eq!(error.kind(), ErrorKind::InvalidData);
        }
        assert_eq!(Error::Timeout.kind(), ErrorKind::TimedOut);
        assert_eq!(Error::Invalid.kind(), ErrorKind::InvalidInput);
        assert_eq!(Error::Overrun.kind(), ErrorKind::Other);
    }
    #[test]
    fn interrupt_mask_updates_keep_the_other_bits() {
        let mut f = Fake::new();
        set_mask(&mut f, N, int::RX | int::RT, true);
        set_mask(&mut f, N, int::TX, true);
        set_mask(&mut f, N, int::RX | int::RT, false);
        assert_eq!(f.regs[&(base(N) + IMSC)], int::TX);
    }
    #[test]
    fn baud_divisor_and_invalid_inputs() {
        assert_eq!(divisors(50_000_000, 115200), Ok((27, 8)));
        assert_eq!(divisors(0, 115200), Err(Error::Invalid));
        assert_eq!(divisors(50_000_000, 0), Err(Error::Invalid));
        assert_eq!(divisors(50_000_000, 1), Err(Error::Invalid));
    }
}
