#[path = "../../shm-firmware/src/copy_stats.rs"]
mod copy_stats;
fn now() -> u32 {
    0
}

#[test]
fn copier_invocation_is_recorded_with_timing_disabled() {
    use copy_stats::{CopyStats, TimedCopy};
    use rp1_hal::shm::SlotCopy;
    struct Copy;
    // SAFETY: synchronous host copy into the test's exclusive destination.
    unsafe impl SlotCopy for Copy {
        type Error = ();
        unsafe fn copy_to_remote(&mut self, src: &[u8], dst: *mut u8) -> Result<(), ()> {
            unsafe { core::ptr::copy_nonoverlapping(src.as_ptr(), dst, src.len()) };
            Ok(())
        }
    }
    let mut stats = CopyStats::default();
    let mut inner = Copy;
    let mut copy = TimedCopy {
        inner: &mut inner,
        stats: &mut stats,
        timed: false,
    };
    let mut dst = [0; 16];
    unsafe { copy.copy_to_remote(&[42; 16], dst.as_mut_ptr()) }.unwrap();
    assert_eq!(dst, [42; 16]);
    assert_eq!(stats.blocks, 1);
    assert_eq!(
        (stats.us, stats.last_end, stats.width8, stats.width16),
        (0, 0, 0, 0)
    );
}
