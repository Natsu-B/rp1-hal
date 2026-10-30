//! Clock authority (`clock.authority = "rp1"`): selects the official clk_sys and reports the
//! rates the drivers must use.
//!
//! After a loader reset RP1 is at cold defaults; [`crate::cold::clocks`] locks PLL_SYS
//! (1 GHz VCO, PRIM /5 /1 = 200 MHz) and starts CLK_UART = XOSC / 1 (HW 2026-10-28). clk_sys
//! is still on XOSC (50 MHz) at that point (HW 2026-10-29: SysTick 50154 cycles / 1004 µs).
//! [`Authority::init`] then moves the clk_sys mux to PLL_SYS, the 200 MHz the official RP1
//! firmware runs at.
//!
//! The switch is the HW-proven PoC sequence (the PoC HAL's `clk_sys::switch`): with PLL_SYS
//! locked on the admitted tuple and CLK_SYS_DIV_INT = 1, write only CLK_SYS_CTRL.SRC[1:0] = 2
//! and poll CLK_SYS_SEL for the one-hot value 4. HW 2026-10-22: 200.000 MHz measured on two
//! boots, and a live XOSC ⇄ PLL_SYS switch from Linux devmem was glitch-free. DOC: Linux
//! clk-rp1 lists clk_sys parents {xosc, -, pll_sys}, `clk_src_mask = 0x3`,
//! `max_freq = 200 MHz`. No PLL value, divider or
//! other clock is written here, and no rate above 200 MHz is possible through this API.
//!
//! What follows clk_sys (HW): the M3 cores and the DesignWare I2C `ic_clk`. What does not:
//! the 1 MHz timer (HW), CLK_UART (separate generator, XOSC / 1) and PLL_SYS PRI_PH (the
//! 100 MHz `uart_apb` that SCMI reports, a PLL output independent of the clk_sys mux, DOC).
//!
//! proc1: only proc0 holds the [`Authority`], so a driver owned by proc1 (ADC, I2C, UART
//! declared under `[proc1]`) gets its [`Clocks`] from [`Clocks::from_proc0`] /
//! [`Clocks::wait_from_proc0`]. [`Authority::init`] publishes the value it returns in a
//! `.shared` cell before returning, so proc1 can only ever obtain a `Clocks` that proc0's
//! init created; launch proc1 after init (or let proc1 wait).
use crate::{Error, read};
use core::sync::atomic::{AtomicU32, Ordering};

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
    linux: crate::linux_blocks::LinuxBlocks,
}

/// Frequencies the drivers rely on. Only [`Authority::init`] creates one, from the registers;
/// proc1 receives a copy of that value through [`Clocks::from_proc0`].
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
    #[inline(never)]
    pub(crate) fn still_current(&self) -> bool {
        self.sys_current_strict() && uart_hz() == Some(self.uart_hz)
    }
    /// clk_sys alone still decodes to [`Clocks::sys_hz`] ([`sys_hz`]: CTRL/DIV/SEL and the
    /// PLL_SYS tuple, read twice). The SPI and I2C init and reconfiguration check: their SCLK
    /// and `ic_clk` depend on clk_sys only, so a different clk_uart does not reject them.
    #[inline(never)]
    pub(crate) fn sys_current_strict(&self) -> bool {
        sys_hz() == Some(self.sys_hz)
    }
    /// The per-call check of the SPI and I2C bus calls: clk_sys is still [`Clocks::sys_hz`],
    /// judged from one read each of CLK_SYS_SEL (the source actually selected) and
    /// CLK_SYS_DIV_INT, plus PLL_SYS CS (lock), FBDIV_INT and PRIM only when SEL is PLL_SYS.
    /// 2 to 5 register reads; clk_uart is not read. One out-of-line copy for SPI and I2C,
    /// placed in ISRAM (instruction fetch from shared SRAM is the slow path).
    ///
    /// Like any check before a transfer it cannot stop a change made while the transfer runs.
    /// It also does not re-read what init verified and nothing in this crate changes later:
    /// CLK_SYS_CTRL (SEL follows it), PLL_SYS PWR and FBDIV_FRAC. [`Clocks::sys_current_strict`]
    /// at init and reconfiguration reads them.
    #[rp1_rt::isram]
    pub(crate) fn sys_current(&self) -> bool {
        sys_is(self.sys_hz, read)
    }
    /// `sys` / `uart` as [`sys_hz`] / [`uart_hz`] read them equal this value.
    #[cfg(test)]
    pub(crate) fn matches(&self, sys: Option<u32>, uart: Option<u32>) -> bool {
        sys == Some(self.sys_hz) && uart == Some(self.uart_hz)
    }
    #[cfg(test)]
    pub(crate) const fn for_test(sys_hz: u32, uart_hz: u32) -> Self {
        Self { sys_hz, uart_hz }
    }
}

/// [`Authority::init`]'s result for proc1: written by proc0 only (init), read by proc1 only.
/// Plain acquire/release loads and stores, no LDREX/STREX (shared SRAM). `.shared` is loaded
/// with the image (`ready` = 0), and the value is published at most once per image load:
/// [`Authority::init`] consumes the one `Authority`. The `withdraw` at the start of init is
/// defensive only; readers do not re-check `ready`, so this is not a protocol for publishing
/// a second value.
struct Handoff {
    ready: AtomicU32,
    sys_hz: AtomicU32,
    uart_hz: AtomicU32,
}
const READY: u32 = 0x434c_4b53;

#[unsafe(link_section = ".shared")]
static HANDOFF: Handoff = Handoff::new();

impl Handoff {
    const fn new() -> Self {
        Self {
            ready: AtomicU32::new(0),
            sys_hz: AtomicU32::new(0),
            uart_hz: AtomicU32::new(0),
        }
    }
    fn withdraw(&self) {
        self.ready.store(0, Ordering::Release);
    }
    /// Only called by [`Authority::init`] with the value it is about to return.
    fn publish(&self, clocks: &Clocks) {
        self.sys_hz.store(clocks.sys_hz, Ordering::Relaxed);
        self.uart_hz.store(clocks.uart_hz, Ordering::Relaxed);
        self.ready.store(READY, Ordering::Release);
        #[cfg(target_arch = "arm")]
        unsafe {
            core::arch::asm!("dsb sy", "sev", options(nostack, preserves_flags));
        }
    }
    fn take(&self) -> Option<Clocks> {
        if self.ready.load(Ordering::Acquire) != READY {
            return None;
        }
        Some(Clocks {
            sys_hz: self.sys_hz.load(Ordering::Relaxed),
            uart_hz: self.uart_hz.load(Ordering::Relaxed),
        })
    }
}

impl Clocks {
    /// proc1's copy of the [`Clocks`] that proc0's [`Authority::init`] returned. Takes the
    /// proc1 token: proc0 already has its own value.
    ///
    /// - [`Error::NotReady`]: init has not succeeded (yet); nothing is published.
    /// - [`Error::Precondition`]: init published a value, but the clock tree no longer
    ///   matches it (the same error the drivers return for a stale `Clocks`).
    pub fn from_proc0(_core: &rp1_rt::Proc1) -> Result<Clocks, Error> {
        published(&HANDOFF, Clocks::still_current)
    }

    /// [`Clocks::from_proc0`], polled for up to `timeout_us` (64-bit 1 MHz timer, which runs
    /// from the cold bring-up on). Tries at least once, also with `timeout_us` = 0.
    /// [`Error::NotReady`] if proc0's init has not published by then; [`Error::Precondition`]
    /// at once if it has published but the tree no longer matches (waiting cannot fix that).
    pub fn wait_from_proc0(core: &rp1_rt::Proc1, timeout_us: u32) -> Result<Clocks, Error> {
        let timer = crate::timer::Timer::new();
        wait_until(timeout_us, || timer.now(), || Self::from_proc0(core))
    }
}

/// The published value, checked against the current tree by `current`.
fn published(handoff: &Handoff, current: impl Fn(&Clocks) -> bool) -> Result<Clocks, Error> {
    let clocks = handoff.take().ok_or(Error::NotReady)?;
    if current(&clocks) {
        Ok(clocks)
    } else {
        Err(Error::Precondition)
    }
}

/// Retries `attempt` while it reports [`Error::NotReady`], until `now() - start >= timeout_us`;
/// then one last attempt (a publish that lands at the deadline still counts). The 64-bit
/// difference cannot wrap within any `u32` timeout, so every timeout up to `u32::MAX` ends.
fn wait_until(
    timeout_us: u32,
    mut now: impl FnMut() -> u64,
    mut attempt: impl FnMut() -> Result<Clocks, Error>,
) -> Result<Clocks, Error> {
    let start = now();
    loop {
        match attempt() {
            Err(Error::NotReady) => {}
            done => return done,
        }
        if now().wrapping_sub(start) >= u64::from(timeout_us) {
            return attempt();
        }
        core::hint::spin_loop();
    }
}

/// [`Authority::init`]'s publication rule: `ready` is cleared first and the value is published
/// only if `body` succeeds.
fn publish_on_success<S>(
    handoff: &Handoff,
    body: impl FnOnce() -> Result<(Clocks, S), Error>,
) -> Result<(Clocks, S), Error> {
    handoff.withdraw();
    let (clocks, rest) = body()?;
    handoff.publish(&clocks);
    Ok((clocks, rest))
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
        Self::take_with(&mut read)
    }
    fn take_with(read: &mut impl FnMut(usize) -> u32) -> Self {
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
        self.cs == PLL_CS_LOCKED
            && self.pwr == 4
            && self.fb_int == PLL_FBDIV_INT
            && self.fb_frac == 0
            && self.prim & !PLL_PRIM_PRI_PH == PLL_PRIM
    }
}
/// PLL_SYS CS locked with REFDIV 1, FBDIV_INT 20, PRIM /5 /1 without PRI_PH (bit 4).
const PLL_CS_LOCKED: u32 = 0x8000_0001;
const PLL_FBDIV_INT: u32 = 20;
const PLL_PRIM: u32 = 0x0005_1000;
const PLL_PRIM_PRI_PH: u32 = 0x10;

/// [`Clocks::sys_current`] over the register reader `read`: each register is read at most once,
/// the PLL ones only when CLK_SYS_SEL selects PLL_SYS.
#[inline(always)]
fn sys_is(hz: u32, mut read: impl FnMut(usize) -> u32) -> bool {
    if read(CLK_SYS_DIV_INT) != 1 {
        return false;
    }
    match read(CLK_SYS_SEL) {
        SEL_XOSC => hz == XOSC_HZ,
        SEL_PLL_SYS => {
            hz == PLL_SYS_HZ
                && read(PLL_SYS_CS) == PLL_CS_LOCKED
                && read(PLL_SYS_FBDIV_INT) == PLL_FBDIV_INT
                && read(PLL_SYS_PRIM) & !PLL_PRIM_PRI_PH == PLL_PRIM
        }
        _ => false,
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

fn sys_once(read: &mut impl FnMut(usize) -> u32) -> Option<u32> {
    let (ctrl, div, sel) = (read(CLK_SYS_CTRL), read(CLK_SYS_DIV_INT), read(CLK_SYS_SEL));
    decode_sys(ctrl, div, sel, &Pll::take_with(read))
}

/// [`sys_hz`] over the register reader `read`.
fn sys_hz_with(mut read: impl FnMut(usize) -> u32) -> Option<u32> {
    let a = sys_once(&mut read);
    (a == sys_once(&mut read)).then_some(a).flatten()
}

/// Current clk_sys rate from the registers (read twice), `None` if unknown or changing.
pub fn sys_hz() -> Option<u32> {
    sys_hz_with(read)
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
    /// `linux` is the set of blocks the declaration leaves to Linux.
    pub const unsafe fn new(linux: crate::linux_blocks::LinuxBlocks) -> Self {
        Self { linux }
    }

    /// Selects clk_sys = `sys_hz` (the declared `clk_sys_hz`; only the official 200 MHz is
    /// accepted), checks PLL_SYS / CLK_UART, and starts the SCMI service that publishes the
    /// RP1 clocks to Linux (authority = rp1 means Linux's DT points its RP1 clock users at
    /// SCMI, so the server is part of the obligation). It also brings up the blocks Linux owns
    /// ([`crate::linux_blocks`]: PLL_SYS_SEC, their clock generators and resets), since Linux
    /// can no longer do that through clk-rp1. Poll the returned [`crate::scmi::Service`] from
    /// proc0's main loop; [`crate::scmi::Service::linux_blocks`] reports the reset release.
    ///
    /// Call after [`crate::cold::clocks`] and before any driver that depends on clk_sys
    /// (I2C). The returned [`Clocks`] is read back from the registers, not from `sys_hz`.
    /// Already on PLL_SYS (warm restart): nothing is written.
    ///
    /// On success, and only then, the same [`Clocks`] is published once for proc1
    /// ([`Clocks::from_proc0`]); on any error proc1 keeps seeing [`Error::NotReady`].
    pub fn init(self, sys_hz: u32) -> Result<(Clocks, crate::scmi::Service), Error> {
        publish_on_success(&HANDOFF, || {
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
            let linux = crate::linux_blocks::bring_up(&mut crate::linux_blocks::Mmio, self.linux)?;
            // SAFETY: `self` is the one clock authority, issued to proc0.
            let scmi = unsafe { crate::scmi::Service::new(linux, self.linux.held_clocks()) };
            Ok((clocks, scmi))
        })
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
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

    /// Review 6: PLL_SYS and CLK_UART on the admitted tuple (what `running_tuple_admitted`
    /// checks) while clk_sys runs from XOSC is not the 200 MHz `Clocks`.
    #[test]
    fn sys_on_xosc_with_the_admitted_pll_and_uart_is_stale() {
        let uart = decode_uart(0x1000_0840, 1, 1);
        assert!(LOCKED.is_200mhz() && uart == Some(XOSC_HZ));
        assert!(PUBLISHED.matches(decode_sys(SRC_PLL_SYS, 1, SEL_PLL_SYS, &LOCKED), uart));
        assert!(!PUBLISHED.matches(decode_sys(SRC_XOSC, 1, SEL_XOSC, &LOCKED), uart));
        assert!(!PUBLISHED.matches(None, uart));
        assert!(!PUBLISHED.matches(Some(PLL_SYS_HZ), None));
    }

    /// clk_sys at 200 MHz from PLL_SYS and clk_uart = XOSC, as `cold::clocks` + init leave them.
    const TREE: [(usize, u32); 11] = [
        (CLK_SYS_CTRL, 0x800 | SRC_PLL_SYS),
        (CLK_SYS_DIV_INT, 1),
        (CLK_SYS_SEL, SEL_PLL_SYS),
        (CLK_UART_CTRL, 0x1000_0840),
        (CLK_UART_DIV_INT, 1),
        (CLK_UART_SEL, 1),
        (PLL_SYS_CS, 0x8000_0001),
        (PLL_SYS_PWR, 4),
        (PLL_SYS_FBDIV_INT, 20),
        (PLL_SYS_FBDIV_FRAC, 0),
        (PLL_SYS_PRIM, 0x0005_1010),
    ];
    /// (fast check, addresses it read in order, strict check) for `hz` on `tree`.
    fn sys_checks(tree: &[(usize, u32)], hz: u32) -> (bool, std::vec::Vec<usize>, bool) {
        let value = |a: usize| tree.iter().find(|r| r.0 == a).unwrap().1;
        let mut reads = std::vec::Vec::new();
        let fast = sys_is(hz, |a| {
            reads.push(a);
            value(a)
        });
        (fast, reads, sys_hz_with(value) == Some(hz))
    }
    fn with(changes: &[(usize, u32)]) -> [(usize, u32); 11] {
        let mut t = TREE;
        for &(a, v) in changes {
            t.iter_mut().find(|r| r.0 == a).unwrap().1 = v;
        }
        t
    }

    /// Review 7: SPI and I2C depend on clk_sys only. A clk_uart change does not reject them;
    /// a clk_sys SEL or DIV change (or a PLL_SYS change under SEL = PLL_SYS) does. The fast
    /// check reads each register once and never a clk_uart one.
    #[test]
    fn sys_checks_ignore_clk_uart_and_reject_sys_changes() {
        let (fast, reads, strict) = sys_checks(&TREE, PLL_SYS_HZ);
        assert!(fast && strict);
        assert_eq!(
            reads,
            [
                CLK_SYS_DIV_INT,
                CLK_SYS_SEL,
                PLL_SYS_CS,
                PLL_SYS_FBDIV_INT,
                PLL_SYS_PRIM
            ]
        );
        for uart in [
            &[(CLK_UART_CTRL, 0)][..],
            &[(CLK_UART_DIV_INT, 2)],
            &[(CLK_UART_SEL, 2)],
            &[(CLK_UART_CTRL, 0x1000_0820), (CLK_UART_SEL, 4)],
        ] {
            let (fast, _, strict) = sys_checks(&with(uart), PLL_SYS_HZ);
            assert!(fast && strict, "{uart:?}");
            // The full check the other drivers use does see it.
            let t = with(uart);
            let u = |a: usize| t.iter().find(|r| r.0 == a).unwrap().1;
            assert_eq!(
                decode_uart(u(CLK_UART_CTRL), u(CLK_UART_DIV_INT), u(CLK_UART_SEL)),
                None
            );
        }
        for sys in [
            &[(CLK_SYS_SEL, SEL_XOSC)][..],
            &[(CLK_SYS_SEL, 2)],
            &[(CLK_SYS_SEL, 0)],
            &[(CLK_SYS_DIV_INT, 2)],
            &[(CLK_SYS_DIV_INT, 0)],
            &[(PLL_SYS_CS, 1)],
            &[(PLL_SYS_FBDIV_INT, 25)],
            // PRIM /5 -> /4 under SEL = PLL_SYS: 250 MHz.
            &[(PLL_SYS_PRIM, 0x0004_1010)],
        ] {
            let (fast, _, strict) = sys_checks(&with(sys), PLL_SYS_HZ);
            assert!(!fast && !strict, "{sys:?}");
        }
        // On XOSC: two reads, no PLL register, and only a 50 MHz value matches.
        let xosc = with(&[(CLK_SYS_CTRL, SRC_XOSC), (CLK_SYS_SEL, SEL_XOSC)]);
        let (fast, reads, strict) = sys_checks(&xosc, XOSC_HZ);
        assert!(fast && strict);
        assert_eq!(reads, [CLK_SYS_DIV_INT, CLK_SYS_SEL]);
        assert!(!sys_checks(&xosc, PLL_SYS_HZ).0);
        assert!(!sys_checks(&TREE, XOSC_HZ).0);
        // XOSC needs no PLL: an unlocked PLL_SYS does not matter there.
        let xosc_unlocked = with(&[
            (CLK_SYS_CTRL, SRC_XOSC),
            (CLK_SYS_SEL, SEL_XOSC),
            (PLL_SYS_CS, 0),
        ]);
        assert!(sys_checks(&xosc_unlocked, XOSC_HZ).0);
    }

    /// Review 8: what only the strict check (init, reconfiguration) reads. One register at a
    /// time: the fast check still passes, the strict one rejects.
    #[test]
    fn only_the_strict_check_sees_ctrl_pwr_and_frac() {
        for change in [
            // CTRL asks for XOSC while SEL still shows PLL_SYS (mid-switch).
            (CLK_SYS_CTRL, 0x800 | SRC_XOSC),
            (PLL_SYS_PWR, 0),
            (PLL_SYS_FBDIV_FRAC, 1),
        ] {
            let (fast, reads, strict) = sys_checks(&with(&[change]), PLL_SYS_HZ);
            assert!(fast && !strict, "{change:?}");
            assert!(!reads.contains(&change.0));
        }
    }

    #[test]
    fn handoff_is_empty_until_published() {
        let h = Handoff::new();
        assert_eq!(h.take(), None);
        let c = Clocks {
            sys_hz: PLL_SYS_HZ,
            uart_hz: XOSC_HZ,
        };
        h.publish(&c);
        assert_eq!(h.take(), Some(c));
        h.withdraw();
        assert_eq!(h.take(), None);
    }

    const PUBLISHED: Clocks = Clocks {
        sys_hz: PLL_SYS_HZ,
        uart_hz: XOSC_HZ,
    };

    #[test]
    fn unpublished_is_not_ready_and_stale_is_precondition() {
        let h = Handoff::new();
        assert_eq!(published(&h, |_| true), Err(Error::NotReady));
        h.publish(&PUBLISHED);
        assert_eq!(published(&h, |_| true), Ok(PUBLISHED));
        assert_eq!(published(&h, |_| false), Err(Error::Precondition));
    }

    /// Runs `wait_until` against a never-published handoff with a timer that starts at
    /// `start` and advances `step` per read. Returns (result, attempts, elapsed at the end).
    fn wait_never_ready(
        timeout_us: u32,
        start: u64,
        step: u64,
    ) -> (Result<Clocks, Error>, u32, u64) {
        let mut clock = start;
        let mut attempts = 0;
        let r = wait_until(
            timeout_us,
            || {
                let t = clock;
                clock = clock.wrapping_add(step);
                t
            },
            || {
                attempts += 1;
                Err(Error::NotReady)
            },
        );
        (r, attempts, clock.wrapping_sub(start).wrapping_sub(step))
    }

    #[test]
    fn wait_zero_timeout_still_tries() {
        let (r, attempts, elapsed) = wait_never_ready(0, 0, 1);
        assert_eq!(r, Err(Error::NotReady));
        // One try, the deadline check, the final try.
        assert_eq!(attempts, 2);
        assert_eq!(elapsed, 1);
    }

    #[test]
    fn wait_normal_timeout_ends_at_the_deadline() {
        let (r, attempts, elapsed) = wait_never_ready(10_000, 5, 1_000);
        assert_eq!(r, Err(Error::NotReady));
        assert_eq!(elapsed, 10_000);
        assert_eq!(attempts, 11);
    }

    #[test]
    fn wait_across_timer_wrap() {
        // The low word wraps (the case a 32-bit RAWL read had to handle) ...
        let (r, _, elapsed) = wait_never_ready(10_000, 0xffff_ff00, 1_000);
        assert_eq!(r, Err(Error::NotReady));
        assert_eq!(elapsed, 10_000);
        // ... and so does the full 64-bit counter.
        let (r, _, elapsed) = wait_never_ready(10_000, u64::MAX - 500, 1_000);
        assert_eq!(r, Err(Error::NotReady));
        assert_eq!(elapsed, 10_000);
    }

    #[test]
    fn wait_near_max_timeout_cannot_skip_the_deadline() {
        // Coarse polling steps over the last microseconds before u32::MAX; with a 32-bit
        // difference the elapsed time would wrap past the deadline and never end.
        let timeout = u32::MAX - 10;
        let (r, _, elapsed) = wait_never_ready(timeout, 0x1234, 0x4000_0000);
        assert_eq!(r, Err(Error::NotReady));
        assert!(elapsed >= u64::from(timeout) && elapsed < u64::from(timeout) + 0x4000_0000);
    }

    #[test]
    fn wait_max_timeout_ends() {
        let (r, _, elapsed) = wait_never_ready(u32::MAX, 7, 0x1000_0000);
        assert_eq!(r, Err(Error::NotReady));
        assert!(elapsed >= u64::from(u32::MAX));
        let (r, attempts, elapsed) = wait_never_ready(u32::MAX, 0, u64::from(u32::MAX));
        assert_eq!(
            (r, attempts, elapsed),
            (Err(Error::NotReady), 2, u64::from(u32::MAX))
        );
    }

    #[test]
    fn wait_returns_at_once_on_success_or_stale() {
        for want in [
            Ok(PUBLISHED),
            Err(Error::Precondition),
            Err(Error::Hardware),
        ] {
            let mut attempts = 0;
            let r = wait_until(
                u32::MAX,
                || 0,
                || {
                    attempts += 1;
                    want
                },
            );
            assert_eq!((r, attempts), (want, 1));
        }
    }

    #[test]
    fn wait_sees_a_publish_during_the_wait() {
        let h = Handoff::new();
        let mut clock = 0u64;
        let r = wait_until(
            10_000,
            || {
                clock += 100;
                if clock == 1_000 {
                    h.publish(&PUBLISHED);
                }
                clock
            },
            || published(&h, |_| true),
        );
        assert_eq!(r, Ok(PUBLISHED));
    }

    #[test]
    fn init_publishes_only_on_success() {
        let h = Handoff::new();
        for e in [Error::Invalid, Error::Precondition, Error::Hardware] {
            assert_eq!(publish_on_success::<()>(&h, || Err(e)), Err(e));
            assert_eq!(h.ready.load(Ordering::Relaxed), 0);
            assert_eq!(h.take(), None);
        }
        assert_eq!(
            publish_on_success(&h, || Ok((PUBLISHED, 9))),
            Ok((PUBLISHED, 9))
        );
        assert_eq!(h.ready.load(Ordering::Relaxed), READY);
        assert_eq!(h.take(), Some(PUBLISHED));
        // A failing init withdraws before it fails (defensive; init runs once per image).
        assert_eq!(
            publish_on_success::<()>(&h, || Err(Error::Hardware)),
            Err(Error::Hardware)
        );
        assert_eq!(h.ready.load(Ordering::Relaxed), 0);
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
