//! Sweep image for interrupt lines whose number is only an estimate (I2C0/2/3, SPI1/3, ADC,
//! PWM1, and proc0's IO_BANK1/2 lines): which NVIC pending bit does each block's interrupt
//! output raise? Markers go to UART0; lab/verify-irq.py --sweep reads them. See sweep.rs.
//!
//! The sweep runs before the PCIe endpoint is brought up. Until then the host has no link to
//! RP1, so Linux, which the loader starts right after this image, cannot touch a register
//! while a block is being raised. The cost is a later link: the sweep must end well before
//! Linux releases PERSTn (about 2.5 s after this image starts), and takes about 0.7 s.
#![no_std]
#![no_main]
use rp1_hal::log;
use rp1_rt::{Proc0, Proc1, entry};
rp1_rt::bind!();
mod blocks;
mod reg;
mod sweep;
use core::sync::atomic::{AtomicU32, Ordering::Relaxed};

static NONCE: AtomicU32 = AtomicU32::new(0);
/// One marker line. Every line carries the boot's nonce, so that a checker can tell a line of
/// another boot in the same capture.
macro_rules! mark {
    ($fmt:literal $(, $arg:expr)* $(,)?) => {
        $crate::rp1::log::write(core::format_args!(
            concat!($fmt, " nonce={:08x}\n") $(, $arg)*,
            $crate::NONCE.load(core::sync::atomic::Ordering::Relaxed)
        )).unwrap()
    };
}
pub(crate) use mark;

/// FNV-1a over the words.
fn hash(seeds: &[u32]) -> u32 {
    seeds.iter().fold(0x811c_9dc5u32, |hash, seed| {
        (hash ^ seed).wrapping_mul(0x0100_0193)
    })
}

#[entry(proc0)]
fn main0(mut core: Proc0, p: rp1::Proc0Resources) -> ! {
    let programmed = unsafe { rp1_hal::cold::clocks() }.unwrap();
    let (clocks, mut scmi) = p.clock_authority.init(200_000_000).unwrap();
    rp1::log::init(&mut core).unwrap();
    log!(
        "proc0 clocks programmed={} sys_hz={} uart_hz={}\n",
        programmed,
        clocks.sys_hz(),
        clocks.uart_hz()
    )
    .unwrap();

    // The drivers' `init` may wait 10 ms, too long for a sweep step, so both run here. PWM1 is
    // left in reset if the fan pin selects a function: its line is then skipped. The I2C and
    // SPI drivers are never initialised: their `init` would route pins.
    let started = reg::now();
    let pwm = blocks::FAN_PIN
        .allows(blocks::FAN_PIN.funcsel())
        .then(|| p.pwm1.init(&clocks));
    let adc = p.adc.init(&clocks);
    log!(
        "p0 pwm1 init {:?} adc init {}\n",
        pwm.as_ref().map(|r| r.is_ok()),
        adc.is_ok()
    )
    .unwrap();
    // Not guaranteed to differ between boots this early; `IRQ_SWEEP_BOOT` below adds a value
    // that usually does (it includes the PERSTn and link waits), though a 32-bit hash is not
    // guaranteed unique either.
    NONCE.store(hash(&[started, reg::now()]), Relaxed);
    sweep::run();

    let mut waits = [0; 2];
    match unsafe { rp1_hal::cold::endpoint_and_syscfg(programmed) } {
        Ok(r) => {
            waits = [r.perstn_wait_us, r.link_wait_us];
            log!(
                "p0 pcie Ok clocks_programmed={} endpoint_programmed={} perstn_wait_us={} link_wait_us={} monitor2=0x{:x}\n",
                r.clocks_programmed,
                r.endpoint_programmed,
                r.perstn_wait_us,
                r.link_wait_us,
                r.monitor2
            )
        }
        Err(e) => log!("p0 pcie Err({:?})\n", e),
    }
    .unwrap();
    // The waits for the host vary by microseconds from boot to boot. Linux reads the value
    // from the link page (lab/check-sweep.sh), which ties its console to these UART0 lines.
    let boot = hash(&[NONCE.load(Relaxed), waits[0], waits[1], reg::now()]);
    reg::wr(rp1_proto::LINK_START as usize + 0x28, boot);
    rp1_proto::barrier();
    mark!("IRQ_SWEEP_BOOT boot={:08x}", boot);

    // From here on the image only serves Linux: the endpoint across PERSTn, and the clocks.
    let mut endpoint = unsafe { rp1_hal::cold::EndpointService::new() };
    loop {
        if let Some(r) = endpoint.poll() {
            log!("p0 pcie perst {:?}\n", r).unwrap();
        }
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

/// The runtime needs an entry for proc1. It is never launched: the sweep is about proc0's NVIC.
#[entry(proc1)]
fn main1(_core: Proc1, _p: rp1::Proc1Resources) -> ! {
    loop {
        rp1_rt::wait_for_event();
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo<'_>) -> ! {
    let _ = log!("IRQ_SWEEP_PANIC\n");
    loop {
        rp1_rt::wait_for_event();
    }
}
