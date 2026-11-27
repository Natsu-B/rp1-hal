//! Cold-boot fixture for vector and terminal delivery, driven through link SRAM and doorbells.
#![no_std]
#![no_main]

use core::sync::atomic::{AtomicBool, Ordering::Relaxed};
use rp1_hal::platform::{self, PcieState};
use rp1_proto::{
    LINK_START, barrier,
    wake::{LATCHED_WAKE, mailbox},
};
use rp1_rt::{Proc0, Proc1, entry, interrupt, status, wait_for_event};

rp1_rt::bind!();

const COMMAND: [usize; 2] = [0x24, 0x28];
const VECTOR_STATUS: usize = 0x2c;
// Each core scans once on entry even if a command arrived before its IRQ was enabled.
static COMMAND_PENDING: AtomicBool = AtomicBool::new(true);

fn read(offset: usize) -> u32 {
    let value = unsafe { ((LINK_START as usize + offset) as *const u32).read_volatile() };
    barrier();
    value
}

fn write(offset: usize, value: u32) {
    unsafe { ((LINK_START as usize + offset) as *mut u32).write_volatile(value) };
    barrier();
}

#[interrupt(proc0)]
fn IO_BANK0() {
    COMMAND_PENDING.store(true, Relaxed);
    unsafe { rp1::Doorbell0::new() }.__ack_and_signal_wake();
}

#[interrupt(proc1)]
fn IO_BANK0() {
    COMMAND_PENDING.store(true, Relaxed);
    unsafe { rp1::Doorbell1::new() }.__ack_and_signal_wake();
}

fn command(core: usize) -> u32 {
    if !COMMAND_PENDING.swap(false, Relaxed) {
        return 0;
    }
    match read(COMMAND[core]) {
        2 => panic!("link-wake fixture panic"),
        3 => {
            // Configurable UsageFault, taken in the commanded core's thread mode.
            unsafe { core::arch::asm!("udf #0", options(nomem, nostack, noreturn)) }
        }
        value => value,
    }
}

#[entry(proc0)]
fn main0(mut core: Proc0, mut p: rp1::Proc0Resources) -> ! {
    for offset in [COMMAND[0], COMMAND[1], VECTOR_STATUS] {
        write(offset, 0);
    }
    // This fixture starts cold with exclusive ownership of the mailbox.
    for offset in (mailbox::START..mailbox::END).step_by(4) {
        write(offset, 0);
    }
    write(mailbox::OFFERED_CAPS, LATCHED_WAKE);
    write(mailbox::BOOT_EPOCH, read(rp1_proto::status::EPOCH));

    let _platform = rp1::platform::init(&mut core, p.platform).unwrap();
    p.irq.IO_BANK0.enable();
    p.proc1.launch(&mut core).unwrap();
    loop {
        if command(0) == 1 {
            status::stop(1);
        }
        wait_for_event();
    }
}

#[entry(proc1)]
fn main1(_core: Proc1, mut p: rp1::Proc1Resources) -> ! {
    p.irq.IO_BANK0.enable();
    write(
        VECTOR_STATUS,
        if p.host_irq.ready().is_ok() { 1 } else { 2 },
    );
    loop {
        let command = command(1);
        let stop = command == 1;
        if stop {
            status::__record_stop_before_park(1);
        }
        if platform::pcie_state() == PcieState::Up {
            let delivery = p.host_irq.pulse_terminals();
            if delivery.is_err()
                || (delivery == Ok(false) && command == 4 && p.host_irq.pulse().is_err())
            {
                write(VECTOR_STATUS, 2);
            }
        }
        if stop {
            status::stop(1);
        }
        wait_for_event();
    }
}
