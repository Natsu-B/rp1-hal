//! PERSTn endpoint re-configuration transaction (review #4).
//!
//! Port of the HW-proven PoC contract
//! (`RP1-hal-PoC-openamp-20260929/examples/minimal/src/endpoint_config.rs`, `Transaction`):
//! the same recipe, and the same guards around *every* MMIO access — READY (PERSTn deasserted and
//! CORE_ALIVE), the expected DBI selector, and the deadline measured from the PERSTn-low sample.
//! A guard failure after the first write is terminal: the partial configuration is never
//! claimed rolled back, cleanup touches only selector/RO and only while the guards still hold.
//!
//! Difference from the PoC: there proc1 had to be held in reset (CTRL0/DONE0 bit31). Here proc1
//! runs by design, so that check is replaced by the exclusion contract of
//! [`super::EndpointService::new`]. All accesses go through one callback so the algorithm,
//! including PRIMASK ordering, is host-testable with fault injection.

pub(super) const SELECTOR: usize = 0x4010_8000;
const DBI: usize = 0x4010_9000;
const RO: usize = DBI + 0x8bc;
pub(super) const MONITOR2: usize = 0x4010_81a4;
/// MONITOR2 bit17 = PERSTn deasserted, bit16 = CORE_ALIVE.
pub(super) const READY: u32 = 3 << 16;
const LEVELS: u32 = 0x1f << 16;
const NORMAL: [usize; 8] = [
    DBI,
    DBI + 4,
    DBI + 8,
    DBI + 0x10,
    DBI + 0x14,
    DBI + 0x18,
    SELECTOR,
    RO,
];
const INBOUND: [usize; 4] = [DBI + 0x114, DBI + 0x118, DBI + 0x100, DBI + 0x104];
/// Writes in recipe order: RO-enable, class, sel1, masks0..2, sel0, BAR0..2, two inbound
/// windows (selector + 4 words each), sel0, RO-restore.
pub const RECIPE_WRITES: u32 = 22;

/// Why a transaction stopped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    InvalidWindow,
    /// The budget from the PERSTn-low sample ran out (checked before every access).
    Deadline,
    /// PERSTn re-asserted or CORE_ALIVE dropped.
    NotReady,
    /// The DBI selector was not the value this transaction last wrote.
    Selector,
    /// The endpoint was not at reset defaults (ID, command, class, BARs).
    Tuple,
    /// DBI read-only write enable was already set.
    RoEnabled,
    /// MONITOR2 link/reset levels changed while the defaults were read.
    LevelDrift,
    CallbackFailure,
    ReadbackMismatch,
    Complete,
}

/// What happened to selector/RO after a failure. Never a configuration rollback.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cleanup {
    NotNeeded,
    Restored,
    /// Guards no longer held, so no further DBI access was made.
    DeferredUnsafe,
    Failed,
}

/// Outcome of one transaction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Receipt {
    pub status: Status,
    pub cleanup: Cleanup,
    pub mask_before: Option<u32>,
    pub mask_restored: bool,
    pub first_us: Option<u32>,
    pub last_us: Option<u32>,
    /// id, cmdstat, classrev, BAR0..2, selector, RO. None means not read.
    pub before: [Option<u32>; 8],
    pub after: [Option<u32>; 8],
    /// One bit per recipe write (see [`RECIPE_WRITES`]). Returned writes are not acceptance.
    pub writes_attempted: u32,
    pub writes_returned: u32,
    /// One bit per exact inbound readback word, BAR1 then BAR2.
    pub inbound_readback: u32,
    /// Cleanup-only bits: selector0, saved RO.
    pub cleanup_attempted: u8,
    pub cleanup_returned: u8,
}

impl Receipt {
    fn new(status: Status) -> Self {
        Self {
            status,
            cleanup: Cleanup::NotNeeded,
            mask_before: None,
            mask_restored: false,
            first_us: None,
            last_us: None,
            before: [None; 8],
            after: [None; 8],
            writes_attempted: 0,
            writes_returned: 0,
            inbound_readback: 0,
            cleanup_attempted: 0,
            cleanup_returned: 0,
        }
    }
    /// A partial transaction leaves the endpoint in an unknown state: only a cold boot recovers.
    pub fn needs_external_recovery(&self) -> bool {
        (self.mask_before.is_some() && !self.mask_restored)
            || (self.writes_attempted != 0 && self.status != Status::Complete)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Op {
    Mask,
    Restore(u32),
    /// Raw 1 µs timer, low word.
    Clock,
    Read(usize),
    Write(usize, u32),
}

pub(super) struct Window {
    pub low_us: u32,
    pub budget_us: u32,
}

type Io<'a> = dyn FnMut(Op) -> Result<u32, ()> + 'a;

fn call(io: &mut Io<'_>, op: Op) -> Result<u32, Status> {
    io(op).map_err(|_| Status::CallbackFailure)
}
fn live(io: &mut Io<'_>) -> Result<u32, Status> {
    let mon = call(io, Op::Read(MONITOR2))?;
    if mon & READY != READY {
        return Err(Status::NotReady);
    }
    Ok(mon)
}
fn time(io: &mut Io<'_>, w: &Window, r: &mut Receipt) -> Result<(), Status> {
    let now = call(io, Op::Clock)?;
    if r.first_us.is_none() {
        r.first_us = Some(now);
    }
    r.last_us = Some(now);
    if now.wrapping_sub(w.low_us) >= w.budget_us {
        Err(Status::Deadline)
    } else {
        Ok(())
    }
}
fn guard(io: &mut Io<'_>, selector: u32, w: &Window, r: &mut Receipt) -> Result<(), Status> {
    live(io)?;
    if call(io, Op::Read(SELECTOR))? != selector {
        return Err(Status::Selector);
    }
    time(io, w, r)
}

/// Runs one transaction with interrupts masked for its whole duration.
pub(super) fn run(w: &Window, io: &mut Io<'_>) -> Receipt {
    let mut r = Receipt::new(Status::InvalidWindow);
    if w.budget_us == 0 || w.budget_us >= 0x8000_0000 {
        return r;
    }
    let mask = match call(io, Op::Mask) {
        Ok(mask) => mask,
        Err(status) => {
            r.status = status;
            return r;
        }
    };
    r.mask_before = Some(mask);
    r.status = match protected(w, io, &mut r) {
        Ok(()) => Status::Complete,
        Err(status) => status,
    };
    if r.writes_attempted != 0 {
        r.cleanup = if r.status == Status::Complete {
            Cleanup::Restored
        } else {
            cleanup(w, io, &mut r)
        };
    }
    r.mask_restored = call(io, Op::Restore(mask)) == Ok(mask);
    if !r.mask_restored {
        r.status = Status::CallbackFailure;
    }
    r
}

fn protected(w: &Window, io: &mut Io<'_>, r: &mut Receipt) -> Result<(), Status> {
    time(io, w, r)?;
    let mon = live(io)?;
    for (index, address) in NORMAL.into_iter().enumerate() {
        guard(io, 0, w, r)?;
        r.before[index] = Some(call(io, Op::Read(address))?);
    }
    if (live(io)? ^ mon) & LEVELS != 0 {
        return Err(Status::LevelDrift);
    }
    if r.before[0] != Some(0x0001_1de4)
        || r.before[1].is_none_or(|v| v & 6 != 0)
        || r.before[2] != Some(2)
        || r.before[3..6] != [Some(0); 3]
    {
        return Err(Status::Tuple);
    }
    let Some(ro) = r.before[7] else {
        return Err(Status::Tuple);
    };
    if ro & 1 != 0 {
        return Err(Status::RoEnabled);
    }
    let recipe = [
        (RO, ro | 1, 0),
        (DBI + 8, 0x0200_0000, 0),
        (SELECTOR, 1, 0),
        (DBI + 0x10, 0x3fff, 1),
        (DBI + 0x14, 0x3f_ffff, 1),
        (DBI + 0x18, 0xffff, 1),
        (SELECTOR, 0, 1),
        (DBI + 0x10, 0xffff_fff0, 0),
        (DBI + 0x14, 0xffff_fff0, 0),
        (DBI + 0x18, 0xffff_fff0, 0),
    ];
    let mut index = 0;
    for (address, value, selector) in recipe {
        guard(io, selector, w, r)?;
        r.writes_attempted |= 1 << index;
        call(io, Op::Write(address, value))?;
        r.writes_returned |= 1 << index;
        index += 1;
    }
    // BAR1 -> RP1 peripherals, BAR2 -> RP1 SRAM; never Linux DDR.
    let mut selected = 0;
    for (region, (selector, target, control)) in [
        (0x23, 0x4000_0000, 0xc000_0100),
        (0x63, 0x2000_0000, 0xc000_0200),
    ]
    .into_iter()
    .enumerate()
    {
        let values = [target, 0xc0, 0, control];
        for (address, value) in
            core::iter::once((SELECTOR, selector)).chain(INBOUND.into_iter().zip(values))
        {
            guard(io, selected, w, r)?;
            r.writes_attempted |= 1 << index;
            call(io, Op::Write(address, value))?;
            r.writes_returned |= 1 << index;
            index += 1;
            selected = selector;
        }
        for (word, (address, expected)) in INBOUND.into_iter().zip(values).enumerate() {
            guard(io, selected, w, r)?;
            if call(io, Op::Read(address))? != expected {
                return Err(Status::ReadbackMismatch);
            }
            r.inbound_readback |= 1 << (region * 4 + word);
        }
    }
    for (address, value) in [(SELECTOR, 0), (RO, ro)] {
        guard(io, selected, w, r)?;
        r.writes_attempted |= 1 << index;
        call(io, Op::Write(address, value))?;
        r.writes_returned |= 1 << index;
        index += 1;
        selected = 0;
    }
    for (index, address) in NORMAL.into_iter().enumerate() {
        guard(io, 0, w, r)?;
        r.after[index] = Some(call(io, Op::Read(address))?);
    }
    guard(io, 0, w, r)?;
    if r.after[0] != r.before[0]
        || r.after[1].is_none_or(|v| v & 6 != 0)
        || r.after[2..]
            != [
                Some(0x0200_0000),
                Some(0xffff_c000),
                Some(0xffc0_0000),
                Some(0xffff_0000),
                Some(0),
                Some(ro),
            ]
    {
        return Err(Status::ReadbackMismatch);
    }
    Ok(())
}

fn cleanup(w: &Window, io: &mut Io<'_>, r: &mut Receipt) -> Cleanup {
    // Never restore class/BARs/ATU; no DBI access once readiness or the deadline is lost.
    if live(io).is_err() || time(io, w, r).is_err() {
        return Cleanup::DeferredUnsafe;
    }
    r.cleanup_attempted |= 1;
    if call(io, Op::Write(SELECTOR, 0)).is_err() {
        return Cleanup::Failed;
    }
    r.cleanup_returned |= 1;
    if call(io, Op::Read(SELECTOR)) != Ok(0) {
        return Cleanup::Failed;
    }
    if live(io).is_err() || time(io, w, r).is_err() {
        return Cleanup::DeferredUnsafe;
    }
    let Some(ro) = r.before[7] else {
        return Cleanup::Failed;
    };
    r.cleanup_attempted |= 2;
    if call(io, Op::Write(RO, ro)).is_err() {
        return Cleanup::Failed;
    }
    r.cleanup_returned |= 2;
    if live(io).is_err() || time(io, w, r).is_err() {
        return Cleanup::DeferredUnsafe;
    }
    if call(io, Op::Read(SELECTOR)) != Ok(0) || call(io, Op::Read(RO)) != Ok(ro) {
        Cleanup::Failed
    } else {
        Cleanup::Restored
    }
}

#[cfg(test)]
mod tests;
