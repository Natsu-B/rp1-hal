//! PR-E lab image. Use the realos platform setup, with only lab alarms as user peripherals.
#![no_std]
#![no_main]

mod protocol;

use core::{arch::asm, cell::RefCell};
use protocol::*;
use rp1_hal::{
    log,
    timer::{Alarm, Timer},
};
use rp1_rt::{
    Proc0, Proc1, entry, interrupt,
    interrupt::{Mutex, Priority, free},
};

rp1_rt::bind!();

const PERIOD_US: u32 = 50_000;
static ALARM0: Mutex<RefCell<Option<Alarm<0>>>> = Mutex::new(RefCell::new(None));
static ALARM1: Mutex<RefCell<Option<Alarm<1>>>> = Mutex::new(RefCell::new(None));

fn word(offset: usize) -> *mut u32 {
    (rp1_proto::LINK_START as usize + offset) as *mut u32
}

fn read(offset: usize) -> u32 {
    unsafe { word(offset).read_volatile() }
}

fn write(offset: usize, value: u32) {
    unsafe { word(offset).write_volatile(value) };
}

fn schedule<const N: u8>(alarm: &mut Alarm<N>) {
    alarm.clear_interrupt();
    alarm
        .schedule((Timer::new().now() as u32).wrapping_add(PERIOD_US))
        .unwrap();
}

fn tick<const N: u8>(alarm: &Mutex<RefCell<Option<Alarm<N>>>>, offset: usize) {
    free(|cs| schedule(alarm.borrow_ref_mut(cs).as_mut().unwrap()));
    // Each word has one ISR writer. Shared SRAM uses plain loads and stores.
    write(offset, read(offset).wrapping_add(1));
}

#[interrupt(proc0)]
fn TIMER_0() {
    tick(&ALARM0, TICKS[0]);
    match Kind::from_word(read(ACK)) {
        Some(Kind::P0IsrPanic) => panic!("lab proc0 timer ISR panic"),
        Some(Kind::P0IsrUdf) => udf(),
        _ => {}
    }
}

#[interrupt(proc1)]
fn TIMER_1() {
    tick(&ALARM1, TICKS[1]);
}

fn wait_for_interrupt() {
    unsafe { asm!("wfi", options(nostack, preserves_flags)) };
}

fn udf() -> ! {
    unsafe { asm!("udf #0", options(noreturn, nostack)) }
}

#[repr(align(64))]
struct DiagnosticStack([u8; 64]);

static mut DIAGNOSTIC_STACK: DiagnosticStack = DiagnosticStack([0; 64]);

const MPU_TYPE: *const u32 = 0xe000_ed90 as *const u32;

fn mpu_regions() -> u32 {
    unsafe { (MPU_TYPE.read_volatile() >> 8) & 0xff }
}

/// Cause a stacking access violation on PSP. The handler still has its original MSP.
fn stacking_fault(regions: u32) -> ! {
    const MPU_CTRL: *mut u32 = 0xe000_ed94 as *mut u32;
    const MPU_RNR: *mut u32 = 0xe000_ed98 as *mut u32;
    const MPU_RBAR: *mut u32 = 0xe000_ed9c as *mut u32;
    const MPU_RASR: *mut u32 = 0xe000_eda0 as *mut u32;
    unsafe {
        let base = (&raw mut DIAGNOSTIC_STACK.0).cast::<u8>();
        asm!("cpsid i", options(nostack, preserves_flags));
        MPU_RNR.write_volatile(regions - 1);
        MPU_RBAR.write_volatile(base as u32);
        // XN, AP=0 (no access), 64 bytes, enabled. Other privileged accesses use the default map.
        MPU_RASR.write_volatile((1 << 28) | (5 << 1) | 1);
        MPU_CTRL.write_volatile(0b111); // PRIVDEFENA | HFNMIENA | ENABLE
        asm!(
            "dsb sy",
            "isb sy",
            "msr psp, {end}",
            "mrs r0, control",
            "orr r0, r0, #2",
            "msr control, r0",
            "isb sy",
            // Take the stacking fault deterministically on UDF before an IRQ can use PSP.
            // With PRIMASK set UDF escalates to HardFault. HFNMIENA keeps MPU enforcement
            // during HardFault.
            "udf #0",
            end = in(reg) base.add(64),
            options(noreturn)
        );
    }
}

#[entry(proc0)]
fn main0(mut core: Proc0, p: rp1::Proc0Resources) -> ! {
    // The loader does not clear the scratch words on every boot.
    for offset in [READY, NONCE, REQUEST, ACK, TICKS[0], TICKS[1]] {
        write(offset, 0);
    }
    rp1_proto::barrier();

    let sys = rp1::platform::init(&mut core, p.platform).unwrap();
    log!("PLAT_INIT ok=1 {}\n", sys.report).unwrap();
    let mut alarm = p.TIMER_ALARM0;
    schedule(&mut alarm);
    alarm.enable_interrupt();
    free(|cs| ALARM0.borrow(cs).replace(Some(alarm)));
    let mut irq = p.irq.TIMER_0;
    irq.set_priority(Priority::new(4).unwrap());
    irq.enable();
    p.proc1.launch(&mut core).unwrap();

    // Mix the boot timing samples as in irq-proof; the verifier also checks the Linux boot ID.
    let nonce = [
        sys.report.perstn_wait_us,
        sys.report.link_wait_us,
        Timer::new().now() as u32,
    ]
    .into_iter()
    .fold(0x811c_9dc5u32, |hash, sample| {
        (hash ^ sample).wrapping_mul(0x0100_0193)
    })
    .max(1);
    let regions = mpu_regions();
    log!(
        "FAULT_BOOT nonce={nonce:08x} kinds={ALLOWED} stkerr={} irq57=unavailable\n",
        u8::from(regions != 0)
    )
    .unwrap();
    write(NONCE, nonce);
    rp1_proto::barrier();
    write(READY, MAGIC);

    let kind = loop {
        if let Some(kind) = Kind::from_word(read(REQUEST))
            && (kind != Kind::Stkerr || regions != 0)
        {
            break kind;
        }
        wait_for_interrupt();
    };
    // ACK is published only once and is the release for the ISR/proc1 stimuli.
    write(ACK, kind as u32);
    rp1_proto::barrier();
    match kind {
        Kind::P0ThreadPanic => panic!("lab proc0 thread panic"),
        Kind::P0ThreadUdf => udf(),
        Kind::Stkerr => stacking_fault(regions),
        _ => loop {
            wait_for_interrupt();
        },
    }
}

#[entry(proc1)]
fn main1(_core: Proc1, p: rp1::Proc1Resources) -> ! {
    let mut alarm = p.TIMER_ALARM1;
    schedule(&mut alarm);
    alarm.enable_interrupt();
    free(|cs| ALARM1.borrow(cs).replace(Some(alarm)));
    let mut irq = p.irq.TIMER_1;
    irq.set_priority(Priority::new(4).unwrap());
    irq.enable();
    loop {
        if read(ACK) == Kind::P1Panic as u32 {
            panic!("lab proc1 thread panic");
        }
        wait_for_interrupt();
    }
}
