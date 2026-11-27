//! Host-only delivery model. A commit may be visible before its IRQ reaches the fd.
#[derive(Default)]
struct Uio {
    count: u32,
    baseline: u32,
    enabled: bool,
    edge: bool,
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
        self.edge |= self.enabled;
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

#[derive(Clone, Copy, Debug, PartialEq)]
enum RequestStep {
    Enable,
    Publish,
    Scan,
    RequestSeen,
    ReplyPublished,
    Pulse,
    OtherPulse,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum RequestOrder {
    Ordered,
    FlagDropped,
    EnableAfterPublish,
    ReplyBeforeFlag,
}

fn request_misses(order: RequestOrder) -> usize {
    use RequestStep::*;
    let host = if order == RequestOrder::EnableAfterPublish {
        [Publish, Scan, Enable]
    } else {
        [Enable, Publish, Scan]
    };
    let peer = [RequestSeen, ReplyPublished, Pulse];
    let mut missed = 0;
    // Interleave both participants. Place an unrelated edge at each boundary
    // and allow its UIO thread to publish a count immediately or after scan.
    for peer_positions in 0u32..64 {
        if peer_positions.count_ones() != 3 {
            continue;
        }
        let mut steps = [OtherPulse; 7];
        let mut host_steps = host.into_iter();
        let mut peer_steps = peer.into_iter();
        for (index, step) in steps[..6].iter_mut().enumerate() {
            *step = if peer_positions & (1 << index) == 0 {
                host_steps.next().unwrap()
            } else {
                peer_steps.next().unwrap()
            };
        }
        let publish = steps.iter().position(|&step| step == Publish).unwrap();
        let seen = steps.iter().position(|&step| step == RequestSeen).unwrap();
        if seen <= publish {
            continue;
        }
        for other in 0..=7 {
            for thread in 0..=7 {
                let mut steps = steps;
                if other < 7 {
                    steps[other..].rotate_right(1);
                }
                let mut uio = Uio::default();
                uio.open();
                let mut flag = false;
                let mut requested = false;
                let mut ready = false;
                let mut found = false;
                for (index, step) in steps.into_iter().enumerate() {
                    if thread == index {
                        uio.irq_thread();
                    }
                    match step {
                        Enable => uio.enable(),
                        Publish => {
                            flag = order == RequestOrder::Ordered
                                || order == RequestOrder::EnableAfterPublish
                        }
                        Scan => found = ready,
                        RequestSeen => requested = flag,
                        ReplyPublished => {
                            ready = true;
                            flag |= order == RequestOrder::ReplyBeforeFlag;
                        }
                        Pulse if requested && !found => {
                            uio.pulse();
                            uio.handler();
                        }
                        OtherPulse if other < 7 => {
                            uio.pulse();
                            uio.handler();
                        }
                        _ => {}
                    }
                    // A poll may take an already readable shared count. The
                    // count is followed by re-enable, then an acquire scan.
                    if steps[..=index].contains(&Scan) && !found && uio.read().is_some() {
                        uio.enable();
                        found = ready;
                    }
                }
                uio.irq_thread();
                if !found && uio.read().is_some() {
                    uio.enable();
                    found = ready;
                }
                missed += usize::from(!found);
            }
        }
    }
    missed
}

#[test]
fn request_ticket_closes_publish_to_poll_race() {
    assert_eq!(request_misses(RequestOrder::Ordered), 0);
}

#[test]
fn request_ticket_mutations_miss_a_reply() {
    for mutation in [
        RequestOrder::FlagDropped,
        RequestOrder::EnableAfterPublish,
        RequestOrder::ReplyBeforeFlag,
    ] {
        assert!(request_misses(mutation) > 0, "{mutation:?}");
    }
}
