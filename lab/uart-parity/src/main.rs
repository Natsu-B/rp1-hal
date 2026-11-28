//! UART IRQ parity with an external peer or PL011 internal loopback.
#![no_std]
#![no_main]

use rp1_hal::{
    fugit::{ExtU32, RateExtU32},
    pac::pl011::int,
    timer::{Deadline, Timer, WaitTimer},
    uart,
};
use rp1_rt::{Proc0, Proc1, entry, interrupt::Slot, wait_for_event};

rp1_rt::bind!();
include!(concat!(env!("OUT_DIR"), "/instance.rs"));

#[inline(always)]
fn now() -> u32 {
    Timer::new().now_32().ticks()
}

#[inline(always)]
fn registers() -> &'static rp1_hal::pac::pl011::RegisterBlock {
    // Both cores read only non-acknowledging registers on the selected UART.
    unsafe { &*Registers::ptr().cast() }
}

const BAUD: u32 = 115_200;
const WATERMARK: usize = 16;
#[derive(Clone, Copy, Default, Debug, PartialEq)]
struct Counts {
    rx: u32,
    rt: u32,
    tx: u32,
    other: u32,
}

impl Counts {
    fn since(self, earlier: Self) -> Self {
        Self {
            rx: self.rx - earlier.rx,
            rt: self.rt - earlier.rt,
            tx: self.tx - earlier.tx,
            other: self.other - earlier.other,
        }
    }
}

#[derive(Clone, Copy, Default)]
struct Entry {
    at: u32,
    ris: u32,
    fr: u32,
    serviced: u32,
}

static PROBE: Slot<(Counts, Entry), 0, 4> = Slot::new((
    Counts {
        rx: 0,
        rt: 0,
        tx: 0,
        other: 0,
    },
    Entry {
        at: 0,
        ris: 0,
        fr: 0,
        serviced: 0,
    },
));

// Wrap the generated dispatcher while preserving its vector and normal service.
#[inline(always)]
fn observe_device(service: impl FnOnce() -> u32) {
    let at = now();
    let ris = registers().ris.read().bits();
    let fr = registers().fr.read().bits();
    let serviced = service();
    PROBE.try_with(|(counts, entry)| {
        *entry = Entry {
            at,
            ris,
            fr,
            serviced,
        };
        match serviced {
            bits if bits == int::RX || bits == int::RX | int::RT => counts.rx += 1,
            int::RT => counts.rt += 1,
            int::TX => counts.tx += 1,
            _ => counts.other += 1,
        }
    });
}

fn counts() -> Counts {
    PROBE.try_with(|(counts, _)| *counts).unwrap()
}

#[cfg(not(any(
    feature = "uart0",
    feature = "uart2",
    feature = "uart3",
    feature = "uart4"
)))]
#[path = "external.rs"]
mod external;

#[cfg(any(
    feature = "uart0",
    feature = "uart2",
    feature = "uart3",
    feature = "uart4"
))]
#[path = "loopback.rs"]
mod loopback;

#[cfg(feature = "uart0")]
use loopback::Report;
