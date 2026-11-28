use super::{BASE, FLEVEL, FLEVEL2, FSTAT};
use crate::timer::{self, Deadline, Instant, Timer};
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

pub struct Tx<const N: u8, I = ()> {
    buffers: Buffers,
    local: PhantomData<(*mut (), I)>,
}

pub struct Rx<const N: u8, I = ()> {
    buffers: Buffers,
    local: PhantomData<(*mut (), I)>,
}

/// An error from a deadline-bounded FIFO operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum WaitError {
    /// The FIFO has zero capacity in the current JOIN mode.
    Disabled,
    /// Another operation is using the IRQ wait slot.
    Busy,
    Timer(timer::Error),
}

impl From<timer::Error> for WaitError {
    fn from(error: timer::Error) -> Self {
        Self::Timer(error)
    }
}

/// An explicitly selected polling FIFO wait.
pub struct Polling<'a, F>(&'a mut F);

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
        impl<const N: u8, I> $type<N, I> {
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

            /// Borrow this FIFO for deadline-bounded polling.
            pub fn polling(&mut self) -> Polling<'_, Self> {
                Polling(self)
            }
        }
    };
}
fifo!(Tx, false, 16, 24);
fifo!(Rx, true, 0, 8);

impl<const N: u8, I> Tx<N, I> {
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

impl<const N: u8, I> Rx<N, I> {
    /// Pop a word immediately, returning None if no word is available.
    #[inline]
    pub fn read(&mut self) -> Option<u32> {
        (!self.is_empty()).then(|| read(BASE + 0x24 + usize::from(N) * 4))
    }
}

impl<const N: u8, I> Polling<'_, Tx<N, I>> {
    /// Write one word by an absolute deadline.
    pub fn write_until(&mut self, word: u32, end: Deadline) -> Result<(), WaitError> {
        rp1_rt::interrupt::ThreadGuard::new().map_err(timer::Error::from)?;
        poll(
            self.0.capacity(),
            end,
            || Timer::new().now(),
            || self.0.write(word).then_some(()),
        )
    }
}

impl<const N: u8, I> Polling<'_, Rx<N, I>> {
    /// Read one word by an absolute deadline.
    pub fn read_until(&mut self, end: Deadline) -> Result<u32, WaitError> {
        rp1_rt::interrupt::ThreadGuard::new().map_err(timer::Error::from)?;
        poll(
            self.0.capacity(),
            end,
            || Timer::new().now(),
            || self.0.read(),
        )
    }
}

fn poll<T>(
    capacity: u8,
    end: Deadline,
    mut now: impl FnMut() -> Instant,
    mut probe: impl FnMut() -> Option<T>,
) -> Result<T, WaitError> {
    if capacity == 0 {
        return Err(WaitError::Disabled);
    }
    end.check(now())?;
    loop {
        if let Some(value) = probe() {
            return Ok(value);
        }
        if now() >= end.instant() {
            return probe().ok_or(timer::Error::Timeout.into());
        }
        core::hint::spin_loop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::cell::Cell;

    #[test]
    fn polling_checks_disabled_before_expiry_and_probes_at_deadline() {
        let now = Cell::new(2);
        let tick = || Instant::from_ticks(now.get());
        let end = Deadline::at(Instant::from_ticks(2));
        assert_eq!(
            poll::<()>(8, end, tick, || panic!("probed an expired wait")),
            Err(WaitError::Timer(timer::Error::Timeout))
        );
        assert_eq!(
            poll::<()>(0, end, tick, || panic!("probed a disabled FIFO")),
            Err(WaitError::Disabled)
        );
        now.set(1);
        let reads = Cell::new(0);
        let result = poll(8, end, tick, || {
            reads.set(reads.get() + 1);
            now.set(2);
            (reads.get() == 2).then_some(42)
        });
        assert_eq!(result, Ok(42));
        assert_eq!(reads.get(), 2);
    }

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
