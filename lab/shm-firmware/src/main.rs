#![no_std]
#![no_main]
#[cfg(feature = "spi")]
use rp1_hal::fugit::RateExtU32;
use rp1_hal::log;
use rp1_link::{Block, OpenError, Progress, flush_window};
#[cfg(not(feature = "ring-dma"))]
use rp1_link::{RingRx, RingTx};
use rp1_proto::{Error, link};
use rp1_rt::{Proc0, Proc1, entry, shared, sync::Spsc};
rp1_rt::bind!();
#[cfg(feature = "adc")]
mod adc_proof;
#[cfg(feature = "ring-dma")]
mod copy_stats;
#[cfg(any(feature = "dma", feature = "ring-dma"))]
mod dma_proof;
#[cfg(feature = "i2c")]
mod i2c_proof;
#[cfg(feature = "sram-profile")]
mod prof;
#[cfg(feature = "ring-dma")]
mod ring_dma;
#[cfg(feature = "ring-dma")]
#[path = "../../ring_dma_protocol.rs"]
mod ring_dma_protocol;
#[cfg(feature = "spi")]
mod spi_proof;
#[cfg(feature = "ring-dma")]
mod stack_watermark;
#[shared]
static VALUES: Spsc<u32, 8> = Spsc::new();
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
type Sample = [u32; 4];
type Control = [u32; 16];
type Bulk = [u32; 262144];
type Wide64 = [u32; 16];
type Wide256 = [u32; 64];
fn now() -> u32 {
    unsafe { (0x400a_c028 as *const u32).read_volatile() }
}
fn link_read(offset: usize) -> u32 {
    unsafe { ((rp1_proto::LINK_START as usize + offset) as *const u32).read_volatile() }
}
fn reg(address: usize) -> u32 {
    unsafe { (address as *const u32).read_volatile() }
}
fn sample(i: u32) -> Sample {
    [i, !i, i ^ 0x5a96_c33c, 0x5250_3148]
}

const ADC_REQUEST: u32 = 0xadc;
const SPI_REQUEST: u32 = 0x5b1;
const PROOF_REQUEST: usize = rp1_proto::LINK_START as usize + 0x28;
#[inline(always)]
fn proof_pending() -> u32 {
    unsafe { (PROOF_REQUEST as *const u32).read_volatile() }
}
fn proof_reply(request: u32, success: bool) {
    unsafe {
        (PROOF_REQUEST as *mut u32).write_volatile(0);
        ((PROOF_REQUEST + 4) as *mut u32).write_volatile(if success { request } else { u32::MAX });
    }
    rp1_proto::barrier();
}

#[entry(proc0)]
fn main0(mut core: Proc0, p: rp1::Proc0Resources) -> ! {
    let sys = match rp1::platform::init(&mut core, p.platform) {
        Ok(sys) => sys,
        // Without the error: its derived Debug costs about 1 KiB of the 48 KiB image.
        Err(_) => {
            let _ = log!("PLAT_INIT ok=0\n");
            panic!("platform init failed");
        }
    };
    log!("PLAT_INIT ok=1 {}\n", sys.report).unwrap();
    let clocks = sys.clocks;
    log!(
        "proc0 clocks sys_hz={} uart_hz={} clk_sys ctrl=0x{:x} div={} sel=0x{:x}\n",
        clocks.sys().raw(),
        clocks.uart().raw(),
        reg(0x4001_8014),
        reg(0x4001_8018),
        reg(0x4001_8020)
    )
    .unwrap();
    let lb = sys.report.linux_blocks;
    log!(
        "proc0 linux_blocks released={:x},{:x},{:x} pending={:x},{:x},{:x}\n",
        lb.released[0],
        lb.released[1],
        lb.released[2],
        lb.pending[0],
        lb.pending[1],
        lb.pending[2]
    )
    .unwrap();
    log!("p0 ready bss=0x{:x}\n", bss_check(0xb55_0000)).unwrap();
    // Lets the verifier catch a profile firmware run with a default client (and vice versa).
    #[cfg(feature = "sram-profile")]
    log!("p0 sram-profile\n").unwrap();
    p.proc1.launch(&mut core).unwrap();
    #[cfg(feature = "i2c")]
    let mut i2c = {
        let i2c = p
            .i2c1
            .init(&clocks, rp1_hal::i2c::Speed::Standard)
            .map_err(|e| *e.error());
        log!("p0 i2c1 init {:?}\n", i2c.as_ref().map(|_| ())).unwrap();
        i2c.ok()
    };
    #[cfg(feature = "i2c")]
    let mut wait_timer = p.wait_timer;
    let mut rx = VALUES.consumer(&mut core);
    let mut attempted = false;
    let mut probed = false;
    let mut pcie = sys.pcie;
    loop {
        // 0xadc and 0x5b1 are proc1's (it owns the ADC and SPI0); the rest are I2C requests.
        let request = proof_pending();
        if request != 0 && !proc1_request(request) {
            #[cfg(feature = "i2c")]
            i2c_proof::run(i2c.as_mut(), request, &mut wait_timer);
            #[cfg(not(feature = "i2c"))]
            proof_reply(request, false);
        }
        if let Some(v) = idle0(&mut rx, probed, attempted) {
            log!("p0 received {}\n", v).unwrap();
        }
        if !probed && link_read(link::SHM_REQUEST) == 2 {
            probed = true;
            let size = rp1::link::__map_for_probe(&mut pcie).unwrap();
            let words = size as usize / 4;
            let pointer = rp1_proto::SHM_WINDOW as *mut u32;
            let start = now();
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
            log!(
                "WINDOW bytes={} words={} errors={} us={} first=0x81000000 last=0x{:x}\n",
                size,
                words,
                errors,
                now().wrapping_sub(start),
                rp1_proto::SHM_WINDOW + size - 4
            )
            .unwrap();
            unsafe {
                ((rp1_proto::LINK_START + 36) as *mut u32).write_volatile(if errors == 0 {
                    1
                } else {
                    2
                });
            }
            rp1_proto::barrier();
        }
        if !attempted && link_read(link::SHM_REQUEST) == 1 {
            attempted = true;
            let enabled = rp1::link::try_enable(&mut pcie);
            log!("p0 shm enable {:?}\n", enabled).unwrap();
        }
    }
}

#[entry(proc1)]
fn main1(mut core: Proc1, p: rp1::Proc1Resources) -> ! {
    #[cfg(feature = "ring-dma")]
    // SAFETY: proc1 owns its private DSRAM; interrupts are still masked at entry.
    let stack = unsafe { stack_watermark::paint() };
    log!("p1 ready bss=0x{:x}\n", bss_check(0xb55_1000)).unwrap();
    // The ADC is declared on proc1: its Clocks come from proc0's platform init
    // (published before proc0 launches proc1).
    #[cfg(any(
        feature = "adc",
        feature = "spi",
        feature = "dma",
        feature = "ring-dma"
    ))]
    let clocks = {
        use rp1_hal::fugit::ExtU32;
        rp1_hal::clocks::Clocks::wait_from_proc0(&core, 10.millis())
    };
    #[cfg(any(
        feature = "adc",
        feature = "spi",
        feature = "dma",
        feature = "ring-dma"
    ))]
    log!("p1 clocks {:?}\n", clocks).unwrap();
    #[cfg(any(feature = "adc", feature = "spi", feature = "dma"))]
    let mut wait_timer = p.wait_timer;
    #[cfg(feature = "adc")]
    let mut adc = {
        let adc = match clocks {
            Ok(c) => p.adc.init(&c).map_err(|e| *e.error()),
            Err(_) => Err(rp1_hal::adc::Error::Precondition),
        };
        log!("p1 adc init {:?}\n", adc.as_ref().map(|_| ())).unwrap();
        let mut adc = adc.ok();
        if let Some(adc) = adc.as_mut() {
            let r = adc_proof::prove(adc, &mut wait_timer, "P1BOOT");
            log!("ADC_RESULT boot {:?}\n", r).unwrap();
        }
        adc
    };
    #[cfg(feature = "spi")]
    let mut spi = {
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
        match &spi {
            Ok(_) => log!("p1 spi0 init Ok\n"),
            Err(e) => log!("p1 spi0 init Err 0x{:x}\n", 0x100 + *e as u32),
        }
        .unwrap();
        spi.ok().map(|bus| (bus, p.pins.gpio8))
    };
    #[cfg(feature = "spi")]
    if let Some(result) = spi_proof::prove(&mut spi, &mut wait_timer, "P1BOOT") {
        spi_proof::log_result("boot", &result);
    }
    #[cfg(any(feature = "dma", feature = "ring-dma"))]
    #[allow(unused_mut)]
    let mut dma = {
        let dma = clocks.ok().and_then(|c| dma_proof::init(p.dma, &c));
        #[cfg(feature = "dma")]
        let dma = dma.and_then(|mut dma| dma_proof::sram(&mut dma).ok().map(|()| dma));
        dma
    };
    #[cfg(feature = "dma")]
    let mut bulk_quarantined = false;
    #[cfg(feature = "ring-dma")]
    // SAFETY: proc1 starts once and exclusively owns this separate DMA scratch reservation.
    let ring_scratch = unsafe { dma_proof::ring_scratch() };
    VALUES.producer(&mut core).push(42).unwrap();
    let doorbell = p.doorbell;
    // Linux may ask for an ADC burst before it enables the shm area.
    let mut init = p.link;
    let endpoints = loop {
        match init.try_open() {
            Ok(endpoints) => break endpoints,
            Err(OpenError::NotReady(pending)) => init = pending,
            Err(OpenError::Reset) => panic!("SHM window lost"),
            Err(OpenError::Attach(error)) => panic!("SHM attach: {error:?}"),
        }
        serve_proc1(
            #[cfg(feature = "adc")]
            adc.as_mut(),
            #[cfg(feature = "spi")]
            &mut spi,
            #[cfg(any(feature = "adc", feature = "spi"))]
            &mut wait_timer,
        );
    };
    let rp1::link::Proc1Link {
        mut up,
        mut down,
        mut latest,
        mut events,
        mut control,
        mut bulk,
        mut ping,
        ..
    } = endpoints;
    #[cfg(feature = "ring-dma")]
    let (mut wide64, mut wide256) = (endpoints.wide64, endpoints.wide256);
    #[cfg(feature = "ring-dma")]
    let mut last_ring_perf = ring_dma_protocol::Perf::default();
    let mut last = 0;
    let mut seen = 0;
    loop {
        let sequence = wait_command(doorbell, &mut ping, &control, seen);
        serve_proc1(
            #[cfg(feature = "adc")]
            adc.as_mut(),
            #[cfg(feature = "spi")]
            &mut spi,
            #[cfg(any(feature = "adc", feature = "spi"))]
            &mut wait_timer,
        );
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
        let mut extra = 0u32;
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
            #[cfg(feature = "dma")]
            3 | 4 | 5 | 9 | 16 if bulk_quarantined => errors = 1,
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
            #[cfg(not(feature = "ring-dma"))]
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
            #[cfg(not(feature = "ring-dma"))]
            10 | 12 => errors = up_batch(&mut up, count, start),
            // `events` stream: seed = batch << 16 | gap_us; 14 notifies after every batch
            // (`flush`), 15 does not (polling baseline on the same ring).
            14 | 15 => {
                (errors, extra) = events_stream(&mut events, command == 14, count, seed, start);
            }
            #[cfg(not(feature = "ring-dma"))]
            11 | 13 => errors = down_batch(&mut down, count, start),
            #[cfg(feature = "dma")]
            16 => {
                // RP1 holds the bulk writer role until all diagnostic writes are verified.
                match dma.as_mut() {
                    Some(d) => {
                        let mut writer = unsafe { bulk.begin_write() };
                        let data = unsafe { writer.as_ptr() };
                        let phys =
                            rp1_proto::SHM_PHYS + (data as u64 - u64::from(rp1_proto::SHM_WINDOW));
                        let (e, out) = dma_proof::ddr(d, &mut wait_timer, data, phys, seed);
                        if e == 0 {
                            writer.commit();
                        } else {
                            // Leave the sequence odd and fence every future use of this mapping.
                            core::mem::forget(writer);
                            bulk_quarantined = true;
                            dma = None;
                        }
                        errors = e;
                        request[8..16].copy_from_slice(&out);
                    }
                    None => errors = 1,
                }
            }
            // RP1 -> Linux ring stream written by the CPU or by DMA (ring_dma); request words
            // 8..14 = ring, mode, notify, batch, gap_us, timed; results replace words 8..15.
            #[cfg(feature = "ring-dma")]
            17 => {
                let p = ring_dma::Params {
                    mode: request[9],
                    notify: request[10] != 0,
                    batch: request[11],
                    gap_us: request[12],
                    timed: request[13] != 0,
                };
                use ring_dma::{Run, sample_into, stream, wide};
                use ring_dma_protocol::BATCH_CAPACITY;
                let run = Run {
                    count,
                    p: &p,
                    start,
                };
                let o = match (dma.as_mut(), request[8]) {
                    (Some(d), 0) => stream::<4, { BATCH_CAPACITY[0] }>(
                        &mut events,
                        d,
                        &ring_scratch,
                        sample_into,
                        run,
                    ),
                    (Some(d), 1) => stream::<16, { BATCH_CAPACITY[1] }>(
                        &mut wide64,
                        d,
                        &ring_scratch,
                        wide::<16>,
                        run,
                    ),
                    (Some(d), 2) => stream::<64, { BATCH_CAPACITY[2] }>(
                        &mut wide256,
                        d,
                        &ring_scratch,
                        wide::<64>,
                        run,
                    ),
                    _ => ring_dma::Out {
                        errors: 1,
                        ..Default::default()
                    },
                };
                errors = o.errors;
                if errors != 0 {
                    dma = None;
                }
                last_ring_perf = o.perf;
                extra = o.pulses;
                request[8..15].copy_from_slice(&[
                    o.ctl_l,
                    o.wraps,
                    o.dma_batches,
                    o.cpu_batches,
                    o.gen_us,
                    o.push_us,
                    o.notify_us,
                ]);
            }
            // Fetch command 17's optional detailed timing, after its response was consumed.
            #[cfg(feature = "ring-dma")]
            18 => request[8..16].copy_from_slice(&<[u32; 8]>::from(last_ring_perf)),
            #[cfg(feature = "sram-profile")]
            9 => {
                // Lab micro-profile (ends with PROF_DONE); RP1 holds the bulk writer role and uses
                // it as DDR scratch. Its SRAM scratch is a linker-placed `.shared` static.
                let mut writer = unsafe { bulk.begin_write() };
                prof::run(unsafe { writer.as_ptr() });
                writer.commit();
            }
            _ => errors = 1,
        }
        flush_window();
        request[4] = 2;
        request[5] = errors;
        request[6] = now().wrapping_sub(start);
        request[7] = if command == 1 { up.dropped() } else { extra };
        unsafe { control.write(&request) };
        flush_window();
        #[cfg(feature = "ring-dma")]
        if command == 17 {
            let free = stack.free_bytes();
            log!("RINGDMA_STACK run_id={} free_bytes={}\n", request[0], free).unwrap();
        }
        log!(
            "p1 shm cmd={} count={} errors={} us={}\n",
            command,
            count,
            errors,
            request[6]
        )
        .unwrap();
    }
}
/// Lab requests proc1 serves (proc0 leaves them alone).
#[inline(always)]
fn proc1_request(request: u32) -> bool {
    matches!(request, ADC_REQUEST | SPI_REQUEST)
}
fn serve_proc1(
    #[cfg(feature = "adc")] adc: Option<&mut rp1::Adc>,
    #[cfg(feature = "spi")] spi: &mut Option<spi_proof::Parts>,
    #[cfg(any(feature = "adc", feature = "spi"))] wait: &mut rp1_hal::timer::WaitTimer,
) {
    match proof_pending() {
        #[cfg(feature = "adc")]
        ADC_REQUEST => adc_proof::run(adc, wait, ADC_REQUEST),
        #[cfg(feature = "spi")]
        SPI_REQUEST => spi_proof::run(spi, wait, SPI_REQUEST),
        request if proc1_request(request) => proof_reply(request, false),
        _ => {}
    }
}
/// proc0 idle loop, from ISRAM: returns as soon as anything needs the cold path (logging, lab
/// request).
#[rp1_rt::isram]
fn idle0(
    rx: &mut rp1_rt::sync::Consumer<'_, u32, 8, Proc1, Proc0>,
    probed: bool,
    attempted: bool,
) -> Option<u32> {
    loop {
        let value = rx.pop();
        let request = link_read(link::SHM_REQUEST);
        if value.is_some() || (!probed && request == 2) || (!attempted && request == 1) || {
            let request = proof_pending();
            request != 0 && !proc1_request(request)
        } {
            return value;
        }
    }
}
const BATCH: usize = 32;
/// Batched RP1→Linux stream: one `head` store per `send_slice`; the loop itself runs from ISRAM.
#[cfg(not(feature = "ring-dma"))]
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
/// `events` stream (commands 14/15): batches of `seed >> 16` (1..=32, 0 = 32) messages,
/// `seed & 0xffff` µs apart, each batch published with one `push_slice` (and, with `notify`,
/// followed by `flush`, which pulses if the reader sleeps). Returns (errors, pulses).
// The ring-dma image needs this ISRAM for the command 17 loop; there it runs from shared SRAM.
#[cfg_attr(not(feature = "ring-dma"), rp1_rt::isram)]
fn events_stream(
    tx: &mut rp1::link::Events,
    notify: bool,
    count: u32,
    seed: u32,
    start: u32,
) -> (u32, u32) {
    let batch = match seed >> 16 {
        0 => BATCH,
        b => (b as usize).min(BATCH),
    };
    let gap = seed & 0xffff;
    let mut buf = [[0u32; 4]; BATCH];
    let mut i = 0;
    let mut pulses = 0;
    while i < count {
        let k = (count - i).min(batch as u32) as usize;
        for (j, b) in buf[..k].iter_mut().enumerate() {
            *b = sample(i + j as u32);
        }
        let mut done = 0;
        while done < k {
            let Ok(n) = tx.push_slice(&buf[done..k]) else {
                return (1, pulses);
            };
            done += n;
            // Also while the ring is full: the reader may have armed meanwhile.
            if notify && tx.flush() == Ok(true) {
                pulses += 1;
            }
            if now().wrapping_sub(start) > 180_000_000 {
                return (1, pulses);
            }
        }
        i += k as u32;
        let t = now();
        while now().wrapping_sub(t) < gap {}
    }
    (0, pulses)
}
/// Batched Linux→RP1 stream: one `tail` store per `recv_into`, every message checked.
#[cfg(not(feature = "ring-dma"))]
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
// ISRAM removes shared-SRAM instruction fetches from the DDR measurement; identical baseline loop.
#[cfg(not(feature = "ring-dma"))]
#[rp1_rt::isram]
unsafe fn repeat_read(pointer: *const u32, count: usize) -> u32 {
    let mut sum = 0u32;
    for _ in 0..count {
        sum = sum.wrapping_add(unsafe { pointer.read_volatile() });
    }
    sum
}
#[cfg(not(feature = "ring-dma"))]
#[rp1_rt::isram]
unsafe fn scan_read(pointer: *const u32, count: usize) -> u32 {
    let mut sum = 0u32;
    for i in 0..count {
        sum = sum.wrapping_add(unsafe { pointer.add(i).read_volatile() });
    }
    sum
}
#[cfg(not(feature = "ring-dma"))]
#[rp1_rt::isram]
unsafe fn scan_write(pointer: *mut u32, count: usize, seed: u32) {
    for i in 0..count {
        unsafe { pointer.add(i).write_volatile(i as u32 ^ seed) };
    }
}
/// Wait for ping, a control-block command, or a peripheral proof request.
#[rp1_rt::isram]
fn wait_command(
    doorbell: rp1::Doorbell1,
    ping: &mut rp1::link::Ping,
    control: &Block<Control>,
    seen: u32,
) -> u32 {
    loop {
        if doorbell.is_pending() {
            doorbell.ack();
            service_ping(ping);
        }
        let sequence = control.sequence();
        if sequence != seen || proc1_request(proof_pending()) {
            return sequence;
        }
    }
}

#[rp1_rt::cold]
fn service_ping(ping: &mut rp1::link::Ping) {
    loop {
        match ping.serve(8, |value| value) {
            Ok(Progress::More) => {}
            Ok(Progress::Idle) => break,
            Err(_) => panic!("ping serve failed"),
        }
    }
}
