//! Host-only delivery model. A commit may be visible before its IRQ reaches the fd.
use std::{
    io::Write,
    os::{fd::AsRawFd, unix::net::UnixStream},
};

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
    let (client, mut kernel) = UnixStream::pair().unwrap();
    let readable = || {
        let mut pfd = libc::pollfd {
            fd: client.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        assert!(unsafe { libc::poll(&mut pfd, 1, 0) } >= 0);
        pfd.revents & libc::POLLIN != 0
    };
    let mut uio = Uio {
        count: u32::MAX,
        ..Uio::default()
    };
    uio.open();
    uio.pulse(); // Before the first enable, the edge may be lost.
    uio.handler();
    assert_eq!(uio.irq_thread(), None);
    assert!(!readable() && !uio.readable());
    uio.enable();
    uio.pulse();
    assert!(!readable() && !uio.readable());
    uio.handler();
    uio.pulse(); // Merged into the first wake while masked.
    assert!(!readable() && !uio.readable());
    let count = uio.irq_thread().unwrap();
    kernel.write_all(&count.to_ne_bytes()).unwrap();
    assert!(readable() && uio.readable());
    assert_eq!(uio.read(), Some(0));
    assert_eq!(uio.read(), None);
    assert!(!uio.enabled, "read alone must leave the source masked");
    uio.open();
    assert!(
        !uio.readable(),
        "a new listener starts at the latest device count"
    );
}

#[test]
fn enable_then_rescan_closes_initial_open_and_shared_irq_races() {
    // Two endpoints share the vector. Endpoint A has work before open or supplies a
    // spurious IRQ. Inject B at every boundary up to and including entry to block.
    for first_open in [false, true] {
        for injection in 0..=5 {
            for thread_before_block in [false, true] {
                let mut uio = Uio::default();
                let mut ready = [false; 2];
                if first_open {
                    uio.enable();
                    uio.pulse();
                    uio.handler();
                    uio.irq_thread();
                    ready[0] = true;
                    uio.open(); // A's IRQ is already in this listener's baseline.
                } else {
                    uio.enable();
                    uio.pulse();
                    uio.handler();
                    uio.irq_thread();
                }
                let mut seen = [false; 2];
                for step in 0..=5 {
                    if step == injection {
                        ready[1] = true;
                        uio.pulse();
                        uio.handler();
                    }
                    match step {
                        1 => {
                            uio.read();
                        }
                        2 => uio.enable(),
                        3 => {}            // arm and read back both new tickets
                        4 => seen = ready, // final acquire scan of all conditions
                        5 if thread_before_block => {
                            uio.irq_thread();
                        }
                        _ => {}
                    }
                }
                if !seen[1] {
                    // Poll can block briefly while the UIO IRQ thread is still pending.
                    uio.irq_thread();
                    assert!(uio.read().is_some(), "injection={injection}");
                    uio.enable();
                    seen = ready;
                }
                assert!(seen[1], "injection={injection}");
                assert!(!first_open || seen[0]);
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum DeadlineResult {
    Ready,
    NotifyLost,
    Timeout,
    PeerDown,
}

#[test]
fn deadline_model_consumes_irq_before_the_final_session_and_readiness_scan() {
    for (ready, drop_edge, peer_down, want) in [
        (true, false, false, DeadlineResult::Ready),
        (true, true, false, DeadlineResult::NotifyLost),
        (false, true, false, DeadlineResult::Timeout),
        (true, false, true, DeadlineResult::PeerDown),
    ] {
        let mut uio = Uio::default();
        uio.open();
        uio.enable();
        assert!(!uio.readable()); // Already armed and blocked before the new commit.
        if !drop_edge {
            uio.pulse();
        }
        uio.handler();
        uio.irq_thread();
        let delivered = uio.read().is_some();
        if delivered {
            uio.enable();
        }
        let result = match (peer_down, ready, delivered) {
            (true, _, _) => DeadlineResult::PeerDown,
            (_, true, true) => DeadlineResult::Ready,
            (_, true, false) => DeadlineResult::NotifyLost,
            _ => DeadlineResult::Timeout,
        };
        assert_eq!(result, want);
    }
}
