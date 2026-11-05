//! `shm-linux dma`: RP1 DMA into Linux DDR (lab/shm-firmware `dma` feature, command 16).
//!
//! Linux fills the `bulk` block with a canary, hands the writer role to RP1 with command 16 and
//! then checks every word of the result: words [0, 128K) are 512 one-KiB DMA copies through the
//! outbound ATU window, [128K, 256K) the same through the PCIe direct-mapped space; copy `k` of
//! half `h` is the firmware pattern with word 0 = `k | h << 16`. A canary left anywhere means a
//! copy went missing or landed elsewhere. Prints the RP1 timings (`DMA_DDR_LINUX`) and
//! `DMA_DDR_PASS` only when RP1 reported no error and every word matches.
use crate::{Bulk, Control, expired, request};
use rp1_linux::{Firmware, Result};
use rp1_proto::Error;

const CANARY: u32 = 0xdead_beef;
const WORDS: usize = 256;
const fn pattern(i: usize, seed: u32) -> u32 {
    (i as u32).wrapping_mul(0x9e37_79b9) ^ seed
}
fn expected(w: usize, seed: u32) -> u32 {
    let i = w % WORDS;
    if i == 0 {
        let copy = w / WORDS;
        (copy % 512) as u32 | ((copy / 512) as u32) << 16
    } else {
        pattern(i, seed)
    }
}

pub fn main() -> Result<()> {
    let mut fw = Firmware::open()?;
    let mut ctrl = fw.block::<Control>("control")?;
    let mut bulk = fw.block::<Bulk>("bulk")?;
    drop(fw);
    {
        let mut writer = unsafe { bulk.begin_write() }?;
        writer.fill(0, 262144, |_| CANARY)?;
        writer.commit();
    }
    let seed = 0x1d3a_5eed;
    let start = request(&mut ctrl, 16, 16, 0, seed)?;
    let report = loop {
        match ctrl.read() {
            Ok(w) if w[0] == 16 && w[4] == 2 => break w,
            Ok(_) | Err(Error::NotReady) => {}
            Err(e) => return Err(e.into()),
        }
        expired(start)?;
    };
    let r = &report[8..16];
    let mbps = |bytes: f64, us: u32| bytes / f64::from(us.max(1));
    println!(
        "DMA_DDR_LINUX rp1_errors={} small16_us_per={:.3} small512_us_per={:.3} cpu_512k_MBps={:.2} atu_1k_MBps={:.2} direct_1k_MBps={:.2} atu_64k_MBps={:.2} direct_64k_MBps={:.2} rp1_readback_bad={}",
        report[5],
        f64::from(r[0]) / 1000.0,
        f64::from(r[1]) / 1000.0,
        mbps(524288.0, r[2]),
        mbps(524288.0, r[3]),
        mbps(524288.0, r[4]),
        mbps(524288.0, r[6]),
        mbps(524288.0, r[7]),
        r[5],
    );
    let (bad, canary, first) = bulk.read_with(|reader| {
        let (mut bad, mut canary, mut first) = (0u32, 0u32, None);
        let mut words = [0; 1024];
        for at in (0..262144).step_by(words.len()) {
            reader.read_words(at, &mut words)?;
            for (i, v) in words.iter().enumerate() {
                if *v != expected(at + i, seed) {
                    bad += 1;
                    canary += u32::from(*v == CANARY);
                    first.get_or_insert((at + i, *v));
                }
            }
        }
        Ok::<_, Error>((bad, canary, first))
    })??;
    println!("DMA_DDR_VERIFY words=262144 bad={bad} canary_left={canary} first_bad={first:x?}");
    if bad == 0 && report[5] == 0 {
        println!("DMA_DDR_PASS");
        Ok(())
    } else {
        Err("DMA DDR proof failed".into())
    }
}
