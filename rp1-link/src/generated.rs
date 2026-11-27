//! Internal support for the declaration code emitted by rp1-build.

use crate::{
    Block, Channel, EnableError, Error, Message, OpenError, Overflow, RingRx, RingTx, Server,
    notify::{Notify, Wake, sealed::Token},
};
use rp1_hal::{
    fugit::ExtU32,
    platform::{Pcie, PcieState},
    timer::{Alarm, Timer},
};
pub use rp1_proto::note::{ShmArea, ShmKind, ShmRegion, Writer};
use rp1_proto::{
    LINK_START, SHM_WINDOW, barrier, link,
    shm::{self, HEADER_SIZE, HeaderInit},
};
pub use rp1_rt::interrupt::Line;

/// # Safety
/// The area, SRAM ranges and digest must come from this firmware's validated declaration and
/// descriptor. Only the proc0 request handler may activate the reservation.
pub unsafe fn try_enable(
    pcie: &mut Pcie,
    area: ShmArea,
    sram_ranges: &[(u16, u16)],
    digest: &[u8; 32],
) -> Result<u32, EnableError> {
    let generation = enable_with(read_link(link::SHM_REQUEST), || unsafe {
        pcie.__publish_shm(area.phys, area.size, || {
            initialize(area.size, sram_ranges, digest)
        })
    })?;
    rp1_rt::send_event();
    Ok(generation)
}

/// Wait for Linux's doorbell or a startup-only timer event.
pub fn enable<const N: u8, const IRQ: u8>(
    alarm: &mut Alarm<N>,
    line: &mut Line<IRQ>,
    mut try_enable: impl FnMut() -> Result<u32, EnableError>,
) -> Result<u32, EnableError> {
    const { assert!(IRQ == N + 26, "use TIMER_N with ALARMN") };
    assert!(
        !line.is_enabled(),
        "link startup needs an idle alarm and disabled IRQ"
    );
    let timer = Timer::new();
    // A disabled NVIC line can latch a WFE event without taking over its handler.
    let result = rp1_rt::interrupt::__with_events_on_pending(|| {
        alarm.enable_interrupt();
        let mut armed = false;
        loop {
            match try_enable() {
                Err(EnableError::NotRequested) => {}
                result => break result,
            }
            let state = rp1_hal::platform::pcie_state();
            if state != PcieState::Up {
                break Err(EnableError::Detector(state));
            }
            if !armed || alarm.try_expired() {
                alarm.clear_interrupt();
                barrier();
                line.unpend();
                armed = alarm.try_arm(timer.now_32() + 10.millis()).is_ok();
                if !armed {
                    continue;
                }
            }
            rp1_rt::wait_for_event();
        }
    });
    alarm.disable_interrupt();
    alarm.cancel();
    alarm.clear_interrupt();
    barrier();
    line.unpend();
    result
}

/// Wait for proc0 to publish the shared-memory window, then attach the endpoints.
/// Sleep while the window is pending. Return immediately on [`OpenError::Reset`] or
/// [`OpenError::Attach`].
pub fn open<I, T>(
    mut init: I,
    mut try_open: impl FnMut(I) -> Result<T, OpenError<I>>,
) -> Result<T, OpenError<core::convert::Infallible>> {
    loop {
        match try_open(init) {
            Ok(endpoints) => return Ok(endpoints),
            Err(OpenError::Reset) => return Err(OpenError::Reset),
            Err(OpenError::Attach(error)) => return Err(OpenError::Attach(error)),
            Err(OpenError::NotReady(token)) => {
                init = token;
                rp1_rt::wait_for_event();
            }
        }
    }
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
    #[cfg(any(test, all(target_arch = "arm", target_os = "none")))]
    return crate::mmio::read(LINK_START as usize + offset);
    #[cfg(not(any(test, all(target_arch = "arm", target_os = "none"))))]
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

/// Attach the RP1 -> Linux ring `r`, with a copy of its writer's notifier. Inlined, so
/// the region's constants fold into `try_open` instead of being stored in the image.
///
/// # Safety
/// Generated `try_open` only, once per region, while the window is Ready: the region comes
/// from this firmware's validated declaration and its notifier's resource was made.
#[inline(always)]
pub unsafe fn ring_tx<T: Message, N: Notify + crate::notify::sealed::Claim>(
    r: ShmRegion<'_>,
) -> Result<RingTx<T, N>, Error> {
    let ring = unsafe {
        shm::RingTx::attach(
            ddr(r),
            r.ddr_len as usize,
            sram(r),
            r.sram_len as usize,
            r.depth,
            overflow(r),
        )
    }?;
    Ok(RingTx::new(ring, N::claim(Token(()))))
}

/// Attach the Linux -> RP1 ring `r` and make it notifiable ([`RingRx`]).
///
/// # Safety
/// As [`ring_tx`].
#[inline(always)]
pub unsafe fn ring_rx<T: Message, W: Wake + crate::notify::sealed::Claim>(
    r: ShmRegion<'_>,
) -> Result<RingRx<T, W>, Error> {
    let ring = unsafe {
        shm::RingRx::attach(
            sram(r),
            r.sram_len as usize,
            ddr(r),
            r.ddr_len as usize,
            r.depth,
            overflow(r),
        )
    }?;
    Ok(RingRx::open(ring, W::claim(Token(()))))
}

/// Assemble the two attached rings of a channel.
pub fn channel<Tx: Message, Rx: Message, N: Notify, W: Wake>(
    tx: RingTx<Tx, N>,
    rx: RingRx<Rx, W>,
) -> Channel<Tx, Rx, N, W> {
    Channel { tx, rx }
}

/// Assemble the two attached rings of an RPC.
pub fn server<Req: Message, Rep: Message, N: Notify, W: Wake>(
    tx: RingTx<Rep, N>,
    rx: RingRx<Req, W>,
) -> Server<Req, Rep, N, W> {
    Server::new(tx, rx)
}

/// Attach the block `region`.
///
/// # Safety
/// As [`ring_tx`].
#[inline(always)]
pub unsafe fn block<T: Message>(region: ShmRegion<'_>) -> Result<Block<T>, Error> {
    unsafe { Block::attach(ddr(region), region.ddr_len as usize) }
}

#[inline]
pub const fn ddr(region: ShmRegion<'_>) -> *mut u8 {
    (SHM_WINDOW + region.ddr_offset) as *mut u8
}

#[inline]
pub const fn sram(region: ShmRegion<'_>) -> *mut u8 {
    (LINK_START + region.sram_offset as u32) as *mut u8
}

const fn overflow(region: ShmRegion<'_>) -> Overflow {
    if region.overwrite {
        Overflow::Overwrite
    } else {
        Overflow::Reject
    }
}

use rp1_proto::wake::{BootEpoch, Request, Session, SessionError, mailbox};

/// B2 uses the existing startup timer with the v3 request/result envelope.
///
/// # Safety
/// As for [`try_enable`]. The complete v3 layout and caps must come from the NOTE.
pub unsafe fn try_enable_v3(
    pcie: &mut Pcie,
    area: ShmArea,
    regions: &[ShmRegion<'_>],
    digest: &[u8; 32],
) -> Result<u32, EnableError> {
    let offered = rp1_proto::wake::LATCHED_WAKE;
    let epoch = read_link(rp1_proto::status::EPOCH);
    let boot = BootEpoch::new(epoch).map_err(|_| EnableError::NotReady)?;
    if read_link(mailbox::BOOT_EPOCH) != epoch {
        write(LINK_START as usize + mailbox::OFFERED_CAPS, offered);
        barrier();
        write(LINK_START as usize + mailbox::BOOT_EPOCH, epoch);
        barrier();
    }
    if read_link(link::SHM_REQUEST) != mailbox::REQUEST_OPCODE {
        return Err(EnableError::NotRequested);
    }
    barrier();
    let request = Request {
        epoch: read_link(mailbox::REQUEST_EPOCH),
        seq: read_link(mailbox::REQUEST_SEQ),
        required_caps: read_link(mailbox::REQUIRED_CAPS),
    };
    let caps = request
        .accept(mailbox::REQUEST_OPCODE, epoch, offered)
        .map_err(|_| EnableError::Mismatch)?;
    let generation = unsafe {
        pcie.__publish_shm_checked(area.phys, area.size, || {
            let base = SHM_WINDOW as *mut u8;
            let header = HeaderInit::begin_v2(base, boot).map_err(|_| EnableError::Mismatch)?;
            for offset in (HEADER_SIZE..area.size as usize).step_by(4) {
                write(SHM_WINDOW as usize + offset, 0);
            }
            for region in regions {
                let start = LINK_START as usize + usize::from(region.sram_offset);
                for offset in (0..usize::from(region.sram_len)).step_by(4) {
                    write(start + offset, 0);
                }
            }
            flush_mapped_window();
            let session = header
                .publish_v2(
                    LINK_START as *mut u8,
                    regions,
                    digest,
                    read(0x400a_c028),
                    caps,
                )
                .map_err(|_| EnableError::Mismatch)?;
            write(LINK_START as usize + mailbox::ACCEPTED_CAPS, caps);
            write(LINK_START as usize + mailbox::RESULT_SEQ, request.seq);
            barrier();
            Ok(session.generation())
        })
    }?;
    rp1_rt::send_event();
    Ok(generation)
}

pub(crate) fn wake_session() -> Result<Session, SessionError> {
    if read_status() != link::SHM_READY {
        return Err(SessionError::Restarted);
    }
    barrier();
    Session::new(
        read_link(mailbox::BOOT_EPOCH),
        read_link(link::SHM_GENERATION),
    )
}

/// # Safety
/// As for [`ring_tx`]. Called only for the v3 IRQ owner's ring.
#[inline(always)]
pub unsafe fn ring_tx_v3<T: Message>(
    r: ShmRegion<'_>,
) -> Result<RingTx<T, crate::notify::TicketIrq>, Error> {
    let ring = unsafe {
        shm::RingTx::attach(
            ddr(r),
            r.ddr_len as usize,
            sram(r),
            r.sram_len as usize,
            r.depth,
            overflow(r),
        )
    }?;
    let notify = unsafe { crate::notify::TicketIrq::attach(ddr(r), sram(r)) }?;
    Ok(RingTx::new(ring, notify))
}

/// # Safety
/// As for [`ring_rx`]. The doorbell belongs to the declared v3 reader.
#[inline(always)]
pub unsafe fn ring_rx_v3<T: Message, const PIN: u8, const CORE: u8>(
    r: ShmRegion<'_>,
) -> Result<RingRx<T, crate::notify::TicketDoorbell<PIN, CORE>>, Error> {
    let ring = unsafe {
        shm::RingRx::attach(
            sram(r),
            r.sram_len as usize,
            ddr(r),
            r.ddr_len as usize,
            r.depth,
            overflow(r),
        )
    }?;
    let wake = unsafe { crate::notify::TicketDoorbell::attach(sram(r), ddr(r)) }?;
    Ok(RingRx::open(ring, wake))
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
