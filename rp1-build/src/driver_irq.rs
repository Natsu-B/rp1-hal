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

// Each row must agree with board::INSTANCES, the verified route and the HAL binding.
// driver_bindings_match_the_board_and_pac and the serial generator test cover this.
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
    "spi": "spi0" => "SPI0", "spi1" => "SPI1", "spi3" => "SPI3";
    "i2c": "i2c0" => "I2C0", "i2c1" => "I2C1", "i2c2" => "I2C2", "i2c3" => "I2C3";
    "adc": "adc" => "ADC_FIFO";
    "pio": "pio" => "PIO_0", "pio" => "PIO_1";
};

impl DriverIrq {
    pub fn line(&self, core: usize) -> Result<Option<&'static board::VerifiedIrq>> {
        let ownership = metadata::peripheral_id(&self.instance.to_ascii_uppercase())?;
        Ok(board::irq_routes_for(core)
            .find(|line| line.name == self.vector && line.owner.ownership() == ownership))
    }

    pub fn number(&self, line: &board::VerifiedIrq) -> Result<u8> {
        if let Some(index) = self.line_index(line) {
            return Ok(index);
        }
        board::find_instance(self.instance)
            .map(|instance| instance.number)
            .ok_or_else(|| format!("unknown IRQ-owning instance {}", self.instance).into())
    }

    pub fn handler_stem(&self, line: &board::VerifiedIrq) -> String {
        if self.line_index(line).is_some() {
            line.name.to_ascii_lowercase()
        } else {
            self.instance.into()
        }
    }

    fn line_index(&self, line: &board::VerifiedIrq) -> Option<u8> {
        (self.module == "pio").then(|| line.irq - rp1_proto::board::PIO_IRQ0 as u8)
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
            let line = driver.line(1).unwrap().unwrap();
            assert_eq!(driver.line(0).unwrap().is_some(), driver.module != "pio");
            assert_eq!(rp1_pac::INTERRUPT_NAMES[usize::from(line.irq)], line.name);
            assert!(driver.number(line).is_ok());
        }
    }

    #[test]
    fn unknown_instance_fails_route_lookup() {
        let driver = DriverIrq {
            instance: "unknown",
            ..DRIVERS[0]
        };
        let error = driver.line(0).err().unwrap();
        assert!(
            error
                .to_string()
                .contains("unknown or unsupported peripheral 'UNKNOWN'")
        );
    }
}
