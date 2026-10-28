#![no_std]
#![no_main]
use rp1_hal::{gpio::Doorbell, log};
use rp1_proto::{Error, link};
use rp1_rt::{Proc0, Proc1, entry, shared, sync::Spsc};
rp1_rt::bind!();
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
fn sample(i: u32) -> Sample {
    [i, !i, i ^ 0x5a96_c33c, 0x5250_3148]
}

#[entry(proc0)]
fn main0(mut core: Proc0, p: rp1::Proc0Resources) -> ! {
    let programmed = unsafe { rp1_hal::cold::clocks() }.unwrap();
    let (_clocks, mut scmi) = p.clock_authority.adopt(200_000_000).unwrap();
    rp1::log::init(&mut core).unwrap();
    log!("proc0 clocks programmed={}\n", programmed).unwrap();
    log!("proc0 pcie {:?}\n", unsafe {
        rp1_hal::cold::endpoint_and_syscfg(programmed)
    })
    .unwrap();
    log!("proc0 ready shm bss=0x{:x}\n", bss_check(0xb55_0000)).unwrap();
    p.proc1.launch(&mut core).unwrap();
    let mut rx = VALUES.consumer(&mut core);
    let mut endpoint = unsafe { rp1_hal::cold::EndpointService::new() };
    let mut attempted = false;
    let mut probed = false;
    loop {
        if let Some(v) = rx.pop() {
            log!("proc0 received {}\n", v).unwrap();
        }
        if let Some(r) = endpoint.poll() {
            log!("proc0 pcie perst {:?}\n", r).unwrap();
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
    VALUES.producer(&mut core).push(42).unwrap();
    let mut doorbell = Doorbell::new(p.pins.gpio27);
    while link_read(link::SHM_STATUS) != 1 {
        service_doorbell(&mut doorbell);
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
        service_doorbell(&mut doorbell);
        // Idle cost is one DDR load (not a 17-word read), so the doorbell stays responsive.
        let sequence = control.sequence();
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
