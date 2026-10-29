//! Bring-up of the RP1 blocks the board leaves to Linux (Ethernet, USB, DMA, ADC, PWM, I2S,
//! audio out, and the UART / I2C / SPI instances the image does not declare).
//!
//! The stock RP1 firmware releases every reset and starts the clocks of these blocks before Linux
//! probes them (HW, stock firmware: RESETS CTRL0/1/2 = 0, PLL_SYS_SEC = 0x8000_0800 (/8 = 125 MHz),
//! CLK_DMA / CLK_ETH / CLK_ETH_TSU / the four USB host clocks = AUXSRC 0, DIV 1, enabled).
//! rp1-hal starts from a reset RP1 instead ([`crate::cold`]), so with `clock.authority = "rp1"`
//! it does the same for the blocks Linux owns. Which resets are Linux's comes from the
//! declaration (rp1-build generates the [`LinuxBlocks`] masks); [`ALLOWED`] bounds them so the
//! processors, PCIe, the PLLs, SYSCFG and the GPIO banks RP1 uses are never touched here.
//!
//! After this, Linux drives the clocks of its blocks through SCMI ([`crate::scmi`]); the USB
//! host clocks have no Linux consumer (the DT USB nodes carry no clocks) and stay on.
use crate::scmi::{Generator, Kind, RP1_CLOCKS, Registers};

const RESETS: usize = 0x4001_4000;
const RESETS_CLR: usize = RESETS + 0x3000;
const RESETS_DONE: usize = RESETS + 0x18;
const PLL_SYS_SEC: usize = 0x4002_0014;
/// IMPL (bit 31) set, RST (bit 16) clear, DIV (12:8) = 8: 1 GHz / 8 = 125 MHz (HW, stock firmware).
pub const PLL_SYS_SEC_125MHZ: u32 = 0x8000_0800;
const SEC_IMPL: u32 = 1 << 31;
const SEC_RST: u32 = 1 << 16;
const SEC_DIV_MASK: u32 = 0x1f << 8;

/// Reset bits (bank 0, 1, 2) that may be released for Linux (regdb resets; ADC = 0:0 and
/// DMA = 0:3 are inferred from the bit order, and the stock firmware releases them too).
pub const ALLOWED: [u32; 3] = [
    // ADC, AUDIO_OUT, DMA, DMA_TICK, ETH_IP, I2C0..6, I2S0..2, PADS_ETH
    1 | (1 << 2) | (1 << 3) | (1 << 4) | (1 << 5) | (0x7f << 7) | (0x7 << 14) | (1 << 25),
    // PWM0/1, SPI0..8, SYS_RIO1/2 (GPIO 28..53 outputs), UART0..5
    (0x3 << 4) | (0x1ff << 10) | (0x3 << 20) | (0x3f << 26),
    // USBHOST0/1, VBUSCTRL
    0x7,
];
/// ADC: its DONE bit stays 0 even under the stock firmware (HW), so it is not waited for.
const NO_DONE: [u32; 3] = [1, 0, 0];
const USB_HOSTS: u32 = 0x3;

/// CLK_USBH0/1_MICROFRAME, CLK_USBH0/1_SUSPEND (CTRL; DIV_INT at +4). AUXSRC 0 = XOSC.
const USB_CLOCKS: [usize; 4] = [0x4001_80f4, 0x4001_8104, 0x4001_8114, 0x4001_8124];
/// Block set-up the stock firmware leaves behind and no Linux driver does (HW, stock firmware
/// register dump 2026-10-29; under rp1-hal these read as reset values without it):
/// - PADS_ETH pads +4..=+0x3c = 0x40 (IE, output enabled). Reset value 0x90 has OD set, so the
///   MDIO / RGMII outputs are off and the PHY is not found ("MDIO device at address 0 is missing").
/// - USBHOSTn_CFG +0 = 1. Without it an access to the xHCI (AXI) never completes and wedges the
///   RP1 fabric for the M3 and PCIe alike (HW: M3 read of 0x4020_c120 hung).
/// - VBUSCTRL +0 = 0x1111, +4 = 0x0200_1111.
/// - ETH_CFG INTE.ETHERNET (+0x20 bit 0). The GEM interrupt reaches the RP1 interrupt
///   controller (MSI-X vector 6) only through this wrapper enable, and macb does not program
///   ETH_CFG. Without it frames go out and come back (HW: GEM frame counters, host capture) but
///   eth0 never takes an interrupt, so ARP never completes.
const PADS_ETH: usize = 0x400f_c000;
const PADS_ETH_BIT: u32 = 1 << 25;
const PAD_ENABLED: u32 = 0x40;
const USBHOST_CFG: [usize; 2] = [0x4016_0000, 0x4016_4000];
const VBUSCTRL: usize = 0x4017_0000;
const VBUSCTRL_BIT: u32 = 1 << 2;
const VBUSCTRL_STOCK: [u32; 2] = [0x1111, 0x0200_1111];
const ETH_CFG_INTE: usize = 0x4010_4020;
const ETH_IP_BIT: u32 = 1 << 5;
/// SCMI clocks that run before Linux probes, as under the stock firmware.
const STARTED: [u32; 3] = [
    crate::scmi::id::DMA,
    crate::scmi::id::ETH,
    crate::scmi::id::ETH_TSU,
];

/// The resets Linux owns, from the declaration. Issued by the generated resources.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LinuxBlocks {
    resets: [u32; 3],
}

impl LinuxBlocks {
    /// # Safety
    /// `resets` must not contain a block the image uses (rp1-build derives it from the
    /// declaration). Bits outside [`ALLOWED`] are a build error.
    pub const unsafe fn new(resets: [u32; 3]) -> Self {
        assert!(
            resets[0] & !ALLOWED[0] == 0
                && resets[1] & !ALLOWED[1] == 0
                && resets[2] & !ALLOWED[2] == 0
        );
        Self { resets }
    }
    pub const fn resets(&self) -> [u32; 3] {
        self.resets
    }
    /// SCMI clock IDs (one bit each) of the Linux-kind generators whose block is not Linux's:
    /// the image declared it (v0.1: only `ADC` can be), so RP1 holds the clock
    /// ([`crate::scmi::Server::holding`]) and [`bring_up`] leaves the generator to its driver.
    pub const fn held_clocks(&self) -> u32 {
        let mut held = 0;
        let mut i = 0;
        while i < CLOCK_BLOCKS.len() {
            let (id, bank, mask) = CLOCK_BLOCKS[i];
            if self.resets[bank] & mask != mask {
                held |= 1 << id;
            }
            i += 1;
        }
        held
    }
}

/// (SCMI clock ID, reset bank, reset bits) of each Linux-kind generator's block(s). The I2S
/// clock is shared by I2S0..2: Linux keeps it only while it owns all three.
const CLOCK_BLOCKS: [(u32, usize, u32); 8] = [
    (crate::scmi::id::DMA, 0, 1 << 3),
    (crate::scmi::id::ETH, 0, ETH_IP_BIT),
    (crate::scmi::id::ETH_TSU, 0, ETH_IP_BIT),
    (crate::scmi::id::ADC, 0, 1),
    (crate::scmi::id::PWM0, 1, 1 << 4),
    (crate::scmi::id::PWM1, 1, 1 << 5),
    (crate::scmi::id::AUDIO_OUT, 0, 1 << 2),
    (crate::scmi::id::I2S, 0, 0x7 << 14),
];

/// What [`bring_up`] did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Report {
    /// Reset bits released (cleared) per bank.
    pub released: [u32; 3],
    /// Released bits whose DONE did not come up within 10 ms.
    pub pending: [u32; 3],
}

pub(crate) trait Hardware: Registers {
    fn now_us(&mut self) -> u32;
}

/// Starts PLL_SYS_SEC, puts every Linux-owned generator on a parent RP1 runs, starts the clocks
/// the stock firmware starts, and releases the resets. Generators of declared blocks
/// ([`LinuxBlocks::held_clocks`]) are not touched. Fails (before touching any reset) only if
/// PLL_SYS_SEC does not read back as programmed. Idempotent: a running set-up is left as it is.
pub(crate) fn bring_up(r: &mut impl Hardware, blocks: LinuxBlocks) -> Result<Report, crate::Error> {
    let sec = r.read(PLL_SYS_SEC);
    if sec != PLL_SYS_SEC_125MHZ {
        if sec & SEC_IMPL == 0 {
            return Err(crate::Error::Precondition);
        }
        // clk-rp1 `rp1_pll_divider_set_rate`: change DIV with RST held, then release RST.
        let held = (sec & !SEC_DIV_MASK) | (8 << 8) | SEC_RST;
        r.write(PLL_SYS_SEC, held);
        r.write(PLL_SYS_SEC, held & !SEC_RST);
        if r.read(PLL_SYS_SEC) != PLL_SYS_SEC_125MHZ {
            return Err(crate::Error::Hardware);
        }
    }
    let held = blocks.held_clocks();
    for (id, spec) in RP1_CLOCKS.iter().enumerate() {
        let Kind::Linux(g) = spec.kind else { continue };
        if held & (1 << id) != 0 {
            continue;
        }
        if g.state(r).is_none() {
            default(r, &g);
        }
        if STARTED.contains(&(id as u32)) {
            g.set_enabled(r, true);
        }
    }
    let resets = blocks.resets;
    if resets[2] & USB_HOSTS != 0 {
        for ctrl in USB_CLOCKS {
            if r.read(ctrl + 4) != 1 {
                r.write(ctrl + 4, 1);
            }
            let v = r.read(ctrl);
            if v & 0x3e0 != 0 || v & crate::scmi::CTRL_ENABLE == 0 {
                r.write(ctrl, (v & !0x3e0) | crate::scmi::CTRL_ENABLE);
            }
        }
    }
    for (bank, mask) in resets.iter().enumerate() {
        if *mask != 0 {
            r.write(RESETS_CLR + 4 * bank, *mask);
        }
    }
    let mut pending = [0; 3];
    let start = r.now_us();
    loop {
        for (bank, mask) in resets.iter().enumerate() {
            pending[bank] = mask & !NO_DONE[bank] & !r.read(RESETS_DONE + 4 * bank);
        }
        if pending == [0; 3] || r.now_us().wrapping_sub(start) > 10_000 {
            break;
        }
    }
    // After the release: these blocks ignore writes while in reset.
    if resets[0] & ETH_IP_BIT != 0 {
        let inte = r.read(ETH_CFG_INTE);
        if inte & 1 == 0 {
            r.write(ETH_CFG_INTE, inte | 1);
        }
    }
    let mut set = |a: usize, v: u32| {
        if r.read(a) != v {
            r.write(a, v);
        }
    };
    if resets[0] & PADS_ETH_BIT != 0 {
        for pad in (PADS_ETH + 4..=PADS_ETH + 0x3c).step_by(4) {
            set(pad, PAD_ENABLED);
        }
    }
    for (host, cfg) in USBHOST_CFG.iter().enumerate() {
        if resets[2] & (1 << host) != 0 {
            set(*cfg, 1);
        }
    }
    if resets[2] & VBUSCTRL_BIT != 0 {
        set(VBUSCTRL, VBUSCTRL_STOCK[0]);
        set(VBUSCTRL + 4, VBUSCTRL_STOCK[1]);
    }
    Ok(Report {
        released: resets,
        pending,
    })
}

fn default(r: &mut impl Registers, g: &Generator) {
    g.program(r, g.parents[0].0, 1);
}

pub(crate) struct Mmio;
impl Registers for Mmio {
    fn read(&mut self, address: usize) -> u32 {
        crate::read(address)
    }
    fn write(&mut self, address: usize, value: u32) {
        crate::write(address, value);
    }
}
impl Hardware for Mmio {
    fn now_us(&mut self) -> u32 {
        crate::read(0x400a_c028)
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use crate::scmi::id;
    use std::collections::BTreeMap;
    use std::vec::Vec;

    #[derive(Default)]
    struct Fake {
        regs: BTreeMap<usize, u32>,
        writes: Vec<(usize, u32)>,
        clock: u32,
        done: [u32; 3],
    }
    impl Registers for Fake {
        fn read(&mut self, address: usize) -> u32 {
            if (RESETS_DONE..RESETS_DONE + 12).contains(&address) {
                return self.done[(address - RESETS_DONE) / 4];
            }
            *self.regs.get(&address).unwrap_or(&0)
        }
        fn write(&mut self, address: usize, value: u32) {
            self.regs.insert(address, value);
            self.writes.push((address, value));
            if (RESETS_CLR..RESETS_CLR + 12).contains(&address) {
                self.done[(address - RESETS_CLR) / 4] |= value;
            }
        }
    }
    impl Hardware for Fake {
        fn now_us(&mut self) -> u32 {
            self.clock += 1000;
            self.clock
        }
    }
    fn cold() -> Fake {
        let mut f = Fake::default();
        // Cold values read on HW under rp1-hal: SEC in reset, generators off with DIV 1.
        f.regs.insert(PLL_SYS_SEC, 0x8001_0000);
        for spec in RP1_CLOCKS {
            if let Kind::Linux(g) = spec.kind {
                f.regs.insert(g.ctrl + 4, 1);
            }
        }
        for c in USB_CLOCKS {
            f.regs.insert(c + 4, 1);
        }
        f
    }
    fn ctrl(id: u32) -> usize {
        match RP1_CLOCKS[id as usize].kind {
            Kind::Linux(g) => g.ctrl,
            Kind::Fixed(_) => unreachable!(),
        }
    }
    const ALL: LinuxBlocks = unsafe { LinuxBlocks::new(ALLOWED) };

    #[test]
    fn cold_bring_up_matches_the_stock_firmware_state() {
        let mut f = cold();
        let report = bring_up(&mut f, ALL).unwrap();
        assert_eq!(report.released, ALLOWED);
        assert_eq!(report.pending, [0; 3]);
        assert_eq!(f.regs[&PLL_SYS_SEC], PLL_SYS_SEC_125MHZ);
        assert_eq!(
            f.writes[..2],
            [(PLL_SYS_SEC, 0x8001_0800), (PLL_SYS_SEC, 0x8000_0800)]
        );
        for id in [id::DMA, id::ETH, id::ETH_TSU] {
            assert_eq!(f.regs[&ctrl(id)], 0x800, "{id}");
        }
        // ADC and I2S stay off on their default parent; PWM / audio out move to XOSC.
        assert_eq!(f.regs.get(&ctrl(id::ADC)).copied().unwrap_or(0), 0);
        assert_eq!(f.regs[&ctrl(id::PWM0)], 2 << 5);
        assert_eq!(f.regs[&ctrl(id::AUDIO_OUT)], 3 << 5);
        for c in USB_CLOCKS {
            assert_eq!(f.regs[&c], 0x800);
        }
        // Resets through the CLR alias, then the set-up of the released blocks.
        let n = f.writes.iter().position(|w| w.0 == RESETS_CLR).unwrap();
        assert_eq!(
            f.writes[n..n + 3],
            [
                (RESETS_CLR, ALLOWED[0]),
                (RESETS_CLR + 4, ALLOWED[1]),
                (RESETS_CLR + 8, ALLOWED[2])
            ]
        );
        assert_eq!(f.writes[n + 3], (ETH_CFG_INTE, 1));
        let after: Vec<_> = f.writes[n + 4..].to_vec();
        assert_eq!(after.len(), 15 + 2 + 2);
        assert!(
            after[..15]
                .iter()
                .all(|&(a, v)| (PADS_ETH + 4..=PADS_ETH + 0x3c).contains(&a) && v == PAD_ENABLED)
        );
        assert_eq!(
            after[15..],
            [
                (USBHOST_CFG[0], 1),
                (USBHOST_CFG[1], 1),
                (VBUSCTRL, 0x1111),
                (VBUSCTRL + 4, 0x0200_1111)
            ]
        );
    }

    #[test]
    fn running_state_is_left_alone() {
        let mut f = cold();
        bring_up(&mut f, ALL).unwrap();
        f.writes.clear();
        bring_up(&mut f, ALL).unwrap();
        assert!(
            f.writes
                .iter()
                .all(|w| (RESETS_CLR..RESETS_CLR + 12).contains(&w.0))
        );
    }

    #[test]
    fn declared_blocks_are_not_released_and_usb_clocks_follow_usb() {
        let mut f = cold();
        // e.g. an image that owns I2C1 and UART0; no USB for Linux.
        let blocks =
            unsafe { LinuxBlocks::new([ALLOWED[0] & !(1 << 8), ALLOWED[1] & !(1 << 26), 0]) };
        let report = bring_up(&mut f, blocks).unwrap();
        assert_eq!(f.regs[&RESETS_CLR] & (1 << 8), 0);
        assert_eq!(f.regs[&(RESETS_CLR + 4)] & (1 << 26), 0);
        assert!(!f.regs.contains_key(&(RESETS_CLR + 8)));
        assert!(!f.regs.contains_key(&USB_CLOCKS[0]));
        assert!(!f.regs.contains_key(&USBHOST_CFG[0]));
        assert!(!f.regs.contains_key(&VBUSCTRL));
        assert_eq!(report.released[2], 0);
    }

    #[test]
    fn declared_adc_is_held_and_its_generator_left_alone() {
        assert_eq!(ALL.held_clocks(), 0);
        let blocks = unsafe { LinuxBlocks::new([ALLOWED[0] & !1, ALLOWED[1], ALLOWED[2]]) };
        assert_eq!(blocks.held_clocks(), 1 << id::ADC);
        // One I2S instance declared would hold the shared I2S clock.
        let i2s = unsafe { LinuxBlocks::new([ALLOWED[0] & !(1 << 15), ALLOWED[1], ALLOWED[2]]) };
        assert_eq!(i2s.held_clocks(), 1 << id::I2S);
        let mut f = cold();
        // A generator state bring_up would otherwise reprogram (DIV_INT 0 = /65536 is valid,
        // so use an unserved parent: AUXSRC 1).
        f.regs.insert(ctrl(id::ADC), 1 << 5);
        bring_up(&mut f, blocks).unwrap();
        assert!(
            !f.writes
                .iter()
                .any(|w| w.0 == ctrl(id::ADC) || w.0 == ctrl(id::ADC) + 4)
        );
        assert_eq!(f.regs[&RESETS_CLR] & 1, 0);
    }

    #[test]
    fn stuck_done_is_reported_not_fatal_and_adc_is_not_waited_for() {
        let mut f = cold();
        f.done = [0; 3];
        struct Stuck(Fake);
        impl Registers for Stuck {
            fn read(&mut self, a: usize) -> u32 {
                let v = self.0.read(a);
                if a == RESETS_DONE + 8 || a == RESETS_DONE {
                    v & !1
                } else {
                    v
                }
            }
            fn write(&mut self, a: usize, v: u32) {
                self.0.write(a, v)
            }
        }
        impl Hardware for Stuck {
            fn now_us(&mut self) -> u32 {
                self.0.now_us()
            }
        }
        let mut s = Stuck(f);
        let report = bring_up(&mut s, ALL).unwrap();
        assert_eq!(report.pending, [0, 0, 1]);
    }

    #[test]
    fn sec_without_impl_or_bad_readback_fails_before_any_reset() {
        let mut f = cold();
        f.regs.insert(PLL_SYS_SEC, 0);
        assert_eq!(bring_up(&mut f, ALL), Err(crate::Error::Precondition));
        assert!(f.writes.is_empty());
    }
}
