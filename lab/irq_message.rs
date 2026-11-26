//! The messages of irq-proof's notification rings (`down0`, `down1`, `events0`, `tick0`), shared
//! by the firmware and shm-linux: `[seq, !seq, generation, crc32(seq, generation)]`.
//!
//! `seq` counts from 0 in each run. `generation` names the run, so a message that a writer of an
//! earlier run left in the ring is told apart from a lost or corrupt one.

pub type Message = [u32; 4];

pub fn message(seq: u32, generation: u32) -> Message {
    [seq, !seq, generation, crc32(seq, generation)]
}

/// Whether `m` is whole: the complement and the checksum agree with `seq` and the generation it
/// carries.
pub fn is_whole(m: &Message) -> bool {
    m[1] == !m[0] && m[3] == crc32(m[0], m[2])
}

/// CRC-32 (IEEE 802.3, reflected, as zlib) of the eight little-endian bytes of `seq` then
/// `generation`. Bitwise: no table in the firmware image.
pub fn crc32(seq: u32, generation: u32) -> u32 {
    let mut crc = !0u32;
    for word in [seq, generation] {
        for byte in word.to_le_bytes() {
            crc ^= u32::from(byte);
            for _ in 0..8 {
                let mask = (crc & 1).wrapping_neg();
                crc = (crc >> 1) ^ (0xedb8_8320 & mask);
            }
        }
    }
    !crc
}
