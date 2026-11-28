use super::{DOWN, LIMIT_US, LabResult, OUTPUT, dma_enables, now, pad_state, reg};
use rp1_hal::{
    gpio::Pin,
    pio::{
        PIO, PIOBuilder, Pins, Program, Rx, ShiftDirection, SyncStart, UninitStateMachine, pio_asm,
    },
    timer::Timer,
};

pub struct Bench {
    pub sm0: Option<UninitStateMachine<0>>,
    pub sm1: Option<UninitStateMachine<1>>,
    pub sm2: Option<UninitStateMachine<2>>,
    pub sm3: Option<UninitStateMachine<3>>,
    pub gpio12: Option<Pin<12>>,
    pub gpio13: Option<Pin<13>>,
    pub gpio22: Option<Pin<22>>,
    pub gpio23: Option<Pin<23>>,
}

// Each branch path takes five instructions at 50 MHz. X timestamps each edge
// at a 10 MHz sample rate, including on paths that push a word.
#[inline(never)]
fn sampler() -> Program<32> {
    pio_asm!(
        "mov x, ~null",
        "low:",
        "jmp pin rise",
        "nop [2]",
        "jmp x-- low",
        "rise:",
        "in x, 32",
        "nop [1]",
        "jmp x-- high",
        "high:",
        "jmp pin stays_high",
        "in x, 32",
        "nop [1]",
        "jmp x-- low",
        "stays_high:",
        "nop [2]",
        "jmp x-- high"
    )
    .program
}

#[inline(never)]
fn set_program() -> Program<32> {
    pio_asm!(
        "pull block",
        "mov x, osr",
        "pulse:",
        "set pins, 1 [9]",
        "set pins, 0 [8]",
        "jmp x-- pulse",
        "park:",
        "jmp park"
    )
    .program
}

#[derive(Clone, Copy)]
struct Case {
    name: &'static str,
    integer: u16,
    fraction: u8,
    cycles: u32,
    pulses: u32,
    side: bool,
    dirs: bool,
    high_ns: u32,
    period_ns: u32,
    sum_low_us: u32,
    sum_high_us: u32,
}

const CASES: [Case; 7] = [
    Case {
        name: "1khz",
        integer: 10000,
        fraction: 0,
        cycles: 20,
        pulses: 10000,
        side: false,
        dirs: false,
        high_ns: 500_000,
        period_ns: 1_000_000,
        sum_low_us: 990_000,
        sum_high_us: 1_010_000,
    },
    Case {
        name: "10khz",
        integer: 1000,
        fraction: 0,
        cycles: 20,
        pulses: 10000,
        side: false,
        dirs: false,
        high_ns: 50_000,
        period_ns: 100_000,
        sum_low_us: 99_000,
        sum_high_us: 101_000,
    },
    Case {
        name: "int",
        integer: 800,
        fraction: 0,
        cycles: 20,
        pulses: 1001,
        side: false,
        dirs: false,
        high_ns: 40_000,
        period_ns: 80_000,
        sum_low_us: 79_200,
        sum_high_us: 80_800,
    },
    Case {
        name: "frac",
        integer: 800,
        fraction: 128,
        cycles: 20,
        pulses: 1001,
        side: false,
        dirs: false,
        high_ns: 40_025,
        period_ns: 80_050,
        sum_low_us: 79_250,
        sum_high_us: 80_850,
    },
    Case {
        name: "side",
        integer: 1000,
        fraction: 0,
        cycles: 20,
        pulses: 1000,
        side: true,
        dirs: false,
        high_ns: 50_000,
        period_ns: 100_000,
        sum_low_us: 99_000,
        sum_high_us: 101_000,
    },
    Case {
        name: "opt_side",
        integer: 2000,
        fraction: 0,
        cycles: 10,
        pulses: 1000,
        side: true,
        dirs: false,
        high_ns: 50_000,
        period_ns: 100_000,
        sum_low_us: 99_000,
        sum_high_us: 101_000,
    },
    Case {
        name: "pindirs",
        integer: 1000,
        fraction: 0,
        cycles: 20,
        pulses: 1000,
        side: true,
        dirs: true,
        high_ns: 50_000,
        period_ns: 100_000,
        sum_low_us: 99_000,
        sum_high_us: 101_000,
    },
];

impl Case {
    #[inline(never)]
    fn program(self) -> Program<32> {
        match self.name {
            "side" => {
                pio_asm!(
                    ".side_set 1",
                    "pull block side 0",
                    "mov x, osr side 0",
                    "pulse:",
                    "nop side 1 [9]",
                    "nop side 0 [8]",
                    "jmp x-- pulse side 0",
                    "park:",
                    "jmp park side 0"
                )
                .program
            }
            "opt_side" => {
                pio_asm!(
                    ".side_set 1 opt",
                    "pull block side 0",
                    "mov x, osr",
                    "pulse:",
                    "nop side 1 [3]",
                    "nop",
                    "nop side 0 [3]",
                    "jmp x-- pulse",
                    "park:",
                    "jmp park"
                )
                .program
            }
            "pindirs" => {
                pio_asm!(
                    ".side_set 1 pindirs",
                    "pull block side 0",
                    "mov x, osr side 0",
                    "pulse:",
                    "nop side 1 [9]",
                    "nop side 0 [8]",
                    "jmp x-- pulse side 0",
                    "park:",
                    "jmp park side 0"
                )
                .program
            }
            _ => set_program(),
        }
    }
}

#[derive(Clone, Copy, Default)]
struct Stamp {
    // Timer reads bracket the last empty observation and the first read of this edge.
    lo: u32,
    hi: u32,
    sample: u32,
}

#[derive(Clone, Copy, Default)]
struct Window {
    first: Stamp,
    last: Stamp,
}

struct Edges {
    count: u32,
    last_empty: u32,
    first: Stamp,
    last: Stamp,
    rising: u32,
    high_min: u32,
    high_max: u32,
    period_min: u32,
    period_max: u32,
    windows: [Window; 10],
}

impl Edges {
    fn new(start: u32) -> Self {
        Self {
            count: 0,
            last_empty: start,
            first: Stamp::default(),
            last: Stamp::default(),
            rising: 0,
            high_min: u32::MAX,
            high_max: 0,
            period_min: u32::MAX,
            period_max: 0,
            windows: [Window::default(); 10],
        }
    }

    #[inline(never)]
    fn probe<const N: u8>(&mut self, rx: &mut Rx<N>, timer: &Timer) {
        let before = now(timer);
        let Some(word) = rx.read() else {
            self.last_empty = before;
            return;
        };
        let edge = Stamp {
            lo: self.last_empty,
            hi: now(timer),
            sample: !word,
        };
        if self.count.is_multiple_of(2) {
            let pulse = self.count / 2;
            if pulse == 0 {
                self.first = edge;
            } else {
                let period = edge.sample.wrapping_sub(self.rising);
                self.period_min = self.period_min.min(period);
                self.period_max = self.period_max.max(period);
            }
            if pulse.is_multiple_of(1000) {
                let window = (pulse / 1000) as usize;
                if window > 0 && window <= self.windows.len() {
                    self.windows[window - 1].last = edge;
                }
                if window < self.windows.len() {
                    self.windows[window].first = edge;
                }
            }
            self.rising = edge.sample;
            self.last = edge;
        } else {
            let high = edge.sample.wrapping_sub(self.rising);
            self.high_min = self.high_min.min(high);
            self.high_max = self.high_max.max(high);
        }
        self.count += 1;
    }

    #[inline(never)]
    fn report(&self, case: Case, channel: u8) -> bool {
        let expected = case.pulses * 2;
        let high_min_ns = self.high_min.saturating_sub(1).saturating_mul(100);
        let high_max_ns = self.high_max.saturating_add(1).saturating_mul(100);
        let period_min_ns = self.period_min.saturating_sub(1).saturating_mul(100);
        let period_max_ns = self.period_max.saturating_add(1).saturating_mul(100);
        // The weak pull-down discharges the PINDIRS fixture during Hi-Z.
        let high_tolerance_ns = if case.dirs { 5_000 } else { 1_000 };
        let mut pass = self.count == expected
            && high_min_ns >= case.high_ns - high_tolerance_ns
            && high_max_ns <= case.high_ns + high_tolerance_ns
            && period_min_ns >= case.period_ns - 1_000
            && period_max_ns <= case.period_ns + 1_000;
        log!("PIO_B_WAVE case={} channel={channel} clkdiv={}:{} cycles={} edges={} first={} last={} high_min_ns={high_min_ns} high_max_ns={high_max_ns} period_min_ns={period_min_ns} period_max_ns={period_max_ns} quantum_ns=100\n",
            case.name, case.integer, case.fraction, case.cycles, self.count, self.first.sample, self.last.sample).unwrap();
        for window in 0..((case.pulses - 1) / 1000) as usize {
            let Window { first, last } = self.windows[window];
            let lo = last.lo.wrapping_sub(first.hi).saturating_sub(1);
            let hi = last.hi.wrapping_sub(first.lo).saturating_add(1);
            pass &= lo >= case.sum_low_us && hi <= case.sum_high_us;
            log!("PIO_B_WINDOW case={} channel={channel} i={window} intervals=1000 first_lo={} first_hi={} last_lo={} last_hi={} sum_lo={lo} sum_hi={hi}\n",
                case.name, first.lo, first.hi, last.lo, last.hi).unwrap();
        }
        pass
    }
}

impl Bench {
    pub fn check(&mut self, pio: &PIO, timer: &Timer) -> LabResult<()> {
        for case in CASES {
            self.single(pio, timer, case)?;
        }
        self.phase(pio, timer)?;
        self.group(pio, timer, false)?;
        self.group(pio, timer, true)?;
        Ok(())
    }

    #[inline(never)]
    fn single(&mut self, pio: &PIO, timer: &Timer, case: Case) -> LabResult<()> {
        let sample = pio.install(&sampler()).map_err(|_| "sample install")?;
        let emit = pio.install(&case.program()).map_err(|_| "wave install")?;
        let (sampler, mut rx, tx) = PIOBuilder::from_installed_program(&sample)
            .clock_divisor_fixed_point(4, 0)
            .jmp_pin(22)
            .autopush(true)
            .pins(Pins::new().input(self.gpio22.take().ok_or("gpio22")?, DOWN))
            .build(self.sm2.take().ok_or("sm2")?)
            .map_err(|_| "sampler build")?;
        let builder = PIOBuilder::from_installed_program(&emit)
            .clock_divisor_fixed_point(case.integer, case.fraction)
            .out_shift(ShiftDirection::Right)
            .pins(Pins::new().output(
                self.gpio12.take().ok_or("gpio12")?,
                OUTPUT,
                case.dirs,
                !case.dirs,
            ));
        let builder = if case.side {
            builder.side_base(12)
        } else {
            builder.set_pins(12, 1)
        };
        let (emitter, erx, mut etx) = builder
            .build(self.sm0.take().ok_or("sm0")?)
            .map_err(|_| "wave build")?;
        if pad_state() != 0xf || !etx.write(case.pulses - 1) {
            return Err("wave preflight");
        }
        let start = now(timer);
        let (emitter, sampler) = (emitter, sampler).sync_start().map_err(|_| "wave start")?;
        if pad_state() & 0xc != 0xc {
            return Err("wave receiver drive");
        }
        let mut edges = Edges::new(start);
        let mut last = None;
        loop {
            edges.probe(&mut rx, timer);
            let time = now(timer);
            if edges.count > case.pulses * 2 || time.wrapping_sub(start) >= LIMIT_US {
                break;
            }
            if edges.count == case.pulses * 2
                && time.wrapping_sub(*last.get_or_insert(time)) > 2 * (case.period_ns / 1_000)
            {
                break;
            }
        }
        let (sm, mut pins) = emitter.stop().uninit(erx, etx);
        self.sm0 = Some(sm);
        self.gpio12 = pins.take();
        let (sm, mut pins) = sampler.stop().uninit(rx, tx);
        self.sm2 = Some(sm);
        self.gpio22 = pins.take();
        emit.uninstall().map_err(|_| "wave uninstall")?;
        sample.uninstall().map_err(|_| "sample uninstall")?;
        let elapsed = now(timer).wrapping_sub(start);
        let pass =
            edges.report(case, 0) && elapsed < LIMIT_US && pad_state() == 0xf && dma_enables() == 0;
        log!("PIO_B_WAVE_END case={} start={start} elapsed={elapsed} pass={} safety={:#x} dma={:#x}\n",
            case.name, u8::from(pass), pad_state(), dma_enables()).unwrap();
        if !pass {
            return Err("wave bounds");
        }
        Ok(())
    }

    fn phase(&mut self, pio: &PIO, timer: &Timer) -> LabResult<()> {
        let emit = pio.install(&set_program()).map_err(|_| "phase install")?;
        let sample = pio
            .install(&sampler())
            .map_err(|_| "phase sampler install")?;
        let case = Case {
            name: "phase",
            pulses: 10001,
            ..CASES[1]
        };
        // Use the same program and divider for both emitters and both samplers.
        macro_rules! emitter {
            ($pin:ident, $sm:ident, $n:expr) => {
                PIOBuilder::from_installed_program(&emit)
                    .set_pins($n, 1)
                    .clock_divisor_fixed_point(case.integer, case.fraction)
                    .pins(Pins::new().output(
                        self.$pin.take().ok_or("phase output")?,
                        OUTPUT,
                        false,
                        true,
                    ))
                    .build(self.$sm.take().ok_or("phase sm")?)
                    .map_err(|_| "phase output build")?
            };
        }
        macro_rules! receiver {
            ($pin:ident, $sm:ident, $n:expr) => {
                PIOBuilder::from_installed_program(&sample)
                    .jmp_pin($n)
                    .clock_divisor_fixed_point(4, 0)
                    .autopush(true)
                    .pins(Pins::new().input(self.$pin.take().ok_or("phase input")?, DOWN))
                    .build(self.$sm.take().ok_or("phase sm")?)
                    .map_err(|_| "phase input build")?
            };
        }
        let (a, arx, mut atx) = emitter!(gpio12, sm0, 12);
        let (b, brx, mut btx) = emitter!(gpio13, sm1, 13);
        let (c, mut crx, ctx) = receiver!(gpio22, sm2, 22);
        let (d, mut drx, dtx) = receiver!(gpio23, sm3, 23);
        if pad_state() != 0xf || !atx.write(case.pulses - 1) || !btx.write(case.pulses - 1) {
            return Err("phase preflight");
        }
        let start = now(timer);
        let (c, d) = (c, d).sync_start().map_err(|_| "phase sampler start")?;
        let (a, b) = (a, b).sync_start().map_err(|_| "phase output start")?;
        if pad_state() & 0xc != 0xc {
            return Err("phase receiver drive");
        }
        let (mut u, mut v) = (Edges::new(start), Edges::new(start));
        let mut last = None;
        loop {
            u.probe(&mut crx, timer);
            v.probe(&mut drx, timer);
            let time = now(timer);
            if u.count > case.pulses * 2
                || v.count > case.pulses * 2
                || time.wrapping_sub(start) >= LIMIT_US
            {
                break;
            }
            if u.count == case.pulses * 2
                && v.count == case.pulses * 2
                && time.wrapping_sub(*last.get_or_insert(time)) > 200
            {
                break;
            }
        }
        let elapsed = now(timer).wrapping_sub(start);
        macro_rules! reclaim {
            ($running:ident, $rx:ident, $tx:ident, $pin:ident, $sm:ident) => {
                let (sm, mut pins) = $running.stop().uninit($rx, $tx);
                self.$sm = Some(sm);
                self.$pin = pins.take();
            };
        }
        reclaim!(a, arx, atx, gpio12, sm0);
        reclaim!(b, brx, btx, gpio13, sm1);
        reclaim!(c, crx, ctx, gpio22, sm2);
        reclaim!(d, drx, dtx, gpio23, sm3);
        emit.uninstall().map_err(|_| "phase emitter uninstall")?;
        sample.uninstall().map_err(|_| "phase sampler uninstall")?;
        // One sample of quantization in either direction bounds each measured difference.
        let first = i64::from(v.first.sample.wrapping_sub(u.first.sample) as i32) * 100;
        let last = i64::from(v.last.sample.wrapping_sub(u.last.sample) as i32) * 100;
        let pass_u = u.report(case, 0);
        let pass_v = v.report(case, 1);
        let pass = pass_u
            && pass_v
            && [first, last]
                .into_iter()
                .all(|sample| (-1900..=1900).contains(&sample))
            && elapsed < LIMIT_US
            && pad_state() == 0xf
            && dma_enables() == 0;
        log!("PIO_B_PHASE first_ns={first} last_ns={last} uncertainty_ns=100 pulses={} start={start} elapsed={elapsed} pass={} safety={:#x} dma={:#x}\n",
            case.pulses, u8::from(pass), pad_state(), dma_enables()).unwrap();
        if !pass {
            return Err("phase bounds");
        }
        Ok(())
    }

    fn group(&mut self, pio: &PIO, timer: &Timer, four: bool) -> LabResult<()> {
        let p = pio
            .install(
                &pio_asm!(
                    "pull block",
                    "mov isr, osr",
                    "push block",
                    "irq set 0 rel",
                    "park:",
                    "jmp park"
                )
                .program,
            )
            .map_err(|_| "group install")?;
        let (a, mut arx, mut atx) = PIOBuilder::from_installed_program(&p)
            .build(self.sm0.take().ok_or("group sm0")?)
            .map_err(|_| "group build")?;
        let (b, mut brx, mut btx) = PIOBuilder::from_installed_program(&p)
            .build(self.sm1.take().ok_or("group sm1")?)
            .map_err(|_| "group build")?;
        let (c, mut crx, mut ctx) = PIOBuilder::from_installed_program(&p)
            .build(self.sm2.take().ok_or("group sm2")?)
            .map_err(|_| "group build")?;
        if !atx.write(1) || !btx.write(2) || !ctx.write(3) {
            return Err("group preload");
        }
        let start = now(timer);
        let expected = if four { 15 } else { 7 };
        let wait = || {
            while reg(0xf000_0034) & 0xf != expected {
                if now(timer).wrapping_sub(start) >= 1_000 {
                    return Err("group flag timeout");
                }
            }
            Ok(())
        };
        let (a, b, c) = if four {
            let (d, mut drx, mut dtx) = PIOBuilder::from_installed_program(&p)
                .build(self.sm3.take().ok_or("group sm3")?)
                .map_err(|_| "group build")?;
            if !dtx.write(4) {
                return Err("group preload");
            }
            let (a, b, c, d) = (a, b, c, d).sync_start().map_err(|_| "group start")?;
            wait()?;
            if drx.read() != Some(4) || drx.read().is_some() {
                return Err("group sm3 completion");
            }
            self.sm3 = Some(d.stop().uninit(drx, dtx).0);
            (a, b, c)
        } else {
            let result = (a, b, c).sync_start().map_err(|_| "group start")?;
            wait()?;
            result
        };
        if [arx.read(), brx.read(), crx.read()] != [Some(1), Some(2), Some(3)]
            || [arx.read(), brx.read(), crx.read()] != [None; 3]
        {
            return Err("group completion");
        }
        self.sm0 = Some(a.stop().uninit(arx, atx).0);
        self.sm1 = Some(b.stop().uninit(brx, btx).0);
        self.sm2 = Some(c.stop().uninit(crx, ctx).0);
        p.uninstall().map_err(|_| "group uninstall")?;
        let remaining = reg(0xf000_0034);
        log!("PIO_B_GROUP sms={} flags={expected:#x} completions={} remaining={remaining:#x} elapsed={} dma={:#x}\n",
            if four { 4 } else { 3 }, if four { 4 } else { 3 }, now(timer).wrapping_sub(start), dma_enables()).unwrap();
        if remaining != 0 || dma_enables() != 0 {
            return Err("group retirement");
        }
        Ok(())
    }
}
