//! Read-only observation of the selected UART on proc1, using the same 1 MHz timer as proc0.
use crate::{now, registers as uart};
use core::sync::atomic::{
    AtomicU32,
    Ordering::{Acquire, Relaxed, Release},
};
use rp1_hal::pac::pl011::int;
use rp1_rt::{send_event, wait_for_event};

pub const BUSY: u32 = 1 << 3;
const TXFE: u32 = 1 << 7;

// Each mailbox has one writer. Shared SRAM supports atomic loads and stores.
#[unsafe(link_section = ".shared")]
static REQUEST: AtomicU32 = AtomicU32::new(0);
#[unsafe(link_section = ".shared")]
static STATE: AtomicU32 = AtomicU32::new(0);
#[unsafe(link_section = ".shared")]
static EDGES: [AtomicU32; 6] = [const { AtomicU32::new(0) }; 6];

fn await_state(state: u32) {
    let start = now();
    while STATE.load(Acquire) != state {
        assert!(now().wrapping_sub(start) <= 10_000, "sampler state {state}");
    }
}

pub fn arm(source: u32) {
    REQUEST.store(source, Release);
    send_event();
    await_state(1);
}

pub fn finish() -> [u32; 6] {
    await_state(2);
    let edges = EDGES.each_ref().map(|word| word.load(Relaxed));
    REQUEST.store(0, Release);
    await_state(0);
    edges
}

// The last negative observation is timestamped before its MMIO read. The
// first positive observation is timestamped after its read. No GPIO is needed.
fn edge(mut reached: impl FnMut() -> bool, on_low: impl FnOnce()) -> [u32; 2] {
    let mut low = now();
    assert!(!reached(), "sampler started after transition");
    on_low();
    loop {
        let before = now();
        if reached() {
            return [low, now()];
        }
        low = before;
    }
}

pub fn sample() -> ! {
    loop {
        let source = REQUEST.load(Acquire);
        if source == 0 {
            wait_for_event();
            continue;
        }
        let irq = edge(
            || uart().ris.read().bits() & source != 0,
            || STATE.store(1, Release),
        );
        let (empty, idle) = if source == int::TX {
            (
                edge(|| uart().fr.read().bits() & TXFE != 0, || {}),
                edge(|| uart().fr.read().bits() & BUSY == 0, || {}),
            )
        } else {
            ([0; 2], [0; 2])
        };
        for (word, value) in EDGES.iter().zip(irq.into_iter().chain(empty).chain(idle)) {
            word.store(value, Relaxed);
        }
        STATE.store(2, Release);
        while REQUEST.load(Acquire) != 0 {
            core::hint::spin_loop();
        }
        STATE.store(0, Release);
    }
}
