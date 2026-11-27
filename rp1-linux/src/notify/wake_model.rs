//! Host-only delivery model. A commit may be visible before its IRQ reaches the fd.
#[derive(Default)]
struct Uio {
    count: u32,
    baseline: u32,
    enabled: bool,
    edge: bool,
    latch_masked: bool,
    thread: bool,
}

impl Uio {
    fn open(&mut self) {
        self.baseline = self.count;
    }

    fn enable(&mut self) {
        self.enabled = true;
    }

    fn pulse(&mut self) {
        self.edge |= self.enabled || self.latch_masked;
    }

    fn handler(&mut self) {
        if self.enabled && std::mem::take(&mut self.edge) {
            self.enabled = false;
            self.thread = true;
        }
    }

    fn irq_thread(&mut self) -> Option<u32> {
        if !std::mem::take(&mut self.thread) {
            return None;
        }
        self.count = self.count.wrapping_add(1);
        Some(self.count)
    }

    fn readable(&self) -> bool {
        self.count != self.baseline
    }

    fn read(&mut self) -> Option<u32> {
        if !self.readable() {
            return None;
        }
        self.baseline = self.count;
        Some(self.count)
    }
}

#[test]
fn fd_ready_requires_thread_publication_after_the_handler() {
    let mut uio = Uio {
        count: u32::MAX,
        ..Uio::default()
    };
    uio.open();
    uio.pulse(); // Before the first enable, the edge may be lost.
    uio.handler();
    assert_eq!(uio.irq_thread(), None);
    assert!(!uio.readable());
    uio.enable();
    uio.pulse();
    assert!(!uio.readable());
    uio.handler();
    uio.pulse(); // Merged into the first wake while masked.
    assert!(!uio.readable());
    assert_eq!(uio.irq_thread(), Some(0));
    assert!(uio.readable());
    assert_eq!(uio.read(), Some(0));
    assert_eq!(uio.read(), None);
    assert!(!uio.enabled, "read alone must leave the source masked");
    uio.open();
    assert!(
        !uio.readable(),
        "a new listener starts at the latest device count"
    );
}

#[derive(Clone, Copy)]
enum Order {
    EnableThenScan,
    ScanThenEnable,
    NeverEnable,
}

fn missed_wakes(order: Order) -> usize {
    // Two endpoints share the vector. Endpoint A has work before open or supplies a
    // spurious IRQ. Inject B at every boundary up to and including entry to block.
    let mut misses = 0;
    for first_open in [false, true] {
        for injection in 0..=5 {
            for thread_before_block in [false, true] {
                let mut uio = Uio::default();
                uio.enable();
                uio.pulse();
                uio.handler();
                uio.irq_thread();
                let mut ready = [first_open, false];
                if first_open {
                    uio.open(); // A's IRQ is already in this listener's baseline.
                }
                let mut seen = [false; 2];
                let (enable, scan) = match order {
                    Order::EnableThenScan => (Some(2), 4),
                    Order::ScanThenEnable => (Some(4), 2),
                    Order::NeverEnable => (None, 4),
                };
                for step in 0..=5 {
                    if step == injection {
                        ready[1] = true;
                        uio.pulse();
                        uio.handler();
                    }
                    if step == 1 {
                        uio.read();
                    }
                    if Some(step) == enable {
                        uio.enable();
                    }
                    if step == scan {
                        seen = ready;
                    }
                    if step == 5 && thread_before_block {
                        uio.irq_thread();
                    }
                }
                if !seen[1] {
                    // Poll may wait for the handler's delayed UIO thread.
                    uio.irq_thread();
                    if uio.read().is_some() {
                        uio.enable();
                        seen = ready;
                    }
                }
                misses += usize::from(!seen[1] || first_open && !seen[0]);
            }
        }
    }
    misses
}

#[test]
fn enable_then_rescan_closes_initial_open_and_shared_irq_races() {
    assert_eq!(missed_wakes(Order::EnableThenScan), 0);
    assert!(missed_wakes(Order::ScanThenEnable) > 0);
    assert!(missed_wakes(Order::NeverEnable) > 0);
}

#[derive(Debug, Eq, PartialEq)]
enum DeadlineResult {
    Ready,
    NotifyLost,
    Timeout,
    PeerDown,
}

#[derive(Clone, Copy)]
enum DeadlineEvent {
    Irq,
    LostEdge,
    Idle,
    Restart,
}

fn at_deadline(event: DeadlineEvent, drain_before_scan: bool) -> DeadlineResult {
    let mut uio = Uio::default();
    uio.open();
    uio.enable();
    // Poll timed out before the UIO count advanced. The commit is visible while
    // the IRQ thread is queued; a lost physical edge has no thread to advance it.
    let mut ready = false;
    let mut live = true;
    match event {
        DeadlineEvent::Irq | DeadlineEvent::Restart => {
            ready = true;
            uio.pulse();
            uio.handler();
        }
        DeadlineEvent::LostEdge => ready = true,
        DeadlineEvent::Idle => {}
    }
    assert!(!uio.readable());
    uio.irq_thread();
    if matches!(event, DeadlineEvent::Restart) {
        live = false;
    }

    let mut drain = || {
        let delivered = uio.read().is_some();
        if delivered {
            uio.enable();
        }
        delivered
    };
    let delivered = drain_before_scan && drain();
    // The final acquire scan is the only one after sleeping. The restart case
    // has both a visible commit and a readable fd; session loss takes precedence.
    let result = match (live, ready, delivered) {
        (false, _, _) => DeadlineResult::PeerDown,
        (true, false, _) => DeadlineResult::Timeout,
        (true, true, true) => DeadlineResult::Ready,
        (true, true, false) => DeadlineResult::NotifyLost,
    };
    if !drain_before_scan {
        drain();
    }
    assert!(!uio.readable());
    result
}

#[test]
fn deadline_takes_a_delayed_irq_before_the_final_session_and_readiness_scan() {
    for (event, expected) in [
        (DeadlineEvent::Irq, DeadlineResult::Ready),
        (DeadlineEvent::LostEdge, DeadlineResult::NotifyLost),
        (DeadlineEvent::Idle, DeadlineResult::Timeout),
        (DeadlineEvent::Restart, DeadlineResult::PeerDown),
    ] {
        assert_eq!(at_deadline(event, true), expected);
    }
    assert_eq!(
        at_deadline(DeadlineEvent::Irq, false),
        DeadlineResult::NotifyLost
    );
}

#[derive(Clone, Copy, Debug)]
enum InitialVector {
    Enabled,
    Readable,
    Replay,
    IrqThreadPending,
}

impl InitialVector {
    fn uio(self) -> Uio {
        let mut uio = Uio {
            latch_masked: true,
            ..Uio::default()
        };
        uio.open();
        uio.enable();
        if !matches!(self, Self::Enabled) {
            uio.pulse();
            uio.handler();
            if !matches!(self, Self::IrqThreadPending) {
                uio.irq_thread();
            }
            if matches!(self, Self::Replay) {
                uio.pulse();
            }
        }
        uio
    }
}

fn intake(uio: &mut Uio) -> bool {
    if uio.read().is_none() {
        return false;
    }
    uio.enable();
    uio.handler();
    true
}

#[test]
fn direct_ticket_closes_the_empty_scan_to_poll_race() {
    #[derive(Clone, Copy, PartialEq, Debug)]
    enum Step {
        Publish,
        FirstScan,
        Intake,
        Arm,
        Force,
        Block,
        ReadFlag,
        Reply,
        FirstService,
        KickedService,
    }
    use Step::*;

    let host = [Publish, FirstScan, Intake, Arm, Force, Block];
    let peer = [ReadFlag, Reply, FirstService, KickedService];
    for positions in 0u16..1 << 10 {
        if positions.count_ones() != 4 {
            continue;
        }
        let (mut host, mut peer) = (host.into_iter(), peer.into_iter());
        let steps = std::array::from_fn::<_, 10, _>(|at| {
            if positions & (1 << at) == 0 {
                host.next().unwrap()
            } else {
                peer.next().unwrap()
            }
        });
        let at = |step| steps.iter().position(|&s| s == step).unwrap();
        if at(ReadFlag) < at(Publish) || at(KickedService) < at(Force) {
            continue;
        }
        for initial in [
            InitialVector::Enabled,
            InitialVector::Readable,
            InitialVector::Replay,
            InitialVector::IrqThreadPending,
        ] {
            for publish_count in 0..=steps.len() {
                let mut uio = initial.uio();
                let (mut ready, mut armed, mut blocked, mut sent, mut found) =
                    (false, false, false, false, false);
                for (at, step) in steps.into_iter().enumerate() {
                    if at == publish_count {
                        uio.irq_thread();
                    }
                    match step {
                        FirstScan if ready => found = true,
                        Intake if !found => {
                            intake(&mut uio);
                            found = ready;
                        }
                        Arm if !found => armed = true,
                        Block if !found => blocked = true,
                        Reply => ready = true,
                        FirstService | KickedService if ready && armed && !sent => {
                            // Only the matching arm can commit a seq and pulse.
                            sent = true;
                            uio.pulse();
                            uio.handler();
                        }
                        _ => {}
                    }
                    if blocked && !found && intake(&mut uio) && sent {
                        found = true;
                    }
                }
                // Drain delayed publication and at most one replay at the poll boundary.
                for _ in 0..2 {
                    uio.irq_thread();
                    if blocked && !found && intake(&mut uio) && sent {
                        found = true;
                    }
                }
                assert!(
                    found,
                    "{initial:?} {steps:?} publication at {publish_count}"
                );
                if matches!(initial, InitialVector::Enabled) {
                    assert!(!uio.edge && !uio.readable(), "RPC left a replay");
                    assert_eq!(uio.count, u32::from(armed));
                }
            }
        }
    }
}
