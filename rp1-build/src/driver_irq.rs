//! Declaration rules for device IRQs owned by synchronous HAL drivers.

use crate::{
    Result, board,
    metadata::{self, Core},
};

pub(crate) struct DriverIrq {
    pub instance: &'static str,
    pub vector: &'static str,
    pub custom: &'static str,
    pub module: &'static str,
}

// Each driver provides Irq<N, CORE>, Parts::__new(token, pins, binding) and
// __on_interrupt::<N, CORE>(). The declaration and handler use the verified
// vector binding for the same block.
pub(crate) const DRIVERS: &[DriverIrq] = &[
    DriverIrq {
        instance: "uart1",
        vector: "UART1",
        custom: "custom_uart1_handler",
        module: "uart",
    },
    DriverIrq {
        instance: "spi0",
        vector: "SPI0",
        custom: "custom_spi0_handler",
        module: "spi",
    },
];

impl DriverIrq {
    pub fn verified_line(&self) -> Result<&'static board::VerifiedIrq> {
        let ownership = metadata::peripheral_id(&self.instance.to_ascii_uppercase())?;
        board::IRQ_VERIFIED
            .iter()
            .find(|line| line.name == self.vector && line.owner.ownership() == ownership)
            .ok_or_else(|| format!("{} has no verified {} IRQ", self.instance, self.vector).into())
    }

    pub fn number(&self) -> Result<u8> {
        board::find_instance(self.instance)
            .map(|instance| instance.number)
            .ok_or_else(|| format!("unknown IRQ-owning instance {}", self.instance).into())
    }

    pub fn is_managed(&self, core: &Core) -> bool {
        core.wait_timer.is_some()
            && core.instances.contains_key(self.instance)
            && !self.is_custom(core)
    }

    pub fn is_custom(&self, core: &Core) -> bool {
        core.custom_driver_handlers.contains_key(self.custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn driver_bindings_match_the_board_and_pac() {
        for driver in DRIVERS {
            let line = driver.verified_line().unwrap();
            assert_eq!(rp1_pac::INTERRUPT_NAMES[usize::from(line.irq)], line.name);
            assert!(driver.number().is_ok());
        }
    }
}
