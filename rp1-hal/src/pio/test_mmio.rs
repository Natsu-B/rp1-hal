//! Thread-local register fixture for the PIO lifecycle trace.
extern crate std;

use core::cell::RefCell;
use std::{collections::BTreeMap, vec::Vec};

#[derive(Default)]
pub(crate) struct Trace {
    pub(crate) registers: BTreeMap<usize, u32>,
    pub(crate) writes: Vec<(usize, u32)>,
    reset_done_next: Option<u32>,
}

std::thread_local! {
    static TRACE: RefCell<Option<Trace>> = const { RefCell::new(None) };
}

pub(crate) fn record(trace: Trace, run: impl FnOnce()) -> Trace {
    TRACE.with(|slot| {
        assert!(slot.replace(Some(trace)).is_none());
        run();
        slot.take().unwrap()
    })
}

pub(crate) fn read(address: usize) -> Option<u32> {
    TRACE.with_borrow_mut(|slot| {
        let trace = slot.as_mut()?;
        let value = trace.registers.get(&address).copied().unwrap_or_default();
        if address == 0x4001_4018
            && let Some(next) = trace.reset_done_next.take()
        {
            trace.registers.insert(address, next);
        }
        Some(value)
    })
}

pub(crate) fn write(address: usize, value: u32) -> bool {
    TRACE.with_borrow_mut(|slot| {
        let Some(trace) = slot.as_mut() else {
            return false;
        };
        trace.writes.push((address, value));
        match address {
            // Leave DONE at its old value for one poll after each reset write.
            0x4001_6000 => {
                trace.reset_done_next = Some(0);
            }
            0x4001_7000 => {
                trace.reset_done_next = Some(value);
            }
            // GPIO native register aliases.
            0x400d_1000..=0x400d_3fff => {
                let register = trace.registers.entry(address & !0x3000).or_default();
                *register = match address & 0x3000 {
                    0x1000 => *register ^ value,
                    0x2000 => *register | value,
                    _ => *register & !value,
                };
            }
            _ => {
                trace.registers.insert(address, value);
            }
        }
        true
    })
}
