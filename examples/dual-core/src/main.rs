#![no_std]
#![no_main]
use rp1_hal::{gpio::Doorbell, log};
use rp1_rt::{Proc0, Proc1, entry, isram, shared, sync::Spsc};
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

#[entry(proc0)]
fn main0(mut core: Proc0, p: rp1::Proc0Resources) -> ! {
    // After a loader reset RP1 is at cold defaults: lock PLL_SYS, then select clk_sys = 200 MHz.
    let programmed = unsafe { rp1_hal::cold::clocks() }.unwrap();
    let (_clocks, mut scmi) = p.clock_authority.init(200_000_000).unwrap();
    rp1::log::init(&mut core).unwrap();
    log!("proc0 clocks programmed={}\n", programmed).unwrap();
    // The host is waiting for the PCIe link; bring the endpoint up before anything slow.
    match unsafe { rp1_hal::cold::endpoint_and_syscfg(programmed) } {
        Ok(r) => log!("proc0 pcie {:?}\n", r).unwrap(),
        Err(e) => log!("proc0 pcie failed {:?}\n", e).unwrap(),
    }
    let private = PRIVATE.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    let bss = bss_check(0xb55_0000 | private);
    log!("proc0 ready private={} bss=0x{:x}\n", private, bss).unwrap();
    LOCAL.get(&core).set(1);
    p.proc1.launch(&mut core).unwrap();
    let mut rx = VALUES.consumer(&mut core);
    // Linux resets the endpoint (PERSTn) when it probes PCIe; re-apply the endpoint config in
    // time. This must be polled without sleeping.
    let mut endpoint = unsafe { rp1_hal::cold::EndpointService::new() };
    loop {
        if let Some(value) = rx.pop() {
            log!("proc0 received {}\n", value).unwrap();
        }
        if let Some(r) = endpoint.poll() {
            log!("proc0 pcie perst {:?}\n", r).unwrap();
        }
        // Linux's arm-scmi asks for the RP1 clocks (serial@34000 uses clk_uart via SCMI).
        if let Some(e) = scmi.poll() {
            log!(
                "proc0 scmi p{:x} m{} {:x?} -> {} votes={:x}\n",
                e.protocol,
                e.message,
                e.args,
                e.status,
                e.votes
            )
            .unwrap();
        }
    }
}

#[entry(proc1)]
fn main1(mut core: Proc1, p: rp1::Proc1Resources) -> ! {
    let private = PRIVATE.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    let bss = bss_check(0xb55_1000 | private);
    log!("proc1 ready private={} bss=0x{:x}\n", private, bss).unwrap();
    // ADC and I2C1 are declared on proc1: their drivers take the Clocks that proc0's
    // `clock_authority.init` published (proc0 launches proc1 only after that, so no wait).
    let clocks = rp1_hal::clocks::Clocks::wait_from_proc0(&core, 10_000).unwrap();
    let mut adc = p.ADC.init(&clocks).unwrap();
    log!("proc1 temp_mc={}\n", adc.read_temperature().unwrap()).unwrap();
    let _i2c = p
        .I2C1
        .init(
            p.pins.gpio2,
            p.pins.gpio3,
            &clocks,
            rp1_hal::i2c::Speed::Standard,
        )
        .unwrap();
    let mut tx = VALUES.producer(&mut core);
    tx.push(increment(core::hint::black_box(41))).unwrap();
    let mut doorbell = Doorbell::new(p.pins.gpio27);
    loop {
        if doorbell.pending() {
            doorbell.acknowledge();
            // Request/reply words occupy the first link cache line, outside SCMI/ABI reservations.
            let value = unsafe { (0x2000_c010 as *const u32).read_volatile() };
            unsafe {
                (0x2000_c014 as *mut u32).write_volatile(value);
            }
            // Linux polls the reply word; v0.1 has no RP1 -> Linux interrupt (design §9).
            rp1_proto::barrier();
        }
        core::hint::spin_loop();
    }
}
#[isram]
fn increment(value: u32) -> u32 {
    value.wrapping_add(1)
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo<'_>) -> ! {
    loop {
        rp1_rt::wait_for_event();
    }
}
