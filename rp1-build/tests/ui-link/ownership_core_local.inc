use rp1_proto::{Message, SharedSafe};
use static_assertions::{assert_impl_all, assert_not_impl_any};

macro_rules! core_local {
    ($($resource:ty),+ $(,)?) => {
        $(
            assert_impl_all!($resource: Send);
            assert_not_impl_any!($resource: Sync, SharedSafe, Message);
        )+
    };
}

core_local!(
    rp1_hal::gpio::Pin<22>,
    rp1_hal::gpio::PinIrq<22, 0>,
    rp1_hal::uart::Tx<1, 0>,
    rp1_hal::uart::Rx<1, 1>,
    rp1_hal::timer::Alarm<0>,
    rp1_rt::interrupt::Line<26>,
    rp1::proc1_irq::TIMER_0,
    rp1::Doorbell1,
    rp1_link::notify::HostIrq,
    rp1_hal::dma::Channel<0>,
    rp1_hal::adc::Token,
    rp1_hal::dma::Token,
    rp1_hal::i2c::Token<1>,
    rp1_hal::pwm::Token<0>,
    rp1_hal::pwm::Channel<0, 0>,
    rp1_hal::spi::Token<0>,
    rp1_hal::uart::Token<1>,
    rp1_hal::platform::Token,
    rp1::Proc0Pins,
    rp1::Proc0Irq,
    rp1::Proc1DmaChannels,
);
assert_not_impl_any!(rp1::Proc0Resources: Send, Sync, SharedSafe, Message);

fn main() {}
