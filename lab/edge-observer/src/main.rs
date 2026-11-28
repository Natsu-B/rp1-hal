//! Bounded stimuli for the ESP32 observer. Linux triggers trials through the RP1 link page.
#![no_std]
#![no_main]

use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering::Relaxed};
use embedded_hal::{
    digital::OutputPin,
    spi::{MODE_0, SpiBus},
};
use rp1_hal::{
    fugit::{ExtU32, RateExtU32},
    gpio::{PadConfig, Pull},
    pwm, spi,
    timer::{Alarm, Instant32, Timer},
};
use rp1_rt::{Proc0, Proc1, entry, interrupt};

rp1_rt::bind!();

// The Linux request and proc1 response occupy the fixture mailbox in the link page.
const MAIL: usize = 0x2000_c400;
const READY: u32 = 0x4544_4745;
const RIO: usize = 0x400e_0000;
const MARKERS: u32 = (1 << 12) | (1 << 13);
const SPI_MARKERS: u32 = (1 << 8) | (1 << 11);
const MARKER_COUNT: usize = 8;
const TICKS: u32 = 256;
const DATA: usize = 6;
const FIRST_TICK: usize = DATA + 2 * MARKER_COUNT * 3;
// PADS_BANK0: output disable, input enable and Schmitt trigger; DRIVE=0 is
// 2 mA, SLEWFAST=0 is slow slew.
const OD: u32 = 1 << 7;
const IE: u32 = 1 << 6;
const PULL_DOWN: u32 = 1 << 2;
const SCHMITT: u32 = 1 << 1;
const INPUT: u32 = OD | IE | SCHMITT;
const RIO_FUNC: u32 = 5;
const RIO_SET: usize = 0x2000;
const RIO_CLR: usize = 0x3000;
const RIO_OE: usize = 4;
const RIO_IN_SYNC: usize = 0xc;
const TIMER: Timer = Timer::new();

static TICK: AtomicU32 = AtomicU32::new(0);
// The runtime gives each core its own .bss. Only proc1 may park these pins.
static PROC1_OWNS_PINS: AtomicBool = AtomicBool::new(false);
static DEADLINE: AtomicU32 = AtomicU32::new(0);
static ERROR: AtomicU32 = AtomicU32::new(0);
static mut ALARM: core::mem::MaybeUninit<Alarm<1>> = core::mem::MaybeUninit::uninit();

fn read(address: usize) -> u32 {
    unsafe { read_volatile(address as *const u32) }
}

fn write(address: usize, value: u32) {
    unsafe { write_volatile(address as *mut u32, value) }
}

fn put(word: usize, value: u32) {
    write(MAIL + word * 4, value);
}

fn now() -> u32 {
    TIMER.now_32().ticks()
}

fn pad(pin: usize) -> usize {
    0x400f_0004 + 4 * pin // PADS_BANK0 GPIO0 pad, then one word per pin
}

fn ctrl(pin: usize) -> usize {
    0x400d_0004 + 8 * pin // IO_BANK0 GPIO0 CTRL, then STATUS/CTRL pairs
}

fn edge(mask: u32, high: bool, before: u32, slot: usize) -> bool {
    write(RIO + if high { RIO_SET } else { RIO_CLR }, mask);
    let expected = if high { mask } else { 0 };
    let (levels, after) = loop {
        let levels = read(RIO + RIO_IN_SYNC) & mask;
        let after = now();
        if levels == expected || after.wrapping_sub(before) >= 5 {
            break (levels, after);
        }
    };
    put(slot, before);
    put(slot + 1, after);
    put(slot + 2, levels);
    levels == expected
}

fn pwm_rate(slow: bool) -> rp1_hal::fugit::HertzU32 {
    if slow { 1.kHz() } else { 10.kHz() }
}

fn markers(group: usize, mask: u32) -> bool {
    let mut good = true;
    for i in 0..MARKER_COUNT {
        // Vary the spacing to avoid sampling the same quantization phase at
        // every edge. The two groups still fit within the ESP32's 5 s window.
        TIMER
            .delay_polling((1137 + 217 * i as u32).micros())
            .unwrap();
        good &= edge(
            mask,
            i % 2 == 0,
            now(),
            DATA + (group * MARKER_COUNT + i) * 3,
        );
    }
    good
}

// Proc1 owns these pads through its generated GPIO, PWM and SPI resources.
// The RIO aliases switch both marker pins in one write. Any alternate route
// must be idle before taking over its pads for the marker sequence.
fn gpio_markers(mask: u32, enable: bool) {
    write(RIO + RIO_CLR + RIO_OE, mask);
    write(RIO + RIO_CLR, mask);
    for pin in [8, 11, 12, 13] {
        if mask & (1 << pin) != 0 {
            write(pad(pin), INPUT | PULL_DOWN);
            write(ctrl(pin), RIO_FUNC);
            if enable {
                write(pad(pin), INPUT & !OD);
            }
        }
    }
    if enable {
        write(RIO + RIO_SET + RIO_OE, mask);
    }
}

#[interrupt(proc1)]
fn TIMER_1() {
    let entered = now();
    // Proc1 main only accesses the alarm while the NVIC line is disabled.
    let alarm = unsafe { &mut *(&raw mut ALARM).cast::<Alarm<1>>() };
    let i = TICK.load(Relaxed);
    let deadline = DEADLINE.load(Relaxed);
    if !edge(1 << 13, i % 2 == 0, entered, FIRST_TICK + i as usize * 3) {
        ERROR.store(1, Relaxed);
    }
    alarm.clear_interrupt();
    let next = deadline.wrapping_add(1000);
    if i + 1 == TICKS {
        alarm.disable_interrupt();
    } else if alarm.try_arm(Instant32::from_ticks(next)).is_err() {
        alarm.disable_interrupt();
        ERROR.store(2, Relaxed);
    }
    DEADLINE.store(next, Relaxed);
    TICK.store(i + 1, Relaxed);
}

#[entry(proc0)]
fn main0(mut core: Proc0, p: rp1::Proc0Resources) -> ! {
    let _sys = rp1::platform::init(&mut core, p.platform).unwrap();
    p.proc1.launch(&mut core).unwrap();
    loop {
        rp1_rt::wait_for_event();
    }
}

#[entry(proc1)]
fn main1(core: Proc1, p: rp1::Proc1Resources) -> ! {
    let clocks = rp1_hal::clocks::Clocks::wait_from_proc0(&core, 10_000.micros()).unwrap();
    PROC1_OWNS_PINS.store(true, Relaxed);
    let mut irq = p.irq.TIMER_1;
    unsafe { (&raw mut ALARM).write(core::mem::MaybeUninit::new(p.TIMER_ALARM1)) };

    let _cs1 = p.pins.gpio7.into_input();
    let sense = PadConfig::INPUT_DEFAULT.with_pull(Pull::None);
    let _in_a = p.pins.gpio22.into_input_with_pad(sense);
    let _in_b = p.pins.gpio23.into_input_with_pad(sense);

    let mut spi = p
        .spi0
        .init(&clocks, spi::Config::new(1.MHz(), MODE_0))
        .unwrap();
    let spi_hz = spi.frequency().raw();
    let mut cs = p.pins.gpio8.into_output(true);
    let pwm_channels = p.pwm0.init(&clocks).unwrap();

    let mut channels = Some((pwm_channels.channel0, pwm_channels.channel1));
    let mut pins = Some((p.pins.gpio12, p.pins.gpio13));
    let mut pwm: Option<(
        rp1_hal::pwm::Output<0, 0, 12>,
        rp1_hal::pwm::Output<0, 1, 13>,
    )> = None;
    let mut spi_pads = None;

    gpio_markers(MARKERS, true);
    put(2, 0);
    put(1, READY);
    let mut last = 0;
    loop {
        let command = read(MAIL);
        if command == 0 || command == last {
            continue;
        }
        last = command;
        let (ok, count) = match command & 0xff {
            1 | 4 if pwm.is_none() => {
                let before = markers(0, MARKERS);
                let jitter = command & 0xff == 4;
                let mut alarm_ok = true;
                if jitter {
                    let alarm = unsafe { &mut *(&raw mut ALARM).cast::<Alarm<1>>() };
                    TICK.store(0, Relaxed);
                    ERROR.store(0, Relaxed);
                    let deadline = now().wrapping_add(40_000);
                    put(5, deadline);
                    DEADLINE.store(deadline, Relaxed);
                    alarm.clear_interrupt();
                    alarm.try_arm(Instant32::from_ticks(deadline)).unwrap();
                    alarm.enable_interrupt();
                    irq.enable();
                    while TICK.load(Relaxed) < TICKS
                        && ERROR.load(Relaxed) == 0
                        && now().wrapping_sub(deadline) as i32 <= 275_000
                    {}
                    irq.disable();
                    let alarm = unsafe { &mut *(&raw mut ALARM).cast::<Alarm<1>>() };
                    alarm.disable_interrupt();
                    alarm.cancel();
                    alarm.clear_interrupt();
                    alarm_ok = TICK.load(Relaxed) == TICKS && ERROR.load(Relaxed) == 0;
                    TIMER.delay_polling(40_000.micros()).unwrap();
                } else {
                    TIMER.delay_polling(2_000_000.micros()).unwrap();
                }
                let after = markers(1, MARKERS);
                let count = 2 * MARKER_COUNT + if jitter { TICKS as usize } else { 0 };
                (before && after && alarm_ok, count * 3)
            }
            2 | 3 if pwm.is_some() => {
                let (a, b) = pwm.take().unwrap();
                let expected = pwm_rate(command & 0xff == 2);
                let window = 130_000_000 / expected.raw();
                let actual = a.actual();
                let period = actual.cycle_ticks() as u32;
                put(DATA, period);
                put(DATA + 1, period / 2);
                put(DATA + 2, actual.clock().raw());
                put(DATA + 3, window);
                let b_actual = b.actual();
                let a = a.enable().unwrap();
                let b = b.enable().unwrap();
                TIMER.delay_polling(window.micros()).unwrap();
                pwm = Some((a.disable().unwrap(), b.disable().unwrap()));
                (actual == b_actual && actual.frequency_hz() == expected, 4)
            }
            5 if spi_pads.is_none() => {
                let mut bus = spi.polling();
                cs.set_low().unwrap();
                TIMER.delay_polling(4.micros()).unwrap();
                let result = bus
                    .write(&[0xa5u8])
                    .and_then(|()| <_ as SpiBus<u8>>::flush(&mut bus));
                TIMER.delay_polling(4.micros()).unwrap();
                cs.set_high().unwrap();
                put(DATA, spi_hz);
                (result.is_ok(), 1)
            }
            6 if spi_pads.is_some() => {
                let before = markers(0, SPI_MARKERS);
                TIMER.delay_polling(40_000.micros()).unwrap();
                let after = markers(1, SPI_MARKERS);
                (before && after, 2 * MARKER_COUNT * 3)
            }
            7 | 12 => {
                if let Some((a, b)) = pwm.take() {
                    let (ch0, pin_a) = a.try_free().unwrap();
                    let (ch1, pin_b) = b.try_free().unwrap();
                    channels = Some((ch0, ch1));
                    pins = Some((pin_a, pin_b));
                }
                gpio_markers(MARKERS, command & 0xff == 7);
                (true, 0)
            }
            8 | 9 if pwm.is_none() => {
                let (pin_a, pin_b) = pins.take().unwrap();
                let (ch0, ch1) = channels.take().unwrap();
                let mut a = ch0.output(pin_a);
                let mut b = ch1.output(pin_b);
                let config = pwm::Config::from_frequency(pwm_rate(command & 0xff == 8));
                let a_ticks = a.configure(config).unwrap().cycle_ticks() as u32;
                let b_ticks = b.configure(config).unwrap().cycle_ticks() as u32;
                a.try_set_duty_ticks(pwm::DutyTicks::new(a_ticks / 2))
                    .unwrap();
                b.try_set_duty_ticks(pwm::DutyTicks::new(b_ticks / 2))
                    .unwrap();
                pwm = Some((a, b));
                (true, 0)
            }
            10 if spi_pads.is_none() => {
                spi_pads = Some([
                    (read(ctrl(8)), read(pad(8))),
                    (read(ctrl(11)), read(pad(11))),
                ]);
                gpio_markers(SPI_MARKERS, true);
                (true, 0)
            }
            11 if spi_pads.is_some() => {
                gpio_markers(SPI_MARKERS, false);
                // Restore the HAL's RIO-high CS and idle-low SPI clock with outputs gated.
                write(RIO + RIO_SET, 1 << 8);
                for (pin, (control, value)) in [8, 11].into_iter().zip(spi_pads.take().unwrap()) {
                    write(ctrl(pin), control);
                    if pin == 8 {
                        write(RIO + RIO_SET + RIO_OE, 1 << pin);
                    }
                    write(pad(pin), value);
                }
                (true, 0)
            }
            _ => (false, 0),
        };
        put(3, u32::from(ok));
        put(4, count as u32);
        unsafe { core::arch::asm!("dmb sy", options(nostack, preserves_flags)) }
        put(2, command);
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
    if PROC1_OWNS_PINS.load(Relaxed) {
        gpio_markers(MARKERS | SPI_MARKERS, false);
    }
    rp1_rt::status::record_panic(info);
    loop {
        rp1_rt::wait_for_event();
    }
}
