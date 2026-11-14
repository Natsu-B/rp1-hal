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
/// Bound of the two waits of the bring-up (reset done, transmitter idle).
const START_TIMEOUT_US: u32 = 10_000;
/// [`Uart::set_timeout_us`] until changed. A full TX FIFO drains in 2.9 ms at 115200 baud.
pub const DEFAULT_TIMEOUT_US: u32 = 100_000;

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
    timeout_us: u32,
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
                timeout_us: DEFAULT_TIMEOUT_US,
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
    wait(
        io,
        RESETS_DONE1,
        reset_bit(n),
        reset_bit(n),
        START_TIMEOUT_US,
    )?;
    let base = base(n);
    wait(io, base + FR, FR_BUSY, 0, START_TIMEOUT_US)?;
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
/// Waits for `address & mask == value` until more than `timeout_us` of the 1 MHz timer have
/// passed, however many polls that takes.
fn wait(
    io: &mut impl Hardware,
    address: usize,
    mask: u32,
    value: u32,
    timeout_us: u32,
) -> Result<(), Error> {
    // The timer wraps every 2^32 us: summing the steps between polls in 64 bits lets every
    // `timeout_us` expire.
    let mut last = io.now_us();
    let mut elapsed_us = 0u64;
    loop {
        if io.read(address) & mask == value {
            return Ok(());
        }
        let now = io.now_us();
        elapsed_us += u64::from(now.wrapping_sub(last));
        last = now;
        if elapsed_us > u64::from(timeout_us) {
            // One more look: the deadline may have passed while this core was stalled.
            return if io.read(address) & mask == value {
                Ok(())
            } else {
                Err(Error::Timeout)
            };
        }
    }
}
/// [`Uart::write`]: waits for room for the first byte only.
fn write(io: &mut impl Hardware, n: u8, bytes: &[u8], timeout_us: u32) -> Result<usize, Error> {
    if bytes.is_empty() {
        return Ok(0);
    }
    let base = base(n);
    wait(io, base + FR, FR_TXFF, 0, timeout_us)?;
    let mut accepted = 0;
    loop {
        io.write(base + DR, u32::from(bytes[accepted]));
        accepted += 1;
        if accepted == bytes.len() || io.read(base + FR) & FR_TXFF != 0 {
            return Ok(accepted);
        }
    }
}
/// Pops the RX FIFO, which must not be empty.
fn take(io: &mut impl Hardware, n: u8) -> Result<u8, Error> {
    let data = io.read(base(n) + DR);
    match receive_error(data) {
        Some(error) => {
            io.write(base(n) + ECR, 0);
            Err(error)
        }
        None => Ok(data as u8),
    }
}
fn read_byte(io: &mut impl Hardware, n: u8) -> Result<Option<u8>, Error> {
    if io.read(base(n) + FR) & FR_RXFE != 0 {
        return Ok(None);
    }
    take(io, n).map(Some)
}
/// [`embedded_io::Read::read`] of one byte.
fn read_wait(io: &mut impl Hardware, n: u8, timeout_us: u32) -> Result<u8, Error> {
    wait(io, base(n) + FR, FR_RXFE, 0, timeout_us)?;
    take(io, n)
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
    /// Bound, in microseconds of the 1 MHz timer, of each wait of this driver: for room in
    /// the TX FIFO ([`Uart::write`]), for a received byte ([`embedded_io::Read`]) and for the
    /// transmitter to drain ([`Uart::flush`]). [`DEFAULT_TIMEOUT_US`] after `init`. A full TX
    /// FIFO needs 330 bit times to drain (32 characters and the one being shifted), which is
    /// longer than the default below 3300 baud.
    pub fn set_timeout_us(&mut self, timeout_us: u32) {
        self.timeout_us = timeout_us;
    }
    /// Queues a prefix of `bytes` for transmission and returns its length, as
    /// [`embedded_io::Write::write`]: waits for room in the TX FIFO for the first byte, then
    /// takes what fits without waiting again. [`Error::Timeout`] means that nothing was
    /// queued, so the same `bytes` can be written again without duplicating data. Queued is
    /// not transmitted: [`Uart::flush`] waits for the line.
    pub fn write(&mut self, bytes: &[u8]) -> Result<usize, Error> {
        write(&mut Mmio, N, bytes, self.timeout_us)
    }
    /// The next received character, `None` if the RX FIFO is empty. A character received with
    /// an error is dropped and the error returned in its place.
    pub fn read_byte(&mut self) -> Result<Option<u8>, Error> {
        read_byte(&mut Mmio, N)
    }
    /// Waits until everything queued has left the shift register (BUSY clear).
    /// [`Error::Timeout`] leaves the rest in transmission; flushing again continues the wait.
    pub fn flush(&mut self) -> Result<(), Error> {
        wait(&mut Mmio, base(N) + FR, FR_BUSY, 0, self.timeout_us)
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
    pub fn free(mut self) -> (Token<N>, Pin<TX>, Pin<RX>) {
        let _ = self.flush();
        Mmio.write(base(N) + IMSC, 0);
        Mmio.write(base(N) + CR, 0);
        (
            Token { _private: () },
            self._tx.release(),
            self._rx.release(),
        )
    }
}
/// Formatting stops at the first [`Error::Timeout`]; what was queued before it stays queued.
impl<const N: u8, const TX: u8, const RX: u8> fmt::Write for Uart<N, TX, RX> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        embedded_io::Write::write_all(self, s.as_bytes()).map_err(|_| fmt::Error)
    }
}
impl<const N: u8, const TX: u8, const RX: u8> embedded_io::ErrorType for Uart<N, TX, RX> {
    type Error = Error;
}
impl<const N: u8, const TX: u8, const RX: u8> embedded_io::Write for Uart<N, TX, RX> {
    fn write(&mut self, b: &[u8]) -> Result<usize, Error> {
        Uart::write(self, b)
    }
    fn flush(&mut self) -> Result<(), Error> {
        Uart::flush(self)
    }
}
/// One byte per call. [`Error::Timeout`] if none arrives within [`Uart::set_timeout_us`];
/// receive errors as in [`Uart::read_byte`].
impl<const N: u8, const TX: u8, const RX: u8> embedded_io::Read for Uart<N, TX, RX> {
    fn read(&mut self, b: &mut [u8]) -> Result<usize, Error> {
        let Some(first) = b.first_mut() else {
            return Ok(0);
        };
        *first = read_wait(&mut Mmio, N, self.timeout_us)?;
        Ok(1)
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
        /// Free TX FIFO slots, and what was pushed.
        tx_room: usize,
        tx: Vec<u8>,
        /// A DR value that enters the RX FIFO once `elapsed_us` reaches the time.
        rx_later: Option<(u64, u32)>,
        now: u32,
        /// Timer step per read, and the time read so far.
        step_us: u32,
        elapsed_us: u64,
    }
    impl Fake {
        fn new() -> Self {
            Self {
                regs: [(RESETS_DONE1, reset_bit(N))].into(),
                writes: Vec::new(),
                rx: VecDeque::new(),
                busy: false,
                tx_room: 32,
                tx: Vec::new(),
                rx_later: None,
                // The 32-bit timer wraps during every bounded wait.
                now: u32::MAX - 500,
                step_us: 100,
                elapsed_us: 0,
            }
        }
    }
    impl Registers for Fake {
        fn read(&mut self, address: usize) -> u32 {
            if address == base(N) + FR {
                if let Some((_, data)) = self.rx_later.take_if(|(at, _)| *at <= self.elapsed_us) {
                    self.rx.push_back(data);
                }
                return if self.busy { FR_BUSY } else { 0 }
                    | if self.tx_room == 0 { FR_TXFF } else { 0 }
                    | if self.rx.is_empty() { FR_RXFE } else { 0 };
            }
            if address == base(N) + DR {
                return self.rx.pop_front().expect("RX underflow");
            }
            self.regs.get(&address).copied().unwrap_or(0)
        }
        fn write(&mut self, address: usize, value: u32) {
            if address == base(N) + DR {
                self.tx_room = self.tx_room.checked_sub(1).expect("TX overflow");
                self.tx.push(value as u8);
                return;
            }
            self.writes.push((address, value));
            self.regs.insert(address, value);
        }
    }
    impl Hardware for Fake {
        fn now_us(&mut self) -> u32 {
            self.now = self.now.wrapping_add(self.step_us);
            self.elapsed_us += u64::from(self.step_us);
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
        assert!((10_000..10_500).contains(&f.elapsed_us));
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
    /// embedded-io `write`: a full FIFO ends the call with the count of the bytes queued, and
    /// only a call that queued nothing times out.
    #[test]
    fn a_write_reports_the_prefix_it_queued() {
        let mut f = Fake::new();
        f.tx_room = 3;
        assert_eq!(write(&mut f, N, b"hello", 1_000), Ok(3));
        assert_eq!(f.tx, b"hel");
        assert_eq!(write(&mut f, N, b"lo", 1_000), Err(Error::Timeout));
        assert_eq!(f.tx, b"hel");
        f.tx_room = 32;
        assert_eq!(write(&mut f, N, b"lo", 1_000), Ok(2));
        assert_eq!(write(&mut f, N, b"", 1_000), Ok(0));
        assert_eq!(f.tx, b"hello");
    }
    /// The bound is time on the timer, not polls: the same wait ends after the same time
    /// whether a poll takes 1 us or 300 us, and across the timer's wrap.
    #[test]
    fn waits_end_after_the_timeout_in_timer_time() {
        for (step_us, timeout_us) in [(1, 2_000), (300, 2_000), (7, 0), (1 << 30, u32::MAX)] {
            let mut f = Fake::new();
            (f.step_us, f.tx_room, f.busy) = (step_us, 0, true);
            for stalled in [
                write(&mut f, N, b"x", timeout_us).map(drop),
                read_wait(&mut f, N, timeout_us).map(drop),
                wait(&mut f, base(N) + FR, FR_BUSY, 0, timeout_us),
            ] {
                assert_eq!(stalled, Err(Error::Timeout));
            }
            // Each wait reads the timer once to start, then runs just past the timeout.
            let each = f.elapsed_us / 3 - u64::from(step_us);
            assert!(
                each > u64::from(timeout_us),
                "{step_us} {timeout_us} {each}"
            );
            assert!(each <= u64::from(timeout_us) + u64::from(step_us));
        }
    }
    #[test]
    fn a_read_waits_for_a_byte_that_arrives_in_time() {
        let mut f = Fake::new();
        f.rx_later = Some((40_000, 0x55));
        assert_eq!(read_wait(&mut f, N, 30_000), Err(Error::Timeout));
        assert_eq!(read_wait(&mut f, N, 30_000), Ok(0x55));
        f.rx_later = Some((0, DR_FE));
        assert_eq!(read_wait(&mut f, N, 30_000), Err(Error::Framing));
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
