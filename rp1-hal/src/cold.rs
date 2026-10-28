//! Cold-start bring-up for an RP1 that was reset and loaded over I2C by a host-side loader.
//!
//! After the PoC loader resets RP1 and starts this image, RP1 is at cold defaults: PLL_SYS is
//! held in reset, CLK_UART is off, UART0 is held in reset and the PCIe endpoint is not
//! configured. The host cannot reach RP1 (BARs read 0xffffffff) until the firmware brings the
//! endpoint up. HW: hwrun1 02-stepA-v1b (`[RP1PCIE] LinkTimeout`, silent UART0).
//!
//! The register sequences below are those of the HW-proven PoC firmware
//! (`RP1-hal-PoC-openamp-20260929/examples/minimal`: `scmi_cold_clock::prepare`,
//! `enable_endpoint_clock_bit26`, `pre_state1_reset_clock_boundary`,
//! `state1_2_perstn_qualified_boundary`, `state3_composite_boundary`,
//! `state5_composite_boundary`), with the same values, order and timeouts. Each step checks its
//! precondition and fails closed before writing anything.
use crate::{Error, read};

const RESETS_CTRL0: usize = 0x4001_4000;
const RESETS_CTRL1: usize = 0x4001_4004;
const RESETS_DONE0: usize = 0x4001_4018;
const RESETS_DONE1: usize = 0x4001_401c;
const RESETS_CLR1: usize = 0x4001_7004;
const PLL_SYS: usize = 0x4002_0000;
const PLL_SYS_PRIM: usize = PLL_SYS + 0x10;
const CLK_UART_CTRL: usize = 0x4001_8054;
const CLK_UART_DIV_INT: usize = 0x4001_8058;

const PLL_RESET: u32 = 1 << 29;
const ENDPOINT_RESET: u32 = 1 << 26;
const UART0_RESET: u32 = 1 << 26;
const SYSCFG_RESET: u32 = 1 << 22;

const PCIE_SEL: usize = 0x4010_8000;
const PCIE_CONTROL: usize = 0x4010_8004;
const PCIE_CFG_188: usize = 0x4010_8188;
const PCIE_PM_CONTROL: usize = 0x4010_8194;
const PCIE_MONITOR2: usize = 0x4010_81a4;
const PCIE_INTR: usize = 0x4010_81a8;
const PCIE_INTE: usize = 0x4010_81ac;
const PCIE_INTS: usize = 0x4010_81b4;
const PCIE_RESET_SEQ_C0: usize = 0x4010_81c0;
const PCIE_RESET_SEQ_C8: usize = 0x4010_81c8;
const DBI: usize = 0x4010_9000;
/// Fresh PERSTn wait. The PoC loader re-initialized the root complex ~0.5 s after start;
/// rp1boot does not touch PCIe, and the VPU firmware leaves PERSTn asserted ("PCI1/PCI2
/// reset" before starting the OS), so the first deassertion is Linux's brcm-pcie probe a few
/// seconds later (HW hwrun5 08-boot7: `Perstn` after the old 1.5 s limit).
const PERSTN_TIMEOUT_US: u64 = 20_000_000;

/// Which bring-up step failed, with the raw value that failed the check.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColdError {
    /// Clock/reset tuple was neither cold defaults nor the admitted running tuple.
    ClockPrecondition(u32),
    PllReset,
    PllLock(u32),
    PllDivider(u32),
    UartClock(u32),
    UartReset,
    ClockFinal(u32),
    EndpointClock,
    PreState1(u32),
    Perstn(u32),
    CoreAlive(u32),
    LinkTimeout(u32),
    Syscfg(u32),
}

/// What [`bring_up`] did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColdReport {
    /// false: clocks were already running (warm) and were left untouched.
    pub clocks_programmed: bool,
    /// false: the PCIe endpoint was already up and was left untouched.
    pub endpoint_programmed: bool,
    /// u32::MAX: no fresh PERSTn event; continued on the deasserted level.
    pub perstn_wait_us: u32,
    pub link_wait_us: u32,
    pub monitor2: u32,
}

fn write(address: usize, value: u32) {
    unsafe { (address as *mut u32).write_volatile(value) };
}
fn modify(address: usize, f: impl FnOnce(u32) -> u32) {
    write(address, f(read(address)));
}
fn dsb() {
    rp1_proto::barrier();
}
fn now_us() -> u64 {
    loop {
        let hi = read(0x400a_c024);
        let lo = read(0x400a_c028);
        if hi == read(0x400a_c024) {
            return (u64::from(hi) << 32) | u64::from(lo);
        }
    }
}
fn wait_us(us: u64) {
    let start = now_us();
    while now_us().wrapping_sub(start) < us {
        core::hint::spin_loop();
    }
}
fn poll_us(us: u64, mut done: impl FnMut() -> bool) -> Option<u32> {
    let start = now_us();
    loop {
        if done() {
            return Some(now_us().wrapping_sub(start) as u32);
        }
        if now_us().wrapping_sub(start) > us {
            return None;
        }
        core::hint::spin_loop();
    }
}

fn clock_snapshot() -> [u32; 10] {
    [
        read(RESETS_CTRL0),
        read(RESETS_DONE0),
        read(RESETS_CTRL1),
        read(RESETS_DONE1),
        read(PLL_SYS),
        read(PLL_SYS + 4),
        read(PLL_SYS + 8),
        read(PLL_SYS + 0xc),
        read(PLL_SYS_PRIM),
        read(CLK_UART_CTRL),
    ]
}

/// PoC `scmi_cold_clock::prepare`: PLL_SYS 1 GHz VCO, PRIM /5 /1 + PRI_PH, CLK_UART = XOSC,
/// UART0 reset released. Returns false (no writes) if the clocks are already running.
fn cold_clocks() -> Result<bool, ColdError> {
    let before = clock_snapshot();
    if before != clock_snapshot() {
        return Err(ColdError::ClockPrecondition(0xffff_ffff));
    }
    let cold = before[0] & PLL_RESET != 0
        && before[1] & PLL_RESET == 0
        && before[2] & UART0_RESET != 0
        && before[3] & UART0_RESET == 0
        && before[4..9] == [1, 0x3f, 0, 0, 0x77000];
    if !cold {
        return if crate::clocks::running_tuple_admitted() {
            Ok(false)
        } else {
            Err(ColdError::ClockPrecondition(before[4]))
        };
    }
    // Release PLL_SYS reset (CTRL0 is read/write; this is the PoC's exact write).
    write(RESETS_CTRL0, before[0] & !PLL_RESET);
    dsb();
    if read(RESETS_CTRL0) & PLL_RESET != 0
        || poll_us(10_000, || read(RESETS_DONE0) & PLL_RESET != 0).is_none()
    {
        return Err(ColdError::PllReset);
    }
    // Core lock: PWR all-down, FBDIV 20, CS refdiv 1, then power up VCO/core.
    write(PLL_SYS + 4, 0x3f);
    write(PLL_SYS + 8, 20);
    write(PLL_SYS + 0xc, 0);
    write(PLL_SYS, 1);
    write(PLL_SYS + 4, 4);
    dsb();
    if poll_us(100_000, || read(PLL_SYS) & (1 << 31) != 0).is_none() {
        return Err(ColdError::PllLock(read(PLL_SYS)));
    }
    let locked = [
        read(PLL_SYS),
        read(PLL_SYS + 4),
        read(PLL_SYS + 8),
        read(PLL_SYS + 0xc),
        read(PLL_SYS_PRIM),
    ];
    if locked != [0x8000_0001, 4, 20, 0, 0x77000] {
        return Err(ColdError::PllLock(locked[0]));
    }
    write(PLL_SYS_PRIM, 0x51000);
    dsb();
    if read(PLL_SYS_PRIM) != 0x51000 {
        return Err(ColdError::PllDivider(read(PLL_SYS_PRIM)));
    }
    write(PLL_SYS_PRIM, 0x51010);
    dsb();
    if read(PLL_SYS_PRIM) != 0x51010 {
        return Err(ColdError::PllDivider(read(PLL_SYS_PRIM)));
    }
    // UART functional clock before UART0 reset release (PoC uart0-functional-clock-before-reset-done).
    write(CLK_UART_DIV_INT, 1);
    dsb();
    if read(CLK_UART_DIV_INT) != 1 {
        return Err(ColdError::UartClock(read(CLK_UART_DIV_INT)));
    }
    write(CLK_UART_CTRL, 0x40);
    dsb();
    if read(CLK_UART_CTRL) & 0xfe0 != 0x40 {
        return Err(ColdError::UartClock(read(CLK_UART_CTRL)));
    }
    write(CLK_UART_CTRL, 0x840);
    dsb();
    if read(CLK_UART_CTRL) & 0xfe0 != 0x840 {
        return Err(ColdError::UartClock(read(CLK_UART_CTRL)));
    }
    write(RESETS_CLR1, UART0_RESET);
    dsb();
    if read(RESETS_CTRL1) & UART0_RESET != 0
        || poll_us(10_000, || read(RESETS_DONE1) & UART0_RESET != 0).is_none()
    {
        return Err(ColdError::UartReset);
    }
    if !crate::clocks::running_tuple_admitted() {
        return Err(ColdError::ClockFinal(read(CLK_UART_CTRL)));
    }
    Ok(true)
}

/// PoC endpoint clock + PCIe state 1..5. Returns (perstn_wait_us, link_wait_us, monitor2).
fn endpoint() -> Result<(u32, u32, u32), ColdError> {
    // Endpoint clock (RESETS CTRL0 bit 26), with the PoC's ~5 ms phases around it.
    wait_us(5_000);
    modify(RESETS_CTRL0, |v| v & !ENDPOINT_RESET);
    dsb();
    if read(RESETS_CTRL0) & ENDPOINT_RESET != 0 {
        return Err(ColdError::EndpointClock);
    }
    wait_us(5_000);
    if poll_us(100_000, || read(RESETS_DONE0) & ENDPOINT_RESET != 0).is_none() {
        return Err(ColdError::EndpointClock);
    }
    wait_us(5_000);

    // Pre-state 1: reset sequence, control, PM, PCIe AUX clock = XOSC / 1.
    modify(PCIE_RESET_SEQ_C8, |v| v | 1 << 1);
    modify(PCIE_RESET_SEQ_C0, |v| v & !(1 << 1));
    modify(PCIE_CONTROL, |v| v & !(1 << 2));
    modify(PCIE_CONTROL, |v| v | 1);
    modify(PCIE_PM_CONTROL, |v| v & !(1 << 3));
    modify(PCIE_CONTROL, |v| v | 1 << 7);
    modify(0x4001_80e4, |v| (v & !0x1e3) | 1);
    write(0x4001_80e8, 1);
    write(0x4001_80ec, 0);
    write(0x4001_a0e4, 1 << 11);
    dsb();
    let control = read(PCIE_CONTROL);
    let sel = read(0x4001_80f0);
    let mut obs = (read(PCIE_RESET_SEQ_C8) >> 1) & 1;
    obs |= (((read(PCIE_RESET_SEQ_C0) >> 1) & 1) ^ 1) << 1;
    obs |= (((control >> 2) & 1) ^ 1) << 2;
    obs |= (control & 1) << 3;
    obs |= (((read(PCIE_PM_CONTROL) >> 3) & 1) ^ 1) << 4;
    obs |= ((control >> 7) & 1) << 5;
    obs |= u32::from(read(0x4001_80e4) & (1 << 11) != 0 && sel == 1) << 6;
    obs |= u32::from(read(0x4001_80e8) == 1) << 7;
    obs |= u32::from(read(0x4001_80ec) == 0) << 8;
    if obs & 0x1ff != 0x1ff {
        return Err(ColdError::PreState1(obs));
    }

    // States 1/2: wait (<=100 ms) for PERSTn deassertion from the host.
    modify(PCIE_INTE, |v| v & !0x7f);
    modify(PCIE_RESET_SEQ_C8, |v| v | 1 << 1);
    modify(PCIE_PM_CONTROL, |v| v | 1 << 1);
    modify(PCIE_PM_CONTROL, |v| v & !(1 << 3));
    modify(PCIE_CONTROL, |v| v & !(1 << 2));
    modify(PCIE_CONTROL, |v| v | 1);
    // Drop a PERSTn event latched before this image started (HW hwrun1 03: the wait returned
    // after 4 us, the link came up before the host re-initialized the root complex, and the
    // host then refused the unassigned BARs). Wait for the host's fresh PERSTn deassertion,
    // as the slower-starting PoC firmware effectively did; if none comes, continue when PERSTn
    // is already deasserted (no host re-init pending).
    write(PCIE_INTR, 1 << 1);
    dsb();
    modify(PCIE_INTE, |v| v | 1 << 1);
    let perstn = match poll_us(PERSTN_TIMEOUT_US, || {
        if read(PCIE_INTS) & (1 << 1) != 0 {
            write(PCIE_INTR, 1 << 1);
            read(PCIE_MONITOR2) & (1 << 17) != 0
        } else {
            false
        }
    }) {
        Some(us) => us,
        None if read(PCIE_MONITOR2) & (1 << 17) != 0 => u32::MAX,
        None => return Err(ColdError::Perstn(read(PCIE_MONITOR2))),
    };
    wait_us(51);

    // State 3: release core reset, wait (<=20 ms) for core alive.
    modify(PCIE_RESET_SEQ_C8, |v| v & !(1 << 1));
    wait_us(2);
    modify(PCIE_CONTROL, |v| v & !(1 << 7));
    modify(PCIE_INTE, |v| v | 1);
    poll_us(20_000, || read(PCIE_MONITOR2) & (1 << 16) != 0)
        .ok_or(ColdError::CoreAlive(read(PCIE_MONITOR2)))?;

    // State 5: DBI config, BARs and inbound ATU windows, then enable LTSSM; wait (<=24 ms) link.
    write(PCIE_SEL, 0);
    modify(DBI + 0x70c, |v| (v & 0xc700_00ff) | 0x2830_3000);
    modify(DBI + 0x80c, |v| (v & 0xffff_ff00) | 0x30);
    modify(DBI + 0x8bc, |v| v | 1);
    write(PCIE_SEL, 0);
    write(DBI + 0x81c, 6);
    modify(DBI + 0x0b0, |v| (v & 0x0000_ffff) | 0x003c_0000);
    write(DBI + 0x0b4, 0);
    write(DBI + 0x0b8, 0x2000);
    write(DBI + 0x008, 0x0200_0000);
    write(PCIE_SEL, 1);
    write(DBI + 0x010, 0x0000_3fff);
    write(DBI + 0x014, 0x003f_ffff);
    write(DBI + 0x018, 0x0000_ffff);
    write(PCIE_SEL, 0);
    write(DBI + 0x010, 0xffff_fff0);
    write(DBI + 0x014, 0xffff_fff0);
    write(DBI + 0x018, 0xffff_fff0);
    write(PCIE_SEL, 0x23);
    write(DBI + 0x114, 0x4000_0000);
    write(DBI + 0x118, 0x0000_00c0);
    write(DBI + 0x100, 0);
    write(DBI + 0x104, 0xc000_0100);
    write(PCIE_SEL, 0x63);
    write(DBI + 0x114, 0x2000_0000);
    write(DBI + 0x118, 0x0000_00c0);
    write(DBI + 0x100, 0);
    write(DBI + 0x104, 0xc000_0200);
    write(PCIE_SEL, 0);
    modify(DBI + 0x8d4, |v| v & !(1 << 8));
    write(PCIE_CFG_188, 0x0100_0000);
    modify(DBI + 0x8bc, |v| v & !1);
    modify(PCIE_CONTROL, |v| v | 1 << 2);
    modify(PCIE_CONTROL, |v| v & !1);
    modify(PCIE_INTE, |v| v | 0x48);
    let link = poll_us(24_000, || read(PCIE_MONITOR2) & (1 << 20) != 0)
        .ok_or(ColdError::LinkTimeout(read(PCIE_MONITOR2)))?;
    Ok((perstn, link, read(PCIE_MONITOR2)))
}

/// PoC `scmi_cold_clock::release_syscfg` without the proc-event route (no SCMI server in v0.1).
fn release_syscfg() -> Result<(), ColdError> {
    let ctrl = read(RESETS_CTRL1);
    if ctrl & SYSCFG_RESET == 0 {
        return Ok(());
    }
    write(RESETS_CLR1, SYSCFG_RESET);
    dsb();
    poll_us(10_000, || read(RESETS_DONE1) & SYSCFG_RESET != 0)
        .map(|_| ())
        .ok_or(ColdError::Syscfg(read(RESETS_DONE1)))
}

/// Program clocks (only if at cold defaults). Call before `Authority::adopt` and `log::init`.
///
/// # Safety
/// proc0 only, once, before any other driver. Requires `clock.authority = "rp1"`.
pub unsafe fn clocks() -> Result<bool, ColdError> {
    cold_clocks()
}

/// Bring the PCIe endpoint up (only if the link is down) and release SYSCFG.
///
/// # Safety
/// proc0 only, once, after [`clocks`]. The host must be waiting for the link.
pub unsafe fn endpoint_and_syscfg(clocks_programmed: bool) -> Result<ColdReport, ColdError> {
    let mut report = ColdReport {
        clocks_programmed,
        endpoint_programmed: false,
        perstn_wait_us: 0,
        link_wait_us: 0,
        monitor2: read(PCIE_MONITOR2),
    };
    if report.monitor2 & (1 << 20) == 0 {
        let (perstn, link, monitor2) = endpoint()?;
        report.endpoint_programmed = true;
        report.perstn_wait_us = perstn;
        report.link_wait_us = link;
        report.monitor2 = monitor2;
    }
    release_syscfg()?;
    Ok(report)
}

impl From<ColdError> for Error {
    fn from(_: ColdError) -> Self {
        Error::Hardware
    }
}

pub mod reconfig;
pub use reconfig::{Cleanup as ReconfigCleanup, Receipt, Status as ReconfigStatus};

/// Re-applies the endpoint DBI configuration after the host resets the endpoint with PERSTn.
///
/// Linux's brcm-pcie probe asserts PERSTn, which resets the RP1 endpoint core to defaults
/// (class 0, BAR1 256 KiB, BAR2 8 MiB, no inbound ATU); the rp1 driver then fails with
/// "BAR 0 not claimed" (HW hwrun1 04). The configuration must be rewritten in the window between
/// PERSTn deassertion + CORE_ALIVE and the host's enumeration. This runs the PoC's guarded
/// transaction ([`reconfig`]): every access re-checks READY, the selector and the 5 ms deadline
/// from the PERSTn-low sample, so a late CORE_ALIVE or a reset in the middle stops it.
/// Call [`EndpointService::poll`] in a tight loop on proc0; never block between calls.
pub struct EndpointService {
    low_at: Option<u32>,
    /// A transaction stopped after writing: the endpoint state is unknown until a cold boot.
    poisoned: bool,
}

/// Outcome of one completed re-configuration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reconfigured {
    /// Time from the last PERSTn-low sample to the last guarded access.
    pub elapsed_us: u32,
}

/// Why a re-configuration did not complete.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReconfigError {
    /// CORE_ALIVE did not follow PERSTn deassertion within the budget; nothing was written.
    CoreAlive(u32),
    /// The guarded transaction stopped. If `receipt.needs_external_recovery()`, the service
    /// refuses further transactions: only a cold boot (RP1 reload) recovers.
    Transaction(Receipt),
    /// An earlier transaction left a partial configuration; this PERSTn epoch was not handled.
    Poisoned,
}

pub const RECONFIG_BUDGET_US: u32 = 5_000;

impl EndpointService {
    /// # Safety
    /// proc0 only, one instance, after [`endpoint_and_syscfg`].
    ///
    /// Exclusion contract (replaces the PoC's "proc1 held in reset" check): while this service
    /// is in use, no other code may access the PCIe DBI selector (0x4010_8000), DBI
    /// (0x4010_9000..) or the outbound PCIe window. In this crate only proc0 code
    /// ([`crate::shm::configure_window`] / [`crate::shm::enable`]) touches the selector/DBI, and
    /// it runs in the same proc0 loop, so it cannot interleave with [`Self::poll`]. proc1 must
    /// not use the outbound window until Linux has published shm (`SHM_STATUS == 1`), which is
    /// after the final PERSTn of a cold boot. A PERSTn after that (live host reset) is unsupported.
    pub const unsafe fn new() -> Self {
        Self {
            low_at: None,
            poisoned: false,
        }
    }

    /// The steady state (PERSTn high, no pending low edge) is one register read from ISRAM;
    /// the reconfiguration transaction itself is cold and runs from shared SRAM.
    #[rp1_rt::isram]
    pub fn poll(&mut self) -> Option<Result<Reconfigured, ReconfigError>> {
        if self.low_at.is_none() && read(reconfig::MONITOR2) & (1 << 17) != 0 {
            return None;
        }
        self.poll_slow()
    }
    #[cold]
    #[inline(never)]
    fn poll_slow(&mut self) -> Option<Result<Reconfigured, ReconfigError>> {
        let mut io = |op| Ok(mmio(op));
        self.poll_with(&mut io)
    }

    fn poll_with(
        &mut self,
        io: &mut dyn FnMut(reconfig::Op) -> Result<u32, ()>,
    ) -> Option<Result<Reconfigured, ReconfigError>> {
        use reconfig::{MONITOR2, Op, READY};
        let t = io(Op::Clock).ok()?;
        let mon = io(Op::Read(MONITOR2)).ok()?;
        if mon & (1 << 17) == 0 {
            self.low_at = Some(t);
            return None;
        }
        let low = self.low_at?;
        if self.poisoned {
            self.low_at = None;
            return Some(Err(ReconfigError::Poisoned));
        }
        if mon & READY != READY {
            if io(Op::Clock).ok()?.wrapping_sub(low) >= RECONFIG_BUDGET_US {
                self.low_at = None;
                return Some(Err(ReconfigError::CoreAlive(mon)));
            }
            return None;
        }
        self.low_at = None;
        let window = reconfig::Window {
            low_us: low,
            budget_us: RECONFIG_BUDGET_US,
        };
        let r = reconfig::run(&window, io);
        if r.status == reconfig::Status::Complete && r.mask_restored {
            let last = r.last_us.unwrap_or(low);
            return Some(Ok(Reconfigured {
                elapsed_us: last.wrapping_sub(low),
            }));
        }
        if r.needs_external_recovery() {
            self.poisoned = true;
        }
        Some(Err(ReconfigError::Transaction(r)))
    }
}

fn mmio(op: reconfig::Op) -> u32 {
    use reconfig::Op;
    match op {
        Op::Mask => mask_interrupts(),
        Op::Restore(mask) => restore_interrupts(mask),
        Op::Clock => read(0x400a_c028),
        Op::Read(address) => read(address),
        Op::Write(address, value) => {
            write(address, value);
            0
        }
    }
}

#[cfg(target_arch = "arm")]
fn mask_interrupts() -> u32 {
    let mask: u32;
    unsafe {
        core::arch::asm!("mrs {0}, PRIMASK", "cpsid i", "isb", out(reg) mask, options(nostack))
    };
    mask
}
#[cfg(target_arch = "arm")]
fn restore_interrupts(mask: u32) -> u32 {
    let restored: u32;
    unsafe {
        core::arch::asm!("dsb sy", "msr PRIMASK, {0}", "isb", in(reg) mask, options(nostack));
        core::arch::asm!("mrs {0}, PRIMASK", out(reg) restored, options(nomem, nostack, preserves_flags));
    }
    restored
}
#[cfg(not(target_arch = "arm"))]
fn mask_interrupts() -> u32 {
    0
}
#[cfg(not(target_arch = "arm"))]
fn restore_interrupts(mask: u32) -> u32 {
    mask
}
