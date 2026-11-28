#![no_std]
#![no_main]

use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{AtomicBool, Ordering::Relaxed};
use rp1_hal::timer::Timer;
use rp1_rt::{Proc0, Proc1, entry};

rp1_rt::bind!();

const PIN_COUNT: usize = 28;
// UART0/1 and the live I²C1 bus are protected, including against pad-bias changes.
const ALLOWED: u32 = ((1 << PIN_COUNT) - 1) & !0xc00f;
const IO: usize = 0x400d_0004;
const PADS: usize = 0x400f_0004;
const RIO: usize = 0x400e_0000;
// SYS_RIO0 reset clear alias (bit 19). Match the GPIO HAL before using RIO.
const RESET_CLR: usize = 0x4001_7004;
const SYS_RIO0_RESET: u32 = 1 << 19;
const SET: usize = 0x2000;
const CLR: usize = 0x3000;
const RIO_FUNC: u32 = 5;
const OD: u32 = 1 << 7;
const IE: u32 = 1 << 6;
const UP: u32 = 1 << 3;
const DOWN: u32 = 1 << 2;
const SCHMITT: u32 = 1 << 1;
// Drive-strength field 0 means 2 mA, with slow slew.
const INPUT: u32 = OD | IE | SCHMITT;
const PULSE_US: u32 = 7000;
const PULSE_START_US: [u32; 6] = [0, 16000, 32000, 51000, 76000, 96000];

// Unused part of the RP1 link page, below SCMI at link offset 0x17c0.
// Linux owns word 0. Proc1 owns words 1 onward.
const MAIL: usize = 0x2000_c400;
const READY: usize = 1;
const DONE: usize = 2;
const PARK_ERROR: usize = 3;
const PASSED_BIAS: usize = 4;
const RESULT: usize = 5;
const BASELINE: usize = 6;
const PULSE_WORDS: usize = 7;
const CHECK_BIAS: u32 = 1;
const DRIVE_DOWN: u32 = 2;
const DRIVE_UP: u32 = 3;
// The runtime gives each core its own .bss. Only proc1 may park these pins.
static PROC1_SCANNING: AtomicBool = AtomicBool::new(false);
static mut READY_COOKIE: u32 = 0x4750_5343;

fn read(address: usize) -> u32 {
    unsafe { read_volatile(address as *const u32) }
}

fn write(address: usize, value: u32) {
    unsafe { write_volatile(address as *mut u32, value) }
}

fn put(word: usize, value: u32) {
    write(MAIL + word * 4, value);
}

fn barrier() {
    unsafe { core::arch::asm!("dmb sy", options(nostack, preserves_flags)) }
}

fn pad(pin: usize) -> usize {
    PADS + 4 * pin
}

fn ctrl(pin: usize) -> usize {
    IO + 8 * pin
}

fn pins() -> impl Iterator<Item = usize> {
    (0..PIN_COUNT).filter(|pin| ALLOWED & (1 << pin) != 0)
}

fn levels() -> u32 {
    read(RIO + 0xc) & ALLOWED
}

fn set_bias(pull: u32) {
    for pin in pins() {
        write(pad(pin), INPUT | pull);
    }
}

fn now() -> u32 {
    Timer::new().now_32().ticks()
}

fn wait_from(start: u32, us: u32) {
    while now().wrapping_sub(start) < us {}
}

#[entry(proc0)]
fn main0(mut core: Proc0, p: rp1::Proc0Resources) -> ! {
    let _sys = rp1::platform::init(&mut core, p.platform).unwrap();
    p.proc1.launch(&mut core).unwrap();
    loop {
        rp1_rt::wait_for_event();
    }
}

fn configure_inputs() -> [(u32, u32); PIN_COUNT] {
    // These bank accesses are limited to proc1-owned pins. Save the prior
    // mux to park it safely after the scan and use RIO for synchronized samples.
    write(RESET_CLR, SYS_RIO0_RESET);
    PROC1_SCANNING.store(true, Relaxed);
    let mut original = [(0, 0); PIN_COUNT];
    for pin in pins() {
        original[pin] = (read(ctrl(pin)), read(pad(pin)));
        write(pad(pin), INPUT);
    }
    write(RIO + CLR + 4, ALLOWED);
    for pin in pins() {
        write(ctrl(pin), RIO_FUNC);
    }
    barrier();
    original
}

fn park(original: &[(u32, u32); PIN_COUNT]) -> u32 {
    write(RIO + CLR + 4, ALLOWED);
    set_bias(0);
    let mut error = read(RIO + 4) & ALLOWED;
    for pin in pins() {
        let (control, pad_value) = original[pin];
        write(ctrl(pin), control);
        write(pad(pin), (pad_value | OD | IE) & !(UP | DOWN));
        if read(ctrl(pin)) != control || read(pad(pin)) & (OD | UP | DOWN) != OD {
            error |= 1 << pin;
        }
    }
    barrier();
    PROC1_SCANNING.store(false, Relaxed);
    error
}

fn check_bias() -> u32 {
    let mut passed = ALLOWED;
    for _ in 0..4 {
        for (pull, expect_high) in [(UP, true), (DOWN, false)] {
            set_bias(pull);
            wait_from(now(), 3000);
            for _ in 0..8 {
                let sample = levels();
                passed &= if expect_high { sample } else { !sample };
                wait_from(now(), 90);
            }
        }
    }
    passed
}

fn drive_pulse(pin: usize, pull: u32, high: bool) -> (u32, u32) {
    let bit = 1 << pin;
    // Keep the drive interval independent of RP1 interrupts and host latency.
    unsafe { core::arch::asm!("cpsid i", options(nostack, preserves_flags)) }
    write(RIO + if high { SET } else { CLR }, bit);
    write(pad(pin), INPUT);
    let start = now();
    write(pad(pin), INPUT & !OD);
    write(RIO + SET + 4, bit);
    wait_from(start, PULSE_US / 2);
    let sample = levels();
    wait_from(start, PULSE_US);
    write(RIO + CLR + 4, bit);
    write(pad(pin), INPUT | pull);
    barrier();
    let elapsed = now().wrapping_sub(start);
    unsafe { core::arch::asm!("cpsie i", options(nostack, preserves_flags)) }
    (elapsed, sample)
}

fn drive(pin: usize, pull: u32) -> u32 {
    set_bias(pull);
    wait_from(now(), 3000);
    let baseline = levels();
    put(BASELINE, baseline);
    let bit = 1 << pin;
    if (baseline & bit != 0) != (pull == UP) || read(RIO + 4) & ALLOWED != 0 {
        return 2;
    }
    let start = now();
    for (index, offset) in PULSE_START_US.into_iter().enumerate() {
        wait_from(start, offset);
        let started = now().wrapping_sub(start);
        let high = index % 2 == 0;
        let (elapsed, sample) = drive_pulse(pin, pull, high);
        let slot = PULSE_WORDS + 3 * index;
        put(slot, started);
        put(slot + 1, elapsed);
        put(slot + 2, sample);
        if elapsed > 10000 || (sample & bit != 0) != high {
            return 3;
        }
    }
    1
}

#[entry(proc1)]
fn main1(_core: Proc1, p: rp1::Proc1Resources) -> ! {
    let _owned_pins = p.pins;
    put(READY, 0);
    put(DONE, 0);
    put(PARK_ERROR, 0);
    put(PASSED_BIAS, 0);
    put(RESULT, 0);
    barrier();
    put(READY, unsafe { read_volatile(&raw const READY_COOKIE) });
    let mut last = 0;
    let mut admitted = 0;
    loop {
        let command = read(MAIL);
        if command == 0 || command == last {
            continue;
        }
        last = command;
        let mode = command & 0xff;
        let pin = ((command >> 8) & 0x1f) as usize;
        let result = match mode {
            CHECK_BIAS => {
                let original = configure_inputs();
                admitted = check_bias();
                put(PASSED_BIAS, admitted);
                put(PARK_ERROR, park(&original));
                1
            }
            DRIVE_DOWN | DRIVE_UP if admitted & (1 << pin) != 0 => {
                let original = configure_inputs();
                let result = drive(pin, if mode == DRIVE_UP { UP } else { DOWN });
                put(PARK_ERROR, park(&original));
                result
            }
            _ => 0,
        };
        put(RESULT, result);
        barrier();
        put(DONE, command);
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
    if PROC1_SCANNING.load(Relaxed) {
        write(RIO + CLR + 4, ALLOWED);
        set_bias(0);
    }
    rp1_rt::status::record_panic(info);
    loop {
        rp1_rt::wait_for_event();
    }
}
