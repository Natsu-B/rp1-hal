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
static ZERO: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);

#[entry(proc0)]
fn main0(mut core: Proc0, p: rp1::Proc0Resources) -> ! {
    // After a loader reset RP1 is at cold defaults: program clocks, then adopt them.
    let programmed = unsafe { rp1_hal::cold::clocks() }.unwrap();
    let (_clocks, mut scmi) = p.clock_authority.adopt(200_000_000).unwrap();
    rp1::log::init(&mut core).unwrap();
    log!("proc0 clocks programmed={}\n", programmed).unwrap();
    // The host is waiting for the PCIe link; bring the endpoint up before anything slow.
    match unsafe { rp1_hal::cold::endpoint_and_syscfg(programmed) } {
        Ok(r) => log!("proc0 pcie {:?}\n", r).unwrap(),
        Err(e) => log!("proc0 pcie failed {:?}\n", e).unwrap(),
    }
    let private = PRIVATE.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    ZERO.store(private, core::sync::atomic::Ordering::Relaxed);
    log!("proc0 ready private={}\n", private).unwrap();
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
    log!("proc1 ready private={}\n", private).unwrap();
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
