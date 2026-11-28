use super::{IRQ0_INTE, IRQ1_INTE, Rx, Tx, WaitError};
use crate::{
    driver_irq::{self, CoreState, Limit, Managed, SlotLine, Source},
    linux_blocks::Mmio,
    scmi::Registers,
    timer::{Deadline, WaitTimer},
};
use rp1_rt::interrupt::{Binding, BlockIrq, Slot};

const IRQ0: u8 = crate::pac::Interrupt::PIO_0 as u8;
const PIO: u16 = rp1_proto::BlockId::Pio as u16;
const INTS_OFFSET: usize = 8;

/// IRQ capability carried by generated SM tokens. Bit N in LINES enables line N.
pub struct Irq<const CORE: u8, const LINES: u8> {
    _private: (),
}

#[derive(Clone, Copy)]
enum IrqLine {
    Irq0,
    Irq1,
}

use IrqLine::{Irq0, Irq1};

pub(crate) struct FifoSource {
    inte: usize,
    armed: u32,
}

impl FifoSource {
    pub(crate) const fn new(inte: usize) -> Self {
        Self { inte, armed: 0 }
    }
}

impl<I: Registers> Source<I> for FifoSource {
    type Error = WaitError;

    fn is_armed(&self) -> bool {
        self.armed != 0
    }

    fn arm(&mut self, io: &mut I, mask: u32) -> Result<bool, WaitError> {
        debug_assert_eq!(self.armed, 0);
        self.armed = mask;
        let enabled = io.read(self.inte);
        io.write(self.inte, enabled | mask);
        Ok(true)
    }

    fn service(&mut self, io: &mut I) -> u32 {
        let pending = io.read(self.inte + INTS_OFFSET) & self.armed;
        if pending != 0 {
            let enabled = io.read(self.inte);
            io.write_relaxed(self.inte, enabled & !pending);
            self.armed &= !pending;
        }
        pending
    }

    fn retire(&mut self, io: &mut I) {
        if self.armed != 0 {
            let enabled = io.read(self.inte);
            io.write(self.inte, enabled & !self.armed);
            self.armed = 0;
        }
    }
}

impl From<driver_irq::Error> for WaitError {
    fn from(error: driver_irq::Error) -> Self {
        match error {
            driver_irq::Error::Busy => Self::Busy,
            driver_irq::Error::IrqUnavailable => Self::Timer(crate::timer::Error::IrqUnavailable),
        }
    }
}

type State<const CORE: u8> = CoreState<FifoSource, SlotLine, CORE>;

macro_rules! states {
    ($name:ident, $core:literal) => {
        fn $name(line: IrqLine) -> &'static State<$core> {
            static ZERO: State<$core> = Slot::new(Managed::new(FifoSource::new(IRQ0_INTE)));
            static ONE: State<$core> = Slot::new(Managed::new(FifoSource::new(IRQ1_INTE)));
            match line {
                Irq0 => &ZERO,
                Irq1 => &ONE,
            }
        }
    };
}
states!(proc0, 0);
states!(proc1, 1);

const fn route<const LINE: u8>() -> IrqLine {
    const { assert!(LINE < 2) };
    if LINE == 0 { Irq0 } else { Irq1 }
}

/// # Safety
/// The declaration must bind LINE once on its PIO-owning CORE and install the
/// matching handler. The route must be validated before applications receive a binding.
#[doc(hidden)]
pub unsafe fn __bind_irq<const LINE: u8, const IRQ: u8, const CORE: u8>(
    binding: Binding<BlockIrq<PIO>, IRQ, CORE>,
) {
    const { assert!(IRQ == IRQ0 + LINE && CORE < 2) };
    let line = SlotLine::new(binding);
    if CORE == 0 {
        driver_irq::bind(proc0(route::<LINE>()), line);
    } else {
        driver_irq::bind(proc1(route::<LINE>()), line);
    }
}

/// # Safety
/// Only the matching PIO handler on the declaring core may call this with
/// LINE, after the corresponding NVIC line is bound.
#[doc(hidden)]
#[inline(always)]
pub unsafe fn __on_interrupt<const LINE: u8, const CORE: u8>() -> u32 {
    const { assert!(CORE < 2) };
    if CORE == 0 {
        driver_irq::on_interrupt(proc0(route::<LINE>()), &mut Mmio)
    } else {
        driver_irq::on_interrupt(proc1(route::<LINE>()), &mut Mmio)
    }
}

/// An IRQ-backed FIFO wait using the core's dedicated alarm.
pub struct Blocking<'a, F> {
    fifo: &'a mut F,
    timer: &'a mut WaitTimer,
    line: IrqLine,
}

macro_rules! blocking {
    ($fifo:ident) => {
        impl<const N: u8, const CORE: u8, const LINES: u8> $fifo<N, Irq<CORE, LINES>> {
            /// Borrow this FIFO and the alarm for an IRQ-backed wait.
            pub fn blocking<'a>(&'a mut self, timer: &'a mut WaitTimer) -> Blocking<'a, Self> {
                Blocking {
                    fifo: self,
                    timer,
                    line: if LINES & 1 != 0 { Irq0 } else { Irq1 },
                }
            }
        }

        impl<const N: u8, const CORE: u8> Blocking<'_, $fifo<N, Irq<CORE, 3>>> {
            /// Route this wait to PIO_1. The default uses PIO_0.
            pub fn on_irq1(mut self) -> Self {
                self.line = Irq1;
                self
            }
        }
    };
}
blocking!(Rx);
blocking!(Tx);

impl<const N: u8, const CORE: u8, const LINES: u8> Blocking<'_, Rx<N, Irq<CORE, LINES>>> {
    /// Read one word by an absolute deadline.
    pub fn read_until(&mut self, end: impl Into<Deadline>) -> Result<u32, WaitError> {
        wait::<CORE, _>(
            self.fifo.capacity(),
            self.line,
            self.timer,
            end.into(),
            1 << N,
            || self.fifo.try_read(),
        )
    }
}

impl<const N: u8, const CORE: u8, const LINES: u8> Blocking<'_, Tx<N, Irq<CORE, LINES>>> {
    /// Write one word by an absolute deadline.
    pub fn write_until(&mut self, word: u32, end: impl Into<Deadline>) -> Result<(), WaitError> {
        wait::<CORE, _>(
            self.fifo.capacity(),
            self.line,
            self.timer,
            end.into(),
            1 << (4 + N),
            || self.fifo.try_write(word).then_some(()),
        )
    }
}

#[inline]
fn wait<const CORE: u8, T>(
    capacity: u8,
    line: IrqLine,
    timer: &mut WaitTimer,
    end: Deadline,
    source: u32,
    mut probe: impl FnMut() -> Option<T>,
) -> Result<T, WaitError> {
    if capacity == 0 {
        return Err(WaitError::Disabled);
    }
    let probe = |_: &mut Mmio| probe().map(Ok);
    let limit = Limit::Absolute(end);
    if CORE == 0 {
        driver_irq::wait_for(proc0(line), &mut Mmio, timer, limit, source, probe)
    } else {
        driver_irq::wait_for(proc1(line), &mut Mmio, timer, limit, source, probe)
    }
}
