use super::{BASE, FLEVEL, FLEVEL2, FSTAT};
use crate::{read, write_relaxed};
use core::marker::PhantomData;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Buffers {
    #[default]
    RxTx,
    OnlyTx,
    OnlyRx,
}

impl Buffers {
    pub(super) const fn bits(self) -> u32 {
        match self {
            Self::RxTx => 0,
            Self::OnlyTx => 1 << 30,
            Self::OnlyRx => 1 << 31,
        }
    }

    const fn capacity(self, rx: bool) -> u8 {
        match (self, rx) {
            (Self::RxTx, _) => 8,
            (Self::OnlyTx, false) | (Self::OnlyRx, true) => 16,
            _ => 0,
        }
    }
}

pub struct Tx<const N: u8> {
    buffers: Buffers,
    local: PhantomData<*mut ()>,
}

pub struct Rx<const N: u8> {
    buffers: Buffers,
    local: PhantomData<*mut ()>,
}

fn level(read: impl Fn(usize) -> u32, sm: u8, rx: bool, capacity: u8) -> u8 {
    if capacity == 0 {
        return 0;
    }
    let shift = 8 * sm + 4 * u8::from(rx);
    let low = ((read(FLEVEL) >> shift) & 15) as u8;
    // FLEVEL saturates at 15. If the FIFO moved between reads, 15 or 16 was
    // still observed during this call.
    if capacity == 16 && low == 15 && read(FLEVEL2) & (1 << shift) != 0 {
        16
    } else {
        low
    }
}

macro_rules! fifo {
    ($type:ident, $rx:expr, $full:expr, $empty:expr) => {
        impl<const N: u8> $type<N> {
            pub(super) fn new(buffers: Buffers) -> Self {
                Self {
                    buffers,
                    local: PhantomData,
                }
            }

            /// Capacity in 32-bit words. A FIFO disabled by JOIN has zero capacity.
            pub const fn capacity(&self) -> u8 {
                self.buffers.capacity($rx)
            }

            /// A snapshot of the number of words available.
            pub fn level(&self) -> u8 {
                level(read, N, $rx, self.capacity())
            }

            pub fn is_empty(&self) -> bool {
                self.capacity() == 0 || read(FSTAT) & (1 << (N + $empty)) != 0
            }

            pub fn is_full(&self) -> bool {
                self.capacity() == 0 || read(FSTAT) & (1 << (N + $full)) != 0
            }
        }
    };
}
fifo!(Tx, false, 16, 24);
fifo!(Rx, true, 0, 8);

impl<const N: u8> Tx<N> {
    /// Write a word immediately, returning false if the FIFO cannot accept it.
    #[inline]
    pub fn write(&mut self, word: u32) -> bool {
        if self.is_full() {
            return false;
        }
        write_relaxed(BASE + 0x14 + usize::from(N) * 4, word);
        true
    }
}

impl<const N: u8> Rx<N> {
    /// Pop a word immediately, returning None if no word is available.
    #[inline]
    pub fn read(&mut self) -> Option<u32> {
        (!self.is_empty()).then(|| read(BASE + 0x24 + usize::from(N) * 4))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joined_level_distinguishes_15_and_16_in_every_lane() {
        for n in 0..4 {
            for rx in [false, true] {
                let shift = 8 * n + 4 * u8::from(rx);
                assert_eq!(level(|_| unreachable!(), n, rx, 0), 0);
                assert_eq!(level(|_| 8 << shift, n, rx, 8), 8);
                assert_eq!(
                    level(|a| if a == FLEVEL { 15 << shift } else { 0 }, n, rx, 16),
                    15
                );
                assert_eq!(
                    level(
                        |a| if a == FLEVEL { 15 << shift } else { 1 << shift },
                        n,
                        rx,
                        16
                    ),
                    16
                );
            }
        }
    }
}
