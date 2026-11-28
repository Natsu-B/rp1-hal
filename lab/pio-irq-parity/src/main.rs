//! PIO FIFO IRQ delivery and timing on the CM5 loopbacks.
#![no_std]
#![no_main]

#[path = "../../shm-firmware/src/stack_watermark.rs"]
mod stack_watermark;

macro_rules! log {
    ($($arg:tt)*) => { write_log(format_args!($($arg)*)) };
}

use core::sync::atomic::{AtomicU32, Ordering};
use embedded_hal::digital::{InputPin, OutputPin};
use rp1_hal::{
    clocks::Clocks,
    gpio::{Input, PadConfig, Pin, Pull},
    pio::{
        InitialOutput, Irq, PIO, PIOBuilder, PIOExt, Pins, UninitStateMachine, WaitError, pio_asm,
    },
    timer::{Deadline, Error, Instant, Timer, WaitTimer},
};
use rp1_proto::board::{PIO_IRQ0, PIO_IRQ1};
use rp1_rt::{Proc0, Proc1, entry, shared};

rp1_rt::bind!();
type PingWord = u32;
type Checked<T> = Result<T, &'static str>;
type Sm<const N: u8> = UninitStateMachine<N, Irq<1, 3>>;

const INTE: [usize; 2] = [0xf000_0150, 0xf000_015c];
const INTF: usize = 4;
const INTS: usize = 8;
const ISER: usize = 0xe000_e104;
const ICER: usize = 0xe000_e184;
const ISPR: usize = 0xe000_e204;
const ICPR: usize = 0xe000_e284;
const LINE_BITS: [u32; 2] = [1 << (PIO_IRQ0 - 32), 1 << (PIO_IRQ1 - 32)];
const LINE_MASK: u32 = LINE_BITS[0] | LINE_BITS[1];
const TRIALS: u32 = 1_000;
const TIMEOUTS: u32 = 100;
const DEADLINE_US: u32 = 1_000;
const RIO_OE_SET: usize = 0x400e_2004;
const RIO_OE_CLEAR: usize = 0x400e_3004;
const PAD12_OD_SET: usize = 0x400f_2004 + 12 * 4;
const PAD12_OD_CLEAR: usize = 0x400f_3004 + 12 * 4;
const GPIO23_CTRL_SET: usize = 0x400d_2004 + 23 * 8;
const GPIO12: u32 = 1 << 12;
const PAD_OD: u32 = 1 << 7;
const OEOVER_DISABLE: u32 = 2 << 14;
const GATE_LIMIT: u32 = 30_000_000;
const STIMULUS_GUARD_US: u32 = 100;
const INPUT: PadConfig = PadConfig::INPUT_DEFAULT.with_pull(Pull::Down);
const OUTPUT: PadConfig = PadConfig::OUTPUT_DEFAULT.with_pull(Pull::None);

#[repr(transparent)]
struct Mailbox(AtomicU32);

// The shared words use only aligned loads and stores. Each has one writer.
unsafe impl rp1_proto::SharedSafe for Mailbox {}

impl Mailbox {
    fn get(&self) -> u32 {
        load(&self.0)
    }

    fn put(&self, value: u32) {
        store(&self.0, value);
    }
}

#[shared]
static COMMAND: Mailbox = Mailbox(AtomicU32::new(0));
#[shared]
static TARGET: Mailbox = Mailbox(AtomicU32::new(0));
#[shared]
static REPLY: Mailbox = Mailbox(AtomicU32::new(0));
#[shared]
static LOWER: Mailbox = Mailbox(AtomicU32::new(0));
#[shared]
static UPPER: Mailbox = Mailbox(AtomicU32::new(0));
#[shared]
static STIMULUS: Mailbox = Mailbox(AtomicU32::new(0));

#[shared]
static FINISHED: Mailbox = Mailbox(AtomicU32::new(0));

const EDGE: u32 = 1;
const PENDING: u32 = 2;
const PARK: u32 = 3;
const QUIET: u32 = 4;

static FORCE: AtomicU32 = AtomicU32::new(0);
static DELIVERIES: [AtomicU32; 2] = [const { AtomicU32::new(0) }; 2];
static ENTRIES: [AtomicU32; 2] = [const { AtomicU32::new(0) }; 2];
static LAST_TICK: AtomicU32 = AtomicU32::new(0);
static LAST_SOURCE: AtomicU32 = AtomicU32::new(0);
static LAST_STATUS: AtomicU32 = AtomicU32::new(0);

fn read(address: usize) -> u32 {
    unsafe { (address as *const u32).read_volatile() }
}

fn write(address: usize, value: u32) {
    unsafe { (address as *mut u32).write_volatile(value) };
    rp1_proto::barrier();
}

fn tick() -> u32 {
    Timer::new().now_32().ticks()
}

fn load(value: &AtomicU32) -> u32 {
    value.load(Ordering::Acquire)
}

fn store(value: &AtomicU32, data: u32) {
    value.store(data, Ordering::Release);
}

fn until(us: u32, mut predicate: impl FnMut() -> bool) -> bool {
    let start = tick();
    while !predicate() {
        if tick().wrapping_sub(start) >= us {
            return false;
        }
    }
    true
}

fn pad(pin: usize) -> u32 {
    read(0x400f_0004 + 4 * pin)
}

fn inputs_isolated() -> bool {
    const PAD_IE: u32 = 1 << 6;
    [22, 23].into_iter().all(|pin| {
        pad(pin) & (PAD_OD | PAD_IE) == (PAD_OD | PAD_IE)
            && read(0x400d_0004 + 8 * pin) & (3 << 14) == OEOVER_DISABLE
    })
}

fn side_edge(drive: &mut Pin<12, rp1_hal::gpio::Output>, sense: &mut Pin<23, Input>) -> bool {
    let target = TARGET.get();
    if !until(DEADLINE_US, || tick().wrapping_sub(target) as i32 >= 0) || sense.is_high().unwrap() {
        return false;
    }
    let stimulus = tick();
    let mut lo = stimulus;
    drive.set_high().unwrap();
    let start = tick();
    let seen = loop {
        let before = tick();
        if sense.is_high().unwrap() {
            STIMULUS.put(stimulus);
            LOWER.put(lo);
            UPPER.put(tick());
            break true;
        }
        lo = before;
        if before.wrapping_sub(start) >= DEADLINE_US {
            break false;
        }
    };
    drive.set_low().unwrap();
    seen && until(DEADLINE_US, || sense.is_low().unwrap())
}

#[entry(proc0)]
fn main0(mut core: Proc0, mut p: rp1::Proc0Resources) -> ! {
    let watermark = unsafe { stack_watermark::paint_core::<0>() };
    let mut sys = rp1::platform::init(&mut core, p.platform).unwrap();
    let mut sense = p.pins.gpio23.into_input_with_pad(INPUT);
    write(GPIO23_CTRL_SET, OEOVER_DISABLE);
    let mut unconfigured = Some(p.pins.gpio12);
    let mut active = None;
    log!("PIO_C_BOOT {}\n", sys.report).unwrap();
    p.proc1.launch(&mut core).unwrap();
    rp1::link::enable(&mut sys.pcie, &mut p.TIMER_ALARM1, &mut p.irq.TIMER_1).unwrap();
    let mut last = 0;
    let mut completed = 0;
    loop {
        let command = COMMAND.get();
        if command != last {
            let result = match command & 7 {
                EDGE => {
                    if let Some(pin) = unconfigured.take() {
                        active = Some(pin.into_output_with_pad(false, OUTPUT));
                    } else {
                        write(RIO_OE_SET, GPIO12);
                        write(PAD12_OD_CLEAR, PAD_OD);
                    }
                    u32::from(!side_edge(active.as_mut().unwrap(), &mut sense))
                }
                PENDING => {
                    let _ = until(DEADLINE_US, || read(ISPR) & LINE_MASK != 0);
                    read(ISPR) & LINE_MASK
                }
                QUIET => {
                    write(ICPR, LINE_MASK);
                    let _ = until(DEADLINE_US, || read(ISPR) & LINE_MASK != 0);
                    read(ISPR) & LINE_MASK
                }
                PARK => {
                    write(PAD12_OD_SET, PAD_OD);
                    write(RIO_OE_CLEAR, GPIO12);
                    0
                }
                _ => unreachable!(),
            };
            if command & 7 != EDGE {
                LOWER.put(result);
            }
            REPLY.put(command | (result & 1) << 31);
            last = command;
        }
        let run = FINISHED.get();
        if run != completed {
            log!(
                "PIO_C_STACK run={run} core=0 free={}\n",
                watermark.free_bytes()
            )
            .unwrap();
            completed = run;
        }
    }
}

struct Peer(u32);

impl Peer {
    fn command(&mut self, mode: u32) {
        self.0 = ((self.0 & !7) + 8) | mode;
        COMMAND.put(self.0);
    }

    fn reply(&self) -> Checked<(u32, u32)> {
        if !until(DEADLINE_US + STIMULUS_GUARD_US, || {
            REPLY.get() & !(1 << 31) == self.0
        }) || REPLY.get() >> 31 != 0
        {
            return Err("peer");
        }
        Ok((LOWER.get(), UPPER.get()))
    }

    fn edge(&mut self) {
        TARGET.put(tick().wrapping_add(STIMULUS_GUARD_US));
        self.command(EDGE);
    }
}

unsafe extern "C" {
    #[link_name = "__rp1_p1_PIO_0"]
    fn generated_irq0();
    #[link_name = "__rp1_p1_PIO_1"]
    fn generated_irq1();
}

#[rp1_rt::isram]
unsafe extern "C" fn irq0() {
    interrupt::<0>();
}
#[rp1_rt::isram]
unsafe extern "C" fn irq1() {
    interrupt::<1>();
}

// Wrap the generated dispatchers to observe ISR entry and source retirement.
fn install_probe_vectors() {
    const VTOR: usize = 0xe000_ed08;
    const VECTOR_WORDS: usize = 80;
    #[repr(C, align(512))]
    struct VectorTable([u32; VECTOR_WORDS]);
    static mut PROBE: VectorTable = VectorTable([0; VECTOR_WORDS]);

    // Proc1 alone owns this copy. Both candidate lines are disabled here.
    unsafe {
        let current = read(VTOR) as *const u32;
        let probe = core::ptr::addr_of_mut!(PROBE).cast::<u32>();
        core::ptr::copy_nonoverlapping(current, probe, VECTOR_WORDS);
        probe
            .add(16 + PIO_IRQ0 as usize)
            .write(irq0 as *const () as u32);
        probe
            .add(16 + PIO_IRQ1 as usize)
            .write(irq1 as *const () as u32);
        write(VTOR, probe as u32);
        core::arch::asm!("isb sy", options(nomem, nostack, preserves_flags));
    }
}

fn interrupt<const LINE: u8>() {
    let time = tick();
    let index = usize::from(LINE);
    let status = read(INTE[index] + INTS);
    let source = if load(&FORCE) == u32::from(LINE) + 1 {
        // Preserve the level until proc0 samples its own disabled NVIC line.
        write(ICER, LINE_BITS[index]);
        status
    } else {
        let enabled = read(INTE[index]);
        // Call the original vector target and measure the bits it retired.
        unsafe {
            if LINE == 0 {
                generated_irq0();
            } else {
                generated_irq1();
            }
        }
        enabled & !read(INTE[index])
    };
    store(&ENTRIES[index], load(&ENTRIES[index]) + 1);
    if source != 0 {
        store(&LAST_STATUS, status);
        store(&LAST_SOURCE, source);
        store(&LAST_TICK, time);
        store(&DELIVERIES[index], load(&DELIVERIES[index]) + 1);
        rp1_rt::send_event();
    }
}

fn force(peer: &mut Peer, run: u32, line: usize) -> Checked<()> {
    let bit = LINE_BITS[line];
    let entries = load(&ENTRIES[line]);
    let other = load(&ENTRIES[1 - line]);
    // Proc0 leaves both NVIC lines disabled, so pending bits from earlier
    // transfers can remain latched after their source is masked.
    peer.command(QUIET);
    let pre0 = peer.reply()?.0;
    write(ICER, bit);
    write(ICPR, LINE_MASK);
    let pre1 = read(ISPR) & LINE_MASK;
    if pre0 | pre1 != 0 {
        return Err("force precondition");
    }
    store(&FORCE, line as u32 + 1);
    // FIFO sources are masked. Force the SM0 flag source without setting its W1C flag.
    const SM0_FLAG: u32 = 1 << 8;
    peer.command(PENDING);
    let start = tick();
    write(INTE[line] + INTF, SM0_FLAG);
    let pending0 = peer.reply();
    let pending1 = read(ISPR) & LINE_MASK;
    write(ISER, bit);
    let seen = until(DEADLINE_US, || load(&ENTRIES[line]) != entries);
    let isr = load(&LAST_TICK);
    let status = load(&LAST_STATUS);
    write(INTE[line] + INTF, 0);
    let cleared = tick();
    write(ICPR, LINE_MASK);
    peer.command(QUIET);
    let quiet0 = peer.reply();
    write(ISER, bit);
    let _ = until(DEADLINE_US, || load(&ENTRIES[line]) > entries + 1);
    let quiet1 = read(ISPR) & LINE_MASK;
    store(&FORCE, 0);
    let quiet0 = quiet0?.0;
    let pending0 = pending0?.0;
    let count = load(&ENTRIES[line]) - entries;
    let cross = load(&ENTRIES[1 - line]) - other;
    log!("PIO_C_FORCE run={run} line={line} start={start} isr={isr} clear={cleared} pre0={pre0:#x} pre1={pre1:#x} p0={pending0:#x} p1={pending1:#x} quiet0={quiet0:#x} quiet1={quiet1:#x} status={status:#x} entries={count} cross={cross}\n").unwrap();
    if !seen
        || pending0 & !bit != 0
        || pending1 != bit
        || quiet0 | quiet1 != 0
        || status != SM0_FLAG
        || count != 1
        || cross != 0
    {
        return Err("force delivery");
    }
    Ok(())
}

#[derive(Default)]
struct Bounds {
    count: u32,
    min: u32,
    max: u32,
    worst: [u32; 3],
}

impl Bounds {
    fn record(&mut self, min: u32, max: u32, endpoints: [u32; 3]) {
        if self.count == 0 || min < self.min {
            self.min = min;
        }
        if self.count == 0 || max > self.max {
            self.max = max;
            self.worst = endpoints;
        }
        self.count += 1;
    }
}

fn sample<const N: u8>(
    pio: &PIO,
    sm: Sm<N>,
    pins: Pins,
    timer: &mut WaitTimer,
    peer: &mut Peer,
    run: u32,
    rx_direction: bool,
) -> Checked<(Sm<N>, Pins)> {
    let code = if rx_direction {
        pio_asm!(
            ".side_set 1",
            ".wrap_target",
            "wait 1 gpio 22 side 0",
            "set x, 21 side 0",
            "mov isr, x side 0",
            "push block side 1",
            "wait 0 gpio 22 side 1",
            ".wrap"
        )
        .program
    } else {
        pio_asm!(
            ".side_set 1",
            ".wrap_target",
            "wait 1 gpio 22 side 0",
            "pull block side 1",
            "mov isr, osr side 1",
            "push block side 1",
            "wait 0 gpio 22 side 1",
            ".wrap"
        )
        .program
    };
    let installed = pio.install(&code).map_err(|_| "install")?;
    let (sm, mut rx, mut tx) = PIOBuilder::from_installed_program(&installed)
        .pins(pins)
        .side_set_pin_base(13)
        .clock_divisor_fixed_point(200, 0)
        .build(sm)
        .map_err(|_| "build")?;
    if !inputs_isolated() {
        return Err("loopback inputs");
    }
    let mut sent = 0;
    if !rx_direction {
        while sent < 8 {
            if !tx.write(sent) {
                return Err("fill");
            }
            sent += 1;
        }
    }
    let sm = sm.start().map_err(|_| "start")?;
    let mut result = Ok(());
    let direction = if rx_direction { "rx" } else { "tx" };
    for line in 0..2 {
        let source = 1 << (N + if rx_direction { 0 } else { 4 });
        let mut latency = Bounds::default();
        let mut timeout = Bounds::default();
        let mut entries = 0;
        let mut fault = [0; 6];
        for index in 0..TRIALS {
            let before = load(&ENTRIES[line]);
            let other = load(&ENTRIES[1 - line]);
            let delivered = load(&DELIVERIES[line]);
            let mut transfer = |end| {
                if rx_direction {
                    let wait = rx.blocking(timer);
                    let mut wait = if line == 0 { wait } else { wait.on_irq1() };
                    wait.read_until(end)
                } else {
                    let wait = tx.blocking(timer);
                    let mut wait = if line == 0 { wait } else { wait.on_irq1() };
                    wait.write_until(sent, end).map(|()| 21)
                }
            };
            if index < TIMEOUTS {
                let start = Timer::new().now();
                let end = Deadline::at(Instant::from_ticks(start.ticks() + u64::from(DEADLINE_US)));
                let outcome = transfer(end);
                let finish = tick();
                let elapsed = finish.wrapping_sub(start.ticks() as u32);
                timeout.record(
                    elapsed.saturating_sub(1),
                    elapsed.saturating_add(1),
                    [start.ticks() as u32, end.instant().ticks() as u32, finish],
                );
                if outcome != Err(WaitError::Timer(Error::Timeout))
                    || (read(INTE[0]) | read(INTE[1])) != 0
                    || load(&ENTRIES[line]) != before
                    || load(&ENTRIES[1 - line]) != other
                    || elapsed < 1_001
                    || elapsed > 1_049
                {
                    fault = [
                        index,
                        elapsed,
                        read(INTE[0]),
                        read(INTE[1]),
                        before,
                        load(&ENTRIES[line]),
                    ];
                    result = Err("timeout retirement");
                    break;
                }
            }
            peer.edge();
            let end = Deadline::at(Instant::from_ticks(
                Timer::new().now().ticks() + u64::from(DEADLINE_US),
            ));
            let value = transfer(end);
            let observation = peer.reply();
            let time = load(&LAST_TICK);
            let count = load(&ENTRIES[line]) - before;
            entries += count;
            if let Ok((mut lo, hi)) = observation {
                if (time.wrapping_sub(lo) as i32) < 0 {
                    lo = STIMULUS.get();
                }
                // Raw INTS and the accepted source establish the lower bound
                // even when the ISR runs before proc0 samples the high level.
                let upper = time.wrapping_sub(lo).saturating_add(1);
                let lower = (time.wrapping_sub(hi) as i32).saturating_sub(1).max(0) as u32;
                latency.record(lower, upper, [lo, hi, time]);
                if hi.wrapping_sub(lo) >= 50 || upper > 50 {
                    fault = [index, lo, hi, time, upper, load(&LAST_STATUS)];
                    result = Err("latency bracket");
                    break;
                }
            }
            if value != Ok(21)
                || observation.is_err()
                || count != 1
                || load(&ENTRIES[1 - line]) != other
                || load(&DELIVERIES[line]) - delivered != 1
                || load(&LAST_SOURCE) != source
                || load(&LAST_STATUS) != source
                || (read(INTE[0]) | read(INTE[1])) != 0
            {
                fault = [
                    index,
                    count,
                    load(&LAST_SOURCE),
                    load(&LAST_STATUS),
                    time,
                    read(INTE[line]),
                ];
                result = Err("incorrect delivery");
                break;
            }
            if rx_direction {
                if rx.read().is_some() {
                    result = Err("extra RX data");
                    break;
                }
            } else {
                let end = Deadline::at(Instant::from_ticks(Timer::new().now().ticks() + 100));
                if rx.polling().read_until(end) != Ok(sent - 8)
                    || rx.read().is_some()
                    || tx.level() != 8
                {
                    result = Err("TX consumed word");
                    break;
                }
                sent += 1;
            }
        }
        log!("PIO_C_FIFO run={run} sm={N} line={line} dir={direction} n={} entries={entries} min={} max={} lo={} hi={} isr={} timeouts={} tmin={} tmax={} start={} deadline={} finish={} result={result:?} fault={fault:?}\n",
            latency.count, latency.min, latency.max, latency.worst[0], latency.worst[1], latency.worst[2],
            timeout.count, timeout.min, timeout.max, timeout.worst[0], timeout.worst[1], timeout.worst[2],
        ).unwrap();
        if result.is_err() {
            break;
        }
    }
    let (sm, pins) = sm.stop().uninit(rx, tx);
    installed.uninstall().map_err(|_| "uninstall")?;
    result.map(|()| (sm, pins))
}

#[entry(proc1)]
fn main1(core: Proc1, mut p: rp1::Proc1Resources) -> ! {
    let watermark = unsafe { stack_watermark::paint() };
    let clocks =
        Clocks::wait_from_proc0(&core, rp1_hal::fugit::MicrosDurationU32::micros(10_000)).unwrap();
    let (pio, sms) = p.pio.split_wait(&clocks).unwrap();
    let mut sm0 = sms.sm0;
    let mut sm1 = sms.sm1;
    let mut sm2 = sms.sm2;
    let mut sm3 = sms.sm3;
    let mut pins =
        Pins::new()
            .input(p.pins.gpio22, INPUT)
            .output(p.pins.gpio13, OUTPUT, InitialOutput::Low);
    install_probe_vectors();
    let mut ping = p.link.open().unwrap().ping;
    let mut peer = Peer(0);
    let mut run = 0;
    log!(
        "PIO_C_READY core=1 clk_hz={} divider=200:0 quantization_us=1\n",
        clocks.sys().raw()
    )
    .unwrap();
    loop {
        let mut triggered = false;
        let _ = ping
            .serve(1, |word| {
                triggered = true;
                word
            })
            .unwrap();
        if !triggered {
            continue;
        }
        run += 1;
        let start = tick();
        let result = (|| {
            force(&mut peer, run, 0)?;
            force(&mut peer, run, 1)?;
            for rx in [true, false] {
                (sm0, pins) = sample(&pio, sm0, pins, &mut p.wait_timer, &mut peer, run, rx)?;
                (sm1, pins) = sample(&pio, sm1, pins, &mut p.wait_timer, &mut peer, run, rx)?;
                (sm2, pins) = sample(&pio, sm2, pins, &mut p.wait_timer, &mut peer, run, rx)?;
                (sm3, pins) = sample(&pio, sm3, pins, &mut p.wait_timer, &mut peer, run, rx)?;
            }
            Ok::<_, &'static str>((sm0, sm1, sm2, sm3, pins))
        })();
        peer.command(PARK);
        let parked = peer.reply().is_ok() && pad(12) & pad(13) & PAD_OD != 0;
        let end = tick();
        let masks = read(INTE[0]) | read(INTE[1]) | read(INTE[0] + INTF) | read(INTE[1] + INTF);
        let elapsed = end.wrapping_sub(start);
        let passed = result.is_ok() && parked && masks == 0 && elapsed < GATE_LIMIT;
        log!("PIO_C_END run={run} pass={} start={start} end={end} masks={masks:#x} parked={} error={:?} stack_free={}\n",
            u8::from(passed), u8::from(parked), result.as_ref().err(), watermark.free_bytes()).unwrap();
        FINISHED.put(run);
        if !passed {
            loop {
                rp1_rt::wait_for_event();
            }
        }
        (sm0, sm1, sm2, sm3, pins) = result.unwrap();
    }
}

// Proc0 writes before launch and after proc1 publishes FINISHED.
fn write_log(args: core::fmt::Arguments<'_>) -> core::fmt::Result {
    struct Uart;
    impl core::fmt::Write for Uart {
        fn write_str(&mut self, text: &str) -> core::fmt::Result {
            const UART0: usize = 0x4003_0000;
            const FLAGS: usize = UART0 + 0x18;
            const TX_FULL: u32 = 1 << 5;
            for byte in text.bytes() {
                if !until(10_000, || read(FLAGS) & TX_FULL == 0) {
                    return Err(core::fmt::Error);
                }
                write(UART0, u32::from(byte));
            }
            Ok(())
        }
    }
    core::fmt::write(&mut Uart, args)
}
