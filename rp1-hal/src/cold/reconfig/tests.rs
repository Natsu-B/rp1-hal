//! Fault injection ported from the PoC's `endpoint_config.rs` tests (review #4).
extern crate std;
use super::*;
use std::{collections::BTreeMap, vec, vec::Vec};

pub(crate) struct Bus {
    pub regs: BTreeMap<usize, u32>,
    shadow: [u32; 3],
    pub trace: Vec<Op>,
    pub mask: u32,
    pub now: u32,
    pub writes: usize,
    fail_writes: Vec<usize>,
    apply_failed: bool,
    pub reset_after: Option<usize>,
    pub timeout_after: Option<usize>,
    ignore_write: Option<usize>,
    fail_read: Option<usize>,
    drift_at: Option<usize>,
    inbound: BTreeMap<(u32, usize), u32>,
}

impl Bus {
    pub fn new(mask: u32) -> Self {
        Self {
            regs: [
                (SELECTOR, 0),
                (MONITOR2, 0x0007_0020),
                (RO, 0x000b_ff40),
                (DBI, 0x0001_1de4),
                (DBI + 4, 0),
                (DBI + 8, 2),
                (DBI + 0x10, 0),
                (DBI + 0x14, 0),
                (DBI + 0x18, 0),
            ]
            .into(),
            shadow: [0; 3],
            trace: Vec::new(),
            mask,
            now: 1100,
            writes: 0,
            fail_writes: Vec::new(),
            apply_failed: true,
            reset_after: None,
            timeout_after: None,
            ignore_write: None,
            fail_read: None,
            drift_at: None,
            inbound: [0x23, 0x63]
                .into_iter()
                .flat_map(|s| INBOUND.map(|a| ((s, a), u32::MAX)))
                .collect(),
        }
    }

    pub fn io(&mut self, op: Op) -> Result<u32, ()> {
        self.trace.push(op);
        if let Op::Mask = op {
            let old = self.mask;
            self.mask = 1;
            return Ok(old);
        }
        if self.drift_at == Some(self.trace.len()) {
            *self.regs.get_mut(&MONITOR2).unwrap() ^= 1 << 20;
        }
        match op {
            Op::Mask => unreachable!(),
            Op::Restore(saved) => {
                self.mask = saved;
                Ok(saved)
            }
            Op::Clock => Ok(self.now),
            Op::Read(address) => {
                if address != MONITOR2 {
                    assert_eq!(self.mask, 1, "every DBI access is under PRIMASK");
                }
                if (DBI..DBI + 0x1000).contains(&address) {
                    assert_eq!(self.regs[&MONITOR2] & READY, READY, "no reset-low DBI read");
                    let selected = self.regs[&SELECTOR];
                    if selected != 0 {
                        assert!(
                            [0x23, 0x63].contains(&selected) && INBOUND.contains(&address),
                            "known inbound views only"
                        );
                        if self.fail_read == Some(address) {
                            return Err(());
                        }
                        return Ok(self.inbound[&(selected, address)]);
                    }
                }
                if self.fail_read == Some(address) {
                    return Err(());
                }
                Ok(self.regs[&address])
            }
            Op::Write(address, value) => {
                assert_eq!(self.mask, 1);
                assert!(
                    [SELECTOR, RO, DBI + 8, DBI + 0x10, DBI + 0x14, DBI + 0x18].contains(&address)
                        || INBOUND.contains(&address),
                    "forbidden write address"
                );
                assert_eq!(self.regs[&MONITOR2] & READY, READY, "no reset-low writes");
                self.writes += 1;
                let fail = self.fail_writes.contains(&self.writes);
                if (!fail || self.apply_failed) && self.ignore_write != Some(self.writes) {
                    if INBOUND.contains(&address) {
                        let selected = self.regs[&SELECTOR];
                        assert!([0x23, 0x63].contains(&selected));
                        self.inbound.insert((selected, address), value);
                    } else if (DBI + 0x10..=DBI + 0x18).contains(&address) {
                        let index = (address - DBI - 0x10) / 4;
                        if self.regs[&SELECTOR] == 1 {
                            self.shadow[index] = value;
                        } else {
                            self.regs.insert(address, value & !self.shadow[index]);
                        }
                    } else if address != DBI + 8 || self.regs[&RO] & 1 != 0 {
                        self.regs.insert(address, value);
                    }
                }
                if self.reset_after == Some(self.writes) {
                    self.regs.insert(MONITOR2, 0x0004_0020);
                }
                if self.timeout_after == Some(self.writes) {
                    self.now = 5100;
                }
                if fail { Err(()) } else { Ok(0) }
            }
        }
    }
}

fn window() -> Window {
    Window {
        low_us: 100,
        budget_us: 5000,
    }
}
fn run_bus(bus: &mut Bus) -> Receipt {
    run(&window(), &mut |op| bus.io(op))
}
fn guard_trace(trace: &mut Vec<Op>) {
    trace.extend([Op::Read(MONITOR2), Op::Read(SELECTOR), Op::Clock]);
}

#[test]
fn exact_recipe_guards_every_access() {
    for saved_mask in [0, 1] {
        let mut bus = Bus::new(saved_mask);
        let r = run_bus(&mut bus);
        assert_eq!((r.status, r.cleanup), (Status::Complete, Cleanup::Restored));
        assert_eq!(
            (r.mask_before, r.mask_restored, bus.mask),
            (Some(saved_mask), true, saved_mask)
        );
        assert_eq!(
            (r.writes_attempted, r.writes_returned),
            (0x3fffff, 0x3fffff)
        );
        assert_eq!(r.inbound_readback, 0xff);
        assert_eq!(
            r.after,
            [
                Some(0x11de4),
                Some(0),
                Some(0x0200_0000),
                Some(0xffff_c000),
                Some(0xffc0_0000),
                Some(0xffff_0000),
                Some(0),
                Some(0xbff40)
            ]
        );
        assert!(!r.needs_external_recovery());
        let mut expected = vec![Op::Mask, Op::Clock, Op::Read(MONITOR2)];
        for address in NORMAL {
            guard_trace(&mut expected);
            expected.push(Op::Read(address));
        }
        expected.push(Op::Read(MONITOR2));
        for (address, value) in [
            (RO, 0xbff41),
            (DBI + 8, 0x0200_0000),
            (SELECTOR, 1),
            (DBI + 0x10, 0x3fff),
            (DBI + 0x14, 0x3f_ffff),
            (DBI + 0x18, 0xffff),
            (SELECTOR, 0),
            (DBI + 0x10, 0xffff_fff0),
            (DBI + 0x14, 0xffff_fff0),
            (DBI + 0x18, 0xffff_fff0),
        ] {
            guard_trace(&mut expected);
            expected.push(Op::Write(address, value));
        }
        for (selector, target, ctrl) in [
            (0x23, 0x4000_0000, 0xc000_0100),
            (0x63, 0x2000_0000, 0xc000_0200),
        ] {
            for (address, value) in [
                (SELECTOR, selector),
                (DBI + 0x114, target),
                (DBI + 0x118, 0xc0),
                (DBI + 0x100, 0),
                (DBI + 0x104, ctrl),
            ] {
                guard_trace(&mut expected);
                expected.push(Op::Write(address, value));
            }
            for address in INBOUND {
                guard_trace(&mut expected);
                expected.push(Op::Read(address));
            }
        }
        for (address, value) in [(SELECTOR, 0), (RO, 0xbff40)] {
            guard_trace(&mut expected);
            expected.push(Op::Write(address, value));
        }
        for address in NORMAL {
            guard_trace(&mut expected);
            expected.push(Op::Read(address));
        }
        guard_trace(&mut expected);
        expected.push(Op::Restore(saved_mask));
        assert_eq!(bus.trace, expected);
        assert_eq!(bus.writes as u32, RECIPE_WRITES);
    }
}

#[test]
fn every_rejected_precondition_has_zero_writes() {
    for (address, value, status) in [
        (MONITOR2, 1 << 16, Status::NotReady),
        (MONITOR2, 1 << 17, Status::NotReady),
        (MONITOR2, 0, Status::NotReady),
        (SELECTOR, 1, Status::Selector),
        (SELECTOR, 0x23, Status::Selector),
        (DBI, 0, Status::Tuple),
        (DBI + 4, 2, Status::Tuple),
        (DBI + 4, 4, Status::Tuple),
        (DBI + 8, 0x0200_0002, Status::Tuple),
        (DBI + 0x10, 1, Status::Tuple),
        (DBI + 0x14, 1, Status::Tuple),
        (DBI + 0x18, 1, Status::Tuple),
        (RO, 0xbff41, Status::RoEnabled),
    ] {
        let mut bus = Bus::new(0);
        bus.regs.insert(address, value);
        let r = run_bus(&mut bus);
        assert_eq!(r.status, status, "address={address:x}");
        assert_eq!(
            (bus.writes, r.writes_attempted, r.cleanup),
            (0, 0, Cleanup::NotNeeded)
        );
        assert_eq!((bus.mask, r.mask_restored), (0, true));
        assert!(!r.needs_external_recovery());
    }
    let mut bus = Bus::new(1);
    bus.drift_at = Some(36); // The level re-check after reading the defaults.
    let r = run_bus(&mut bus);
    assert_eq!((r.status, bus.writes, bus.mask), (Status::LevelDrift, 0, 1));
    for address in NORMAL.into_iter().chain([MONITOR2]) {
        let mut bus = Bus::new(0);
        bus.fail_read = Some(address);
        let r = run_bus(&mut bus);
        assert_eq!(
            (r.status, bus.writes, bus.mask),
            (Status::CallbackFailure, 0, 0)
        );
    }
}

#[test]
fn mutation_failures_clean_up_without_claiming_rollback() {
    for failed in 1..=RECIPE_WRITES as usize {
        for apply in [false, true] {
            let mut bus = Bus::new(0);
            bus.fail_writes = vec![failed];
            bus.apply_failed = apply;
            let r = run_bus(&mut bus);
            assert_eq!(
                (r.status, r.cleanup),
                (Status::CallbackFailure, Cleanup::Restored)
            );
            assert_eq!(r.writes_attempted, (1 << failed) - 1);
            assert_eq!(r.writes_returned, (1 << (failed - 1)) - 1);
            assert_eq!((r.cleanup_attempted, r.cleanup_returned), (3, 3));
            assert_eq!(
                (bus.regs[&SELECTOR], bus.regs[&RO], bus.mask),
                (0, 0xbff40, 0)
            );
            assert!(r.needs_external_recovery());
        }
        let mut bus = Bus::new(1);
        bus.ignore_write = Some(failed);
        let r = run_bus(&mut bus);
        assert_ne!(
            r.status,
            Status::Complete,
            "returned write is not acceptance"
        );
        assert!(r.needs_external_recovery());
        assert!(r.mask_restored);
    }
}

/// Review #4: PERSTn re-asserted, or the deadline passing, right after *each* write stops the
/// transaction before the next access, and cleanup makes no further DBI access.
#[test]
fn reset_loss_and_deadline_after_every_write_are_terminal() {
    for changed in 1..=RECIPE_WRITES as usize {
        for reset in [false, true] {
            let mut bus = Bus::new(0);
            if reset {
                bus.reset_after = Some(changed);
            } else {
                bus.timeout_after = Some(changed);
            }
            let r = run_bus(&mut bus);
            assert_eq!(
                r.status,
                if reset {
                    Status::NotReady
                } else {
                    Status::Deadline
                }
            );
            assert_eq!(r.cleanup, Cleanup::DeferredUnsafe);
            assert_eq!(bus.writes, changed);
            assert_eq!(r.writes_attempted, (1 << changed) - 1);
            assert_eq!(r.cleanup_attempted, 0);
            assert_eq!(bus.mask, 0);
            assert!(r.needs_external_recovery());
        }
    }
}

#[test]
fn deadline_boundaries_and_wrap() {
    for (low, now, expected) in [
        (100, 5100, Status::Deadline),
        (100, 5099, Status::Complete),
        (u32::MAX - 100, 50, Status::Complete),
        (u32::MAX - 100, 4900, Status::Deadline),
    ] {
        let mut bus = Bus::new(1);
        bus.now = now;
        let r = run(
            &Window {
                low_us: low,
                budget_us: 5000,
            },
            &mut |op| bus.io(op),
        );
        assert_eq!(r.status, expected, "low={low} now={now}");
        if expected != Status::Complete {
            assert_eq!(bus.writes, 0);
        }
    }
    for budget in [0, 0x8000_0000] {
        let mut bus = Bus::new(0);
        let r = run(
            &Window {
                low_us: 100,
                budget_us: budget,
            },
            &mut |op| bus.io(op),
        );
        assert_eq!(r.status, Status::InvalidWindow);
        assert!(bus.trace.is_empty());
    }
}

#[test]
fn cleanup_never_exceeds_the_deadline() {
    for reset in [false, true] {
        let mut bus = Bus::new(0);
        bus.fail_writes = vec![1];
        if reset {
            bus.reset_after = Some(2);
        } else {
            bus.timeout_after = Some(2);
        }
        let r = run_bus(&mut bus);
        assert_eq!(
            (r.status, r.cleanup),
            (Status::CallbackFailure, Cleanup::DeferredUnsafe)
        );
        assert_eq!(
            (r.cleanup_attempted, r.cleanup_returned, bus.writes),
            (1, 1, 2)
        );
        assert!(r.needs_external_recovery());
        assert!(r.mask_restored);
    }
}

mod service {
    use super::*;
    use crate::cold::{EndpointService, ReconfigError};

    fn poll(
        s: &mut EndpointService,
        bus: &mut Bus,
    ) -> Option<Result<crate::cold::Reconfigured, ReconfigError>> {
        s.poll_with(&mut |op| bus.io(op))
    }

    #[test]
    fn normal_epoch_completes_once() {
        let mut s = unsafe { EndpointService::new() };
        let mut bus = Bus::new(0);
        bus.regs.insert(MONITOR2, 0);
        bus.now = 100;
        assert_eq!(poll(&mut s, &mut bus), None);
        bus.regs.insert(MONITOR2, 0x0007_0020);
        bus.now = 210;
        let r = poll(&mut s, &mut bus).unwrap().unwrap();
        assert_eq!(r.elapsed_us, 110);
        assert_eq!(bus.writes as u32, RECIPE_WRITES);
        // High and stable afterwards: no new transaction.
        assert_eq!(poll(&mut s, &mut bus), None);
        assert_eq!(bus.writes as u32, RECIPE_WRITES);
    }

    /// Review #4 scenario 1: CORE_ALIVE is already up when proc0 finally polls, but the 5 ms
    /// budget from PERSTn-low has passed. The old port reconfigured anyway; now nothing is written.
    #[test]
    fn late_poll_after_the_deadline_writes_nothing() {
        let mut s = unsafe { EndpointService::new() };
        let mut bus = Bus::new(0);
        bus.regs.insert(MONITOR2, 0);
        bus.now = 100;
        assert_eq!(poll(&mut s, &mut bus), None);
        bus.regs.insert(MONITOR2, 0x0007_0020);
        bus.now = 100 + 5_000;
        match poll(&mut s, &mut bus) {
            Some(Err(ReconfigError::Transaction(r))) => {
                assert_eq!(r.status, Status::Deadline);
                assert!(!r.needs_external_recovery());
            }
            other => panic!("{other:?}"),
        }
        assert_eq!(bus.writes, 0);
    }

    #[test]
    fn core_alive_timeout_and_partial_write_poisoning() {
        let mut s = unsafe { EndpointService::new() };
        let mut bus = Bus::new(0);
        bus.regs.insert(MONITOR2, 0);
        bus.now = 100;
        poll(&mut s, &mut bus);
        bus.regs.insert(MONITOR2, 0x0006_0020 & !(1 << 16)); // PERSTn high, no CORE_ALIVE.
        bus.now = 200;
        assert_eq!(poll(&mut s, &mut bus), None);
        bus.now = 5_100;
        assert!(matches!(
            poll(&mut s, &mut bus),
            Some(Err(ReconfigError::CoreAlive(_)))
        ));
        // Scenario 2: PERSTn drops again in the middle of the writes.
        bus.regs.insert(MONITOR2, 0);
        bus.now = 6_000;
        poll(&mut s, &mut bus);
        bus.regs.insert(MONITOR2, 0x0007_0020);
        bus.reset_after = Some(5);
        match poll(&mut s, &mut bus) {
            Some(Err(ReconfigError::Transaction(r))) => {
                assert_eq!(
                    (r.status, r.cleanup),
                    (Status::NotReady, Cleanup::DeferredUnsafe)
                );
                assert!(r.needs_external_recovery());
            }
            other => panic!("{other:?}"),
        }
        assert_eq!(bus.writes, 5);
        // The endpoint state is unknown: later epochs are refused, never retried.
        bus.regs.insert(MONITOR2, 0);
        poll(&mut s, &mut bus);
        bus.regs.insert(MONITOR2, 0x0007_0020);
        assert_eq!(poll(&mut s, &mut bus), Some(Err(ReconfigError::Poisoned)));
        assert_eq!(bus.writes, 5);
    }
}
