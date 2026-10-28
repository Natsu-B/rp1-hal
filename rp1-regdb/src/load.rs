use std::{fs, path::Path};

use anyhow::{Context, Result, bail};

use crate::model::{Database, DeviceFile, PeripheralDefinition, PeripheralFile};

pub fn load_database(device_path: &Path) -> Result<Database> {
    let root = device_path
        .parent()
        .context("device TOML has no parent directory")?;
    let device_text = fs::read_to_string(device_path)
        .with_context(|| format!("reading {}", device_path.display()))?;
    let device_file: DeviceFile = toml::from_str(&device_text)
        .with_context(|| format!("parsing {}", device_path.display()))?;
    if device_file.schema != "rp1-regdb-v3" {
        bail!(
            "{}: unsupported schema {}",
            device_path.display(),
            device_file.schema
        );
    }

    let mut peripherals = Vec::new();
    for relative in &device_file.data_files {
        let path = root.join(relative);
        let text =
            fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
        let mut file: PeripheralFile =
            toml::from_str(&text).with_context(|| format!("parsing {}", path.display()))?;
        for register in &mut file.registers {
            register.source = path.clone();
        }
        peripherals.push(PeripheralDefinition {
            peripheral: file.peripheral,
            registers: file.registers,
            source: path,
        });
    }

    Ok(Database {
        schema: device_file.schema,
        device: device_file.device,
        core: device_file.core,
        boot_rom: device_file.boot_rom,
        peripherals,
        interrupts: device_file.interrupt,
        clocks: device_file.clock,
        gpio_banks: device_file.gpio_bank,
        address_spaces: device_file.address_space,
        pci_endpoint: device_file.pci_endpoint,
        root: root.to_owned(),
    })
}
