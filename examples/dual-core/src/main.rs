#![no_std]
#![no_main]
use core::sync::atomic::{AtomicU32, Ordering::Relaxed};
use rp1_hal::{
    fugit::{ExtU32, RateExtU32, TimerInstantU32},
    log,
    timer::{Alarm, Timer},
};
#[cfg(not(feature = "link-wake"))]
use rp1_link::Progress;
use rp1_rt::{Proc0, Proc1, entry, interrupt, interrupt::Slot, isram, shared, sync::Spsc};
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

// Ordinary statics have one DSRAM copy per core, shared only by that core's main and
// handlers, so a core-local Slot or a plain atomic is enough.
// Proc0 main installs the alarm before enabling the heartbeat IRQ.
static ALARM0: Slot<Option<Alarm<0>>, 0> = Slot::empty();
static TICKS: AtomicU32 = AtomicU32::new(0);
type Ping = u32;
type Pong = u32;
type Nonce = u32;

#[cfg(feature = "ping-test")]
#[path = "../../../lab/ping/firmware.rs"]
mod ping_test;

/// proc0's heartbeat: count the tick, tell Linux (the declared `heartbeat`), arm the next one.
#[interrupt(proc0)]
fn TIMER_0() {
    ALARM0.with(|alarm| {
        alarm.clear_interrupt();
        alarm.try_arm(next_tick()).unwrap();
    });
    // The handler is the only writer: a load and a store, no read-modify-write.
    TICKS.store(TICKS.load(Relaxed) + 1, Relaxed);
    rp1_rt::status::beat();
    #[cfg(not(any(feature = "ping-poll", feature = "link-wake")))]
    // Older Linux clients do not ring after freeing channel reply slots.
    rp1_rt::send_event();
}

fn next_tick() -> TimerInstantU32<1_000_000> {
    Timer::new().now_32() + 1.secs()
}

#[entry(proc0)]
fn main0(mut core: Proc0, mut p: rp1::Proc0Resources) -> ! {
    // Clocks (200 MHz clk_sys), Linux's blocks, the log, the PCIe endpoint (waits for Linux's
    // probe, about 3 s after RP1 starts) and the SCMI clock server.
    let mut sys = match rp1::platform::init(&mut core, p.platform) {
        Ok(sys) => sys,
        Err(_) => {
            let _ = log!("PLAT_INIT ok=0\n");
            panic!("platform init failed");
        }
    };
    #[cfg(feature = "link-wake")]
    let startup_end = rp1_hal::timer::Deadline::after(10.secs()).unwrap();
    log!("PLAT_INIT ok=1 {}\n", sys.report).unwrap();
    let private = PRIVATE.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    let bss = bss_check(0xb55_0000 | private);
    log!("proc0 ready private={} bss=0x{:x}\n", private, bss).unwrap();
    LOCAL.get(&core).set(1);
    // v3 polls only this startup ID probe (one 100 ms deadline) to fit the ISRAM reply loop.
    let mut bus = p
        .spi0
        .init(
            &sys.clocks,
            rp1_hal::spi::Config::new(10.MHz(), embedded_hal::spi::MODE_0),
        )
        .unwrap();
    let frequency = bus.frequency().raw();
    #[cfg(not(feature = "link-wake"))]
    let spi = bus.blocking(&mut p.wait_timer);
    #[cfg(feature = "link-wake")]
    let spi = bus.polling();
    let mut device = rp1_hal::spi::Device::new(spi, p.pins.gpio8);
    let mut id = [0u8; 2];
    use embedded_hal::spi::{Operation, SpiDevice};
    device
        .transaction(&mut [Operation::Write(&[0x9f]), Operation::Read(&mut id)])
        .unwrap();
    log!("proc0 spi0 {frequency} Hz id={id:x?}\n").unwrap();
    #[cfg(not(feature = "link-wake"))]
    start_heartbeat(p.TIMER_ALARM0, &mut p.irq.TIMER_0);
    p.proc1.launch(&mut core).unwrap();
    #[cfg(feature = "link-wake")]
    {
        ping_test::enable_startup(&mut sys.pcie, p.irq.IO_BANK0, p.wait_timer, startup_end);
        start_heartbeat(p.TIMER_ALARM0, &mut p.irq.TIMER_0);
    }
    #[cfg(not(feature = "link-wake"))]
    rp1::link::enable(&mut sys.pcie, &mut p.TIMER_ALARM1, &mut p.irq.TIMER_1)
        .expect("SHM enable failed");

    let mut rx = VALUES.consumer(&mut core);
    let mut logged = 0;
    #[cfg(feature = "ping-poll")]
    let mut channel = Some(p.link.open().expect("channel open failed").channel);
    loop {
        #[cfg(feature = "link-wake")]
        ping_test::check_proc0_panic();
        #[cfg(feature = "ping-poll")]
        if let Some(endpoint) = channel.as_mut()
            && let Err(error) = endpoint.respond(1, |nonce| nonce)
        {
            let _ = log!("channel stopped: {error}\n");
            channel = None;
        }
        let ticks = TICKS.load(Relaxed);
        if ticks != logged {
            logged = ticks;
            log!("proc0 tick {}\n", ticks).unwrap();
        }
        if let Some(value) = rx.pop() {
            log!("proc0 received {}\n", value).unwrap();
        }
        #[cfg(not(feature = "ping-poll"))]
        rp1_rt::wait_for_event();
    }
}

fn start_heartbeat(
    mut alarm: Alarm<0>,
    irq: &mut interrupt::Binding<interrupt::AlarmIrq<0>, 26, 0>,
) {
    alarm.enable_interrupt();
    alarm.try_arm(next_tick()).unwrap();
    ALARM0.put(alarm).ok().unwrap();
    irq.enable();
}

#[entry(proc1)]
fn main1(mut core: Proc1, p: rp1::Proc1Resources) -> ! {
    #[cfg(feature = "link-wake")]
    let startup_end = rp1_hal::timer::Deadline::after(10.secs()).unwrap();
    let private = PRIVATE.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    let bss = bss_check(0xb55_1000 | private);
    log!("proc1 ready private={} bss=0x{:x}\n", private, bss).unwrap();
    // ADC and I2C1 are declared on proc1: their drivers take the Clocks that proc0's
    // platform init published (proc0 launches proc1 only after that, so no wait).
    let clocks = rp1_hal::clocks::Clocks::wait_from_proc0(&core, 10.millis()).unwrap();
    let mut adc = p.adc.init(&clocks).unwrap();
    // Proc0 owns the three firmware alarms, so proc1 opts into ADC polling.
    log!(
        "proc1 temp_mc={}\n",
        adc.read_temperature_polling().unwrap()
    )
    .unwrap();
    let _i2c = p.i2c1.init(&clocks, rp1_hal::i2c::Speed::Standard).unwrap();
    let mut tx = VALUES.producer(&mut core);
    tx.push(increment(core::hint::black_box(41))).unwrap();
    #[cfg(not(any(feature = "ping-poll", feature = "link-wake")))]
    let link = {
        let mut irq = p.irq.IO_BANK0;
        p.link.open(&mut irq).expect("link open failed")
    };
    #[cfg(feature = "ping-poll")]
    let link = p.link.open().expect("link open failed");
    #[cfg(feature = "link-wake")]
    ping_test::open_and_run(
        p.link,
        p.irq.IO_BANK0,
        p.wait_timer,
        startup_end,
        p.host_irq,
    );
    #[cfg(not(feature = "link-wake"))]
    run_link(link)
}

#[cfg(not(feature = "link-wake"))]
fn run_link(link: rp1::link::Proc1Link) -> ! {
    let mut ping = link.ping;
    #[cfg(not(feature = "ping-poll"))]
    let mut channel = Some(link.channel);
    #[cfg(feature = "ping-test")]
    let mut probe = ping_test::Probe::new();
    loop {
        let progress = ping
            .serve(8, |value| {
                #[cfg(feature = "ping-test")]
                let value = probe.reply(value);
                value
            })
            .expect("ping serve failed");
        #[cfg(not(feature = "ping-poll"))]
        {
            if let Some(endpoint) = channel.as_mut() {
                match endpoint.respond(8, |nonce| nonce) {
                    Ok(Progress::More) => continue,
                    Ok(Progress::Idle) => {}
                    Err(error) => {
                        let _ = log!("channel stopped: {error}\n");
                        channel = None;
                    }
                }
            }
            // Both endpoints share the doorbell. Check both before going to sleep.
            if progress == Progress::Idle {
                rp1_rt::wait_for_event();
            }
        }
        #[cfg(feature = "ping-poll")]
        let _ = progress;
    }
}

#[isram]
fn increment(value: u32) -> u32 {
    value.wrapping_add(1)
}
