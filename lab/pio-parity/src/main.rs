//! Gate A: one push, one pop and reset on the declaring core.
#![no_std]
#![no_main]

#[path = "../../shm-firmware/src/stack_watermark.rs"]
mod stack_watermark;

use rp1_hal::{log, pac, pio::PioPin, timer::Timer};
use rp1_proto::board::{PIO_IRQ0, PIO_IRQ1};
use rp1_rt::{Proc0, Proc1, entry, shared, sync::Spsc};

rp1_rt::bind!();
type PingWord = u32;

const TRIALS: usize = 100;
const TIMEOUT_US: u32 = 1_000;
const QUIET_US: u32 = 1_000;
const RESET_PIO: u32 = 1 << 27;
const RX_EMPTY: u32 = 1 << 8;
const DREQ_ENABLE_MASK: u16 = 0xaaaa;
const DREQ_RX_STATUS_MASK: u16 = 0x5500;
const NVIC_PIO: u32 = (1 << (PIO_IRQ0 - 32)) | (1 << (PIO_IRQ1 - 32));
const RESET_FSTAT: u32 = 0x0f00_0f00;
const PROGRAM: [u32; 4] = [0xe035, 0xa0c1, 0x8020, 0x0003];
const _: () = assert!(rp1::PioParts::SM_MASK == 1);

#[shared]
static COMPLETED: Spsc<u32, 2> = Spsc::new();

#[derive(Clone, Copy, Default)]
struct Trial {
    release_at: u32,
    done_at: u32,
    raw_irq: u32,
    raw_dreq: u16,
    reset_dreq: u16,
    reset_done: bool,
    pc: u8,
    rx: u32,
    pushed: u8,
    popped: bool,
    empty: bool,
}

impl Trial {
    fn upper_us(self) -> u32 {
        self.done_at.wrapping_sub(self.release_at).saturating_add(1)
    }

    fn passed(self) -> bool {
        self.reset_done
            && self.upper_us() <= TIMEOUT_US
            && self.pc == 3
            && self.rx == 21
            && self.pushed == 1
            && self.popped
            && self.empty
            && self.raw_irq == 0
            && self.raw_dreq & (DREQ_ENABLE_MASK | DREQ_RX_STATUS_MASK) == 0
    }
}

#[entry(proc0)]
fn main0(mut core: Proc0, mut p: rp1::Proc0Resources) -> ! {
    let watermark = unsafe { stack_watermark::paint_core::<0>() };
    let mut sys = rp1::platform::init(&mut core, p.platform).unwrap();
    log!("PIO_BOOT PLAT_INIT ok=1 {}\n", sys.report).unwrap();
    p.proc1.launch(&mut core).unwrap();
    rp1::link::enable(&mut sys.pcie, &mut p.TIMER_ALARM1, &mut p.irq.TIMER_1).unwrap();
    let mut completed = COMPLETED.consumer(&mut core);
    loop {
        if let Some(run) = completed.pop() {
            log!(
                "PIO_STACK core=0 run={run} free={}\n",
                watermark.free_bytes()
            )
            .unwrap();
        }
        rp1_rt::wait_for_event();
    }
}

fn input_only<const N: u8>(_pin: PioPin<N>) -> bool {
    const IO_BANK0_RESET_CLEAR: *mut u32 = 0x4001_7004 as *mut u32;
    const IO_BANK0_RESET: u32 = 1 << 19;
    const RIO_OE_CLEAR: *mut u32 = 0x400e_3004 as *mut u32;
    const PAD_INPUT: u32 = (1 << 7) | (1 << 6) | (1 << 2) | (1 << 1);
    const FUNCSEL_RIO: u32 = 5;
    const ROUTE_MASK: u32 = (0x3f << 12) | 0x1f;
    const XOR_ALIAS: usize = 0x1000;
    // Isolate the declared pad before changing its mux.
    let io = unsafe { &*pac::IO_BANK0::PTR.cast::<pac::generated::io_bank0::RegisterBlock>() };
    let pads =
        unsafe { &*pac::PADS_BANK0::PTR.cast::<pac::generated::pads_bank0::RegisterBlock>() };
    let ctrl = io.gpio_ctrl(N.into());
    let disable = pac::generated::io_bank0::gpio_ctrl::oeover::DISABLE;
    let ctrl_xor = (pac::IO_BANK0::PTR.addr() + XOR_ALIAS + 4 + usize::from(N) * 8) as *mut u32;
    unsafe {
        IO_BANK0_RESET_CLEAR.write_volatile(IO_BANK0_RESET);
        pads.gpio(N.into()).write_bits_unchecked(PAD_INPUT);
        RIO_OE_CLEAR.write_volatile(1 << N);
        let route = (disable << 14) | FUNCSEL_RIO;
        ctrl_xor.write_volatile((ctrl.read().bits() ^ route) & ROUTE_MASK);
    };
    let pad = pads.gpio(N.into()).read();
    pad.ie().bits() && pad.od().bits() && ctrl.read().oeover().bits() == disable
}

#[entry(proc1)]
fn main1(mut core: Proc1, p: rp1::Proc1Resources) -> ! {
    // Proc1 owns its private stack. Paint before opening the polling endpoint.
    let watermark = unsafe { stack_watermark::paint() };
    let pads = [
        input_only(p.pins.gpio22),
        input_only(p.pins.gpio23),
        input_only(p.pins.gpio12),
        input_only(p.pins.gpio13),
    ];
    let probe = Probe::new(p.pio);
    let mut completed = COMPLETED.producer(&mut core);
    let mut ping = p.link.open().unwrap().ping;
    let mut trials = [Trial::default(); TRIALS];
    let mut run = 0u32;
    let pads_ok = pads.into_iter().all(|ok| ok);
    log!("PIO_READY core=1 pads_ie_od_oeoff={pads:?} clk_hz=200000000 clkdiv=1.0\n").unwrap();
    loop {
        let mut outcome = None;
        let _ = ping
            .serve(1, |value| {
                run += 1;
                if !pads_ok {
                    return !value;
                }
                let timer = Timer::new();
                let start = timer.now_32().ticks();
                let entry_done = probe.done();
                let mut count = 0;
                for trial in &mut trials {
                    *trial = probe.trial(&timer);
                    count += 1;
                    if !trial.passed() {
                        break;
                    }
                }
                // Flush FIFOs and instruction state before restoring the entry reset condition.
                let restored =
                    probe.reset(&timer, false) && (!entry_done || probe.reset(&timer, true));
                let end = timer.now_32().ticks();
                let pass = restored
                    && count == TRIALS
                    && trials.iter().all(|trial| trial.passed())
                    && end.wrapping_sub(start) < 30_000_000;
                outcome = Some((count, start, end, entry_done, restored, pass));
                // The reply acknowledges the trigger; the UART records determine the verdict.
                value
            })
            .unwrap();
        if let Some((count, start, end, entry_done, restored, pass)) = outcome {
            for (index, t) in trials[..count].iter().enumerate() {
                log!(
                    "PIO_TRIAL run={run} i={index} release={} done={} seen={} upper={} pc={} rx={} push={} pop={} empty={} irq={:#x} dreq={:#x} reset_dreq={:#x}\n",
                    t.release_at, t.done_at, u8::from(t.reset_done), t.upper_us(), t.pc, t.rx,
                    t.pushed, u8::from(t.popped), u8::from(t.empty), t.raw_irq, t.raw_dreq, t.reset_dreq,
                ).unwrap();
            }
            let bounds = trials[..count].iter().map(|trial| trial.upper_us());
            log!(
                "PIO_END run={run} pass={} trials={count} start={start} end={end} reset_upper_min={} reset_upper_max={} entry_done={} restored={} stack_free={}\n",
                u8::from(pass), bounds.clone().min().unwrap(), bounds.max().unwrap(),
                u8::from(entry_done), u8::from(restored), watermark.free_bytes(),
            ).unwrap();
            completed.push(run).unwrap();
            rp1_rt::send_event();
        }
    }
}

struct Probe {
    _owner: rp1::PioParts,
    pio: &'static pac::generated::pio_m3::RegisterBlock,
    resets: &'static pac::generated::resets::RegisterBlock,
}

impl Probe {
    fn new(owner: rp1::PioParts) -> Self {
        // The declaration excludes Linux and the other core from both PIO windows.
        unsafe {
            Self {
                _owner: owner,
                pio: &*pac::PIO_M3::PTR.cast(),
                resets: &*pac::RESETS::PTR.cast(),
            }
        }
    }

    fn done(&self) -> bool {
        self.resets.done0().read().pio_apbs().bits()
    }

    fn set_reset(&self, released: bool) {
        // Only PIO_APBS changes. Other reset bits belong to other drivers.
        unsafe {
            if released {
                self.resets.ctrl0().atomic_clear_bits_unchecked(RESET_PIO);
            } else {
                self.resets.ctrl0().atomic_set_bits_unchecked(RESET_PIO);
            }
        }
    }

    fn reset(&self, timer: &Timer, released: bool) -> bool {
        self.set_reset(released);
        let start = timer.now_32().ticks();
        while self.done() != released {
            if timer.now_32().ticks().wrapping_sub(start) >= TIMEOUT_US {
                return false;
            }
        }
        true
    }

    fn sample_quiet(&self, trial: &mut Trial) {
        let pio = self.pio;
        // INTR includes TX-not-full even with no enabled IRQ. Sample masked status and
        // core-local pending, plus the enable and candidate raw request bits.
        let pending = unsafe { (0xe000_e204 as *const u32).read_volatile() };
        trial.raw_irq |=
            pio.irq0_ints().read().bits() | pio.irq1_ints().read().bits() | (pending & NVIC_PIO);
        trial.raw_dreq |= self.dreq();
    }

    fn dreq(&self) -> u16 {
        let mut flags = 0;
        // Pack bits 31:30 per direction: TX0..3, then RX0..3.
        for sm in 0..4 {
            let tx = (self.pio.sm_dmatx(sm).read().bits() >> 30) as u16;
            let rx = (self.pio.sm_dmarx(sm).read().bits() >> 30) as u16;
            flags |= tx << (2 * sm) | rx << (8 + 2 * sm);
        }
        flags
    }

    fn trial(&self, timer: &Timer) -> Trial {
        let mut t = Trial::default();
        if !self.reset(timer, false) {
            return t;
        }
        // Bracket reset release through the first DONE sample. The upper bound adds one tick.
        t.release_at = timer.now_32().ticks();
        self.set_reset(true);
        loop {
            t.reset_done = self.done();
            t.done_at = timer.now_32().ticks();
            if t.reset_done || t.upper_us() >= TIMEOUT_US {
                break;
            }
        }
        if !t.reset_done || t.upper_us() > TIMEOUT_US {
            return t;
        }
        let pio = self.pio;
        if pio.ctrl().read().bits() & 0xfff != 0
            || pio.fstat().read().bits() != RESET_FSTAT
            || pio.flevel().read().bits() != 0
        {
            return t;
        }
        t.reset_dreq = self.dreq();
        if t.reset_dreq & (DREQ_ENABLE_MASK | DREQ_RX_STATUS_MASK) != 0 {
            return t;
        }
        // Keep TX threshold zero and RX above both FIFO levels exercised here.
        // Leave all DREQ enables clear.
        for sm in 0..4 {
            unsafe {
                pio.sm_dmatx(sm).write_disabled_threshold(0);
                pio.sm_dmarx(sm).write_disabled_threshold(2);
            }
            if pio.sm_dmatx(sm).read().bits() & 0x1f != 0
                || pio.sm_dmarx(sm).read().bits() & 0x1f != 2
            {
                return t;
            }
        }
        let quiet = timer.now_32().ticks();
        while timer.now_32().ticks().wrapping_sub(quiet) < QUIET_US {
            self.sample_quiet(&mut t);
        }
        // SM0 uses no pins, side-set, autopush or JOIN.
        unsafe {
            pio.sm_pinctrl(0).write_bits_unchecked(0);
            pio.sm_clkdiv(0).write_bits_unchecked(0x0001_0000);
            pio.sm_execctrl(0).write_bits_unchecked(0x0000_3000);
            pio.sm_shiftctrl(0).write_bits_unchecked(0);
            for (i, instruction) in PROGRAM.into_iter().enumerate() {
                pio.instr_mem(i).write_bits_unchecked(instruction);
            }
            pio.ctrl().write_bits_unchecked(0x110);
            pio.sm_instr(0).write_bits_unchecked(0);
            pio.ctrl().write_bits_unchecked(1);
        }
        let start = timer.now_32().ticks();
        while timer.now_32().ticks().wrapping_sub(start) < TIMEOUT_US {
            if pio.fstat().read().bits() & RX_EMPTY == 0 && pio.sm_addr(0).read().bits() == 3 {
                break;
            }
        }
        unsafe { pio.ctrl().write_bits_unchecked(0) };
        self.sample_quiet(&mut t);
        t.pc = pio.sm_addr(0).read().bits() as u8;
        t.pushed = ((pio.flevel().read().bits() >> 4) & 0xf) as u8;
        if pio.fstat().read().bits() & RX_EMPTY == 0 {
            t.rx = pio.rxf(0).pop();
            t.popped = true;
        }
        t.empty = pio.fstat().read().bits() & RX_EMPTY != 0 && pio.flevel().read().bits() == 0;
        t
    }
}
