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

#[test]
fn clear_before_final_scan_and_sticky_event_cover_every_boundary() {
    for inject in 0..=4 {
        let mut rp1 = Rp1 {
            intf: true,
            ..Rp1::default()
        }; // unrelated IRQ holds the level
        let mut ready = false;
        let mut awake = false;
        for step in 0..=4 {
            if step == inject {
                rp1.publish();
            }
            match step {
                0 | 2 | 4 => rp1.handler(),
                1 => ready = rp1.ready,
                3 => awake = rp1.wfe(),
                _ => unreachable!(),
            }
        }
        // Either the final scan saw the commit, a retained event woke WFE, or the
        // new force bit reached the handler after WFE began.
        assert!(ready || awake || rp1.wfe(), "injection={inject}");
    }

    let mut rp1 = Rp1 {
        intf: true,
        ..Rp1::default()
    };
    assert!(!rp1.ready); // Broken handler checks data before clearing the force.
    rp1.publish(); // The new SET merges into the old, still-asserted force.
    rp1.clear();
    assert!(!rp1.wfe() && !rp1.intf && rp1.ready);
}
