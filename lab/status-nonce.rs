//! Shared by the standalone PWM and SCMI lab images. The link scratch word is lab-only.
use rp1_hal::{platform::InitReport, timer::Timer};

pub fn publish(report: &InitReport) -> u32 {
    let nonce = [
        report.perstn_wait_us,
        report.link_wait_us,
        Timer::new().now() as u32,
    ]
    .into_iter()
    .fold(0x811c_9dc5u32, |hash, sample| {
        (hash ^ sample).wrapping_mul(0x0100_0193)
    })
    .clamp(1, u32::MAX - 1);
    // Link page + 0x30. The SCMI lab uses +0x28 and +0x2c for its trigger and ACK.
    unsafe { (0x2000_c030 as *mut u32).write_volatile(nonce) };
    unsafe { core::arch::asm!("dsb sy", options(nostack, preserves_flags)) };
    nonce
}
