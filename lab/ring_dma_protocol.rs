//! Command 17/18 mode numbers, stage capacities and timing response layout.
pub const CPU: u32 = 0;
pub const DMA: u32 = 1;
pub const AUTO: u32 = 2;
pub const EARLY_HEAD: u32 = 3;
pub const BIT30_ZERO: u32 = 4;
pub const GENERATE_DMA: u32 = 5;
pub const GENERATE_CPU: u32 = 6;
pub const GENERATE_AUTO: u32 = 7;

pub const STAGE_BYTES: usize = 4096;
pub const BATCH_CAPACITY: [usize; 3] = [
    STAGE_BYTES / rp1_proto::shm::slot_size(16),
    STAGE_BYTES / rp1_proto::shm::slot_size(64),
    STAGE_BYTES / rp1_proto::shm::slot_size(256),
];

#[derive(Clone, Copy, Default)]
pub struct Perf {
    pub copy_us: u32,
    pub prepare_us: u32,
    pub publish_us: u32,
    pub blocked_us: u32,
    pub blocks: u32,
    pub two_block_batches: u32,
    pub width8: u32,
    pub width16: u32,
}
impl From<[u32; 8]> for Perf {
    fn from(words: [u32; 8]) -> Self {
        let [
            copy_us,
            prepare_us,
            publish_us,
            blocked_us,
            blocks,
            two_block_batches,
            width8,
            width16,
        ] = words;
        Self {
            copy_us,
            prepare_us,
            publish_us,
            blocked_us,
            blocks,
            two_block_batches,
            width8,
            width16,
        }
    }
}
impl From<Perf> for [u32; 8] {
    fn from(p: Perf) -> Self {
        [
            p.copy_us,
            p.prepare_us,
            p.publish_us,
            p.blocked_us,
            p.blocks,
            p.two_block_batches,
            p.width8,
            p.width16,
        ]
    }
}
