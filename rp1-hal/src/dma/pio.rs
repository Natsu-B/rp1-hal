use super::{
    __stop_until, Block, BusAddr, CLEANUP_US, Channel, EngineClaim, Error, Lease, RawTransfer,
    State, observe, program_block, wait_block,
};
use crate::{
    linux_blocks::Mmio,
    pac::{
        PIO,
        dw_axi_dmac::{MAX_BLOCK_TS, cfg2, ctl_h, ctl_l},
    },
    pio::{Rx, Tx, sm_reg},
    scmi::Registers,
    timer::{self, Deadline, Timer, WaitTimer, read_instant},
};
use core::{marker::PhantomData, mem::ManuallyDrop};

const DREQ_ENABLE: u32 = 1 << 31;

// Only this module can construct an endpoint or attach it to a DMA channel.
#[derive(Clone, Copy, Debug)]
enum Direction {
    ToPio,
    FromPio,
}

#[derive(Clone, Copy, Debug)]
struct Endpoint {
    sm: u8,
    direction: Direction,
}

impl Endpoint {
    fn control(self) -> usize {
        sm_reg(
            self.sm,
            match self.direction {
                Direction::ToPio => 0x18,
                Direction::FromPio => 0x1c,
            },
        )
    }

    fn block(self, memory: BusAddr, len: u32) -> Result<Block, Error> {
        let count = len / 4;
        if !len.is_multiple_of(4)
            || !memory.0.is_multiple_of(4)
            || count == 0
            || count > MAX_BLOCK_TS
        {
            return Err(Error::Invalid);
        }
        let rx = matches!(self.direction, Direction::FromPio);
        let fifo_offset = usize::from(self.sm) * 4 + usize::from(rx) * 0x10;
        let fifo = BusAddr::from_proc(PIO::PTR.wrapping_add(fifo_offset).addr())?;
        // Linux RP1 bindings assign consecutive TX/RX request IDs to each SM.
        let request = 56 + u32::from(self.sm) * 2 + u32::from(rx);
        let (src, dst, fixed, cfg_l, cfg_h) = match self.direction {
            Direction::ToPio => (
                memory,
                fifo,
                ctl_l::DINC_NOINC,
                request << cfg2::DST_PER_SHIFT,
                (cfg2::TT_FC_MEM_TO_PER << cfg2::TT_FC_SHIFT) | cfg2::HS_SEL_SRC,
            ),
            Direction::FromPio => (
                fifo,
                memory,
                ctl_l::SINC_NOINC,
                request << cfg2::SRC_PER_SHIFT,
                (cfg2::TT_FC_PER_TO_MEM << cfg2::TT_FC_SHIFT) | cfg2::HS_SEL_DST,
            ),
        };
        Ok(Block {
            src,
            dst,
            count,
            ctl_l: fixed | (2 << ctl_l::SRC_TR_WIDTH_SHIFT) | (2 << ctl_l::DST_TR_WIDTH_SHIFT),
            // A single 32-bit beat per AXI burst also handles a tail shorter than the FIFO.
            ctl_h: ctl_h::ARLEN_EN | ctl_h::AWLEN_EN,
            cfg_l,
            cfg_h,
        })
    }

    fn open(self, io: &mut impl Registers, capacity: u8) {
        let threshold = match self.direction {
            Direction::ToPio => capacity - 1,
            Direction::FromPio => 1,
        };
        io.write(self.control(), DREQ_ENABLE | u32::from(threshold));
    }

    fn mask(self, io: &mut impl Registers) -> Result<(), Error> {
        let control = self.control();
        io.write(control, 0);
        if io.read(control) & DREQ_ENABLE != 0 {
            return Err(Error::Recovering);
        }
        Ok(())
    }
}

/// DMA request capability issued when the same core declares DMA and PIO.
#[derive(Debug)]
pub struct PioRequests {
    endpoint: Option<Endpoint>,
    _local: PhantomData<*mut ()>,
}

impl PioRequests {
    pub(super) const fn new() -> Self {
        Self {
            endpoint: None,
            _local: PhantomData,
        }
    }
}

pub trait Gate {
    fn check(&self) -> Result<(), Error> {
        Ok(())
    }
    fn mask(&mut self, io: &mut impl Registers) -> Result<(), Error>;
}

impl Gate for () {
    fn mask(&mut self, _: &mut impl Registers) -> Result<(), Error> {
        Ok(())
    }
}

impl Gate for PioRequests {
    fn check(&self) -> Result<(), Error> {
        if self.endpoint.is_some() {
            return Err(Error::Recovering);
        }
        Ok(())
    }

    fn mask(&mut self, io: &mut impl Registers) -> Result<(), Error> {
        if let Some(endpoint) = self.endpoint {
            endpoint.mask(io)?;
            self.endpoint = None;
        }
        Ok(())
    }
}

/// A FIFO reserved for DMA. The original FIFO is available again after proven retirement.
pub struct PioFifo<F> {
    fifo: F,
    quarantined: bool,
}

impl<F> PioFifo<F> {
    /// True if the DMA claim has not been returned.
    pub fn is_quarantined(&self) -> bool {
        self.quarantined
    }

    /// Borrow the CPU FIFO when it is available.
    pub fn try_cpu(&mut self) -> Option<&mut F> {
        (!self.quarantined).then_some(&mut self.fifo)
    }

    /// Return the CPU FIFO so it can be used to release or reconfigure the SM.
    /// A lost DMA claim keeps the FIFO reserved for the firmware epoch.
    pub fn try_into_inner(self) -> Result<F, Self> {
        if self.quarantined {
            Err(self)
        } else {
            Ok(self.fifo)
        }
    }
}

impl<const N: u8, I> Tx<N, I> {
    /// Reserve this TX FIFO for DMA.
    pub fn into_dma(self) -> PioFifo<Self> {
        PioFifo {
            fifo: self,
            quarantined: false,
        }
    }
}

impl<const N: u8, I> Rx<N, I> {
    /// Reserve this RX FIFO for DMA.
    pub fn into_dma(self) -> PioFifo<Self> {
        PioFifo {
            fifo: self,
            quarantined: false,
        }
    }
}

trait Fifo {
    const ENDPOINT: Endpoint;
    fn dma_capacity(&self) -> u8;
}

impl<const SM: u8, I> Fifo for Tx<SM, I> {
    const ENDPOINT: Endpoint = Endpoint {
        sm: SM,
        direction: Direction::ToPio,
    };

    fn dma_capacity(&self) -> u8 {
        self.capacity()
    }
}

impl<const SM: u8, I> Fifo for Rx<SM, I> {
    const ENDPOINT: Endpoint = Endpoint {
        sm: SM,
        direction: Direction::FromPio,
    };

    fn dma_capacity(&self) -> u8 {
        self.capacity()
    }
}

impl<const N: u8> Channel<N, PioRequests> {
    /// Transfer the lease as 32-bit words to a TX FIFO. DMA completion means all words
    /// reached the FIFO, with SM execution and waveform completion tracked separately.
    pub fn try_start_to_pio<'t, const SM: u8, I>(
        &'t mut self,
        src: &'t mut Lease<'_>,
        tx: &'t mut PioFifo<Tx<SM, I>>,
    ) -> Result<PioTransfer<'t>, Error> {
        self.start_fifo(&mut Mmio, src, tx)
    }

    /// Fill the lease with 32-bit words from an RX FIFO. Both endpoints remain borrowed
    /// until reclamation or proven cancellation.
    pub fn try_start_from_pio<'t, const SM: u8, I>(
        &'t mut self,
        rx: &'t mut PioFifo<Rx<SM, I>>,
        dst: &'t mut Lease<'_>,
    ) -> Result<PioTransfer<'t>, Error> {
        self.start_fifo(&mut Mmio, dst, rx)
    }

    fn start_fifo<'t, F: Fifo>(
        &'t mut self,
        io: &mut impl Registers,
        lease: &'t mut Lease<'_>,
        fifo: &'t mut PioFifo<F>,
    ) -> Result<PioTransfer<'t>, Error> {
        let capacity = fifo.fifo.dma_capacity();
        self.start_pio(io, lease, &mut fifo.quarantined, capacity, F::ENDPOINT)
    }

    fn start_pio<'t>(
        &'t mut self,
        io: &mut impl Registers,
        lease: &'t mut Lease<'_>,
        quarantined: &'t mut bool,
        capacity: u8,
        endpoint: Endpoint,
    ) -> Result<PioTransfer<'t>, Error> {
        if *quarantined {
            return Err(Error::Busy);
        }
        if capacity == 0 {
            return Err(Error::Invalid);
        }
        if self.state != State::Idle {
            return Err(Error::Recovering);
        }
        self.gate.check()?;
        let block = endpoint.block(lease.address()?, lease.len())?;
        endpoint.mask(io)?;
        *quarantined = true;
        let mut lease = lease.hand_to_engine();
        self.gate.endpoint = Some(endpoint);
        if let Err(error) = program_block(io, N, &mut self.state, block) {
            self.gate.endpoint = None;
            lease.return_to_cpu();
            *quarantined = false;
            return Err(error);
        }
        endpoint.open(io, capacity);
        Ok(PioTransfer {
            raw: ManuallyDrop::new(RawTransfer::new(N, &mut self.state)),
            gate: &mut self.gate,
            lease,
            fifo_held: quarantined,
        })
    }
}

/// A single block between a shared-SRAM lease and a PIO FIFO.
#[must_use]
pub struct PioTransfer<'t> {
    // Drop must close the PIO gate before it can attempt the channel stop.
    raw: ManuallyDrop<RawTransfer<'t>>,
    gate: &'t mut PioRequests,
    lease: EngineClaim<'t>,
    fifo_held: &'t mut bool,
}

impl PioTransfer<'_> {
    /// Observe completion once. A bus error initiates cancellation.
    pub fn try_complete(&mut self) -> Option<Result<(), Error>> {
        self.poll(&mut Mmio)
    }

    fn poll(&mut self, io: &mut impl Registers) -> Option<Result<(), Error>> {
        let raw = &mut *self.raw;
        if *raw.state != State::Idle {
            let result = observe(io, raw.n, raw.state, &mut raw.terminal_status);
            if let Some(result) = result {
                raw.result.get_or_insert(result);
                if let Err(error @ Error::Bus(_)) = result {
                    let _ = self.cancel(io, error);
                }
            }
        }
        if matches!(*self.raw.state, State::Idle | State::Aborted) && self.gate.mask(io).is_ok() {
            self.raw.result
        } else {
            None
        }
    }

    /// Mask DREQ and make one bounded check for a graceful stop.
    pub fn try_cancel(&mut self) -> Result<(), Error> {
        self.cancel(&mut Mmio, Error::Cancelled)
    }

    fn cancel(&mut self, io: &mut impl Registers, reason: Error) -> Result<(), Error> {
        let end = Deadline::after_ticks(read_instant(io), CLEANUP_US)?;
        self.gate.mask(io)?;
        self.raw.stop(io, reason)?;
        // Include the proof itself in the stop budget.
        end.check(read_instant(io)).map_err(|_| Error::Recovering)
    }

    /// Request cancellation and recheck using the dedicated timer, within 1 ms.
    pub fn cancel_until(&mut self, wait: &mut WaitTimer, end: Deadline) -> Result<(), Error> {
        let limit = Deadline::after_ticks(Timer::new().now(), CLEANUP_US)?;
        __stop_until(wait, end, || self.try_cancel())?;
        limit
            .check(Timer::new().now())
            .map_err(|_| Error::Recovering)
    }

    /// Explicitly poll for completion against one absolute deadline in thread mode.
    pub fn wait_polling(&mut self, end: Deadline) -> Result<(), Error> {
        rp1_rt::interrupt::ThreadGuard::new().map_err(timer::Error::from)?;
        self.wait_accepted(&mut Mmio, end)
    }

    fn wait_accepted(&mut self, io: &mut impl Registers, end: Deadline) -> Result<(), Error> {
        let raw = &mut *self.raw;
        let result = raw.result.unwrap_or_else(|| {
            wait_block(io, end, |io| {
                observe(io, raw.n, raw.state, &mut raw.terminal_status)
            })
        });
        raw.result.get_or_insert(result);
        match result {
            Ok(()) => self.gate.mask(io),
            Err(error) => {
                self.cancel(io, error)?;
                Err(error)
            }
        }
    }

    /// Channel interrupt status captured before the terminal ACK.
    pub fn terminal_status(&self) -> u32 {
        self.raw.terminal_status
    }

    /// Return the endpoints and the transfer outcome after a proven stop.
    /// An unconfirmed stop returns the handle for a later recheck.
    pub fn try_reclaim(mut self) -> Result<Result<(), Error>, Self> {
        match self.poll(&mut Mmio) {
            Some(result) if *self.raw.state == State::Idle => {
                self.return_endpoints();
                Ok(result)
            }
            _ => Err(self),
        }
    }

    fn return_endpoints(&mut self) {
        self.lease.return_to_cpu();
        *self.fifo_held = false;
    }

    fn release_on_drop(&mut self, io: &mut impl Registers) {
        if *self.fifo_held && self.cancel(io, Error::Cancelled).is_ok() {
            self.return_endpoints();
        }
    }
}

impl Drop for PioTransfer<'_> {
    fn drop(&mut self) {
        self.release_on_drop(&mut Mmio);
    }
}

#[cfg(test)]
#[path = "pio_tests.rs"]
mod tests;
