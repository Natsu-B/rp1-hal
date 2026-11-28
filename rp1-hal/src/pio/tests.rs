extern crate std;
use super::*;
use crate::cold::RESETS_DONE0;
use core::cell::RefCell;

#[test]
fn reset_does_not_release_until_assertion_is_seen() {
    assert_eq!(
        (RESET_SET, RESET_CLEAR, RESET_DONE),
        (0x4001_6000, 0x4001_7000, RESETS_DONE0)
    );
    let state = RefCell::new((100, RESET, std::vec::Vec::new()));
    let mut reset = Reset::Fresh;
    let mut step = || {
        reset.step(
            |address| {
                let state = state.borrow();
                if address == TIMER_RAWL {
                    state.0
                } else {
                    assert_eq!(address, RESET_DONE);
                    state.1
                }
            },
            |address, value| state.borrow_mut().2.push((address, value)),
        )
    };
    assert_eq!(step(), Err(Error::NotReady));
    assert_eq!(state.borrow().2, [(RESET_SET, RESET)]);
    state.borrow_mut().1 = 0;
    assert_eq!(step(), Err(Error::NotReady));
    assert_eq!(state.borrow().2, [(RESET_SET, RESET), (RESET_CLEAR, RESET)]);
    state.borrow_mut().1 = RESET;
    assert_eq!(step(), Ok(()));
}

#[test]
fn reset_timeout_reasserts_reset_even_when_the_timer_wraps() {
    let time = Cell::new(u32::MAX - 7);
    let writes = RefCell::new(std::vec::Vec::new());
    let mut reset = Reset::Fresh;
    let mut step = || {
        reset.step(
            |address| if address == TIMER_RAWL { time.get() } else { 0 },
            |address, value| writes.borrow_mut().push((address, value)),
        )
    };
    assert_eq!(step(), Err(Error::NotReady));
    time.set(time.get().wrapping_add(1_000));
    assert_eq!(step(), Err(Error::Timeout));
    assert_eq!(step(), Err(Error::Timeout));
    assert_eq!(
        *writes.borrow(),
        [(RESET_SET, RESET), (RESET_CLEAR, RESET), (RESET_SET, RESET)]
    );
}

pub(super) fn block(pins: u32) -> PIO {
    PIO {
        clocks: Clocks::for_test(200_000_000, 50_000_000),
        pins,
        used: Cell::new(0),
        active: [const { Cell::new(0) }; 4],
        local: PhantomData,
    }
}
