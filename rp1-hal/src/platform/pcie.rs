//! Outbound region 0 and SHM publication, owned by proc0's PCIe endpoint.
//!
//! Section A maps the reservation with interrupts masked by PRIMASK. The fill runs unmasked
//! so IRQ53 can report a host reset. Section B rechecks the detector and edge latch with the
//! same mask before publishing Ready. If a reset starts after those checks, IRQ53 overwrites
//! Ready with SHM_WINDOW_LOST as soon as interrupts are unmasked.

use super::{
    Pcie, PcieState,
    perstn::{self, PERSTN_EDGE},
};
use crate::{
    cold::{LINK, PCIE_INTR, PCIE_MONITOR2},
    timer::{Deadline, Timer},
};
use rp1_proto::{
    barrier,
    link::{self, SHM_READY},
    wake::{StartupError, mailbox},
};

const SELECT: usize = 0x4010_8000;
/// The SELECT value that maps the outbound ATU registers into the DBI.
const SELECT_ATU: u32 = 3;
const DBI: usize = 0x4010_9000;
/// DBI command register: memory space (bit 1) and bus master (bit 2) enabled by Linux.
const DBI_COMMAND: usize = DBI + 4;
const MEMORY_AND_MASTER: u32 = 6;
/// Outbound region 0 in the DBI while SELECT = [`SELECT_ATU`]: 8 words, word 1 is the enable.
const REGION0: usize = DBI + 0x100;
const REGION0_ENABLE: usize = 1;
/// PCIE_MONITOR2 bits 16 (core alive), 17 (PERSTn deasserted) and the link.
const ENDPOINT_UP: u32 = (3 << 16) | LINK;
const SHM_GENERATION: usize = rp1_proto::LINK_START as usize + link::SHM_GENERATION;
const SHM_STATUS: usize = rp1_proto::LINK_START as usize + link::SHM_STATUS;

fn read(at: usize) -> u32 {
    unsafe { (at as *const u32).read_volatile() }
}
fn write(at: usize, value: u32) {
    unsafe { (at as *mut u32).write_volatile(value) }
}
fn word(offset: usize) -> usize {
    rp1_proto::LINK_START as usize + offset
}

/// Why the PCIe window transaction did not publish the reservation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnableError {
    /// Linux has not supplied the expected request word.
    /// Returned by rp1-link's generated `try_enable` and probe entries; this module never does.
    NotRequested,
    /// The PERSTn detector is not `Up`: the host reset the endpoint, or a reset would not be
    /// seen. Nothing was written.
    Detector(PcieState),
    /// The reservation was already published, or region 0 previously read back wrong.
    Pinned,
    /// A PERSTn edge is latched, the link is down, the DBI selector is in use, the endpoint is
    /// not enabled or the loader handoff does not match the declaration. Nothing was written.
    NotReady,
    /// Region 0 read back differently from what was written. Later calls return `Pinned`.
    Mismatch,
    /// The host reset the endpoint while the reservation was being initialized: not
    /// published, `SHM_STATUS` is `SHM_WINDOW_LOST`.
    WindowLost,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StartupEnableError {
    /// The window transaction could not proceed.
    Window(EnableError),
    /// The core-local one-shot could not be used.
    Wait(crate::timer::Error),
    /// Proc0 committed this failure for the current boot.
    Rejected(StartupError),
}

impl From<EnableError> for StartupEnableError {
    fn from(error: EnableError) -> Self {
        Self::Window(error)
    }
}

#[derive(Clone, Copy)]
struct Window {
    phys: u64,
    size: u32,
}

impl Pcie {
    /// Map region 0 and publish Ready after `initialize` returns the DDR generation.
    /// The callback runs outside both PRIMASK sections.
    ///
    /// # Safety
    /// Only generated/link code may call this, on proc0 after Linux requests SHM. The PA and
    /// size must be the validated reservation from the firmware declaration. The callback
    /// exclusively initializes it and completes its posted writes before returning.
    #[doc(hidden)]
    pub unsafe fn __publish_shm(
        &mut self,
        phys: u64,
        size: u32,
        initialize: impl FnOnce() -> u32,
    ) -> Result<u32, EnableError> {
        unsafe { self.__publish_shm_checked(phys, size, || Ok(initialize())) }
    }

    /// The v3 initialization can reject an exhausted generation before Ready.
    ///
    /// # Safety
    /// As for [`Self::__publish_shm`].
    #[doc(hidden)]
    pub unsafe fn __publish_shm_checked(
        &mut self,
        phys: u64,
        size: u32,
        initialize: impl FnOnce() -> Result<u32, EnableError>,
    ) -> Result<u32, EnableError> {
        publish(
            &mut Mmio,
            &mut self.shm_pinned,
            Window { phys, size },
            |_| initialize(),
        )
    }

    /// Map once, leaving the reservation unpublished until the startup transaction commits.
    ///
    /// # Safety
    /// As for [`Self::__publish_shm`]. Only generated v3 startup code may call this.
    #[doc(hidden)]
    pub unsafe fn __map_startup_window(&mut self, phys: u64, size: u32) -> Result<(), EnableError> {
        map_window(&mut Mmio, &mut self.shm_pinned, Window { phys, size })?;
        // A partially initialized reservation cannot be reused in this boot.
        self.shm_pinned = true;
        Ok(())
    }

    /// Commit the v3 startup result after checking the reset latch and the absolute deadline.
    ///
    /// # Safety
    /// Only the generated proc0 startup owner may call this. On success the validated
    /// reservation must be fully initialized and its outbound writes drained.
    #[doc(hidden)]
    pub unsafe fn __commit_startup(
        &mut self,
        seq: u32,
        result: Result<(u32, u32), StartupError>,
        end: Deadline,
    ) -> Result<u32, StartupEnableError> {
        let result = commit_startup(&mut Mmio, seq, result, || {
            Timer::new().now() >= end.instant()
        });
        rp1_rt::send_event();
        result
    }

    /// Map region 0 for the pre-Ready lab probe without pinning or publishing it.
    ///
    /// # Safety
    /// Only generated/link lab code may call this, on proc0 in response to the probe request.
    /// The PA and size must be the validated reservation from the firmware declaration.
    #[doc(hidden)]
    pub unsafe fn __map_shm_for_probe(&mut self, phys: u64, size: u32) -> Result<(), EnableError> {
        map_window(&mut Mmio, &mut self.shm_pinned, Window { phys, size })
    }
}

/// The hardware accesses exercised by the PERSTn model.
trait Io {
    fn read(&mut self, address: usize) -> u32;
    fn write(&mut self, address: usize, value: u32);
    fn barrier(&mut self);
    /// Runs `f` with this core's interrupts masked by PRIMASK, IRQ53 included.
    fn masked<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T;
    /// The PERSTn detector's own state on proc0.
    fn detector(&mut self) -> PcieState;
}

/// Section A, the fill, then section B (module documentation).
fn publish<I: Io>(
    io: &mut I,
    pinned: &mut bool,
    window: Window,
    initialize: impl FnOnce(&mut I) -> Result<u32, EnableError>,
) -> Result<u32, EnableError> {
    map_window(io, pinned, window)?;
    let generation = initialize(io)?;
    // Section B. IRQ53 cannot run between these checks and the stores.
    io.masked(|io| {
        if io.detector() != PcieState::Up || io.read(PCIE_INTR) & PERSTN_EDGE != 0 {
            return Err(EnableError::WindowLost);
        }
        io.write(SHM_GENERATION, generation);
        io.barrier();
        io.write(SHM_STATUS, SHM_READY);
        *pinned = true;
        Ok(generation)
    })
}

/// Check on either core before touching the window or attempting a startup MSI-X pulse.
#[doc(hidden)]
pub fn __startup_window_live() -> bool {
    rp1_proto::status::PcieState::Up == perstn::pcie_state() && endpoint_up(&mut Mmio)
}

fn endpoint_up(io: &mut impl Io) -> bool {
    io.read(PCIE_MONITOR2) & ENDPOINT_UP == ENDPOINT_UP && io.read(PCIE_INTR) & PERSTN_EDGE == 0
}

fn commit_startup(
    io: &mut impl Io,
    seq: u32,
    result: Result<(u32, u32), StartupError>,
    expired: impl FnOnce() -> bool,
) -> Result<u32, StartupEnableError> {
    let result = io.masked(|io| -> Result<_, EnableError> {
        let result = result.and_then(|ready| {
            if expired() {
                Err(StartupError::Timeout)
            } else {
                Ok(ready)
            }
        });
        if io.detector() != PcieState::Up || !endpoint_up(io) {
            return Err(EnableError::WindowLost);
        }
        if io.read(SHM_STATUS) != 0 {
            return Err(EnableError::Pinned);
        }
        let (generation, caps, status) = match result {
            Ok((generation, caps)) => (generation, caps, SHM_READY),
            Err(error) => (0, 0, error as u32),
        };
        io.write(SHM_GENERATION, generation);
        io.write(word(mailbox::ACCEPTED_CAPS), caps);
        io.write(word(mailbox::RESULT_SEQ), seq);
        io.barrier();
        if !endpoint_up(io) {
            return Err(EnableError::WindowLost);
        }
        io.write(SHM_STATUS, status);
        let _ = io.read(SHM_STATUS);
        io.barrier();
        Ok(result
            .map(|(generation, _)| generation)
            .map_err(StartupEnableError::Rejected))
    })?;
    // IRQ53 can run on unmask. An edge during the commit supersedes either outcome.
    if io.detector() != PcieState::Up || !endpoint_up(io) {
        return Err(EnableError::WindowLost.into());
    }
    result
}

fn map_window(io: &mut impl Io, pinned: &mut bool, window: Window) -> Result<(), EnableError> {
    io.masked(|io| checked_map(io, pinned, window))
}

/// Section A, with interrupts masked: the checks, then region 0 written and read back.
fn checked_map(io: &mut impl Io, pinned: &mut bool, window: Window) -> Result<(), EnableError> {
    let state = io.detector();
    if state != PcieState::Up {
        return Err(EnableError::Detector(state));
    }
    if *pinned {
        return Err(EnableError::Pinned);
    }
    let ready = io.read(PCIE_INTR) & PERSTN_EDGE == 0
        && io.read(PCIE_MONITOR2) & ENDPOINT_UP == ENDPOINT_UP
        && io.read(SELECT) == 0
        && io.read(DBI_COMMAND) & MEMORY_AND_MASTER == MEMORY_AND_MASTER
        && io.read(word(link::HANDOFF_MAGIC)) == link::MAGIC
        && io.read(word(link::HANDOFF_PHYS_LOW)) == window.phys as u32
        && io.read(word(link::HANDOFF_PHYS_HIGH)) == (window.phys >> 32) as u32
        && io.read(word(link::HANDOFF_SIZE)) == window.size;
    if !ready {
        return Err(EnableError::NotReady);
    }
    let region = region0(window);
    io.write(SELECT, SELECT_ATU);
    // Disabled while the other words change, enabled last.
    io.write(REGION0 + 4 * REGION0_ENABLE, 0);
    for (i, value) in region.iter().enumerate() {
        if i != REGION0_ENABLE {
            io.write(REGION0 + i * 4, *value);
        }
    }
    io.write(REGION0 + 4 * REGION0_ENABLE, region[REGION0_ENABLE]);
    io.barrier();
    let valid = region
        .iter()
        .enumerate()
        .all(|(i, v)| io.read(REGION0 + i * 4) == *v);
    io.write(SELECT, 0);
    if !valid {
        // The window's state is unknown: enable nothing on it until RP1 is reloaded.
        *pinned = true;
        return Err(EnableError::Mismatch);
    }
    Ok(())
}

/// The 8 words of region 0 for `window`. PoC mapping: fabric 0x80_0100_0000 -> PCIe
/// 0x10_0000_0000 + CPU PA.
fn region0(window: Window) -> [u32; 8] {
    let target = 0x10_0000_0000u64 + window.phys;
    [
        0,
        0x8000_0000,
        0x0100_0000,
        0x80,
        0x0100_0000 + window.size - 1,
        target as u32,
        (target >> 32) as u32,
        0,
    ]
}

struct Mmio;

impl Io for Mmio {
    fn read(&mut self, address: usize) -> u32 {
        read(address)
    }
    fn write(&mut self, address: usize, value: u32) {
        write(address, value);
    }
    fn barrier(&mut self) {
        barrier();
    }
    fn masked<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        #[cfg(all(target_arch = "arm", target_os = "none"))]
        return rp1_rt::interrupt::__primask_free(|| f(self));
        #[cfg(not(all(target_arch = "arm", target_os = "none")))]
        f(self)
    }
    fn detector(&mut self) -> PcieState {
        perstn::state()
    }
}

#[cfg(test)]
mod tests;
