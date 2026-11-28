//! Same-boot SPI flush timing with an independent wire-idle observer.
#![no_std]
#![no_main]

use core::{
    mem::offset_of,
    sync::atomic::{AtomicU32, Ordering::Relaxed},
};
use embedded_hal::spi::MODE_0;
use rp1_hal::{
    fugit::{ExtU32, RateExtU32},
    log, pac,
    spi::{CompletePins, Config, Controller, Error, Irq, Spi, Try},
    timer::{Deadline, Timer, WaitTimer},
};
use rp1_rt::{Proc0, Proc1, entry};

rp1_rt::bind!();

#[path = "../../spi-parity/src/idle_probe.rs"]
mod idle_probe;
#[path = "../../spi-parity/src/platform_trace.rs"]
mod platform_trace;

const TRIALS: usize = platform_trace::TRIALS;
const REPEAT: u32 = u32::from_le_bytes(*b"SPIR");

static TX: AtomicU32 = AtomicU32::new(0);
static STRAY: AtomicU32 = AtomicU32::new(0);
static ALARMS: AtomicU32 = AtomicU32::new(0);

#[inline(always)]
fn now() -> u32 {
    Timer::new().now_32().ticks()
}

#[inline(always)]
fn device(dispatch: impl FnOnce() -> u32) {
    let status = dispatch();
    if status & pac::dw_ssi::int::TXEI != 0 {
        TX.store(TX.load(Relaxed) + 1, Relaxed);
    }
    if status == 0 {
        STRAY.store(STRAY.load(Relaxed) + 1, Relaxed);
    }
}

#[inline(always)]
fn alarm() {
    // SAFETY: this wraps the generated ISR for this build's ALARM1.
    unsafe { WaitTimer::__on_interrupt::<1>() };
    ALARMS.store(ALARMS.load(Relaxed) + 1, Relaxed);
}

type Driver<const N: u8, P> = Spi<N, P, Controller, Irq<N, 0>>;

struct Case<const N: u8, P: CompletePins<N>> {
    spi: Driver<N, P>,
    regs: *const u8,
    stats: platform_trace::Summary,
    tails: usize,
    over_16: usize,
}

impl<const N: u8, P: CompletePins<N>> Case<N, P> {
    fn new(spi: Driver<N, P>, regs: *const u8) -> Self {
        Self {
            spi,
            regs,
            stats: Default::default(),
            tails: 0,
            over_16: 0,
        }
    }

    fn reg(&self, offset: usize) -> *mut u32 {
        self.regs.wrapping_add(offset).cast_mut().cast()
    }

    fn rate(&mut self, timer: &mut WaitTimer, hz: u32, sys: u32) -> Result<(), Error> {
        assert_eq!(self.spi.blocking(timer).set_frequency(hz.Hz())?.raw(), hz);
        // SAFETY: read the declared block and the platform clock registers.
        let (baud, control, threshold, id) = unsafe {
            (
                self.reg(offset_of!(pac::dw_ssi::RegisterBlock, baudr))
                    .read_volatile(),
                self.reg(offset_of!(pac::dw_ssi::RegisterBlock, ctrlr0))
                    .read_volatile(),
                self.reg(offset_of!(pac::dw_ssi::RegisterBlock, txftlr))
                    .read_volatile(),
                self.reg(offset_of!(pac::dw_ssi::RegisterBlock, idr))
                    .read_volatile(),
            )
        };
        let clock = |addr: usize| unsafe { (addr as *const u32).read_volatile() };
        assert_eq!(sys / baud, hz);
        log!(
            "TAIL_REG instance={N} hz={hz} sys={sys} ctrl={:08x} div={} sel={:08x} baud={baud} ctrlr0={control:08x} txftlr={threshold:08x} id={id:08x} depth={}\n",
            clock(0x4001_8014), clock(0x4001_8018), clock(0x4001_8020), self.spi.fifo_depth()
        ).unwrap();
        Ok(())
    }

    fn flush(&mut self, timer: &mut WaitTimer, hz: u32, trial: usize) -> Result<(), Error> {
        use pac::dw_ssi::sr;
        let depth = self.spi.fifo_depth();
        let burst_us = depth as u32 * (8_000_000 / hz);
        // SAFETY: this case owns the controller. SER zero primes the FIFO with
        // the shifter held, and SR remains valid for the proc1 observer.
        let select = self.reg(offset_of!(pac::dw_ssi::RegisterBlock, ser));
        let status = self.reg(offset_of!(pac::dw_ssi::RegisterBlock, sr));
        unsafe { select.write_volatile(0) };
        for word in 0..depth as u8 {
            assert_eq!(self.spi.try_write_word(word)?, Try::Ready(()));
        }
        assert_eq!(unsafe { status.read_volatile() } & sr::TFE, 0);

        let tx = TX.load(Relaxed);
        let stray = STRAY.load(Relaxed);
        let alarms = ALARMS.load(Relaxed);
        let end = Deadline::after(100.millis())?;
        let window = idle_probe::arm(status, burst_us);
        let launch = now();
        unsafe { select.write_volatile(1) };
        let start = platform_trace::begin();
        self.spi.blocking(timer).flush_until(end)?;
        let returned = platform_trace::end();
        let (last_busy, first_idle) = idle_probe::bracket(window, returned);
        let tail_alarms = ALARMS.load(Relaxed) - alarms;
        assert_eq!(
            unsafe { status.read_volatile() } & (sr::BUSY | sr::TFE),
            sr::TFE
        );
        assert_eq!(TX.load(Relaxed) - tx, 1);
        assert_eq!(STRAY.load(Relaxed) - stray, 0);
        let earliest = burst_us - 1_000_000 / hz - 1;
        let elapsed = returned.wrapping_sub(launch);
        assert!((earliest..earliest + 100).contains(&elapsed));
        assert!(hz != 100_000 || tail_alarms > 0);
        self.tails += usize::from(tail_alarms != 0);
        let flush = platform_trace::Flush {
            start,
            returned,
            last_busy,
            first_idle,
            tail_alarms,
        };
        // Interleaved records identify the controller in trial % 4.
        let key = trial * 4 + usize::from(N);
        let upper = platform_trace::assess(&mut self.stats, hz, key, flush);
        self.over_16 += usize::from(upper >= 17);
        for word in 0..depth as u8 {
            assert_eq!(self.spi.try_read_word::<u8>()?, Try::Ready(word));
        }
        Ok(())
    }

    fn report(&mut self, hz: u32, trials: usize) {
        let stats = core::mem::take(&mut self.stats);
        log!(
            "TAIL_RATE instance={N} hz={hz} trials={trials} tails={} preempted={} max_plain={} max_preempted={} failed={} ge17={}\n",
            self.tails, stats.preempted, stats.max_plain, stats.max_preempted, stats.failed, self.over_16,
        ).unwrap();
        self.tails = 0;
        self.over_16 = 0;
    }
}

#[entry(proc0)]
fn main(mut core: Proc0, mut p: rp1::Proc0Resources) -> ! {
    let sys = rp1::platform::init(&mut core, p.platform).unwrap();
    platform_trace::install();
    p.proc1.launch(&mut core).unwrap();
    let config = Config::new(1.MHz(), MODE_0);
    let mut spi0 = Case::new(
        p.spi0
            .init(&sys.clocks, config)
            .map_err(|error| *error.error())
            .unwrap(),
        pac::SPI0::PTR,
    );
    let mut spi1 = Case::new(
        p.spi1
            .init(&sys.clocks, config)
            .map_err(|error| *error.error())
            .unwrap(),
        pac::SPI1::PTR,
    );
    let mut spi3 = Case::new(
        p.spi3
            .init(&sys.clocks, config)
            .map_err(|error| *error.error())
            .unwrap(),
        pac::SPI3::PTR,
    );
    let timer = &mut p.wait_timer;
    spi0.spi.blocking(timer).set_loopback(true).unwrap();
    spi1.spi.blocking(timer).set_loopback(true).unwrap();
    spi3.spi.blocking(timer).set_loopback(true).unwrap();
    let sys = sys.clocks.sys().raw();
    // Match the preceding fixture's warm-up and put the early trials in Linux boot.
    for hz in [1_000_000, 10_000_000] {
        let mut bus = spi0.spi.blocking(timer);
        bus.set_frequency(hz.Hz()).unwrap();
        let tx = [0x5au8; 1_000];
        let mut rx = [0; 1_000];
        for _ in 0..100 {
            let end = Deadline::after(100.millis()).unwrap();
            bus.transfer_until(&mut rx, &tx, end).unwrap();
            bus.flush_until(end).unwrap();
            assert_eq!(rx, tx);
        }
    }

    for phase in ["early", "linux"] {
        if phase == "linux" {
            platform_trace::wait_for_linux();
        }
        log!("TAIL_BEGIN phase={phase}\n").unwrap();
        platform_trace::inventory(&[19, 20, 22], &[27]);
        for hz in [1_000_000, 100_000] {
            spi0.rate(timer, hz, sys).unwrap();
            spi1.rate(timer, hz, sys).unwrap();
            spi3.rate(timer, hz, sys).unwrap();
            for i in 0..TRIALS {
                spi0.flush(timer, hz, i).unwrap();
                spi1.flush(timer, hz, i).unwrap();
                spi3.flush(timer, hz, i).unwrap();
            }
            spi0.report(hz, TRIALS);
            spi1.report(hz, TRIALS);
            spi3.report(hz, TRIALS);
        }
        platform_trace::inventory(&[19, 20, 22], &[27]);
        platform_trace::platform_totals();
        log!("TAIL_END phase={phase}\n").unwrap();
    }
    platform_trace::wait_for_host("TAIL_AWAIT_REPEAT", REPEAT);
    const HZ: u32 = 1_000_000;
    log!("TAIL_BEGIN phase=repeat\n").unwrap();
    spi3.rate(timer, HZ, sys).unwrap();
    for i in 0..5 * TRIALS {
        spi3.flush(timer, HZ, i).unwrap();
    }
    spi3.report(HZ, 5 * TRIALS);
    platform_trace::platform_totals();
    log!("TAIL_END phase=repeat\n").unwrap();
    log!("TAIL_FINISHED\n").unwrap();
    loop {
        rp1_rt::wait_for_event();
    }
}

#[entry(proc1)]
fn observe(_core: Proc1, _p: rp1::Proc1Resources) -> ! {
    idle_probe::observe()
}
