//! ADC_FIFO delivery, deadline, legacy OneShot and one-shot recovery on the internal sensor.
#![no_std]
#![no_main]

use core::sync::atomic::{AtomicBool, AtomicU32, Ordering::Relaxed};
use embedded_hal_0_2::adc::OneShot;
use rp1_hal::{
    adc::{Error, TEMPERATURE},
    log,
    pac::{self, generated::adc::RegisterBlock},
    timer::{Deadline, Timer, WaitTimer},
};
use rp1_rt::{Proc0, Proc1, entry, interrupt::Slot};

rp1_rt::bind!();

const START_ONCE: u32 = 1 << 2;
const THRESHOLD: u32 = 1;

static IRQ: AtomicU32 = AtomicU32::new(0);
static STRAY: AtomicU32 = AtomicU32::new(0);
static ALARM: AtomicU32 = AtomicU32::new(0);
static DROPPED: AtomicU32 = AtomicU32::new(0);
static SUPPRESS: AtomicBool = AtomicBool::new(false);
static MASKED: Slot<(), 0, 4> = Slot::new(());

fn now() -> u32 {
    Timer::new().now_32().ticks()
}

#[inline(always)]
fn registers() -> &'static RegisterBlock {
    // SAFETY: this fixture owns the ADC on proc0. The PAC uses volatile accesses.
    unsafe { &*pac::ADC::PTR.cast::<RegisterBlock>() }
}

#[inline(always)]
fn count(counter: &AtomicU32) {
    // Each counter has exactly one ISR writer. Thread mode reads it after the trial.
    counter.store(counter.load(Relaxed) + 1, Relaxed);
}

#[inline(always)]
fn observe_adc(dispatch: impl FnOnce() -> u32) {
    let adc = registers();
    if SUPPRESS.load(Relaxed)
        && adc.ints().read().bits() == THRESHOLD
        && !adc.fcs().read().empty().bits()
    {
        // Lose the IRQ publication and its sample. Mask first, as the source is level-triggered.
        // SAFETY: INTE is owned by the active test and the FIFO is nonempty.
        unsafe { adc.inte().write_bits_unchecked(0) };
        adc.fifo().read();
        count(&DROPPED);
    } else if dispatch() == THRESHOLD {
        count(&IRQ);
    } else {
        count(&STRAY);
    }
}

#[inline(always)]
fn observe_wait_alarm() {
    // SAFETY: called by the generated handler for ALARM1.
    unsafe { WaitTimer::__on_interrupt::<1>() };
    count(&ALARM);
}

#[derive(Clone, Copy)]
struct Counts {
    irq: u32,
    stray: u32,
    alarm: u32,
    dropped: u32,
}

impl Counts {
    fn read() -> Self {
        Self {
            irq: IRQ.load(Relaxed),
            stray: STRAY.load(Relaxed),
            alarm: ALARM.load(Relaxed),
            dropped: DROPPED.load(Relaxed),
        }
    }

    fn since(self) -> Self {
        let end = Self::read();
        Self {
            irq: end.irq - self.irq,
            stray: end.stray - self.stray,
            alarm: end.alarm - self.alarm,
            dropped: end.dropped - self.dropped,
        }
    }
}

fn ready() {
    let start = now();
    loop {
        let cs = registers().cs().read();
        if cs.ready().bits() && !cs.start_once().bits() {
            break;
        }
        assert!(
            now().wrapping_sub(start) <= 1_000,
            "ADC did not become ready"
        );
    }
}

fn unsaturated(sample: u16) {
    assert!((1..=4094).contains(&sample));
}

fn irq_samples(adc: &mut rp1::Adc, wait: &mut WaitTimer) -> Result<(), Error> {
    let mut adc = adc.blocking(wait);
    for i in 0..100 {
        let counts = Counts::read();
        let start = now();
        let raw = adc.read(TEMPERATURE)?;
        let elapsed = now().wrapping_sub(start);
        let Counts {
            irq,
            stray,
            alarm,
            dropped,
        } = counts.since();
        log!("ADC_IRQ_SAMPLE i={i} raw={raw} us={elapsed} irq={irq} stray={stray} alarm={alarm} dropped={dropped}\n").unwrap();
    }
    Ok(())
}

fn legacy_samples(adc: &mut rp1::Adc) {
    let mut input = TEMPERATURE;
    let counts = Counts::read();
    for i in 0..100 {
        let start = now();
        assert_eq!(OneShot::read(adc, &mut input), Err(nb::Error::WouldBlock));
        let raw = loop {
            match OneShot::read(adc, &mut input) {
                Ok(raw) => break raw,
                Err(nb::Error::WouldBlock) => assert!(now().wrapping_sub(start) <= 1_000),
                Err(error) => panic!("legacy OneShot error: {error:?}"),
            }
        };
        let elapsed = now().wrapping_sub(start);
        log!("ADC_LEGACY_SAMPLE i={i} raw={raw} us={elapsed}\n").unwrap();
    }
    let Counts {
        irq,
        stray,
        alarm,
        dropped,
    } = counts.since();
    log!("ADC_LEGACY_END trials=100 would_block=100 irq={irq} stray={stray} alarm={alarm} dropped={dropped}\n").unwrap();
}

fn faults(adc: &mut rp1::Adc, wait: &mut WaitTimer) -> Result<(), Error> {
    let counts = Counts::read();
    let start = now();
    // SAFETY: IRQ52 is idle and owned by this test. Only its FIFO bit is forced.
    unsafe { registers().intf().write_bits_unchecked(THRESHOLD) };
    while STRAY.load(Relaxed) == counts.stray {
        assert!(
            now().wrapping_sub(start) <= 100,
            "forced ADC IRQ did not retire"
        );
    }
    let elapsed = now().wrapping_sub(start);
    assert_eq!(registers().ints().read().bits(), 0);
    let Counts {
        irq,
        stray,
        alarm,
        dropped,
    } = counts.since();
    log!(
        "ADC_STRAY us={elapsed} irq={irq} stray={stray} alarm={alarm} dropped={dropped} lowered=1\n"
    )
    .unwrap();

    for i in 0..10 {
        let counts = Counts::read();
        SUPPRESS.store(true, Relaxed);
        let start = now();
        let result = adc.blocking(wait).read(TEMPERATURE);
        let elapsed = now().wrapping_sub(start);
        SUPPRESS.store(false, Relaxed);
        assert_eq!(result, Err(Error::Timeout));
        assert_eq!(adc.try_sample(), Err(Error::NoConversion));
        let Counts {
            irq,
            stray,
            alarm,
            dropped,
        } = counts.since();
        let retry_counts = Counts::read();
        let retry_start = now();
        let raw = adc.blocking(wait).read(TEMPERATURE)?;
        let retry_us = now().wrapping_sub(retry_start);
        let retry = retry_counts.since();
        log!("ADC_TIMEOUT i={i} us={elapsed} irq={irq} stray={stray} alarm={alarm} dropped={dropped}\n").unwrap();
        log!(
            "ADC_REARM i={i} raw={raw} us={retry_us} irq={} stray={} alarm={} dropped={}\n",
            retry.irq,
            retry.stray,
            retry.alarm,
            retry.dropped
        )
        .unwrap();
    }

    let counts = Counts::read();
    adc.try_start(TEMPERATURE)?;
    assert_eq!(adc.try_start(TEMPERATURE), Err(Error::Busy));
    ready();
    for _ in 0..17 {
        // SAFETY: the preceding conversion is complete and the lab owns the ADC.
        unsafe { registers().cs().atomic_set_bits_unchecked(START_ONCE) };
        ready();
    }
    assert!(registers().fcs().read().over().bits());
    assert_eq!(adc.try_sample(), Err(Error::Overrun));
    let fcs = registers().fcs().read();
    assert!(fcs.empty().bits() && !fcs.over().bits());
    let Counts {
        irq,
        stray,
        alarm,
        dropped,
    } = counts.since();
    log!("ADC_OVERRUN injected=1 cleared=1 busy=1 irq={irq} stray={stray} alarm={alarm} dropped={dropped}\n").unwrap();
    Ok(())
}

#[entry(proc0)]
fn main(mut core: Proc0, mut p: rp1::Proc0Resources) -> ! {
    let sys = rp1::platform::init(&mut core, p.platform).unwrap();
    let mut adc = p.adc.init(&sys.clocks).unwrap();
    let wait = &mut p.wait_timer;
    log!("ADC_PARITY_READY\n").unwrap();
    irq_samples(&mut adc, wait).unwrap();
    legacy_samples(&mut adc);
    faults(&mut adc, wait).unwrap();

    let counts = Counts::read();
    let expired = Deadline::at(Timer::new().now());
    assert_eq!(
        adc.blocking(wait).read_until(TEMPERATURE, expired),
        Err(Error::Timeout)
    );
    assert_eq!(
        MASKED.try_with(|_| adc.blocking(wait).read(TEMPERATURE)),
        Some(Err(Error::Masked))
    );
    assert_eq!(
        MASKED.try_with(|_| adc.read_polling(TEMPERATURE)),
        Some(Err(Error::Masked))
    );
    assert!(registers().fcs().read().empty().bits());
    let Counts {
        irq,
        stray,
        alarm,
        dropped,
    } = counts.since();
    log!("ADC_ADMISSION expired=1 masked_irq=1 masked_poll=1 irq={irq} stray={stray} alarm={alarm} dropped={dropped}\n").unwrap();

    adc.try_start(TEMPERATURE).unwrap();
    let mut adc = match adc.try_free() {
        Err(error) if *error.error() == Error::Busy => error.into_driver(),
        _ => panic!("release did not retain the active ADC"),
    };
    ready();
    adc.try_cancel().unwrap();
    assert!(registers().fcs().read().empty().bits());
    assert_eq!(adc.try_sample(), Err(Error::NoConversion));
    let counts = Counts::read();
    let start = now();
    let raw = adc.blocking(wait).read(TEMPERATURE).unwrap();
    let elapsed = now().wrapping_sub(start);
    let Counts {
        irq,
        stray,
        alarm,
        dropped,
    } = counts.since();
    log!("ADC_CANCEL raw={raw} us={elapsed} irq={irq} stray={stray} alarm={alarm} dropped={dropped} retained=1\n").unwrap();

    let parts = adc.try_free().unwrap();
    let mut adc = parts.init(&sys.clocks).unwrap();
    let counts = Counts::read();
    let start = now();
    let raw = adc.blocking(wait).read(TEMPERATURE).unwrap();
    let elapsed = now().wrapping_sub(start);
    let Counts {
        irq,
        stray,
        alarm,
        dropped,
    } = counts.since();
    log!("ADC_REINIT raw={raw} us={elapsed} irq={irq} stray={stray} alarm={alarm} dropped={dropped}\n").unwrap();

    let counts = Counts::read();
    let start = now();
    let raw = adc.read_polling(TEMPERATURE).unwrap();
    let elapsed = now().wrapping_sub(start);
    unsaturated(raw);
    let Counts {
        irq,
        stray,
        alarm,
        dropped,
    } = counts.since();
    log!(
        "ADC_POLL raw={raw} us={elapsed} irq={irq} stray={stray} alarm={alarm} dropped={dropped}\n"
    )
    .unwrap();
    log!("ADC_PARITY_FIRMWARE_END result=PASS\n").unwrap();
    loop {
        rp1_rt::wait_for_event();
    }
}

#[entry(proc1)]
fn idle(_core: Proc1, _p: rp1::Proc1Resources) -> ! {
    loop {
        rp1_rt::wait_for_event();
    }
}
