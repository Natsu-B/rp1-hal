//! Platform bring-up on proc0: the clocks, the blocks Linux owns, the log, the PCIe endpoint
//! and the SCMI clock server, once per image, before any driver.
//!
//! The firmware calls the generated `rp1::platform::init(&mut core, p.platform)`, which runs
//! [`Token::init`]. The steps, in order:
//!
//! 0. `PLAT_PCIE` = Initializing ([`rp1_proto::status::PLAT_PCIE`], written by `perstn`).
//! 1. Cold clocks: PLL_SYS and CLK_UART, if RP1 is at cold defaults ([`crate::cold`]).
//! 2. clk_sys = PLL_SYS and the blocks Linux owns; on success the [`Clocks`] are published for
//!    proc1 ([`Clocks::wait_from_proc0`]).
//! 3. The declared log.
//! 4. A link that is already up is a warm start: refused ([`InitError::Warm`]) before anything
//!    is written to the SCMI channel or the endpoint.
//! 5. Quiesce: endpoint clock, LTSSM enable dropped and read back, then the link is read again
//!    and must be down.
//! 6. The SCMI channel is cleared and the server installed with zeroed counters. Linux needs
//!    the link to write the channel, and the link cannot come up while LTSSM enable is 0 (DesignWare
//!    `app_ltssm_enable`, HYP on RP1), so nothing writes the channel while it is cleared.
//!    Requests that arrive before IRQ57 is enabled wait in the mailbox.
//! 7. Endpoint bring-up: PERSTn wait (up to 20 s), core alive, DBI, LTSSM enable, link.
//! 8. SYSCFG reset released.
//! 9. Publish SCMI Serving with zero requests, then enable proc0 IRQ57.
//! 10. Arm the PERSTn detector on proc0 IRQ53 (`perstn`). A forced PCIe interrupt source
//!     leaves it off (`DetectorOff`), and init still succeeds.
//! 11. Return. Init writes no handler state and no status word after the ISER of its line:
//!     a handler that ran at once keeps what it wrote (rule E1).
//!
//! Init returns once Linux's PCIe probe has released PERSTn and
//! the link is up: about 3 s (HW 2026-11, cold boot, reboot and poweroff: the PERSTn wait
//! ended after 3.06 s each time).
use crate::{Error, clocks::Clocks, cold, linux_blocks, scmi};
use core::{fmt, marker::PhantomData};
use scmi::irq;

pub(crate) mod perstn;

pub use crate::cold::ColdError;
pub use irq::{ScmiState, ScmiStats, scmi_state};
mod pcie;
pub use pcie::EnableError;

pub use perstn::{DetectorReport, LostDetail, PcieState, pcie_state};

/// The platform resource: one per image, issued to proc0 only. [`Token::init`] consumes it, so
/// the bring-up runs once.
pub struct Token {
    _proc0: PhantomData<core::cell::Cell<()>>,
    linux: linux_blocks::LinuxBlocks,
    sys_hz: u32,
}

/// What init brought up.
pub struct System {
    pub clocks: Clocks,
    pub pcie: Pcie,
    pub report: InitReport,
}

/// The PCIe endpoint init brought up. proc0 only (not `Send`).
pub struct Pcie {
    _proc0: PhantomData<core::cell::Cell<()>>,
    /// The SHM transaction published the reservation, or outbound region 0 read back wrong:
    /// the window stays as it is until RP1 is reloaded.
    shm_pinned: bool,
}

impl Pcie {
    /// What IRQ53 read when it saw a host reset; None while [`pcie_state`] is not `Lost`.
    pub fn lost_detail(&self) -> Option<LostDetail> {
        perstn::lost_detail()
    }
}

/// Snapshot IRQ57's counters on proc0 with interrupts masked.
pub fn scmi_stats(_pcie: &Pcie) -> ScmiStats {
    irq::stats()
}

/// The values init read on its way (for the log).
///
/// On `Ok`, `warm` / `ltssm_off` / `link_before_scmi` (as printed) are always 0 / 1 / 0: each
/// other value makes init fail. They record that the checks ran; the facts a log checker
/// needs are `ok=1` and `perstn_wait_us` (u32::MAX: no fresh PERSTn release).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InitReport {
    /// false: the clocks were already running and were left untouched.
    pub clocks_programmed: bool,
    /// PCIE_MONITOR2 before the endpoint was touched (step 4).
    pub monitor2_at_start: u32,
    /// LTSSM enable read back as 0 after the quiesce (step 5).
    pub ltssm_off: bool,
    /// The link read as up after the quiesce, before the SCMI channel was cleared (step 5).
    pub link_before_scmi: bool,
    /// u32::MAX: no fresh PERSTn event; continued on the deasserted level.
    pub perstn_wait_us: u32,
    pub link_wait_us: u32,
    /// PCIE_MONITOR2 once the link was up.
    pub monitor2: u32,
    pub linux_blocks: linux_blocks::Report,
    pub detector: DetectorReport,
}

/// The step that failed. Steps after it did not run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InitError {
    Clocks(ColdError),
    Authority(Error),
    Log(Error),
    /// The link was already up (PCIE_MONITOR2 bit 20): not a cold start.
    Warm {
        monitor2: u32,
    },
    Quiesce(ColdError),
    Endpoint(ColdError),
    Syscfg(ColdError),
}

impl Token {
    /// # Safety
    /// Issued once to proc0 by the generated resources. `linux` is the set of blocks the
    /// declaration leaves to Linux, `sys_hz` the declared clk_sys rate.
    pub const unsafe fn new(linux: linux_blocks::LinuxBlocks, sys_hz: u32) -> Self {
        Self {
            _proc0: PhantomData,
            linux,
            sys_hz,
        }
    }

    /// Runs the steps of this module. `log` initializes the declared log (step 3).
    /// `before_link` runs between the log and the endpoint, while Linux cannot reach RP1 yet;
    /// only the lab's interrupt sweep needs it.
    ///
    /// Called by the generated `rp1::platform::init`; use that.
    #[doc(hidden)]
    pub fn init(
        self,
        core: &mut rp1_rt::Proc0,
        log: impl FnOnce(&mut rp1_rt::Proc0) -> Result<(), Error>,
        before_link: impl FnOnce(&Clocks),
    ) -> Result<System, InitError> {
        let mut steps = Rp1 {
            linux: self.linux,
            sys_hz: self.sys_hz,
        };
        let (clocks, report) = run(&mut steps, |_| log(core), |_, c| before_link(c))?;
        Ok(System {
            clocks,
            pcie: Pcie {
                _proc0: PhantomData,
                shm_pinned: false,
            },
            report,
        })
    }
}

/// `warm=0 ltssm_off=1 link_before_scmi=0 perstn_wait_us=… link_wait_us=… monitor2=0x…
/// clocks_programmed=1 inte=0x2 intf=0x0 intr_at_arm_bit1=0 nvic53=1 vec53_ok=1
/// intr_at_arm=0x…`: one line of `key=value` fields, flags as 0 / 1. `intr_at_arm_bit1` is the
/// PERSTn edge bit, `-` when PCIE_INTR was not read (`intf` nonzero).
impl fmt::Display for InitReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let d = &self.detector;
        write!(
            f,
            "warm={} ltssm_off={} link_before_scmi={} perstn_wait_us={} link_wait_us={} monitor2=0x{:x} clocks_programmed={} \
             inte=0x{:x} intf=0x{:x} intr_at_arm_bit1=",
            u8::from(self.monitor2_at_start & cold::LINK != 0),
            u8::from(self.ltssm_off),
            u8::from(self.link_before_scmi),
            self.perstn_wait_us,
            self.link_wait_us,
            self.monitor2,
            u8::from(self.clocks_programmed),
            d.inte,
            d.intf,
        )?;
        // `write_str`, not `{}`: a `str` argument links the formatter's padding code (~0.7 KiB).
        f.write_str(match d.perstn_edge_at_arm() {
            Some(false) => "0",
            Some(true) => "1",
            None => "-",
        })?;
        write!(
            f,
            " nvic53={} vec53_ok={} intr_at_arm=0x{:x}",
            u8::from(d.nvic53),
            u8::from(d.vec53_ok),
            d.intr_at_arm,
        )
    }
}

/// The steps of init, apart from their order ([`run`]), so the host tests can check the order.
trait Steps {
    /// Sets the PERSTn detector and its `PLAT_PCIE` word to `Initializing`.
    fn pcie_initializing(&mut self);
    fn clocks(&mut self) -> Result<bool, ColdError>;
    /// clk_sys and the Linux blocks; publishes the [`Clocks`] for proc1 on success.
    fn authority(&mut self) -> Result<(Clocks, linux_blocks::Report), Error>;
    fn monitor2(&mut self) -> u32;
    /// Returns PCIE_CONTROL as read back after LTSSM enable was dropped.
    fn quiesce(&mut self) -> Result<u32, ColdError>;
    /// Clears the SCMI channel (the first write to it) and sets up the server.
    fn install_scmi(&mut self);
    /// Returns (perstn_wait_us, link_wait_us, monitor2).
    fn bring_up(&mut self) -> Result<(u32, u32, u32), ColdError>;
    fn release_syscfg(&mut self) -> Result<(), ColdError>;
    fn enable_scmi(&mut self);
    fn arm_perstn(&mut self) -> DetectorReport;
}

/// Steps 0..11 of the module documentation.
fn run<S: Steps>(
    s: &mut S,
    log: impl FnOnce(&mut S) -> Result<(), Error>,
    before_link: impl FnOnce(&mut S, &Clocks),
) -> Result<(Clocks, InitReport), InitError> {
    s.pcie_initializing();
    let clocks_programmed = s.clocks().map_err(InitError::Clocks)?;
    let (clocks, linux_blocks) = s.authority().map_err(InitError::Authority)?;
    log(s).map_err(InitError::Log)?;
    // Before the warm check on purpose: the lab's sweep keeps the position it had before
    // `init` existed (after the log, before the endpoint), where its hardware results were
    // taken. A warm start still fails init after it, so a sweep log of such a boot ends in
    // `PLAT_INIT ok=0`, which lab/verify-irq.py rejects.
    before_link(s, &clocks);
    let monitor2_at_start = s.monitor2();
    if monitor2_at_start & cold::LINK != 0 {
        return Err(InitError::Warm {
            monitor2: monitor2_at_start,
        });
    }
    let control = s.quiesce().map_err(InitError::Quiesce)?;
    // A link that came up before LTSSM enable was dropped would let Linux write the channel
    // while it is being cleared.
    let reread = s.monitor2();
    if reread & cold::LINK != 0 {
        return Err(InitError::Quiesce(ColdError::LinkUp(reread)));
    }
    s.install_scmi();
    let (perstn_wait_us, link_wait_us, monitor2) = s.bring_up().map_err(InitError::Endpoint)?;
    s.release_syscfg().map_err(InitError::Syscfg)?;
    s.enable_scmi();
    let detector = s.arm_perstn();
    let report = InitReport {
        clocks_programmed,
        monitor2_at_start,
        ltssm_off: control & cold::LTSSM_ENABLE == 0,
        link_before_scmi: reread & cold::LINK != 0,
        perstn_wait_us,
        link_wait_us,
        monitor2,
        linux_blocks,
        detector,
    };
    Ok((clocks, report))
}

/// The steps on RP1's registers.
struct Rp1 {
    linux: linux_blocks::LinuxBlocks,
    sys_hz: u32,
}

impl Steps for Rp1 {
    fn pcie_initializing(&mut self) {
        perstn::initializing(&mut perstn::Mmio)
    }
    fn clocks(&mut self) -> Result<bool, ColdError> {
        cold::clocks()
    }
    fn authority(&mut self) -> Result<(Clocks, linux_blocks::Report), Error> {
        crate::clocks::init(self.linux, self.sys_hz)
    }
    fn monitor2(&mut self) -> u32 {
        cold::monitor2()
    }
    fn quiesce(&mut self) -> Result<u32, ColdError> {
        cold::quiesce()
    }
    fn install_scmi(&mut self) {
        // SAFETY: The proc0 token is single-use. The link is quiesced and IRQ57 is still
        // disabled. Init has clock authority and reserves channel 0 and its shmem for SCMI.
        unsafe { irq::install(self.linux.held_clocks()) };
    }
    fn bring_up(&mut self) -> Result<(u32, u32, u32), ColdError> {
        cold::bring_up()
    }
    fn release_syscfg(&mut self) -> Result<(), ColdError> {
        cold::release_syscfg()
    }
    fn enable_scmi(&mut self) {
        irq::enable(&mut irq::Mmio)
    }
    fn arm_perstn(&mut self) -> DetectorReport {
        perstn::arm(&mut perstn::Mmio, perstn::Handlers::linked())
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use std::{format, vec::Vec};

    const LINK: u32 = cold::LINK;
    const CLOCKS: Clocks = Clocks::for_test(200_000_000, 50_000_000);
    const PERSTN_US: u32 = 3_062_273;
    const LINK_US: u32 = 8_204;
    const UP: u32 = 0x001b_0000 | LINK;
    /// A normal arming, with the PCIE_INTR latches expected after the bring-up.
    const ARMED: DetectorReport = DetectorReport {
        inte: 0x2,
        intf: 0,
        intr_at_arm: perstn::fake::BRING_UP_LATCHES,
        nvic53: true,
        vec53_ok: true,
    };

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Ev {
        PcieInitializing,
        Clocks,
        Authority,
        Log,
        BeforeLink,
        /// A PCIE_MONITOR2 read and whether it showed the link.
        Monitor2(bool),
        Quiesce,
        ScmiReset,
        BringUp,
        Syscfg,
        EnableScmi,
        ArmPerstn,
    }

    /// Where a link-up is injected, relative to the quiesce.
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Rise {
        Never,
        /// After the warm check, before the quiesce starts.
        BeforeQuiesce,
        /// Inside the quiesce, before LTSSM enable is dropped.
        InQuiesce,
        /// After LTSSM enable was dropped and the link was read again: only possible if H1
        /// (no link while LTSSM enable is 0) is false.
        AfterReread,
    }

    /// The init-level model of RP1 and the host. Rule H1 (DOC/HYP): while LTSSM enable is 0
    /// the link does not come up; an injection that breaks it is recorded in `h1_violated`.
    struct Fake {
        trace: Vec<Ev>,
        ltssm: bool,
        link: bool,
        rise: Rise,
        h1_violated: bool,
        authority: Result<(), Error>,
        quiesce: Result<(), ColdError>,
        monitor2_reads: u32,
    }

    impl Fake {
        fn cold() -> Self {
            Self {
                trace: Vec::new(),
                ltssm: false,
                link: false,
                rise: Rise::Never,
                h1_violated: false,
                authority: Ok(()),
                quiesce: Ok(()),
                monitor2_reads: 0,
            }
        }
        fn raise_link(&mut self) {
            self.h1_violated |= !self.ltssm;
            self.link = true;
        }
        fn run(&mut self) -> Result<(Clocks, InitReport), InitError> {
            run(
                self,
                |f| {
                    f.trace.push(Ev::Log);
                    Ok(())
                },
                |f, c| {
                    assert_eq!(*c, CLOCKS);
                    f.trace.push(Ev::BeforeLink);
                },
            )
        }
        fn has(&self, ev: Ev) -> bool {
            self.trace.contains(&ev)
        }
    }

    impl Steps for Fake {
        fn arm_perstn(&mut self) -> DetectorReport {
            self.trace.push(Ev::ArmPerstn);
            ARMED
        }
        fn pcie_initializing(&mut self) {
            self.trace.push(Ev::PcieInitializing);
        }
        fn clocks(&mut self) -> Result<bool, ColdError> {
            self.trace.push(Ev::Clocks);
            Ok(true)
        }
        fn authority(&mut self) -> Result<(Clocks, linux_blocks::Report), Error> {
            self.trace.push(Ev::Authority);
            self.authority.map(|()| {
                (
                    CLOCKS,
                    linux_blocks::Report {
                        released: [1, 2, 3],
                        pending: [0; 3],
                    },
                )
            })
        }
        fn monitor2(&mut self) -> u32 {
            self.monitor2_reads += 1;
            self.trace.push(Ev::Monitor2(self.link));
            let value = if self.link { UP } else { 0 };
            // Read 1 is the warm check, read 2 the one after the quiesce.
            match (self.rise, self.monitor2_reads) {
                (Rise::BeforeQuiesce, 1) | (Rise::AfterReread, 2) => self.raise_link(),
                _ => {}
            }
            value
        }
        fn quiesce(&mut self) -> Result<u32, ColdError> {
            if self.rise == Rise::InQuiesce {
                self.raise_link();
            }
            self.quiesce?;
            // Dropping LTSSM enable does not take an established link down (assumed, the
            // conservative case).
            self.ltssm = false;
            self.trace.push(Ev::Quiesce);
            Ok(0x0000_0081)
        }
        fn install_scmi(&mut self) {
            self.trace.push(Ev::ScmiReset);
        }
        fn bring_up(&mut self) -> Result<(u32, u32, u32), ColdError> {
            assert!(!self.ltssm, "bring-up starts with LTSSM enable off");
            self.trace.push(Ev::BringUp);
            self.ltssm = true;
            self.link = true;
            Ok((PERSTN_US, LINK_US, UP))
        }
        fn release_syscfg(&mut self) -> Result<(), ColdError> {
            self.trace.push(Ev::Syscfg);
            Ok(())
        }
        fn enable_scmi(&mut self) {
            self.trace.push(Ev::EnableScmi);
        }
    }

    /// I1: cold start. The SCMI channel is first written after LTSSM enable read back as 0
    /// and the link read again as down; IRQ57 follows SYSCFG release, IRQ53 comes last.
    #[test]
    fn cold_start_clears_the_channel_only_after_the_quiesce() {
        let mut f = Fake::cold();
        let (clocks, report) = f.run().unwrap();
        assert_eq!(
            f.trace,
            [
                Ev::PcieInitializing,
                Ev::Clocks,
                Ev::Authority,
                Ev::Log,
                Ev::BeforeLink,
                Ev::Monitor2(false),
                Ev::Quiesce,
                Ev::Monitor2(false),
                Ev::ScmiReset,
                Ev::BringUp,
                Ev::Syscfg,
                Ev::EnableScmi,
                Ev::ArmPerstn,
            ]
        );
        assert_eq!(clocks, CLOCKS);
        assert_eq!(
            report,
            InitReport {
                clocks_programmed: true,
                monitor2_at_start: 0,
                ltssm_off: true,
                link_before_scmi: false,
                perstn_wait_us: PERSTN_US,
                link_wait_us: LINK_US,
                monitor2: UP,
                linux_blocks: linux_blocks::Report {
                    released: [1, 2, 3],
                    pending: [0; 3],
                },
                detector: ARMED,
            }
        );
        assert!(!f.h1_violated);
    }

    /// I2: the link is up when init looks: refused, and nothing reaches the SCMI channel, the
    /// endpoint (quiesce, DBI) or SYSCFG.
    #[test]
    fn warm_start_is_refused_before_the_endpoint_and_the_channel() {
        let mut f = Fake::cold();
        f.ltssm = true;
        f.link = true;
        assert_eq!(f.run().unwrap_err(), InitError::Warm { monitor2: UP });
        assert_eq!(f.trace.last(), Some(&Ev::Monitor2(true)));
        for ev in [Ev::Quiesce, Ev::ScmiReset, Ev::BringUp, Ev::Syscfg] {
            assert!(!f.has(ev), "{ev:?}");
        }
    }

    /// I3: LTSSM enable inherited on, link down at the warm check, link up before or during
    /// the quiesce: the second read sees it and the channel is not touched.
    #[test]
    fn link_up_during_the_quiesce_is_refused_before_the_channel() {
        for rise in [Rise::BeforeQuiesce, Rise::InQuiesce] {
            let mut f = Fake::cold();
            f.ltssm = true;
            f.rise = rise;
            assert_eq!(
                f.run().unwrap_err(),
                InitError::Quiesce(ColdError::LinkUp(UP))
            );
            assert!(!f.has(Ev::ScmiReset) && !f.has(Ev::BringUp));
            assert!(!f.h1_violated);
        }
    }

    /// I3, forced variant: a link that comes up after LTSSM enable is 0 and after the second
    /// read is not seen, and the channel is cleared while Linux could reach it. The model can
    /// only produce it by breaking H1: the ordering relies on H1.
    #[test]
    fn link_up_after_the_reread_needs_a_model_violation() {
        let mut f = Fake::cold();
        f.ltssm = true;
        f.rise = Rise::AfterReread;
        assert!(f.run().is_ok());
        assert!(f.has(Ev::ScmiReset));
        assert!(f.h1_violated, "only reachable if H1 is false");
    }

    /// A quiesce that fails its read-back check stops init before the channel.
    #[test]
    fn failed_quiesce_stops_before_the_channel() {
        let mut f = Fake::cold();
        f.quiesce = Err(ColdError::PreState1(0x1fb));
        assert_eq!(
            f.run().unwrap_err(),
            InitError::Quiesce(ColdError::PreState1(0x1fb))
        );
        assert_eq!(f.trace.last(), Some(&Ev::Monitor2(false)));
        assert!(!f.has(Ev::ScmiReset));
    }

    /// I4: the Linux blocks fail (inside step 2): no log, no SCMI server, nothing after. The
    /// [`Clocks`] are not published: `clocks::init` publishes only on success
    /// (`clocks::tests::init_publishes_only_on_success`).
    #[test]
    fn failed_linux_blocks_stop_before_scmi() {
        let mut f = Fake::cold();
        f.authority = Err(Error::Hardware);
        assert_eq!(f.run().unwrap_err(), InitError::Authority(Error::Hardware));
        assert_eq!(f.trace, [Ev::PcieInitializing, Ev::Clocks, Ev::Authority]);
    }

    #[test]
    fn report_prints_the_plat_init_fields() {
        let mut f = Fake::cold();
        let (_, report) = f.run().unwrap();
        assert_eq!(
            format!("{report}"),
            "warm=0 ltssm_off=1 link_before_scmi=0 perstn_wait_us=3062273 link_wait_us=8204 monitor2=0x1b0000 clocks_programmed=1 \
             inte=0x2 intf=0x0 intr_at_arm_bit1=0 nvic53=1 vec53_ok=1 intr_at_arm=0x49"
        );
        let warm = InitReport {
            monitor2_at_start: UP,
            perstn_wait_us: u32::MAX,
            ..report
        };
        assert!(format!("{warm}").starts_with("warm=1 "));
        assert!(format!("{warm}").contains(" perstn_wait_us=4294967295 "));
        let arm_fault = InitReport {
            detector: DetectorReport {
                inte: 0,
                intf: 0x4,
                intr_at_arm: 0,
                nvic53: false,
                vec53_ok: true,
            },
            ..report
        };
        assert!(format!("{arm_fault}").ends_with(
            " inte=0x0 intf=0x4 intr_at_arm_bit1=- nvic53=0 vec53_ok=1 intr_at_arm=0x0"
        ));
    }

    /// RP1's registers for the real `cold::quiesce_with` and `scmi::reset_channel`: the step
    /// model above cannot see a missing read-back check. Unset registers read 0; `sticky` bits
    /// of PCIE_CONTROL cannot be cleared.
    struct Regs {
        values: std::collections::BTreeMap<usize, u32>,
        sticky: u32,
        /// (write?, address) in program order.
        log: Vec<(bool, usize)>,
        now: u64,
    }
    const AUX_CTRL: usize = 0x4001_80e4;
    impl Regs {
        fn cold(control: u32, sticky: u32) -> Self {
            let values = [
                (cold::PCIE_CONTROL, control),
                (cold::RESETS_DONE0, 1 << 26),
                // PCIe AUX clock selected: XOSC.
                (0x4001_80f0, 1),
            ];
            Self {
                values: values.into_iter().collect(),
                sticky,
                log: Vec::new(),
                now: 0,
            }
        }
        fn writes_in(&self, range: core::ops::Range<usize>) -> Vec<usize> {
            let i = self.log.iter().enumerate();
            i.filter(|(_, (w, a))| *w && range.contains(a))
                .map(|(i, _)| i)
                .collect()
        }
    }
    impl cold::Io for Regs {
        fn read(&mut self, address: usize) -> u32 {
            self.log.push((false, address));
            self.values.get(&address).copied().unwrap_or(0)
        }
        fn write(&mut self, address: usize, value: u32) {
            self.log.push((true, address));
            let old = self.values.get(&address).copied().unwrap_or(0);
            let (address, value) = match address {
                // The SET alias of the AUX clock control.
                a if a == AUX_CTRL + 0x2000 => (
                    AUX_CTRL,
                    self.values.get(&AUX_CTRL).copied().unwrap_or(0) | value,
                ),
                cold::PCIE_CONTROL => (address, value | (old & self.sticky)),
                _ => (address, value),
            };
            self.values.insert(address, value);
        }
        fn now_us(&mut self) -> u64 {
            self.now += 500;
            self.now
        }
    }
    impl scmi::Transport for Regs {
        fn take_event(&mut self) -> bool {
            unreachable!()
        }
        fn read(&mut self, _: usize) -> u32 {
            unreachable!()
        }
        fn write(&mut self, word: usize, value: u32) {
            cold::Io::write(self, scmi::SHMEM + 4 * word, value);
        }
        fn barrier(&mut self) {}
        fn notify(&mut self) {
            unreachable!()
        }
    }
    /// The real quiesce, link read and channel reset on [`Regs`]; the other steps succeed.
    impl Steps for Regs {
        fn pcie_initializing(&mut self) {}
        fn clocks(&mut self) -> Result<bool, ColdError> {
            Ok(true)
        }
        fn authority(&mut self) -> Result<(Clocks, linux_blocks::Report), Error> {
            Ok((
                CLOCKS,
                linux_blocks::Report {
                    released: [0; 3],
                    pending: [0; 3],
                },
            ))
        }
        fn monitor2(&mut self) -> u32 {
            cold::Io::read(self, cold::PCIE_MONITOR2)
        }
        fn quiesce(&mut self) -> Result<u32, ColdError> {
            cold::quiesce_with(self)
        }
        fn install_scmi(&mut self) {
            scmi::reset_channel(self);
        }
        fn bring_up(&mut self) -> Result<(u32, u32, u32), ColdError> {
            Ok((PERSTN_US, LINK_US, UP))
        }
        fn release_syscfg(&mut self) -> Result<(), ColdError> {
            Ok(())
        }
        fn enable_scmi(&mut self) {}
        fn arm_perstn(&mut self) -> DetectorReport {
            DetectorReport::default()
        }
    }
    const SHMEM: core::ops::Range<usize> = scmi::SHMEM..scmi::SHMEM + 256;
    /// NVIC ISER0..7 and ISPR0..7: no line is enabled or pended.
    const NVIC_SET: [core::ops::Range<usize>; 2] =
        [0xe000_e100..0xe000_e120, 0xe000_e200..0xe000_e220];

    /// I1 / I3 on the registers: LTSSM enable that stays set after the quiesce's write fails
    /// init with no write to the SCMI area; when it clears, the area is first written after
    /// the PCIE_CONTROL read-back and the second link read. Failed init leaves the NVIC alone.
    #[test]
    fn ltssm_read_back_guards_the_scmi_area() {
        let mut stuck = Regs::cold(cold::LTSSM_ENABLE, cold::LTSSM_ENABLE);
        assert_eq!(
            run(&mut stuck, |_| Ok(()), |_, _| {}).unwrap_err(),
            InitError::Quiesce(ColdError::PreState1(0x1fb))
        );
        assert!(
            stuck
                .writes_in(cold::PCIE_CONTROL..cold::PCIE_CONTROL + 4)
                .len()
                >= 3
        );
        assert!(stuck.writes_in(SHMEM).is_empty());

        let mut r = Regs::cold(cold::LTSSM_ENABLE, 0);
        let (_, report) = run(&mut r, |_| Ok(()), |_, _| {}).unwrap();
        assert!(report.ltssm_off);
        let first = r.writes_in(SHMEM)[0];
        let reads = |address| {
            let i = r.log.iter().enumerate();
            i.filter(move |(_, e)| **e == (false, address))
                .map(|(i, _)| i)
        };
        assert!(reads(cold::PCIE_CONTROL).all(|i| i < first));
        assert_eq!(reads(cold::PCIE_MONITOR2).count(), 2);
        assert!(reads(cold::PCIE_MONITOR2).all(|i| i < first));
        assert_eq!(r.writes_in(SHMEM).len(), 65, "64 zeroes, then FREE");
        for range in NVIC_SET {
            assert!(stuck.writes_in(range).is_empty());
        }
    }
}
