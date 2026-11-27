//! Hardware proof image for interrupts: handlers on both cores (alarms, IO_BANK0, the wrapped
//! I2C1 and SPI0 lines), the user interrupt mask, the default handler, `log!` from a
//! handler, stack and masking figures. Markers go to UART0; lab/verify-irq.py reads them.
//!
//! The system part (proc0's clocks, PCIe endpoint, SCMI and shm bring-up; proc1's shm command
//! loop) is that of lab/shm-firmware without its peripheral proofs, so the Linux-side SHM,
//! ping and UART1 regressions run in the same boot. Proc1 serves ping in its IO_BANK0 handler.
//!
//! Once shm is up, the notification proofs of [`rings`] run on commands from shm-linux: each
//! core's doorbell handler drains its Linux -> RP1 ring, and proc0 (the `host_irq` owner) sends
//! to Linux from main and from its alarm handler.
#![no_std]
#![no_main]
use rp1_hal::{
    fugit::{ExtU32, RateExtU32},
    log,
};
use rp1_link::{Block, OpenError, RingRx, RingTx, flush_window};
use rp1_proto::{Error, link};
use rp1_rt::{Proc0, Proc1, entry, shared, sync::Spsc};
rp1_rt::bind!();
mod cs_stats;
#[path = "../../irq_message.rs"]
mod irq_message;
mod proc0;
mod proc1;
mod proof;
mod rings;
mod stack;
use proof::{mark, now};
use rings::{Reader, Ticks, command};

/// proc1's figures for the final lines ([`proc1::Report`]).
#[shared]
static REPORT: Spsc<u32, 8> = Spsc::new();
/// The boot's nonce, from proc0 (which makes it) to proc1.
#[shared]
static NONCE: Spsc<u32, 2, Proc0, Proc1> = Spsc::new();

type Sample = [u32; 4];
type Control = [u32; 16];
type Bulk = [u32; 262144];
fn link_read(offset: usize) -> u32 {
    proof::rd(rp1_proto::LINK_START as usize + offset)
}
fn sample(i: u32) -> Sample {
    [i, !i, i ^ 0x5a96_c33c, 0x5250_3148]
}

/// A value that differs from boot to boot: the two waits for the host during the cold bring-up
/// (PERSTn, link) and the time after them, which vary by microseconds. Not guaranteed unique;
/// the checker also separates boots by the position of the BEGIN line.
fn make_nonce(seeds: [u32; 3]) -> u32 {
    // FNV-1a over the words.
    seeds.iter().fold(0x811c_9dc5u32, |hash, seed| {
        (hash ^ seed).wrapping_mul(0x0100_0193)
    })
}

#[entry(proc0)]
fn main0(mut core: Proc0, p: rp1::Proc0Resources) -> ! {
    // SAFETY: proc0, and no NVIC line is enabled yet.
    let stack = unsafe { stack::paint(0) };
    let sys = match rp1::platform::init(&mut core, p.platform) {
        Ok(sys) => sys,
        Err(e) => {
            let _ = log!("PLAT_INIT ok=0 {:?}\n", e);
            panic!("platform init failed");
        }
    };
    let clocks = sys.clocks;
    log!(
        "PLAT_INIT ok=1 {} sys_hz={} uart_hz={}\n",
        sys.report,
        clocks.sys().raw(),
        clocks.uart().raw()
    )
    .unwrap();
    let (perstn, link) = (sys.report.perstn_wait_us, sys.report.link_wait_us);
    let nonce = make_nonce([perstn, link, now()]);
    proof::set_nonce(nonce);
    proof::publish_nonce();
    // `sweep=0`: this image raises none of the lines that only lab/irq-sweep proved.
    mark!("IRQ_PROOF_BEGIN sweep=0");
    NONCE.producer(&mut core).push(nonce).unwrap();
    p.proc1.launch(&mut core).unwrap();
    let i2c = p
        .i2c1
        .init(&clocks, rp1_hal::i2c::Speed::Standard)
        .map_err(|e| *e.error());
    log!("p0 i2c1 init {:?}\n", i2c.as_ref().map(|_| ())).unwrap();

    let mut system = System {
        pcie: sys.pcie,
        probed: false,
        attempted: false,
    };
    // The system services keep running while proc0's items wait for their interrupts.
    let doorbell = p.doorbell;
    proc0::run(p.TIMER_ALARM0, i2c.ok(), doorbell, p.irq, &mut || {
        system.handle()
    });

    let mut rx = REPORT.consumer(&mut core);
    let mut report = [0; proc1::Report::WORDS];
    let mut words = 0;
    let mut init = Some(p.link);
    let mut commands: Option<Commands0> = None;
    loop {
        let word = idle0(&mut rx, &system, commands.as_ref());
        system.handle();
        // proc0 enables the window itself, so its endpoints open right after that.
        if commands.is_none() && system.attempted {
            commands = open0(&mut init, doorbell);
        }
        if let Some(commands) = commands.as_mut() {
            commands.serve();
        }
        if let Some(word) = word
            && words < report.len()
        {
            report[words] = word;
            words += 1;
            if words == report.len() {
                proc0::finish(stack.free_bytes(), &proc1::Report::from_words(report));
            }
        }
    }
}

/// The two shm requests Linux makes through the link page.
struct System {
    pcie: rp1_hal::platform::Pcie,
    probed: bool,
    attempted: bool,
}
impl System {
    /// The slow part of one round: serve a shm request.
    fn handle(&mut self) {
        let request = link_read(link::SHM_REQUEST);
        if !self.probed && request == 2 {
            self.probed = true;
            probe_window(&mut self.pcie);
        }
        if !self.attempted && request == 1 {
            self.attempted = true;
            let enabled = rp1::link::try_enable(&mut self.pcie);
            log!("p0 shm enable {:?}\n", enabled).unwrap();
        }
    }
}

/// shm-linux's whole-window check before it enables the area: pattern every word, read back.
fn probe_window(pcie: &mut rp1_hal::platform::Pcie) {
    let size = rp1::link::__map_for_probe(pcie).unwrap();
    let words = size as usize / 4;
    let pointer = rp1_proto::SHM_WINDOW as *mut u32;
    for i in 0..words {
        unsafe { pointer.add(i).write_volatile(i as u32 ^ 0xa596_c33c) };
    }
    // This probe mapped the reservation before Ready.
    unsafe { rp1_link::__flush_probe_window() };
    let mut errors = 0;
    for i in 0..words {
        if unsafe { pointer.add(i).read_volatile() } != (i as u32 ^ 0xa596_c33c) {
            errors += 1;
        }
    }
    log!("WINDOW bytes={} errors={}\n", size, errors).unwrap();
    proof::wr(
        rp1_proto::LINK_START as usize + 36,
        if errors == 0 { 1 } else { 2 },
    );
    rp1_proto::barrier();
}

/// proc0's idle loop once its items are done, from ISRAM: returns as soon as anything needs
/// the slow path.
#[rp1_rt::isram]
fn idle0(
    rx: &mut rp1_rt::sync::Consumer<'_, u32, 8, Proc1, Proc0>,
    system: &System,
    commands: Option<&Commands0>,
) -> Option<u32> {
    loop {
        let word = rx.pop();
        let request = link_read(link::SHM_REQUEST);
        let command = commands.is_some_and(|c| c.control.sequence() != c.seen);
        if word.is_some()
            || command
            || (!system.probed && request == 2)
            || (!system.attempted && request == 1)
        {
            return word;
        }
    }
}

/// Opens proc0's endpoints and hands `down0` and `tick0` to its handlers. Messages Linux sent
/// before the handler had the reader rang a doorbell that found nothing to drain, so raise it
/// once more.
fn open0(
    init: &mut Option<rp1::link::Proc0LinkInit>,
    doorbell: rp1::Doorbell0,
) -> Option<Commands0> {
    match init.take()?.try_open() {
        Ok(rp1::link::Proc0Link {
            down0,
            events0,
            tick0,
            control0,
        }) => {
            proc0::DOWN0.put(Reader::new(down0)).ok().unwrap();
            proc0::TICK0.put(Ticks::new(tick0)).ok().unwrap();
            doorbell.raise();
            Some(Commands0 {
                control: control0,
                events: events0,
                seen: 0,
                last: 0,
            })
        }
        Err(OpenError::NotReady(pending)) => {
            *init = Some(pending);
            None
        }
        Err(OpenError::Reset) => panic!("SHM window lost"),
        Err(OpenError::Attach(error)) => panic!("SHM attach: {error:?}"),
    }
}

/// proc0's commands on `control0` (the [`rings::command`] numbers), answered in the block as
/// proc1 answers its own: word 4 = 2, word 5 = errors, word 6 = µs.
struct Commands0 {
    control: Block<Control>,
    events: rp1::link::Events0,
    /// The block's sequence and the request id last served.
    seen: u32,
    last: u32,
}
impl Commands0 {
    fn serve(&mut self) {
        let sequence = self.control.sequence();
        if sequence == self.seen {
            return;
        }
        let Ok(mut request) = self.control.read() else {
            return;
        };
        self.seen = sequence;
        if request[0] == self.last || request[0] == 0 {
            return;
        }
        self.last = request[0];
        let (command, count, seed) = (request[1], request[2], request[3]);
        let start = now();
        let errors = match command {
            command::EVENTS => {
                rings::send_events(&mut self.events, count, seed, request[8], request[9])
            }
            command::TICKS => {
                proc0::start_ticks(count, request[8], seed);
                0
            }
            command::TICK_REPORT => {
                let mut errors = 1;
                proc0::TICK0.with(|ticks| {
                    request[8..11].copy_from_slice(&ticks.words());
                    errors = 0;
                });
                request[11] = proc0::tick_late_max_us();
                errors
            }
            _ => rings::down_command(&proc0::DOWN0, &mut request).unwrap_or(1),
        };
        request[4] = 2;
        request[5] = errors;
        request[6] = now().wrapping_sub(start);
        request[7] = 0;
        unsafe { self.control.write(&request) };
        flush_window();
        log!(
            "p0 shm cmd={} count={} errors={} us={}\n",
            command,
            count,
            errors,
            request[6]
        )
        .unwrap();
    }
}

#[entry(proc1)]
fn main1(mut core: Proc1, p: rp1::Proc1Resources) -> ! {
    // SAFETY: proc1, and no NVIC line is enabled yet.
    let stack = unsafe { stack::paint(1) };
    // proc0 pushed the nonce before it launched this core.
    proof::set_nonce(NONCE.consumer(&mut core).pop().unwrap_or(0));
    let clocks = rp1_hal::clocks::Clocks::wait_from_proc0(&core, 10.millis());
    let spi = match clocks {
        Ok(c) => p
            .spi0
            .init(
                &c,
                rp1_hal::spi::Config::new(1.MHz(), embedded_hal::spi::MODE_0),
            )
            .map_err(|e| *e.error()),
        Err(_) => Err(rp1_hal::spi::Error::Precondition),
    };
    log!("p1 spi0 init {}\n", spi.is_ok()).unwrap();
    let doorbell = p.doorbell;
    proc1::run(proc1::Parts {
        alarm1: p.TIMER_ALARM1,
        alarm2: p.TIMER_ALARM2,
        spi: spi.ok(),
        doorbell,
        irq: p.irq,
    });

    let mut report = REPORT.producer(&mut core);
    let mut init = p.link;
    let rp1::link::Proc1Link {
        mut up,
        mut down,
        mut latest,
        mut control,
        mut bulk,
        down1,
        ping,
    } = loop {
        match init.try_open() {
            Ok(endpoints) => break endpoints,
            Err(OpenError::NotReady(pending)) => init = pending,
            Err(OpenError::Reset) => panic!("SHM window lost"),
            Err(OpenError::Attach(error)) => panic!("SHM attach: {error:?}"),
        }
    };
    proc1::DOWN1.put(Reader::new(down1)).ok().unwrap();
    proc1::PING.put(ping).ok().unwrap();
    // As on proc0: a doorbell before the handler had the reader found nothing to drain.
    doorbell.raise();
    let mut last = 0;
    let mut seen = 0;
    // The command loop of lab/shm-firmware for shm-linux's default suite (commands 1..8, 10..13),
    // plus the `down1` commands of [`rings`].
    loop {
        let sequence = wait_command(&control, seen);
        report_when_due(&mut report, &stack);
        if sequence == seen {
            continue;
        }
        let Ok(mut request) = control.read() else {
            continue;
        };
        seen = sequence;
        if request[0] == last || request[0] == 0 {
            continue;
        }
        last = request[0];
        let (command, count, seed) = (request[1], request[2], request[3]);
        let start = now();
        let mut errors = 0u32;
        match command {
            1 | 6 => {
                let tx = if command == 1 { &mut up } else { &mut latest };
                for i in 0..count {
                    loop {
                        match tx.send(&sample(i)) {
                            Ok(()) => break,
                            Err(Error::Full) => {}
                            Err(_) => {
                                errors += 1;
                                break;
                            }
                        }
                        if now().wrapping_sub(start) > 180_000_000 {
                            errors += 1;
                            break;
                        }
                    }
                }
            }
            2 => {
                let mut i = 0;
                while i < count {
                    match down.recv() {
                        Ok(Some(s)) => {
                            if s != sample(i) {
                                errors += 1;
                            }
                            i += 1;
                        }
                        Ok(None) => {}
                        Err(_) => {
                            errors += 1;
                            break;
                        }
                    }
                    if now().wrapping_sub(start) > 180_000_000 {
                        errors += 1;
                        break;
                    }
                }
            }
            3 => {
                // Handshake transfers the sole-writer role to RP1 until the response is published.
                let mut writer = unsafe { bulk.begin_write() };
                writer.fill(0, 262144, |i| i as u32 ^ seed).unwrap();
                writer.commit();
                flush_window();
            }
            4 => {
                let checked = bulk.read_with(|reader| {
                    let mut errors = 0;
                    for i in 0..262144 {
                        if reader.read_word(i).unwrap() != (i as u32 ^ seed) {
                            errors += 1;
                        }
                    }
                    errors
                });
                errors = checked.unwrap_or(1);
            }
            5 => {
                let pointer = unsafe { bulk.raw().as_ptr() };
                let mut sum = 0u32;
                let t = now();
                sum = sum.wrapping_add(unsafe { repeat_read(pointer, 4096) });
                request[8] = now().wrapping_sub(t);
                let t = now();
                sum = sum.wrapping_add(unsafe { scan_read(pointer, 262144) });
                request[9] = now().wrapping_sub(t);
                core::hint::black_box(sum);
                let mut writer = unsafe { bulk.begin_write() };
                let pointer = unsafe { writer.as_ptr() };
                let t = now();
                unsafe { scan_write(pointer, 262144, seed) };
                writer.commit();
                flush_window();
                request[10] = now().wrapping_sub(t);
                let local = [seed; 4];
                let t = now();
                core::hint::black_box(unsafe { repeat_read(local.as_ptr(), 4096) });
                request[11] = now().wrapping_sub(t);
            }
            7 => {
                // Isolated pop cost: Linux filled `down` beforehand and stays off the link page.
                let mut n = 0;
                while n < count {
                    match down.recv() {
                        Ok(Some(s)) if s == sample(n) => n += 1,
                        _ => break,
                    }
                }
                errors = count - n;
            }
            8 => {
                // Isolated push cost: the ring is empty and Linux stays off the link page.
                for i in 0..count {
                    if up.send(&sample(i)).is_err() {
                        errors += 1;
                    }
                }
            }
            10 | 12 => errors = up_batch(&mut up, count, start),
            11 | 13 => errors = down_batch(&mut down, count, start),
            _ => errors = rings::down_command(&proc1::DOWN1, &mut request).unwrap_or(1),
        }
        flush_window();
        request[4] = 2;
        request[5] = errors;
        request[6] = now().wrapping_sub(start);
        request[7] = if command == 1 { up.dropped() } else { 0 };
        unsafe { control.write(&request) };
        flush_window();
        // The stack figure in the final lines is taken before these commands; this one tells
        // whether the default stack also holds them.
        log!(
            "p1 shm cmd={} count={} errors={} us={} stack_free={}\n",
            command,
            count,
            errors,
            request[6],
            stack.free_bytes()
        )
        .unwrap();
    }
}

/// Send proc1's report after the first 1000 ping requests.
fn report_when_due(
    report: &mut rp1_rt::sync::Producer<'_, u32, 8, Proc1, Proc0>,
    stack: &stack::Watermark,
) {
    if proc1::report_due() {
        proc1::report(report, stack.free_bytes());
    }
}

/// proc1 idle loop, from ISRAM: until the control block's sequence changes or the report is
/// due (returns `seen` then). Idle cost is one DDR load.
#[rp1_rt::isram]
fn wait_command(control: &Block<Control>, seen: u32) -> u32 {
    loop {
        let sequence = control.sequence();
        if sequence != seen || proc1::report_due() {
            return sequence;
        }
    }
}

const BATCH: usize = 32;
/// Batched RP1→Linux stream: one `head` store per `send_slice`; the loop itself runs from ISRAM.
#[rp1_rt::isram]
fn up_batch(tx: &mut RingTx<Sample>, count: u32, start: u32) -> u32 {
    let mut buf = [[0u32; 4]; BATCH];
    let mut i = 0;
    while i < count {
        let k = (count - i).min(BATCH as u32) as usize;
        for (j, b) in buf[..k].iter_mut().enumerate() {
            *b = sample(i + j as u32);
        }
        let mut done = 0;
        while done < k {
            let Ok(n) = tx.send_slice(&buf[done..k]) else {
                return 1;
            };
            done += n;
            if now().wrapping_sub(start) > 180_000_000 {
                return 1;
            }
        }
        i += k as u32;
    }
    0
}
/// Batched Linux→RP1 stream: one `tail` store per `recv_into`, every message checked.
#[rp1_rt::isram]
fn down_batch(rx: &mut RingRx<Sample>, count: u32, start: u32) -> u32 {
    let mut buf = [[0u32; 4]; BATCH];
    let mut i = 0;
    let mut errors = 0;
    while i < count {
        let want = (count - i).min(BATCH as u32) as usize;
        match rx.recv_into(&mut buf[..want]) {
            Ok(n) => {
                for b in &buf[..n] {
                    let s = sample(i);
                    // Word compares: an array `!=` would call memcmp in shared SRAM.
                    if (b[0] ^ s[0]) | (b[1] ^ s[1]) | (b[2] ^ s[2]) | (b[3] ^ s[3]) != 0 {
                        errors += 1;
                    }
                    i += 1;
                }
            }
            Err(_) => return errors + 1,
        }
        if now().wrapping_sub(start) > 180_000_000 {
            return errors + 1;
        }
    }
    errors
}
// ISRAM removes shared-SRAM instruction fetches from the DDR measurement.
#[rp1_rt::isram]
unsafe fn repeat_read(pointer: *const u32, count: usize) -> u32 {
    let mut sum = 0u32;
    for _ in 0..count {
        sum = sum.wrapping_add(unsafe { pointer.read_volatile() });
    }
    sum
}
#[rp1_rt::isram]
unsafe fn scan_read(pointer: *const u32, count: usize) -> u32 {
    let mut sum = 0u32;
    for i in 0..count {
        sum = sum.wrapping_add(unsafe { pointer.add(i).read_volatile() });
    }
    sum
}
#[rp1_rt::isram]
unsafe fn scan_write(pointer: *mut u32, count: usize, seed: u32) {
    for i in 0..count {
        unsafe { pointer.add(i).write_volatile(i as u32 ^ seed) };
    }
}
