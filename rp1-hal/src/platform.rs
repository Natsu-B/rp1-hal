//! Platform bring-up on proc0: the clocks, the blocks Linux owns, the log, the PCIe endpoint
//! and the SCMI clock server, once per image, before any driver.
//!
//! The firmware calls the generated `rp1::platform::init(&mut core, p.platform)`, which runs
//! [`Token::init`]. The steps, in order:
//!
//! 0. `PLAT_PCIE` = Initializing ([`rp1_proto::status::PLAT_PCIE`]).
//! 1. Cold clocks: PLL_SYS and CLK_UART, if RP1 is at cold defaults ([`crate::cold`]).
//! 2. clk_sys = PLL_SYS and the blocks Linux owns; on success the [`Clocks`] are published for
//!    proc1 ([`Clocks::from_proc0`]).
//! 3. The declared log.
//! 4. A link that is already up is a warm start: refused ([`InitError::Warm`]) before anything
//!    is written to the SCMI channel or the endpoint.
//! 5. Quiesce: endpoint clock, LTSSM enable dropped and read back, then the link is read again
//!    and must be down.
//! 6. The SCMI channel is cleared and the server set up. Linux needs the link to write the
//!    channel, and the link cannot come up while LTSSM enable is 0 (DesignWare
//!    `app_ltssm_enable`, HYP on RP1), so nothing writes the channel while it is cleared.
//!    Requests that arrive from here on wait in the mailbox until proc0's loop polls them.
//! 7. Endpoint bring-up: PERSTn wait (up to 20 s), core alive, DBI, LTSSM enable, link.
//! 8. SYSCFG reset released.
//!
//! Init enables no interrupt line. It returns once Linux's PCIe probe has released PERSTn and
//! the link is up: about 3 s (HW 2026-11, cold boot, reboot and poweroff: the PERSTn wait
//! ended after 3.06 s each time).
use crate::{Error, clocks::Clocks, cold, linux_blocks, scmi};
use core::{fmt, marker::PhantomData};

pub use crate::cold::ColdError;

/// The platform resource: one per image, issued to proc0 only. [`Token::init`] consumes it, so
/// the bring-up runs once.
pub struct Token {
    linux: linux_blocks::LinuxBlocks,
    sys_hz: u32,
}

/// What init brought up.
pub struct System {
    pub clocks: Clocks,
    pub pcie: Pcie,
    pub report: InitReport,
    scmi: scmi::Service,
}

/// The PCIe endpoint init brought up. proc0 only (not `Send`).
pub struct Pcie {
    _proc0: PhantomData<*mut ()>,
}

/// The values init read on its way (for the log).
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
        Self { linux, sys_hz }
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
        let (clocks, report, scmi) = run(&mut steps, |_| log(core), |_, c| before_link(c))?;
        Ok(System {
            clocks,
            pcie: Pcie {
                _proc0: PhantomData,
            },
            report,
            scmi,
        })
    }
}

impl System {
    /// Answers one pending SCMI request, if any ([`scmi::Event`] for the log). Call it from
    /// proc0's loop; it never blocks. The idle check runs from ISRAM.
    #[doc(hidden)]
    #[inline(always)]
    pub fn poll_services(&mut self) -> Option<scmi::Event> {
        self.scmi.poll()
    }
}

/// `warm=0 ltssm_off=1 link_before_scmi=0 perstn_wait_us=… link_wait_us=… monitor2=0x…
/// clocks_programmed=1`: one line of `key=value` fields, flags as 0 / 1.
impl fmt::Display for InitReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "warm={} ltssm_off={} link_before_scmi={} perstn_wait_us={} link_wait_us={} monitor2=0x{:x} clocks_programmed={}",
            u8::from(self.monitor2_at_start & cold::LINK != 0),
            u8::from(self.ltssm_off),
            u8::from(self.link_before_scmi),
            self.perstn_wait_us,
            self.link_wait_us,
            self.monitor2,
            u8::from(self.clocks_programmed),
        )
    }
}

/// The steps of init, apart from their order ([`run`]), so the host tests can check the order.
trait Steps {
    type Scmi;
    /// Writes the `PLAT_PCIE` status word.
    fn set_pcie_state(&mut self, word: u32);
    fn clocks(&mut self) -> Result<bool, ColdError>;
    /// clk_sys and the Linux blocks; publishes the [`Clocks`] for proc1 on success.
    fn authority(&mut self) -> Result<(Clocks, linux_blocks::Report), Error>;
    fn monitor2(&mut self) -> u32;
    /// Returns PCIE_CONTROL as read back after LTSSM enable was dropped.
    fn quiesce(&mut self) -> Result<u32, ColdError>;
    /// Clears the SCMI channel (the first write to it) and sets up the server.
    fn start_scmi(&mut self) -> Self::Scmi;
    /// Returns (perstn_wait_us, link_wait_us, monitor2).
    fn bring_up(&mut self) -> Result<(u32, u32, u32), ColdError>;
    fn release_syscfg(&mut self) -> Result<(), ColdError>;
}

/// Steps 0..8 of the module documentation.
fn run<S: Steps>(
    s: &mut S,
    log: impl FnOnce(&mut S) -> Result<(), Error>,
    before_link: impl FnOnce(&mut S, &Clocks),
) -> Result<(Clocks, InitReport, S::Scmi), InitError> {
    s.set_pcie_state(rp1_proto::status::PLAT_PCIE_INITIALIZING);
    let clocks_programmed = s.clocks().map_err(InitError::Clocks)?;
    let (clocks, linux_blocks) = s.authority().map_err(InitError::Authority)?;
    log(s).map_err(InitError::Log)?;
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
    let scmi = s.start_scmi();
    let (perstn_wait_us, link_wait_us, monitor2) = s.bring_up().map_err(InitError::Endpoint)?;
    s.release_syscfg().map_err(InitError::Syscfg)?;
    let report = InitReport {
        clocks_programmed,
        monitor2_at_start,
        ltssm_off: control & cold::LTSSM_ENABLE == 0,
        link_before_scmi: reread & cold::LINK != 0,
        perstn_wait_us,
        link_wait_us,
        monitor2,
        linux_blocks,
    };
    Ok((clocks, report, scmi))
}

/// The steps on RP1's registers.
struct Rp1 {
    linux: linux_blocks::LinuxBlocks,
    sys_hz: u32,
}

impl Steps for Rp1 {
    type Scmi = scmi::Service;
    fn set_pcie_state(&mut self, word: u32) {
        let address = rp1_proto::LINK_START as usize + rp1_proto::status::PLAT_PCIE;
        // A plain store: proc0's init is the only writer of this word.
        unsafe { (address as *mut u32).write_volatile(word) };
        rp1_proto::barrier();
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
    fn start_scmi(&mut self) -> scmi::Service {
        // SAFETY: the one platform token, on proc0; the Linux-owned generators and mailbox
        // channel 0 belong to this server from here on.
        unsafe { scmi::Service::new(self.linux.held_clocks()) }
    }
    fn bring_up(&mut self) -> Result<(u32, u32, u32), ColdError> {
        cold::bring_up()
    }
    fn release_syscfg(&mut self) -> Result<(), ColdError> {
        cold::release_syscfg()
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

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Ev {
        PcieState(u32),
        Clocks,
        Authority,
        Log,
        BeforeLink,
        /// A PCIE_MONITOR2 read and whether it showed the link.
        Monitor2(bool),
        /// The quiesce, with LTSSM enable as read back.
        Quiesce(bool),
        ScmiReset,
        BringUp,
        Syscfg,
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
        fn run(&mut self) -> Result<(Clocks, InitReport, ()), InitError> {
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
        type Scmi = ();
        fn set_pcie_state(&mut self, word: u32) {
            self.trace.push(Ev::PcieState(word));
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
            self.trace.push(Ev::Quiesce(self.ltssm));
            Ok(0x0000_0081)
        }
        fn start_scmi(&mut self) {
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
    }

    /// I1: cold start. The SCMI channel is first written after LTSSM enable read back as 0
    /// and the link read again as down; the PCIe state goes first. The steps have no way to
    /// write the NVIC, so no line is enabled.
    #[test]
    fn cold_start_clears_the_channel_only_after_the_quiesce() {
        let mut f = Fake::cold();
        let (clocks, report, ()) = f.run().unwrap();
        assert_eq!(
            f.trace,
            [
                Ev::PcieState(rp1_proto::status::PLAT_PCIE_INITIALIZING),
                Ev::Clocks,
                Ev::Authority,
                Ev::Log,
                Ev::BeforeLink,
                Ev::Monitor2(false),
                Ev::Quiesce(false),
                Ev::Monitor2(false),
                Ev::ScmiReset,
                Ev::BringUp,
                Ev::Syscfg,
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
        for ev in [Ev::Quiesce(false), Ev::ScmiReset, Ev::BringUp, Ev::Syscfg] {
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
        assert_eq!(
            f.trace,
            [
                Ev::PcieState(rp1_proto::status::PLAT_PCIE_INITIALIZING),
                Ev::Clocks,
                Ev::Authority
            ]
        );
    }

    #[test]
    fn report_prints_the_plat_init_fields() {
        let mut f = Fake::cold();
        let (_, report, ()) = f.run().unwrap();
        assert_eq!(
            format!("{report}"),
            "warm=0 ltssm_off=1 link_before_scmi=0 perstn_wait_us=3062273 link_wait_us=8204 monitor2=0x1b0000 clocks_programmed=1"
        );
        let warm = InitReport {
            monitor2_at_start: UP,
            perstn_wait_us: u32::MAX,
            ..report
        };
        assert!(format!("{warm}").starts_with("warm=1 "));
        assert!(format!("{warm}").contains(" perstn_wait_us=4294967295 "));
    }
}
