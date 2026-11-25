use super::*;
use core::cell::{Cell, RefCell};
extern crate std;
use std::panic::{AssertUnwindSafe, catch_unwind, panic_any};
use std::vec::Vec;

#[derive(Debug, PartialEq, Eq)]
enum Event {
    ReadCfsr,
    ReadFrame(usize),
    WriteFrame(usize, u32),
    Record(u32, u32, u32),
    ClearCfsr(u32),
    Basepri(u32),
    Sync,
    Stop,
}
use Event::*;

struct Stopped;

struct Fake {
    frame: Option<Cell<[u32; 8]>>,
    cfsr: u32,
    events: RefCell<Vec<Event>>,
}

impl Fake {
    fn new(cfsr: u32, frame: Option<[u32; 8]>) -> Self {
        Self {
            frame: frame.map(Cell::new),
            cfsr,
            events: RefCell::default(),
        }
    }
}

impl FaultCpu for Fake {
    fn cfsr(&self) -> u32 {
        self.events.borrow_mut().push(ReadCfsr);
        self.cfsr
    }

    fn ipsr(&self) -> u32 {
        6
    }

    fn read_frame(&self, index: usize) -> u32 {
        self.events.borrow_mut().push(ReadFrame(index));
        self.frame.as_ref().expect("invalid frame").get()[index]
    }

    fn write_frame(&self, index: usize, value: u32) {
        self.events.borrow_mut().push(WriteFrame(index, value));
        let frame = self.frame.as_ref().expect("invalid frame");
        let mut words = frame.get();
        words[index] = value;
        frame.set(words);
    }

    fn record(&self, exception: u32, pc: u32, cfsr: u32) {
        self.events.borrow_mut().push(Record(exception, pc, cfsr));
    }

    fn clear_cfsr(&self, bits: u32) {
        self.events.borrow_mut().push(ClearCfsr(bits));
    }

    fn write_basepri(&self, mask: u8) {
        self.events.borrow_mut().push(Basepri(mask.into()));
    }

    fn sync(&self) {
        self.events.borrow_mut().push(Sync);
    }

    fn stop(&self) -> ! {
        self.events.borrow_mut().push(Stop);
        panic_any(Stopped)
    }
}

#[test]
fn p2_returns_to_park_with_only_pc_and_xpsr_changed() {
    const PC: u32 = 0x2000_1234;
    const PARK: u32 = 0x2000_5679;
    const CFSR: u32 = (1 << 8) | (1 << 10);
    const T: u32 = 1 << 24;
    for ipsr in [0, 16 + 42] {
        for align in [0, 1 << 9] {
            let parked_xpsr = T | ipsr | align;
            // Plain Thumb state, IT state, ICI state, NZCVQ, then T cleared.
            for state in [
                T,
                T | (1 << 25) | (0b1100 << 12),
                T | (0b0101 << 12),
                T | 0xf800_0000,
                0,
            ] {
                let frame = [1, 2, 3, 4, 5, 6, PC, ipsr | align | state];
                let fake = Fake::new(CFSR, Some(frame));
                configurable_fault(&fake, PARK);
                assert_eq!(
                    fake.frame.unwrap().get(),
                    [1, 2, 3, 4, 5, 6, PARK & !1, parked_xpsr]
                );
                assert_eq!(
                    fake.events.into_inner(),
                    [
                        ReadCfsr,
                        ReadFrame(6),
                        Record(6, PC, CFSR),
                        ClearCfsr(CFSR),
                        WriteFrame(6, PARK & !1),
                        ReadFrame(7),
                        WriteFrame(7, parked_xpsr),
                        Basepri(0x40),
                        Sync,
                    ]
                );
            }
        }
    }
}

#[test]
fn p3_terminal_configurable_faults_never_access_a_frame_or_return() {
    // MSTKERR, MUNSTKERR, STKERR, UNSTKERR, INVPC.
    for bit in [4, 3, 12, 11, 18] {
        let fake = Fake::new(1 << bit, None);
        let result = catch_unwind(AssertUnwindSafe(|| configurable_fault(&fake, 0)));
        assert!(result.unwrap_err().is::<Stopped>());
        assert_eq!(
            fake.events.into_inner(),
            [ReadCfsr, Record(6, 0, 1 << bit), Stop]
        );
    }

    // P2e checks EXC_RETURN[2] dispatches to the PSP entry before reading MSP or calling Rust.
    let fake = Fake::new(0, None);
    let result = catch_unwind(AssertUnwindSafe(|| psp_fault(&fake)));
    assert!(result.unwrap_err().is::<Stopped>());
    assert_eq!(fake.events.into_inner(), [ReadCfsr, Record(6, 0, 0), Stop]);
}

#[test]
fn hardfault_records_the_pc_only_with_a_usable_msp_frame() {
    for (cfsr, exc_return) in [
        (0, 0xffff_fffd),
        (1 << 4, 0xffff_fff9),
        (1 << 11, 0xffff_fff1),
    ] {
        let fake = Fake::new(cfsr, None);
        let result = catch_unwind(AssertUnwindSafe(|| hardfault(&fake, exc_return)));
        assert!(result.unwrap_err().is::<Stopped>());
        assert_eq!(
            fake.events.into_inner(),
            [ReadCfsr, Record(3, 0, cfsr), Stop]
        );
    }

    let fake = Fake::new(INVPC, Some([0, 0, 0, 0, 0, 0, 42, 0]));
    let result = catch_unwind(AssertUnwindSafe(|| hardfault(&fake, 0xffff_fff9)));
    assert!(result.unwrap_err().is::<Stopped>());
    assert_eq!(
        fake.events.into_inner(),
        [ReadCfsr, ReadFrame(6), Record(3, 42, INVPC), Stop]
    );
}
