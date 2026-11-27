//! The RP1 doorbell force survives until CLR. The core's SEV event survives until WFE.
#[derive(Default)]
struct Rp1 {
    ready: bool,
    intf: bool,
    event: bool,
}

impl Rp1 {
    fn publish(&mut self) {
        self.ready = true;
        self.intf = true;
    }

    fn clear(&mut self) {
        self.intf = false;
    }

    fn handler(&mut self) {
        if self.intf {
            self.clear();
            self.event = true;
        }
    }

    fn wfe(&mut self) -> bool {
        core::mem::take(&mut self.event)
    }
}

fn missed_events(clear_after_scan: bool) -> usize {
    let mut misses = 0;
    for inject in 0..=5 {
        let mut rp1 = Rp1 {
            intf: true,
            ..Rp1::default()
        }; // unrelated IRQ holds the level
        let mut ready = false;
        let mut awake = false;
        for step in 0..=5 {
            if step == inject {
                rp1.publish();
            }
            match step {
                0 if clear_after_scan => rp1.event = true, // Signal before deferring the CLR.
                0 => rp1.handler(),
                1 => {
                    assert!(rp1.wfe()); // Resume from the unrelated IRQ.
                }
                2 => ready = rp1.ready,
                3 if clear_after_scan => rp1.clear(),
                3 => {}
                4 => {
                    rp1.handler();
                    awake = rp1.wfe();
                }
                5 => rp1.handler(),
                _ => unreachable!(),
            }
        }
        // The final scan saw the commit, a retained event woke WFE, or the new
        // force reached the handler after WFE began.
        misses += usize::from(!(ready || awake || rp1.wfe()));
    }
    misses
}

#[test]
fn clear_before_final_scan_and_sticky_event_cover_every_boundary() {
    assert_eq!(missed_events(false), 0);
    assert!(missed_events(true) > 0);
}
