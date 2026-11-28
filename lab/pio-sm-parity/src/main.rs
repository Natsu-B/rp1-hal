//! Gate B with PIO edge samples tied to the independent 1 MHz timer.
#![no_std]
#![no_main]

macro_rules! log {
    ($($arg:tt)*) => { crate::write_log(format_args!($($arg)*)) };
}

mod fifo;
#[path = "../../shm-firmware/src/stack_watermark.rs"]
mod stack_watermark;
mod wave;

use rp1_hal::{
    clocks::Clocks,
    gpio::{PadConfig, Pull},
    pio::{PIO, PIOExt, PioPin},
    timer::Timer,
};
use rp1_rt::{Proc0, Proc1, entry, shared, sync::Spsc};

rp1_rt::bind!();
type PingWord = u32;
type LabResult<T> = Result<T, &'static str>;

const LIMIT_US: u32 = 30_000_000;
const DOWN: PadConfig = PadConfig::INPUT_DEFAULT.with_pull(Pull::Down);
const OUTPUT: PadConfig = PadConfig::OUTPUT_DEFAULT.with_pull(Pull::None);

#[shared]
static COMPLETED: Spsc<u32, 2> = Spsc::new();

#[entry(proc0)]
fn main0(mut core: Proc0, mut p: rp1::Proc0Resources) -> ! {
    let watermark = unsafe { stack_watermark::paint_core::<0>() };
    let mut sys = rp1::platform::init(&mut core, p.platform).unwrap();
    log!("PIO_B_BOOT {}\n", sys.report).unwrap();
    p.proc1.launch(&mut core).unwrap();
    rp1::link::enable(&mut sys.pcie, &mut p.TIMER_ALARM1, &mut p.irq.TIMER_1).unwrap();
    let mut completed = COMPLETED.consumer(&mut core);
    loop {
        if let Some(run) = completed.pop() {
            log!(
                "PIO_B_STACK run={run} core=0 free={}\n",
                watermark.free_bytes()
            )
            .unwrap();
        }
        rp1_rt::wait_for_event();
    }
}

#[entry(proc1)]
fn main1(mut core: Proc1, p: rp1::Proc1Resources) -> ! {
    let watermark = unsafe { stack_watermark::paint() };
    let clocks =
        Clocks::wait_from_proc0(&core, rp1_hal::fugit::MicrosDurationU32::micros(10_000)).unwrap();
    let timer = Timer::new();
    let (pio, sms) = p.pio.split_wait(&clocks).unwrap();
    let mut w = wave::Bench {
        sm0: Some(sms.sm0),
        sm1: Some(sms.sm1),
        sm2: Some(sms.sm2),
        sm3: Some(sms.sm3),
        gpio22: Some(input_only(p.pins.gpio22)),
        gpio23: Some(input_only(p.pins.gpio23)),
        gpio12: Some(input_only(p.pins.gpio12)),
        gpio13: Some(input_only(p.pins.gpio13)),
    };
    let mut done = COMPLETED.producer(&mut core);
    let mut ping = p.link.open().unwrap().ping;
    let mut run = 0u32;
    log!(
        "PIO_B_READY core=1 clk_hz={} safety={:#x}\n",
        clocks.sys().raw(),
        pad_state()
    )
    .unwrap();
    loop {
        let mut trigger = false;
        let _ = ping
            .serve(1, |value| {
                trigger = true;
                value
            })
            .unwrap();
        if trigger {
            run += 1;
            let start = now(&timer);
            log!("PIO_B_BEGIN run={run} start={start}\n").unwrap();
            let result = check(&pio, &mut w, &timer);
            let end = now(&timer);
            log!("PIO_B_END run={run} result={result:?} start={start} end={end} safety={:#x} dma={:#x} stack_free={}\n",
                pad_state(), dma_enables(), watermark.free_bytes()).unwrap();
            done.push(run).unwrap();
            rp1_rt::send_event();
        }
    }
}

fn check(pio: &PIO, bench: &mut wave::Bench, timer: &Timer) -> LabResult<()> {
    if pad_state() != 0xf || dma_enables() != 0 {
        return Err("unsafe entry");
    }
    fifo::lifecycle(pio, &mut bench.sm0, timer)?;
    fifo::lifecycle(pio, &mut bench.sm1, timer)?;
    fifo::lifecycle(pio, &mut bench.sm2, timer)?;
    fifo::lifecycle(pio, &mut bench.sm3, timer)?;
    bench.check(pio, timer)?;
    if pad_state() != 0xf || dma_enables() != 0 {
        return Err("unsafe exit");
    }
    Ok(())
}

fn input_only<const N: u8>(pin: PioPin<N>) -> PioPin<N> {
    let ctrl = 0x400d_0004 + usize::from(N) * 8;
    let pad = 0x400f_0004 + usize::from(N) * 4;
    // This declaration owns N. Set OD and IE before switching its mux.
    unsafe {
        (pad as *mut u32).write_volatile(0xc6);
        (0x400e_3004 as *mut u32).write_volatile(1 << N);
        (ctrl as *mut u32).write_volatile((2 << 14) | 5);
    }
    pin
}

fn pad_state() -> u8 {
    let mut result = 0;
    for (bit, n) in [12, 13, 22, 23].into_iter().enumerate() {
        let ctrl = reg(0x400d_0004 + n * 8);
        let pad = reg(0x400f_0004 + n * 4);
        result |= u8::from(pad & 0xc0 == 0xc0 && ctrl >> 14 & 3 == 2) << bit;
    }
    result
}

fn dma_enables() -> u8 {
    let mut result = 0;
    for n in 0..4 {
        result |= ((reg(0xf000_00e4 + 0x20 * n) >> 31) as u8) << n;
        result |= ((reg(0xf000_00e8 + 0x20 * n) >> 31) as u8) << (4 + n);
    }
    result
}

fn reg(address: usize) -> u32 {
    unsafe { (address as *const u32).read_volatile() }
}

fn now(timer: &Timer) -> u32 {
    timer.now_32().ticks()
}

// Gate B emits through the platform-initialized UART0. Proc0 writes before
// launching proc1 and after it hands the UART back with COMPLETED.
fn write_log(args: core::fmt::Arguments<'_>) -> core::fmt::Result {
    struct Sink;
    impl core::fmt::Write for Sink {
        fn write_str(&mut self, text: &str) -> core::fmt::Result {
            let timer = Timer::new();
            for byte in text.bytes() {
                let start = now(&timer);
                while reg(0x4003_0018) & (1 << 5) != 0 {
                    if now(&timer).wrapping_sub(start) >= 10_000 {
                        return Err(core::fmt::Error);
                    }
                }
                unsafe { (0x4003_0000 as *mut u32).write_volatile(u32::from(byte)) };
            }
            Ok(())
        }
    }
    core::fmt::write(&mut Sink, args)
}
