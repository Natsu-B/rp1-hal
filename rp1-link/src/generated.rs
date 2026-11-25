//! Internal support for the declaration code emitted by rp1-build.

use crate::{EnableError, Error, OpenError, Overflow};
use rp1_hal::platform::Pcie;
pub use rp1_proto::note::{ShmArea, ShmKind, ShmRegion, Writer};
use rp1_proto::{
    LINK_START, SHM_WINDOW, barrier, link,
    shm::{HEADER_SIZE, HeaderInit},
};

/// # Safety
/// The area, SRAM ranges and digest must come from this firmware's validated declaration and
/// descriptor. Only the proc0 request handler may activate the reservation.
pub unsafe fn try_enable(
    pcie: &mut Pcie,
    area: ShmArea,
    sram_ranges: &[(u16, u16)],
    digest: &[u8; 32],
) -> Result<u32, EnableError> {
    enable_with(read_link(link::SHM_REQUEST), || unsafe {
        pcie.__publish_shm(area.phys, area.size, || {
            initialize(area.size, sram_ranges, digest)
        })
    })
}

fn enable_with(
    request: u32,
    publish: impl FnOnce() -> Result<u32, EnableError>,
) -> Result<u32, EnableError> {
    if request != 1 {
        return Err(EnableError::NotRequested);
    }
    publish()
}

/// # Safety
/// The area must come from this firmware's validated declaration. Only the proc0 request
/// handler may map and probe the reservation.
pub unsafe fn map_probe(pcie: &mut Pcie, area: ShmArea) -> Result<u32, EnableError> {
    probe_with(read_link(link::SHM_REQUEST), || {
        unsafe { pcie.__map_shm_for_probe(area.phys, area.size) }?;
        Ok(area.size)
    })
}

fn probe_with(
    request: u32,
    map: impl FnOnce() -> Result<u32, EnableError>,
) -> Result<u32, EnableError> {
    if request != 2 {
        return Err(EnableError::NotRequested);
    }
    map()
}

fn read_link(offset: usize) -> u32 {
    #[cfg(all(target_arch = "arm", target_os = "none"))]
    return read(LINK_START as usize + offset);
    #[cfg(not(all(target_arch = "arm", target_os = "none")))]
    {
        let _ = offset;
        0
    }
}

fn read(address: usize) -> u32 {
    unsafe { (address as *const u32).read_volatile() }
}

fn write(address: usize, value: u32) {
    unsafe { (address as *mut u32).write_volatile(value) }
}

fn initialize(size: u32, sram_ranges: &[(u16, u16)], digest: &[u8; 32]) -> u32 {
    const BASE: usize = SHM_WINDOW as usize;
    const TIMER_RAWL: usize = 0x400a_c028;
    let previous = if read(BASE) == u32::from_le_bytes(*b"RP1S")
        && read(BASE + 4) == u32::from_le_bytes(*b"HM\0\0")
    {
        read(BASE + 12)
    } else {
        0
    };
    // Section A mapped the reservation and this transaction exclusively owns initialization.
    let mut header = unsafe { HeaderInit::begin(BASE as *mut u8, previous) };
    // This is a Device aperture. The fill must use aligned volatile words.
    for offset in (HEADER_SIZE..size as usize).step_by(4) {
        write(BASE + offset, 0);
    }
    for &(start, len) in sram_ranges {
        let start = LINK_START as usize + usize::from(start);
        for offset in (0..usize::from(len)).step_by(4) {
            write(start + offset, 0);
        }
    }
    header.write(digest, sram_ranges.len() as u32, read(TIMER_RAWL));
    unsafe { flush_mapped_window() };
    let generation = header.publish();
    unsafe { flush_mapped_window() };
    generation
}

/// Read back through the outbound window before it has been published. Used by initialization
/// and by the lab probe.
///
/// # Safety
/// The outbound window must already have been mapped by this transaction or the lab probe.
#[inline]
pub unsafe fn flush_mapped_window() {
    barrier();
    let _ = read(SHM_WINDOW as usize);
    barrier();
}

/// # Safety
/// The token and attachment code must come from rp1-build. The token must be the owning
/// core's one right to attach these endpoints.
pub unsafe fn try_open<I, T>(
    init: I,
    attach: impl FnOnce() -> Result<T, Error>,
) -> Result<T, OpenError<I>> {
    open_with(init, read_status, attach)
}

fn open_with<I, T>(
    init: I,
    mut status: impl FnMut() -> u32,
    attach: impl FnOnce() -> Result<T, Error>,
) -> Result<T, OpenError<I>> {
    match status() {
        0 => return Err(OpenError::NotReady(init)),
        link::SHM_WINDOW_LOST => return Err(OpenError::Reset),
        link::SHM_READY => {}
        _ => return Err(OpenError::Attach(Error::Invalid)),
    }
    barrier();
    let endpoints = attach().map_err(OpenError::Attach)?;
    match status() {
        link::SHM_READY => Ok(endpoints),
        link::SHM_WINDOW_LOST => Err(OpenError::Reset),
        _ => Err(OpenError::Attach(Error::Invalid)),
    }
}

pub(crate) fn read_status() -> u32 {
    read_link(link::SHM_STATUS)
}

#[inline]
pub const fn ddr(region: ShmRegion<'_>) -> *mut u8 {
    (SHM_WINDOW + region.ddr_offset) as *mut u8
}

#[inline]
pub const fn sram(region: ShmRegion<'_>) -> *mut u8 {
    (LINK_START + region.sram_offset as u32) as *mut u8
}

pub const fn overflow(region: ShmRegion<'_>) -> Overflow {
    if region.overwrite {
        Overflow::Overwrite
    } else {
        Overflow::Reject
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enable_runs_only_for_request_1() {
        for request in [0, 2, 3] {
            assert_eq!(
                enable_with(request, || panic!("publish without request 1")),
                Err(EnableError::NotRequested)
            );
        }
        assert_eq!(enable_with(1, || Ok(7)), Ok(7));
        assert_eq!(
            enable_with(1, || Err(EnableError::Pinned)),
            Err(EnableError::Pinned)
        );
    }

    #[test]
    fn probe_maps_only_for_request_2() {
        for request in [0, 1, 3] {
            assert_eq!(
                probe_with(request, || panic!("map without request 2")),
                Err(EnableError::NotRequested)
            );
        }
        assert_eq!(probe_with(2, || Ok(0x40_0000)), Ok(0x40_0000));
    }

    #[test]
    fn pending_returns_the_token_and_ready_attaches_once() {
        let pending = open_with(
            42,
            || 0,
            || -> Result<(), Error> { panic!("attach while pending") },
        );
        let Err(OpenError::NotReady(init)) = pending else {
            panic!("expected the token")
        };
        let mut reads = 0;
        let mut attaches = 0;
        let ready = open_with(
            init,
            || {
                reads += 1;
                link::SHM_READY
            },
            || {
                attaches += 1;
                Ok(23)
            },
        );
        assert_eq!(ready.unwrap(), 23);
        assert_eq!((reads, attaches), (2, 1));
    }

    #[test]
    fn lost_and_invalid_status_do_not_attach() {
        let attach = || -> Result<(), Error> { panic!("attach without Ready") };
        assert!(matches!(
            open_with((), || link::SHM_WINDOW_LOST, attach),
            Err(OpenError::Reset)
        ));
        assert!(matches!(
            open_with((), || 2, attach),
            Err(OpenError::Attach(Error::Invalid))
        ));
    }

    #[test]
    fn attachment_failure_or_loss_after_ready_consumes_the_token() {
        let mut reads = 0;
        let result = open_with(
            (),
            || {
                reads += 1;
                if reads == 1 {
                    link::SHM_READY
                } else {
                    link::SHM_WINDOW_LOST
                }
            },
            || Ok(23),
        );
        assert_eq!(reads, 2);
        assert!(matches!(result, Err(OpenError::Reset)));
        assert!(matches!(
            open_with((), || link::SHM_READY, || Err::<(), _>(Error::Mismatch)),
            Err(OpenError::Attach(Error::Mismatch))
        ));
    }
}
