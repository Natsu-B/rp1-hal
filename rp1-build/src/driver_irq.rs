//! Declaration rules for device IRQs owned by synchronous HAL drivers.

use crate::declaration::Core;

pub(crate) struct DriverIrq {
    pub instance: &'static str,
    pub vector: &'static str,
    pub custom: &'static str,
    pub module: &'static str,
}

// Each driver provides Irq<CORE>, Parts::__new(token, pins, binding) and
// __on_interrupt::<CORE>(). The declaration and handler use the verified
// vector binding for the same block.
pub(crate) const DRIVERS: &[DriverIrq] = &[DriverIrq {
    instance: "uart1",
    vector: "UART1",
    custom: "custom_uart1_handler",
    module: "uart",
}];

impl DriverIrq {
    pub fn is_managed(&self, core: &Core) -> bool {
        core.wait_timer.is_some()
            && core.instances.contains_key(self.instance)
            && !self.is_custom(core)
    }

    pub fn is_custom(&self, core: &Core) -> bool {
        core.custom_driver_handlers.contains_key(self.custom)
    }
}
