//! RP1 bank-0 function selection, independent of board ownership policy.
//!
//! DOC: Signal names and FUNCSEL values come from Raspberry Pi Ltd.'s
//! [RP1 Peripherals](https://datasheets.raspberrypi.com/rp1/rp1-peripherals.pdf),
//! build 2023-11-07 (`b9b6f74-clean`), Table 4, pp. 15–16. SPI SIO roles follow
//! §3.6.1.1: on masters SIO[0] is MOSI and SIO[1] is MISO; SPI4 is a slave,
//! with SIO[0] MISO (TXD) and SIO[1] MOSI (RXD). SIO[2:3] retain their lane names.
//! The source TOML preserves the datasheet spellings, including `CSn`.
//!
//! STATIC: All 28 × 9 function families were compared with Raspberry Pi Linux
//! [`pinctrl-rp1.c`](https://github.com/raspberrypi/linux/blob/rpi-6.12.y/drivers/pinctrl/pinctrl-rp1.c)
//! `PIN()` rows (2026-10-30). Linux groups SYS_RIO as `gpio`, UART0 infrared as
//! `ir`, UART0 modem lines individually, audio as `aaud`/`mic`, SDIO0 as `sd0`,
//! and MIPI TE as `dsiN_te_ext`; it does not specify individual signal roles.
//! `rp1-regdb` tests can repeat this comparison with `RP1_LINUX_PINCTRL` set to
//! the driver source file. Tests also compare the existing HAL I2C/SPI routes.
//! This crate makes no hardware validation claim.
//!
//! Regenerate the checked-in table with `cargo run -p rp1-regdb -- generate-pinmux`;
//! add `--check` to reject stale output. There is no build script or runtime dependency.
//!
//! ```
//! use rp1_pinmux::{Signal, UartSignal, funcsel, candidate_pins};
//! let tx = Signal::Uart(2, UartSignal::Tx);
//! assert_eq!(funcsel(4, tx), Some(2));
//! assert_eq!(candidate_pins(tx).collect::<Vec<_>>(), [4]);
//! ```
#![no_std]

mod generated;
pub use generated::BANK0;

/// A bank-0 signal. Tuple instance numbers are hardware instance IDs.
/// `None` represents an unassigned cell, not FUNCSEL 31 (NULL).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Signal {
    None,
    Uart(u8, UartSignal),
    I2c(u8, I2cSignal),
    Spi(u8, SpiSignal),
    Pwm(u8, PwmSignal),
    I2s(u8, I2sSignal),
    Sdio(u8, SdioSignal),
    Dpi(DpiSignal),
    Gpclk(u8),
    MipiDsiTe(u8),
    AudioOut(AudioOutSignal),
    AudioIn(AudioInSignal),
    SysRio(u8),
    ProcRio(u8),
    Pio(u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UartSignal {
    Tx,
    Rx,
    Cts,
    Rts,
    IrRx,
    IrTx,
    Ri,
    Dtr,
    Dcd,
    Dsr,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum I2cSignal {
    Sda,
    Scl,
}

/// MOSI/MISO use bus direction names, including on slave instance SPI4.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpiSignal {
    Sclk,
    Mosi,
    Miso,
    Sio2,
    Sio3,
    Cs(u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PwmSignal {
    Channel0,
    Channel1,
    Channel2,
    Channel3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum I2sSignal {
    Sclk,
    Ws,
    Sdi(u8),
    Sdo(u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SdioSignal {
    Clk,
    Cmd,
    Dat(u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DpiSignal {
    Pclk,
    De,
    Vsync,
    Hsync,
    D(u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioOutSignal {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioInSignal {
    Clk,
    Dat0,
    Dat1,
}

/// Look up FUNCSEL (a0..a8). Invalid pins, absent signals, and `None` return `None`.
pub fn funcsel(pin: u8, signal: Signal) -> Option<u8> {
    if signal == Signal::None {
        return None;
    }
    BANK0
        .get(usize::from(pin))?
        .iter()
        .position(|&entry| entry == signal)
        .map(|index| index as u8)
}

/// Candidate GPIO numbers in ascending order, without allocation.
/// Nonexistent instances/lanes and `Signal::None` produce an empty iterator.
pub fn candidate_pins(signal: Signal) -> impl Iterator<Item = u8> {
    (0..28).filter(move |&pin| funcsel(pin, signal).is_some())
}
