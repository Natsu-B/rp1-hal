extern crate std;
use core::cell::Cell;

std::thread_local! {
    static MASK: Cell<u8> = const { Cell::new(0) };
    static NEXT: Cell<Option<fn()>> = const { Cell::new(None) };
    static PENDING: Cell<Option<fn()>> = const { Cell::new(None) };
}

pub(super) fn primask_disable() -> u8 {
    MASK.replace(1)
}

pub(super) fn set_primask(saved: u8) {
    MASK.set(saved);
    if saved == 0
        && let Some(handler) = PENDING.take()
    {
        handler();
    }
}

pub(crate) fn preempt_at_next_point(handler: fn()) {
    NEXT.set(Some(handler));
}

pub(crate) fn interrupt_point() {
    if let Some(handler) = NEXT.take() {
        if MASK.get() == 0 {
            handler();
        } else {
            PENDING.set(Some(handler));
        }
    }
}
