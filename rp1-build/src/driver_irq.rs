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

// The same route supplies the binding and the generated handler.
macro_rules! drivers {
    ($($module:literal: $($instance:literal => $vector:literal),+;) +) => {
        &[$($(DriverIrq {
            instance: $instance,
            vector: $vector,
            custom: concat!("custom_", $instance, "_handler"),
            module: $module,
        },)+)+]
    };
}

pub(crate) const DRIVERS: &[DriverIrq] = drivers! {
    "uart": "uart0" => "UART0", "uart1" => "UART1", "uart2" => "UART2",
        "uart3" => "UART3", "uart4" => "UART4";
    "spi": "spi0" => "SPI0", "spi1" => "SPI1", "spi2" => "SPI2",
        "spi3" => "SPI3", "spi5" => "SPI5";
    "i2c": "i2c0" => "I2C0", "i2c1" => "I2C1", "i2c2" => "I2C2", "i2c3" => "I2C3";
    "adc": "adc" => "ADC_FIFO";
};

impl DriverIrq {
    pub fn line(&self) -> Result<&'static board::VerifiedIrq> {
        let ownership = metadata::peripheral_id(&self.instance.to_ascii_uppercase())?;
        board::irq_routes()
            .find(|line| line.name == self.vector && line.owner.ownership() == ownership)
            .ok_or_else(|| format!("{} has no declared {} IRQ", self.instance, self.vector).into())
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
            let line = driver.line().unwrap();
            assert_eq!(rp1_pac::INTERRUPT_NAMES[usize::from(line.irq)], line.name);
            assert!(driver.number().is_ok());
        }
    }
}
