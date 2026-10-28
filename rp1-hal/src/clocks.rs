//! Clock authority for v0.1: adopt the clock tree that the EEPROM-stage RP1 firmware left behind.
//!
//! v0.1 does not program PLLs or the clk_sys mux. No cold-start sequence for those has been
//! proven on hardware; the PoC only ever *read* PLL_SYS / CLK_UART and admitted one exact tuple
//! (`RP1-hal-PoC-openamp-20260929/crates/rp1-hal/src/clock_adopt.rs`, HW-observed). We check the
//! same tuple and fail closed if anything differs. See docs/design.md §9.
use crate::{Error, read};

const PLL_SYS_CS: usize = 0x4002_0000;
const PLL_SYS_PWR: usize = 0x4002_0004;
const PLL_SYS_FBDIV_INT: usize = 0x4002_0008;
const PLL_SYS_FBDIV_FRAC: usize = 0x4002_000c;
const PLL_SYS_PRIM: usize = 0x4002_0010;
const CLK_UART_CTRL: usize = 0x4001_8054;
const CLK_UART_DIV_INT: usize = 0x4001_8058;
const CLK_UART_SEL: usize = 0x4001_8060;

/// Proof that this image holds clock authority (`clock.authority = "rp1"`), issued to proc0.
pub struct Authority {
    _private: (),
}

/// Frequencies the drivers may rely on after adoption.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Clocks {
    pub sys_hz: u32,
    pub uart_hz: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Snapshot {
    cs: u32,
    pwr: u32,
    fb_int: u32,
    fb_frac: u32,
    prim: u32,
    uart_ctrl: u32,
    uart_div: u32,
    uart_sel: u32,
}

impl Snapshot {
    fn take() -> Self {
        Self {
            cs: read(PLL_SYS_CS),
            pwr: read(PLL_SYS_PWR),
            fb_int: read(PLL_SYS_FBDIV_INT),
            fb_frac: read(PLL_SYS_FBDIV_FRAC),
            prim: read(PLL_SYS_PRIM),
            uart_ctrl: read(CLK_UART_CTRL),
            uart_div: read(CLK_UART_DIV_INT),
            uart_sel: read(CLK_UART_SEL),
        }
    }

    /// 50 MHz XOSC * 20 = 1 GHz VCO, PRIM postdiv 5*1 (PRI_PH either way), CLK_UART = XOSC / 1.
    fn admitted(&self) -> bool {
        self.cs == 0x8000_0001
            && self.pwr == 4
            && self.fb_int == 20
            && self.fb_frac == 0
            && self.prim & !0x10 == 0x0005_1000
            && uart_clock_admitted(self.uart_ctrl, self.uart_div, self.uart_sel)
    }
}

pub(crate) fn uart_clock_admitted(ctrl: u32, div: u32, sel: u32) -> bool {
    ctrl == 0x1000_0840 && div == 1 && sel == 1
}

/// Reads CLK_UART twice; returns whether it matches the 50 MHz XOSC tuple.
pub(crate) fn uart_clock_is_50mhz() -> bool {
    let a = (
        read(CLK_UART_CTRL),
        read(CLK_UART_DIV_INT),
        read(CLK_UART_SEL),
    );
    let b = (
        read(CLK_UART_CTRL),
        read(CLK_UART_DIV_INT),
        read(CLK_UART_SEL),
    );
    a == b && uart_clock_admitted(b.0, b.1, b.2)
}

impl Authority {
    /// # Safety
    /// Issued once to proc0 by the generated resources. Linux must not own the clock generator.
    pub const unsafe fn new() -> Self {
        Self { _private: () }
    }

    /// Admit the existing 200 MHz clk_sys / 50 MHz clk_uart configuration without writing anything.
    ///
    /// `sys_hz` is the declared value (`clk_sys_hz`); only 200 MHz is accepted. The CLK_SYS mux
    /// itself is not checked because its register encoding has not been observed on hardware (HYP).
    pub fn adopt(self, sys_hz: u32) -> Result<Clocks, Error> {
        if sys_hz != 200_000_000 {
            return Err(Error::Invalid);
        }
        let a = Snapshot::take();
        let b = Snapshot::take();
        if a != b || !b.admitted() {
            return Err(Error::Precondition);
        }
        Ok(Clocks {
            sys_hz,
            uart_hz: 50_000_000,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_observed_tuple_is_admitted() {
        let ok = Snapshot {
            cs: 0x8000_0001,
            pwr: 4,
            fb_int: 20,
            fb_frac: 0,
            prim: 0x0005_1010,
            uart_ctrl: 0x1000_0840,
            uart_div: 1,
            uart_sel: 1,
        };
        assert!(ok.admitted());
        assert!(
            Snapshot {
                prim: 0x0005_1000,
                ..ok
            }
            .admitted()
        );
        assert!(
            !Snapshot {
                prim: 0x0007_7000,
                ..ok
            }
            .admitted()
        );
        assert!(!Snapshot { cs: 1, ..ok }.admitted());
        assert!(!Snapshot { uart_sel: 2, ..ok }.admitted());
    }
}
