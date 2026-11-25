#![no_std]
#![no_main]
use core::cell::RefCell;
use core::sync::atomic::{AtomicU32, Ordering::Relaxed};
use rp1_hal::{
    gpio::PinIrq,
    log,
    timer::{Alarm, Timer},
};
use rp1_proto::link;
use rp1_rt::{
    Proc0, Proc1, entry, interrupt,
    interrupt::{Mutex, free},
    isram, shared,
    sync::Spsc,
};
rp1_rt::bind!();

#[shared]
static VALUES: Spsc<u32, 8> = Spsc::new();
#[rp1_rt::proc0]
static LOCAL: core::cell::Cell<u32> = core::cell::Cell::new(0);
static PRIVATE: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(7);
/// Zero-initialised, so it lands in .bss. Each core zeroes its own DSRAM copy at startup;
/// reading it before any write checks that (review #10), and the store keeps it non-constant.
static BSS: [core::sync::atomic::AtomicU32; 4] =
    [const { core::sync::atomic::AtomicU32::new(0) }; 4];
fn bss_check(marker: u32) -> u32 {
    let mut seen = 0;
    for word in &BSS {
        seen |= word.load(core::sync::atomic::Ordering::Relaxed);
        word.store(marker, core::sync::atomic::Ordering::Relaxed);
    }
    seen
}

// Ordinary statics have one DSRAM copy per core: each is shared only between one core's main
// and that core's handlers, so a core-local `Mutex` or a plain atomic is enough.
static ALARM0: Mutex<RefCell<Option<Alarm<0>>>> = Mutex::new(RefCell::new(None));
static TICKS: AtomicU32 = AtomicU32::new(0);
static DOORBELL: Mutex<RefCell<Option<PinIrq<27, 1>>>> = Mutex::new(RefCell::new(None));
const TICK_US: u32 = 1_000_000;
#[cfg(feature = "panic-request")]
const PANIC_PROC0: u32 = 0x0dea_d000;
#[cfg(feature = "panic-request")]
const PANIC_PROC1: u32 = 0x0dea_d001;

/// proc0's heartbeat: count the tick, tell Linux (the declared `heartbeat`), arm the next one.
#[interrupt(proc0)]
fn TIMER_0() {
    free(|cs| {
        if let Some(alarm) = ALARM0.borrow_ref_mut(cs).as_mut() {
            alarm.clear_interrupt();
            alarm.schedule(next_tick()).unwrap();
        }
    });
    // The handler is the only writer: a load and a store, no read-modify-write.
    TICKS.store(TICKS.load(Relaxed) + 1, Relaxed);
    rp1_rt::status::beat();
}
/// Linux's doorbell (examples/linux-doorbell): it wrote a request word and forced GPIO27 in
/// PROC1_INTF. Drop the force, then echo the request as the reply, which Linux polls for.
#[interrupt(proc1)]
fn IO_BANK0() {
    free(|cs| {
        if let Some(pin) = DOORBELL.borrow_ref_mut(cs).as_mut() {
            pin.unforce();
        }
    });
    let page = rp1_proto::LINK_START as usize;
    unsafe {
        let request = ((page + link::DOORBELL_REQUEST) as *const u32).read_volatile();
        #[cfg(feature = "panic-request")]
        if request == PANIC_PROC1 {
            panic!("proc1 panic requested by Linux");
        }
        ((page + link::DOORBELL_REPLY) as *mut u32).write_volatile(request);
    }
    rp1_proto::barrier();
}
fn next_tick() -> u32 {
    (Timer::new().now() as u32).wrapping_add(TICK_US)
}

#[entry(proc0)]
fn main0(mut core: Proc0, p: rp1::Proc0Resources) -> ! {
    // Clocks (200 MHz clk_sys), Linux's blocks, the log, the PCIe endpoint (waits for Linux's
    // probe, about 3 s after RP1 starts) and the SCMI clock server.
    let sys = match rp1::platform::init(&mut core, p.platform) {
        Ok(sys) => sys,
        Err(e) => {
            let _ = log!("PLAT_INIT ok=0 {:?}\n", e);
            panic!("platform init failed");
        }
    };
    log!("PLAT_INIT ok=1 {}\n", sys.report).unwrap();
    let private = PRIVATE.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    let bss = bss_check(0xb55_0000 | private);
    log!("proc0 ready private={} bss=0x{:x}\n", private, bss).unwrap();
    LOCAL.get(&core).set(1);
    // SPI0 on proc0 with its own Clocks: SCLK/MOSI/MISO = GPIO11/10/9, GPIO8 as chip select.
    let bus = p
        .spi0
        .init(
            &sys.clocks,
            rp1_hal::spi::Config::new(10_000_000, embedded_hal::spi::MODE_0),
        )
        .unwrap();
    let mut device = rp1_hal::spi::Device::new(bus, p.pins.gpio8);
    let mut id = [0u8; 2];
    {
        use embedded_hal::spi::{Operation, SpiDevice};
        // A typical register read: command byte out, two bytes back, one CS assertion.
        device
            .transaction(&mut [Operation::Write(&[0x9f]), Operation::Read(&mut id)])
            .unwrap();
    }
    log!(
        "proc0 spi0 {} Hz id={:x?}\n",
        device.bus_mut().frequency_hz(),
        id
    )
    .unwrap();
    let mut alarm = p.TIMER_ALARM0;
    alarm.clear_interrupt();
    alarm.enable_interrupt();
    alarm.schedule(next_tick()).unwrap();
    free(|cs| ALARM0.borrow(cs).replace(Some(alarm)));
    let mut irq = p.irq;
    irq.TIMER_0.enable();
    p.proc1.launch(&mut core).unwrap();
    let mut rx = VALUES.consumer(&mut core);
    let mut logged = 0;
    loop {
        #[cfg(feature = "panic-request")]
        {
            let page = rp1_proto::LINK_START as usize;
            let request =
                unsafe { ((page + link::DOORBELL_REQUEST) as *const u32).read_volatile() };
            if request == PANIC_PROC0 {
                panic!("proc0 panic requested by Linux");
            }
        }
        let ticks = TICKS.load(Relaxed);
        if ticks != logged {
            logged = ticks;
            log!("proc0 tick {}\n", ticks).unwrap();
        }
        if let Some(value) = rx.pop() {
            log!("proc0 received {}\n", value).unwrap();
        }
    }
}

#[entry(proc1)]
fn main1(mut core: Proc1, p: rp1::Proc1Resources) -> ! {
    let private = PRIVATE.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    let bss = bss_check(0xb55_1000 | private);
    log!("proc1 ready private={} bss=0x{:x}\n", private, bss).unwrap();
    // ADC and I2C1 are declared on proc1: their drivers take the Clocks that proc0's
    // platform init published (proc0 launches proc1 only after that, so no wait).
    let clocks = rp1_hal::clocks::Clocks::wait_from_proc0(&core, 10_000).unwrap();
    let mut adc = p.adc.init(&clocks).unwrap();
    log!("proc1 temp_mc={}\n", adc.read_temperature().unwrap()).unwrap();
    let _i2c = p.i2c1.init(&clocks, rp1_hal::i2c::Speed::Standard).unwrap();
    let mut tx = VALUES.producer(&mut core);
    tx.push(increment(core::hint::black_box(41))).unwrap();
    let mut irq = p.irq;
    free(|cs| DOORBELL.borrow(cs).replace(Some(irq.gpio27)));
    irq.IO_BANK0.enable();
    log!("proc1 doorbell armed\n").unwrap();
    // Everything else happens in IO_BANK0.
    loop {
        rp1_rt::wait_for_event();
    }
}
#[isram]
fn increment(value: u32) -> u32 {
    value.wrapping_add(1)
}
