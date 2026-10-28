//! Clock authority (`clock.authority = "rp1"`): selects the official clk_sys and reports the
//! rates the drivers must use.
//!
//! After a loader reset RP1 is at cold defaults; [`crate::cold::clocks`] locks PLL_SYS
//! (1 GHz VCO, PRIM /5 /1 = 200 MHz) and starts CLK_UART = XOSC / 1 (HW: hwrun1). clk_sys is
//! still on XOSC (50 MHz) at that point (HW: SysTick 50154 cycles / 1004 µs,
//! `<lab-log>`). [`Authority::init`] then moves the clk_sys mux to
//! PLL_SYS, the 200 MHz the official RP1 firmware runs at.
//!
//! The switch is the HW-proven PoC sequence (`RP1-hal-PoC-openamp-20260929`
//! `codex/clk-sys-200-20261022` `clk_sys::switch`): with PLL_SYS locked on the admitted tuple
//! and CLK_SYS_DIV_INT = 1, write only CLK_SYS_CTRL.SRC[1:0] = 2 and poll CLK_SYS_SEL for the
//! one-hot value 4. HW: 200.000 MHz measured on two boots (`<lab-log>`),
//! and a live XOSC ⇄ PLL_SYS switch from Linux devmem was glitch-free
//! (`<lab-log>`). DOC: Linux clk-rp1 lists clk_sys parents
//! {xosc, -, pll_sys}, `clk_src_mask = 0x3`, `max_freq = 200 MHz`. No PLL value, divider or
//! other clock is written here, and no rate above 200 MHz is possible through this API.
//!
//! What follows clk_sys (HW): the M3 cores and the DesignWare I2C `ic_clk`. What does not:
//! the 1 MHz timer (HW), CLK_UART (separate generator, XOSC / 1) and PLL_SYS PRI_PH (the
//! 100 MHz `uart_apb` that SCMI reports, a PLL output independent of the clk_sys mux, DOC).
use crate::{Error, read};

const CLK_SYS_CTRL: usize = 0x4001_8014;
const CLK_SYS_DIV_INT: usize = 0x4001_8018;
const CLK_SYS_SEL: usize = 0x4001_8020;
const CLK_UART_CTRL: usize = 0x4001_8054;
const CLK_UART_DIV_INT: usize = 0x4001_8058;
const CLK_UART_SEL: usize = 0x4001_8060;
const PLL_SYS_CS: usize = 0x4002_0000;
const PLL_SYS_PWR: usize = 0x4002_0004;
const PLL_SYS_FBDIV_INT: usize = 0x4002_0008;
const PLL_SYS_FBDIV_FRAC: usize = 0x4002_000c;
const PLL_SYS_PRIM: usize = 0x4002_0010;

pub const XOSC_HZ: u32 = 50_000_000;
/// PLL_SYS primary output: 50 MHz × 20 / 5 / 1. Also the official clk_sys rate.
pub const PLL_SYS_HZ: u32 = 200_000_000;

/// CLK_SYS_CTRL.SRC values and the matching one-hot CLK_SYS_SEL (DOC clk-rp1, HW SEL 1 / 4).
const SRC_XOSC: u32 = 0;
const SRC_PLL_SYS: u32 = 2;
const SEL_XOSC: u32 = 1 << SRC_XOSC;
const SEL_PLL_SYS: u32 = 1 << SRC_PLL_SYS;
const SWITCH_TIMEOUT_US: u32 = 10_000;

/// Proof that this image holds clock authority (`clock.authority = "rp1"`), issued to proc0.
pub struct Authority {
    _private: (),
}

/// Frequencies the drivers rely on. Only [`Authority::init`] creates one, from the registers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Clocks {
    sys_hz: u32,
    uart_hz: u32,
}

impl Clocks {
    /// clk_sys: the M3 core clock and the I2C `ic_clk`.
    pub const fn sys_hz(&self) -> u32 {
        self.sys_hz
    }
    /// clk_uart (UART0..5 functional clock).
    pub const fn uart_hz(&self) -> u32 {
        self.uart_hz
    }
    /// The tree still matches what this value was created from (drivers check before use).
    pub(crate) fn still_current(&self) -> bool {
        sys_hz() == Some(self.sys_hz) && uart_hz() == Some(self.uart_hz)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Pll {
    cs: u32,
    pwr: u32,
    fb_int: u32,
    fb_frac: u32,
    prim: u32,
}

impl Pll {
    fn take() -> Self {
        Self {
            cs: read(PLL_SYS_CS),
            pwr: read(PLL_SYS_PWR),
            fb_int: read(PLL_SYS_FBDIV_INT),
            fb_frac: read(PLL_SYS_FBDIV_FRAC),
            prim: read(PLL_SYS_PRIM),
        }
    }
    /// Locked, 50 MHz × 20 = 1 GHz VCO, PRIM /5 /1 (PRI_PH bit 4 either way). The only tuple
    /// written by `cold::clocks` and observed on hardware.
    fn is_200mhz(&self) -> bool {
        self.cs == 0x8000_0001
            && self.pwr == 4
            && self.fb_int == 20
            && self.fb_frac == 0
            && self.prim & !0x10 == 0x0005_1000
    }
}

/// clk_sys rate from (CTRL, DIV_INT, SEL, PLL_SYS), or `None` for anything not understood.
fn decode_sys(ctrl: u32, div: u32, sel: u32, pll: &Pll) -> Option<u32> {
    if div != 1 {
        return None;
    }
    match (ctrl & 3, sel) {
        (SRC_XOSC, SEL_XOSC) => Some(XOSC_HZ),
        (SRC_PLL_SYS, SEL_PLL_SYS) if pll.is_200mhz() => Some(PLL_SYS_HZ),
        _ => None,
    }
}

fn sys_once() -> Option<u32> {
    decode_sys(
        read(CLK_SYS_CTRL),
        read(CLK_SYS_DIV_INT),
        read(CLK_SYS_SEL),
        &Pll::take(),
    )
}

/// Current clk_sys rate from the registers (read twice), `None` if unknown or changing.
pub fn sys_hz() -> Option<u32> {
    let a = sys_once();
    (a == sys_once()).then_some(a).flatten()
}

fn decode_uart(ctrl: u32, div: u32, sel: u32) -> Option<u32> {
    // The exact CTRL value `cold::clocks` leaves (ENABLE bit 11, AUXSRC 2 << 5) as read on HW.
    (ctrl == 0x1000_0840 && div == 1 && sel == 1).then_some(XOSC_HZ)
}

/// Current clk_uart rate from the registers (read twice), `None` if unknown or changing.
pub fn uart_hz() -> Option<u32> {
    let once = || {
        decode_uart(
            read(CLK_UART_CTRL),
            read(CLK_UART_DIV_INT),
            read(CLK_UART_SEL),
        )
    };
    let a = once();
    (a == once()).then_some(a).flatten()
}

/// PLL_SYS and CLK_UART are on the tuple `cold::clocks` programs (clk_sys mux not included).
pub(crate) fn running_tuple_admitted() -> bool {
    let a = Pll::take();
    a == Pll::take() && a.is_200mhz() && uart_hz() == Some(XOSC_HZ)
}

trait Registers {
    fn read(&mut self, address: usize) -> u32;
    fn write_ctrl(&mut self, value: u32);
    fn now_us(&mut self) -> u32;
}
struct Mmio;
impl Registers for Mmio {
    fn read(&mut self, address: usize) -> u32 {
        read(address)
    }
    fn write_ctrl(&mut self, value: u32) {
        crate::write(CLK_SYS_CTRL, value);
    }
    fn now_us(&mut self) -> u32 {
        read(0x400a_c028)
    }
}

/// Moves clk_sys from XOSC to PLL_SYS (or accepts it already there). Returns the new rate.
fn select_pll_sys(r: &mut impl Registers) -> Result<u32, Error> {
    let pll = Pll {
        cs: r.read(PLL_SYS_CS),
        pwr: r.read(PLL_SYS_PWR),
        fb_int: r.read(PLL_SYS_FBDIV_INT),
        fb_frac: r.read(PLL_SYS_FBDIV_FRAC),
        prim: r.read(PLL_SYS_PRIM),
    };
    let ctrl = r.read(CLK_SYS_CTRL);
    let div = r.read(CLK_SYS_DIV_INT);
    let sel = r.read(CLK_SYS_SEL);
    if !pll.is_200mhz() {
        return Err(Error::Precondition);
    }
    match decode_sys(ctrl, div, sel, &pll) {
        Some(PLL_SYS_HZ) => return Ok(PLL_SYS_HZ),
        Some(XOSC_HZ) => {}
        _ => return Err(Error::Precondition),
    }
    // Only SRC[1:0] changes; the mux is glitchless (HW, live switch from Linux).
    r.write_ctrl((ctrl & !3) | SRC_PLL_SYS);
    let start = r.now_us();
    loop {
        let sel = r.read(CLK_SYS_SEL);
        if sel == SEL_PLL_SYS {
            return Ok(PLL_SYS_HZ);
        }
        if r.now_us().wrapping_sub(start) > SWITCH_TIMEOUT_US {
            // The write was issued: the clock state is unknown, report it as such.
            return Err(Error::Hardware);
        }
    }
}

impl Authority {
    /// # Safety
    /// Issued once to proc0 by the generated resources. Linux must not own the clock generator.
    pub const unsafe fn new() -> Self {
        Self { _private: () }
    }

    /// Selects clk_sys = `sys_hz` (the declared `clk_sys_hz`; only the official 200 MHz is
    /// accepted), checks PLL_SYS / CLK_UART, and starts the SCMI service that publishes the
    /// RP1 clocks to Linux (authority = rp1 means Linux's DT points its RP1 clock users at
    /// SCMI, so the server is part of the obligation). Poll the returned
    /// [`crate::scmi::Service`] from proc0's main loop.
    ///
    /// Call after [`crate::cold::clocks`] and before any driver that depends on clk_sys
    /// (I2C). The returned [`Clocks`] is read back from the registers, not from `sys_hz`.
    /// Already on PLL_SYS (warm restart): nothing is written.
    pub fn init(self, sys_hz: u32) -> Result<(Clocks, crate::scmi::Service), Error> {
        if sys_hz != PLL_SYS_HZ {
            return Err(Error::Invalid);
        }
        if !running_tuple_admitted() {
            return Err(Error::Precondition);
        }
        select_pll_sys(&mut Mmio)?;
        let clocks = Clocks {
            sys_hz: self::sys_hz().ok_or(Error::Hardware)?,
            uart_hz: uart_hz().ok_or(Error::Precondition)?,
        };
        if clocks.sys_hz != sys_hz {
            return Err(Error::Hardware);
        }
        // SAFETY: `self` is the one clock authority, issued to proc0.
        let scmi = unsafe { crate::scmi::Service::new() };
        Ok((clocks, scmi))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOCKED: Pll = Pll {
        cs: 0x8000_0001,
        pwr: 4,
        fb_int: 20,
        fb_frac: 0,
        prim: 0x0005_1010,
    };

    #[test]
    fn only_the_observed_pll_tuple_is_200mhz() {
        assert!(LOCKED.is_200mhz());
        assert!(
            Pll {
                prim: 0x0005_1000,
                ..LOCKED
            }
            .is_200mhz()
        );
        for bad in [
            Pll { cs: 1, ..LOCKED },
            Pll { pwr: 5, ..LOCKED },
            Pll {
                fb_int: 21,
                ..LOCKED
            },
            Pll {
                fb_frac: 1,
                ..LOCKED
            },
            Pll {
                prim: 0x0007_7000,
                ..LOCKED
            },
            Pll {
                prim: 0x0004_1010,
                ..LOCKED
            },
        ] {
            assert!(!bad.is_200mhz(), "{bad:?}");
        }
    }

    #[test]
    fn sys_rate_follows_the_selected_source() {
        assert_eq!(decode_sys(0, 1, 1, &LOCKED), Some(50_000_000));
        assert_eq!(decode_sys(2, 1, 4, &LOCKED), Some(200_000_000));
        // Other CTRL bits are preserved, not interpreted.
        assert_eq!(decode_sys(0x800 | 2, 1, 4, &LOCKED), Some(200_000_000));
        // Mid-switch, divided, unlocked PLL, unknown source.
        assert_eq!(decode_sys(2, 1, 1, &LOCKED), None);
        assert_eq!(decode_sys(2, 2, 4, &LOCKED), None);
        assert_eq!(decode_sys(2, 1, 4, &Pll { cs: 1, ..LOCKED }), None);
        assert_eq!(decode_sys(1, 1, 2, &LOCKED), None);
        // XOSC needs no PLL.
        assert_eq!(
            decode_sys(0, 1, 1, &Pll { cs: 0, ..LOCKED }),
            Some(50_000_000)
        );
    }

    #[test]
    fn uart_rate_is_the_xosc_tuple_only() {
        assert_eq!(decode_uart(0x1000_0840, 1, 1), Some(50_000_000));
        assert_eq!(decode_uart(0x0000_0840, 1, 1), None);
        assert_eq!(decode_uart(0x1000_0840, 2, 1), None);
        assert_eq!(decode_uart(0x1000_0840, 1, 2), None);
    }

    struct Fake {
        regs: [(usize, u32); 8],
        sels: [u32; 3],
        sel_reads: usize,
        writes: usize,
        clock: u32,
    }
    impl Fake {
        fn cold() -> Self {
            Self {
                regs: [
                    (PLL_SYS_CS, LOCKED.cs),
                    (PLL_SYS_PWR, LOCKED.pwr),
                    (PLL_SYS_FBDIV_INT, LOCKED.fb_int),
                    (PLL_SYS_FBDIV_FRAC, LOCKED.fb_frac),
                    (PLL_SYS_PRIM, LOCKED.prim),
                    (CLK_SYS_CTRL, 0),
                    (CLK_SYS_DIV_INT, 1),
                    (CLK_SYS_SEL, 1),
                ],
                sels: [1, 0, 4],
                sel_reads: 0,
                writes: 0,
                clock: 0,
            }
        }
        fn set(&mut self, address: usize, value: u32) {
            self.regs.iter_mut().find(|r| r.0 == address).unwrap().1 = value;
        }
    }
    impl Registers for Fake {
        fn read(&mut self, address: usize) -> u32 {
            if address == CLK_SYS_SEL && self.writes > 0 {
                let v = self.sels[self.sel_reads.min(2)];
                self.sel_reads += 1;
                return v;
            }
            self.regs.iter().find(|r| r.0 == address).unwrap().1
        }
        fn write_ctrl(&mut self, value: u32) {
            self.set(CLK_SYS_CTRL, value);
            self.writes += 1;
        }
        fn now_us(&mut self) -> u32 {
            self.clock = self.clock.wrapping_add(1_000);
            self.clock
        }
    }

    #[test]
    fn switch_writes_src_only_and_waits_for_one_hot_sel() {
        let mut f = Fake::cold();
        f.set(CLK_SYS_CTRL, 0x800);
        assert_eq!(select_pll_sys(&mut f), Ok(200_000_000));
        assert_eq!(f.writes, 1);
        assert_eq!(f.read(CLK_SYS_CTRL), 0x802);
        assert_eq!(f.sel_reads, 3);
    }

    #[test]
    fn already_on_pll_sys_writes_nothing() {
        let mut f = Fake::cold();
        f.set(CLK_SYS_CTRL, 2);
        f.set(CLK_SYS_SEL, 4);
        assert_eq!(select_pll_sys(&mut f), Ok(200_000_000));
        assert_eq!(f.writes, 0);
    }

    #[test]
    fn refuses_without_writing() {
        for (address, value) in [
            (PLL_SYS_CS, 1),
            (PLL_SYS_FBDIV_INT, 21),
            (PLL_SYS_PRIM, 0x77000),
            (CLK_SYS_DIV_INT, 2),
            (CLK_SYS_SEL, 2),
            (CLK_SYS_CTRL, 1),
        ] {
            let mut f = Fake::cold();
            f.set(address, value);
            assert_eq!(select_pll_sys(&mut f), Err(Error::Precondition));
            assert_eq!(f.writes, 0);
        }
    }

    #[test]
    fn timeout_is_bounded_and_reported_as_hardware() {
        let mut f = Fake::cold();
        f.sels = [1; 3];
        assert_eq!(select_pll_sys(&mut f), Err(Error::Hardware));
        assert_eq!(f.writes, 1);
        assert!(f.sel_reads <= 12);
    }
}
