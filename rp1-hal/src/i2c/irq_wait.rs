//! I2C interrupts and synchronous controller adapter.

use super::*;
use crate::{
    driver_irq::{self, Limit, Source},
    timer::WaitTimer,
};

/// Capability carried by I2C parts when the declaration reserves the IRQ for the HAL.
pub struct Irq<const N: u8, const CORE: u8>(PhantomData<core::cell::Cell<()>>);

driver_irq::device_irq!(I2cSource, Io,
    0 => I2c0, I2C0;
    1 => I2c1, I2C1;
    2 => I2c2, I2C2;
    3 => I2c3, I2C3;
);

/// # Safety
/// Called only by the generated I2C handler on CORE for instance N.
/// Returns the serviced device bits for the parity fixture.
#[doc(hidden)]
#[inline(always)]
pub unsafe fn __on_interrupt<const N: u8, const CORE: u8>() -> u32 {
    // A pending NVIC entry can outlive controller init recovery.
    if read(RESETS_DONE0) & reset_bit(N) == 0 {
        return 0;
    }
    service_irq::<N, CORE, _>(&mut Mmio::new(N))
}

impl From<driver_irq::Error> for Error {
    fn from(error: driver_irq::Error) -> Self {
        match error {
            driver_irq::Error::Busy => Self::Busy,
            driver_irq::Error::IrqUnavailable => Self::IrqUnavailable,
        }
    }
}

struct I2cSource<const N: u8> {
    mask: u32,
}

impl<const N: u8> I2cSource<N> {
    const fn new() -> Self {
        Self { mask: 0 }
    }
}

impl<const N: u8, I: Io> Source<I> for I2cSource<N> {
    type Error = Error;

    fn is_armed(&self) -> bool {
        self.mask != 0
    }

    fn arm(&mut self, io: &mut I, mask: u32) -> Result<bool, Error> {
        if self.mask != 0 {
            return Ok(false);
        }
        if io.read(MASK) != 0 {
            // Silencing a stray level is safe on the exclusively managed line.
            mask_interrupts(io);
            return Err(Error::IrqUnavailable);
        }
        self.mask = mask | ERRORS;
        enable_interrupt(io, self.mask);
        Ok(true)
    }

    fn service(&mut self, io: &mut I) -> u32 {
        let asserted = io.read(INTR_STAT);
        let pending = asserted & self.mask;
        if pending != 0 {
            self.mask = 0;
        }
        if asserted != 0 {
            // Silence stray levels too, preserving an armed wait on a stray-only entry.
            io.write(MASK, self.mask);
            io.read(INTR_STAT);
        }
        // STOP and TX_ABRT are latched. Leave them and TX_ABRT_SOURCE for the
        // thread. The ISR never touches the FIFOs, user buffers or the timer.
        pending
    }

    fn retire(&mut self, io: &mut I) {
        mask_interrupts(io);
        io.read(INTR_STAT);
        self.mask = 0;
    }
}

fn probe(io: &mut impl Io, mask: u32) -> Option<Result<(), Error>> {
    let raw = io.read(RAW);
    match fault(io, raw) {
        Ok(()) => (raw & mask != 0).then_some(Ok(())),
        Err(error) => Some(Err(error)),
    }
}

/// Seven-bit embedded-hal transactions using the I2C IRQ, wait alarm and WFE.
pub struct Blocking<'a, P: CompletePins<N>, const CORE: u8, const N: u8 = 1> {
    bus: &'a mut I2c<N, P, Controller, Irq<N, CORE>>,
    timer: &'a mut WaitTimer,
    config: WaitConfig,
}

impl<P: CompletePins<N>, const CORE: u8, const N: u8> I2c<N, P, Controller, Irq<N, CORE>> {
    /// Borrow the bus for synchronous transactions in unmasked thread mode.
    pub fn blocking<'a>(&'a mut self, timer: &'a mut WaitTimer) -> Blocking<'a, P, CORE, N> {
        Blocking {
            bus: self,
            timer,
            config: WaitConfig::new(),
        }
    }
}

impl<const SDA: u8, const SCL: u8, const CORE: u8, const N: u8>
    I2c<N, PinTuple<SDA, SCL>, Controller, Irq<N, CORE>>
where
    Pin<SDA>: Sda<N>,
    Pin<SCL>: Scl<N>,
{
    /// Wait for disable and idle, then return parts with the IRQ capability.
    /// On failure the error keeps the controller and its pins.
    pub fn free_until(
        mut self,
        timer: &mut WaitTimer,
        end: Deadline,
    ) -> Result<Parts<N, PinTuple<SDA, SCL>, Irq<N, CORE>>, crate::FreeError<Self, Error>> {
        match self.blocking(timer).cancel_until(end) {
            Ok(()) => self.try_free(),
            Err(error) => Err(crate::FreeError {
                driver: self,
                error,
            }),
        }
    }
}

impl<P: CompletePins<N>, const CORE: u8, const N: u8> Blocking<'_, P, CORE, N> {
    pub fn with_wait_config(mut self, config: WaitConfig) -> Self {
        self.config = config;
        self
    }

    /// Set speed on a disabled controller in unmasked thread mode.
    pub fn set_speed(&mut self, speed: Speed) -> Result<(), Error> {
        self.bus.state.ready()?;
        self.check_context()?;
        let mut io = self.bus.io();
        let clocks = self.bus.clocks;
        if !io.sys_current_strict(&clocks) {
            return Err(Error::Precondition);
        }
        let counts = timing(clocks.sys().raw(), speed)?;
        if !stopped(&mut io) {
            return Err(Error::Busy);
        }
        configure(&mut io, counts);
        Ok(())
    }

    /// Sleep for at least this duration using the same dedicated alarm.
    pub fn delay(&mut self, duration: Duration) -> Result<(), Error> {
        Ok(self.timer.delay(duration)?)
    }

    /// Request cancellation and wait until the controller is disabled and idle.
    /// Call `I2c::try_recover` before submitting another transaction.
    pub fn cancel_until(&mut self, end: Deadline) -> Result<(), Error> {
        self.timer.thread_context()?;
        end.check(Timer::new().now())?;
        let mut io = self.bus.io();
        match self.bus.cancel_with(&mut io) {
            Err(Error::Busy) => wait_stopped(&mut io, self.timer, end),
            result => result,
        }
    }

    /// Run all operations against one absolute deadline. Empty buffers are
    /// skipped. Adjacent buffers in the same direction continue, direction
    /// changes issue repeated START and the final command carries STOP.
    /// Disable and idle confirmation may take up to 1 ms beyond the deadline.
    /// On error a prefix may already have reached the wire.
    pub fn transaction_until(
        &mut self,
        address: Address7,
        ops: &mut [Operation<'_>],
        end: Deadline,
    ) -> Result<(), Error> {
        self.run(address.0, ops, end)
    }

    fn check_context(&mut self) -> Result<(), Error> {
        self.timer.thread_context()?;
        Ok(check_irq::<N, CORE>()?)
    }

    fn run(&mut self, address: u8, ops: &mut [Operation<'_>], end: Deadline) -> Result<(), Error> {
        self.check_context()?;
        let mut io = self.bus.io();
        admit(&mut io, &self.bus.clocks, self.bus.state, address, end)?;
        let cursor = scan_ops(&mut io, ops, end)?;
        if cursor.last.is_none() {
            return Ok(());
        }
        end.check(io.instant())?;
        begin(&mut io, address)?;
        self.bus.state = State::Poisoned;
        let result = transfer(
            &mut io,
            ops,
            cursor,
            self.bus.rx_depth,
            end,
            |io, mask, threshold, end| {
                // Wait for a batch without asking for more data than has been queued.
                if mask == RX_FULL {
                    io.write(RX_TL, threshold);
                }
                wait_for_irq::<N, CORE, _, _>(io, self.timer, Limit::Absolute(end), mask, |io| {
                    probe(io, mask)
                })
            },
        );
        io.write(RX_TL, 0);
        request_stop(&mut io, result.is_err());
        let cleanup = Deadline::after(Duration::from_ticks(CLEANUP_US))
            .map_err(Error::from)
            .and_then(|end| wait_stopped(&mut io, self.timer, end));
        finish(&mut io, &mut self.bus.state, result, cleanup)
    }
}

impl<P: CompletePins<N>, const CORE: u8, const N: u8> embedded_hal::i2c::ErrorType
    for Blocking<'_, P, CORE, N>
{
    type Error = Error;
}

impl<P: CompletePins<N>, const CORE: u8, const N: u8> embedded_hal::i2c::I2c
    for Blocking<'_, P, CORE, N>
{
    fn transaction(&mut self, address: u8, ops: &mut [Operation<'_>]) -> Result<(), Error> {
        self.run(address, ops, default_deadline(self.config)?)
    }
}

// The block has no IRQ for disable completion. Only after STOP or abort/disable
// is requested, check on the alarm at bounded intervals until idle or deadline.
fn wait_stopped(io: &mut impl Io, timer: &mut WaitTimer, end: Deadline) -> Result<(), Error> {
    while !stopped(io) {
        let now = io.instant();
        end.check(now)?;
        let next = now + Duration::from_ticks(10);
        timer.delay_until(next.min(end.instant()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Model {
        raw: u32,
        mask: u32,
        wrap: u32,
        abort: u32,
        during_arm: u32,
    }

    impl Io for Model {
        fn read(&mut self, offset: usize) -> u32 {
            match offset {
                MASK => self.mask,
                RAW => self.raw,
                INTR_STAT => self.raw & self.mask,
                IRQ_WRAP => self.wrap,
                ABORT_SOURCE => self.abort,
                _ => panic!("unexpected IRQ register read"),
            }
        }

        fn write(&mut self, offset: usize, value: u32) {
            match offset {
                MASK => {
                    self.mask = value;
                    if value != 0 {
                        self.raw |= core::mem::take(&mut self.during_arm);
                    }
                }
                IRQ_WRAP => self.wrap = value,
                _ => panic!("unexpected IRQ register write"),
            }
        }

        fn now(&mut self) -> u32 {
            unreachable!()
        }
        fn instant(&mut self) -> Instant {
            unreachable!()
        }
        fn sys_current(&mut self, _: &Clocks) -> bool {
            unreachable!()
        }
        fn sys_current_strict(&mut self, _: &Clocks) -> bool {
            unreachable!()
        }
        fn release_reset(&mut self, _: u32) -> Result<(), Error> {
            unreachable!()
        }
    }

    #[test]
    fn instance_routes_match_register_base_and_clock() {
        use crate::pac;

        assert_eq!(
            IRQ_ROUTES,
            &[(0, 32, 7), (1, 33, 8), (2, 34, 9), (3, 35, 10)]
        );
        for (n, address, clock) in [
            (0, pac::I2C0::PTR, pac::I2C0::CLOCK_DEPENDENCY),
            (1, pac::I2C1::PTR, pac::I2C1::CLOCK_DEPENDENCY),
            (2, pac::I2C2::PTR, pac::I2C2::CLOCK_DEPENDENCY),
            (3, pac::I2C3::PTR, pac::I2C3::CLOCK_DEPENDENCY),
        ] {
            assert_eq!(Mmio::new(n).base, address as usize, "instance {n}");
            assert_eq!(clock, "ClockId::Sys", "instance {n}");
        }
    }

    #[test]
    fn device_readiness_before_during_after_arm_and_missing_delivery() {
        for mask in [RX_FULL, TX_EMPTY, STOP_DET] {
            for phase in 0..4 {
                let mut io = Model {
                    raw: if phase == 0 { mask } else { 0 },
                    during_arm: if phase == 1 { mask } else { 0 },
                    ..Model::default()
                };
                let mut source = I2cSource::<1>::new();
                let ready = driver_irq::probe_and_arm(
                    &mut io,
                    |io| probe(io, mask),
                    |io| source.arm(io, mask),
                );
                assert_eq!(ready, (phase < 2).then_some(Ok(())));
                assert_eq!(io.mask, if phase == 0 { 0 } else { mask | ERRORS });
                if phase == 2 {
                    io.raw = mask;
                    assert_eq!(source.service(&mut io), mask);
                    assert_eq!(probe(&mut io, mask), Some(Ok(())));
                }
                if phase == 3 {
                    assert_eq!(source.service(&mut io), 0);
                    assert_eq!(probe(&mut io, mask), None);
                }
                source.retire(&mut io);
                assert_eq!((io.mask, io.wrap), (0, 0));
            }
        }
    }

    #[test]
    fn stale_stray_and_abort_are_qualified_without_acknowledging_new_state() {
        use crate::pac::dw_i2c::intr::ACTIVITY;

        let mut io = Model {
            wrap: 8,
            ..Model::default()
        };
        let mut source = I2cSource::<1>::new();
        assert_eq!(source.arm(&mut io, STOP_DET), Ok(true));
        source.retire(&mut io);
        io.raw = STOP_DET;
        assert_eq!(source.service(&mut io), 0);
        assert_eq!(source.arm(&mut io, RX_FULL), Ok(true));

        io.mask |= ACTIVITY;
        io.raw |= ACTIVITY;
        assert_eq!(source.service(&mut io), 0);
        assert_eq!(io.mask, RX_FULL | ERRORS);
        assert_eq!(probe(&mut io, RX_FULL), None);

        io.raw |= RX_FULL | TX_ABRT | TX_OVER;
        io.abort = 8;
        assert_eq!(source.service(&mut io), RX_FULL | TX_ABRT | TX_OVER);
        assert_eq!(io.mask, 0);
        assert_eq!(source.service(&mut io), 0);
        assert_eq!(probe(&mut io, RX_FULL), Some(Err(Error::DataNack)));

        source.retire(&mut io);
        assert_eq!(io.wrap, 8);
        io.raw = RX_OVER;
        assert_eq!(probe(&mut io, RX_FULL), Some(Err(Error::Fifo)));

        io.mask = ACTIVITY;
        assert_eq!(source.arm(&mut io, RX_FULL), Err(Error::IrqUnavailable));
        assert_eq!(io.mask, 0);
    }
}
