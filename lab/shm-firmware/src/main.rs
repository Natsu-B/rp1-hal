#![no_std]
#![no_main]
use rp1_hal::{gpio::Doorbell, log};
use rp1_proto::{Error, link};
use rp1_rt::{Proc0, Proc1, entry, shared, sync::Spsc};
rp1_rt::bind!();
mod adc_proof;
mod i2c_proof;
mod prof;
mod spi_proof;
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

#[entry(proc0)]
fn main0(mut core: Proc0, p: rp1::Proc0Resources) -> ! {
    let programmed = unsafe { rp1_hal::cold::clocks() }.unwrap();
    let (clocks, mut scmi) = p.clock_authority.init(200_000_000).unwrap();
    rp1::log::init(&mut core).unwrap();
    log!(
        "proc0 clocks programmed={} sys_hz={} uart_hz={} clk_sys ctrl=0x{:x} div={} sel=0x{:x}\n",
        programmed,
        clocks.sys_hz(),
        clocks.uart_hz(),
        reg(0x4001_8014),
        reg(0x4001_8018),
        reg(0x4001_8020)
    )
    .unwrap();
    let lb = scmi.linux_blocks();
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
    // Fields printed one by one: the derived Debug of the whole result costs ~1 KiB of the
    // 48 KiB shared SRAM image.
    match unsafe { rp1_hal::cold::endpoint_and_syscfg(programmed) } {
        Ok(r) => log!(
            "proc0 pcie Ok clocks_programmed={} endpoint_programmed={} perstn_wait_us={} link_wait_us={} monitor2=0x{:x}\n",
            r.clocks_programmed,
            r.endpoint_programmed,
            r.perstn_wait_us,
            r.link_wait_us,
            r.monitor2
        ),
        Err(e) => log!("proc0 pcie Err({:?})\n", e),
    }
    .unwrap();
    log!("proc0 ready shm bss=0x{:x}\n", bss_check(0xb55_0000)).unwrap();
    p.proc1.launch(&mut core).unwrap();
    let i2c = p.I2C1.init(
        p.pins.gpio2,
        p.pins.gpio3,
        &clocks,
        rp1_hal::i2c::Speed::Standard,
    );
    log!("proc0 i2c1 init {:?}\n", i2c.as_ref().map(|_| ())).unwrap();
    let mut i2c = i2c.ok();
    let mut rx = VALUES.consumer(&mut core);
    let mut endpoint = unsafe { rp1_hal::cold::EndpointService::new() };
    let mut attempted = false;
    let mut probed = false;
    loop {
        // 0xadc and 0x5b1 are proc1's (it owns the ADC and SPI0); the rest are I2C requests.
        let request = i2c_proof::pending();
        if request != 0 && !proc1_request(request) {
            i2c_proof::run(i2c.as_mut(), request);
        }
        let (value, perst, scmi_event) =
            idle0(&mut rx, &mut endpoint, &mut scmi, probed, attempted);
        if let Some(v) = value {
            log!("proc0 received {}\n", v).unwrap();
        }
        if let Some(r) = perst {
            log!("proc0 pcie perst {:?}\n", r).unwrap();
        }
        if let Some(e) = scmi_event {
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
        if !probed && link_read(link::SHM_REQUEST) == 2 {
            probed = true;
            let area = unsafe { rp1_hal::shm::configure_window(rp1::SHM_NOTE) }.unwrap();
            let words = area.size as usize / 4;
            let pointer = rp1_proto::SHM_WINDOW as *mut u32;
            let start = now();
            for i in 0..words {
                unsafe { pointer.add(i).write_volatile(i as u32 ^ 0xa596_c33c) };
            }
            rp1_hal::shm::complete_posted_writes();
            let mut errors = 0;
            for i in 0..words {
                if unsafe { pointer.add(i).read_volatile() } != (i as u32 ^ 0xa596_c33c) {
                    errors += 1;
                }
            }
            log!(
                "WINDOW bytes={} words={} errors={} us={} first=0x81000000 last=0x{:x}\n",
                area.size,
                words,
                errors,
                now().wrapping_sub(start),
                rp1_proto::SHM_WINDOW + area.size - 4
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
            let enabled = unsafe { rp1_hal::shm::enable(rp1::SHM_NOTE, &rp1::SHM_NOTE_SHA256) };
            log!("proc0 shm enable {:?}\n", enabled).unwrap();
        }
    }
}

#[entry(proc1)]
fn main1(mut core: Proc1, p: rp1::Proc1Resources) -> ! {
    log!("proc1 ready shm bss=0x{:x}\n", bss_check(0xb55_1000)).unwrap();
    // The ADC is declared on proc1: its Clocks come from proc0's clock_authority.init
    // (published before proc0 launches proc1).
    let clocks = rp1_hal::clocks::Clocks::wait_from_proc0(&core, 10_000);
    log!("proc1 clocks {:?}\n", clocks).unwrap();
    let adc = match clocks {
        Ok(c) => p.ADC.init(&c),
        Err(_) => Err(rp1_hal::adc::Error::Precondition),
    };
    log!("proc1 adc init {:?}\n", adc.as_ref().map(|_| ())).unwrap();
    let mut adc = adc.ok();
    if let Some(adc) = adc.as_mut() {
        let r = adc_proof::prove(adc, "P1BOOT");
        log!("ADC_RESULT boot {:?}\n", r).unwrap();
    }
    let spi = match clocks {
        Ok(c) => p.SPI0.init(
            p.pins.gpio11,
            p.pins.gpio10,
            p.pins.gpio9,
            &c,
            rp1_hal::spi::Config::new(1_000_000, embedded_hal::spi::MODE_0),
        ),
        Err(_) => Err(rp1_hal::spi::Error::Precondition),
    };
    match &spi {
        Ok(_) => log!("proc1 spi0 init Ok\n"),
        Err(e) => log!("proc1 spi0 init Err 0x{:x}\n", 0x100 + *e as u32),
    }
    .unwrap();
    let mut spi = spi
        .ok()
        .map(|bus| rp1_hal::spi::Device::new(bus, p.pins.gpio8));
    if let Some(dev) = spi.as_mut() {
        spi_proof::log_result("boot", &spi_proof::prove(dev, "P1BOOT"));
    }
    VALUES.producer(&mut core).push(42).unwrap();
    let mut doorbell = Doorbell::new(p.pins.gpio27);
    // Linux may ask for an ADC burst before it enables the shm area.
    while link_read(link::SHM_STATUS) != 1 {
        service_doorbell(&mut doorbell);
        serve_proc1(adc.as_mut(), spi.as_mut());
    }
    let mut area = unsafe { rp1_hal::shm::Area::attach(rp1::SHM_NOTE, 2) }.unwrap();
    let mut up = area.ring_tx::<Sample>("up").unwrap();
    let mut down = area.ring_rx::<Sample>("down").unwrap();
    let mut latest = area.ring_tx::<Sample>("latest").unwrap();
    let mut control = area.block::<Control>("control").unwrap();
    let mut bulk = area.block::<Bulk>("bulk").unwrap();
    let mut last = 0;
    let mut seen = 0;
    loop {
        let sequence = wait_command(&mut doorbell, &control, seen);
        serve_proc1(adc.as_mut(), spi.as_mut());
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
                        match tx.push(&sample(i)) {
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
                    match down.pop() {
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
                rp1_hal::shm::complete_posted_writes();
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
                rp1_hal::shm::complete_posted_writes();
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
                    match down.pop() {
                        Ok(Some(s)) if s == sample(n) => n += 1,
                        _ => break,
                    }
                }
                errors = count - n;
            }
            8 => {
                // Isolated push cost: the ring is empty and Linux stays off the link page.
                for i in 0..count {
                    if up.push(&sample(i)).is_err() {
                        errors += 1;
                    }
                }
            }
            10 | 12 => errors = up_batch(&mut up, count, start),
            11 | 13 => errors = down_batch(&mut down, count, start),
            9 => {
                // Lab micro-profile (ends with PROF_DONE); RP1 holds the bulk writer role and uses
                // it as DDR scratch. Its SRAM scratch is a linker-placed `.shared` static.
                let mut writer = unsafe { bulk.begin_write() };
                prof::run(unsafe { writer.as_ptr() });
                writer.commit();
            }
            _ => errors = 1,
        }
        rp1_hal::shm::complete_posted_writes();
        request[4] = 2;
        request[5] = errors;
        request[6] = now().wrapping_sub(start);
        request[7] = if command == 1 { up.dropped() } else { 0 };
        unsafe { control.write(&request) };
        rp1_hal::shm::complete_posted_writes();
        log!(
            "proc1 shm cmd={} count={} errors={} us={}\n",
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
    matches!(request, adc_proof::REQUEST | spi_proof::REQUEST)
}
fn serve_proc1(adc: Option<&mut rp1_hal::adc::Adc>, spi: Option<&mut spi_proof::Dev>) {
    match i2c_proof::pending() {
        adc_proof::REQUEST => adc_proof::run(adc, adc_proof::REQUEST),
        spi_proof::REQUEST => spi_proof::run(spi, spi_proof::REQUEST),
        _ => {}
    }
}
type Perst = Option<Result<rp1_hal::cold::Reconfigured, rp1_hal::cold::ReconfigError>>;
/// proc0 idle loop, from ISRAM (with the ISRAM fast paths of the polls it calls): returns as
/// soon as anything needs the cold path (logging, SCMI request, PERST, lab request).
#[rp1_rt::isram]
fn idle0(
    rx: &mut rp1_rt::sync::Consumer<'_, u32, 8, Proc1, Proc0>,
    endpoint: &mut rp1_hal::cold::EndpointService,
    scmi: &mut rp1_hal::scmi::Service,
    probed: bool,
    attempted: bool,
) -> (Option<u32>, Perst, Option<rp1_hal::scmi::Event>) {
    loop {
        let value = rx.pop();
        let perst = endpoint.poll();
        let event = scmi.poll();
        let request = link_read(link::SHM_REQUEST);
        if value.is_some()
            || perst.is_some()
            || event.is_some()
            || (!probed && request == 2)
            || (!attempted && request == 1)
            || !matches!(
                i2c_proof::pending(),
                0 | adc_proof::REQUEST | spi_proof::REQUEST
            )
        {
            return (value, perst, event);
        }
    }
}
const BATCH: usize = 32;
/// Batched RP1→Linux stream: one `head` store per `push_slice`; the loop itself runs from ISRAM.
#[rp1_rt::isram]
fn up_batch(tx: &mut rp1_hal::shm::RingTx<Sample>, count: u32, start: u32) -> u32 {
    let mut buf = [[0u32; 4]; BATCH];
    let mut i = 0;
    while i < count {
        let k = (count - i).min(BATCH as u32) as usize;
        for (j, b) in buf[..k].iter_mut().enumerate() {
            *b = sample(i + j as u32);
        }
        let mut done = 0;
        while done < k {
            done += tx.push_slice(&buf[done..k]);
            if now().wrapping_sub(start) > 180_000_000 {
                return 1;
            }
        }
        i += k as u32;
    }
    0
}
/// Batched Linux→RP1 stream: one `tail` store per `pop_into`, every message checked.
#[rp1_rt::isram]
fn down_batch(rx: &mut rp1_hal::shm::RingRx<Sample>, count: u32, start: u32) -> u32 {
    let mut buf = [[0u32; 4]; BATCH];
    let mut i = 0;
    let mut errors = 0;
    while i < count {
        let want = (count - i).min(BATCH as u32) as usize;
        match rx.pop_into(&mut buf[..want]) {
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
/// proc1 idle loop, from ISRAM: answer doorbells until the control block's sequence changes
/// or Linux asks for an ADC or SPI burst (returns `seen` then).
/// Idle cost is one DDR load (not a 17-word read), so the doorbell stays responsive.
#[rp1_rt::isram]
fn wait_command(doorbell: &mut Doorbell, control: &rp1_hal::shm::Block<Control>, seen: u32) -> u32 {
    loop {
        service_doorbell(doorbell);
        let sequence = control.sequence();
        if sequence != seen || proc1_request(i2c_proof::pending()) {
            return sequence;
        }
    }
}
#[inline(always)]
fn service_doorbell(doorbell: &mut Doorbell) {
    if doorbell.pending() {
        doorbell.acknowledge();
        unsafe {
            ((rp1_proto::LINK_START as usize + link::DOORBELL_REPLY) as *mut u32)
                .write_volatile(link_read(link::DOORBELL_REQUEST));
        }
        rp1_proto::barrier();
    }
}
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo<'_>) -> ! {
    loop {
        rp1_rt::wait_for_event();
    }
}
