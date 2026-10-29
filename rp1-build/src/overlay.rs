use crate::{Result, metadata::Declaration};
use std::{path::Path, process::Command};

pub fn compile(source: &Path, destination: &Path) -> Result<()> {
    let output = Command::new(std::env::var_os("RP1_DTC").unwrap_or_else(|| "dtc".into()))
        .args(["-@", "-I", "dts", "-O", "dtb", "-o"])
        .arg(destination)
        .arg(source)
        .output()?;
    if !output.status.success() {
        return Err(format!("dtc failed: {}", String::from_utf8_lossy(&output.stderr)).into());
    }
    Ok(())
}
pub fn apply(base: &Path, overlay: &Path, destination: &Path) -> Result<()> {
    if base == destination {
        return Err("base DTB must remain read-only; choose another output".into());
    }
    let output = Command::new("fdtoverlay")
        .arg("-i")
        .arg(base)
        .arg("-o")
        .arg(destination)
        .arg(overlay)
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "fdtoverlay failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(())
}
/// RP1 GPIO numbers that `dtb` withholds from Linux (`gpio-reserved-ranges` of the node
/// labelled `rp1_gpio`).
pub fn reserved_gpios(dtb: &Path) -> Result<std::collections::BTreeSet<u32>> {
    Ok(rp1_dt::check::reserved_gpios(&rp1_dt::fdt::Fdt::parse(
        &std::fs::read(dtb)?,
    )?)?)
}

/// rpi-cm5 profile: the 40-pin header GPIO 2..27 is RP1's to declare (Linux keeps 0..1 for UART1,
/// e.g. `dtoverlay=uart1-pi5`). GPIO 28..53 are board lines of Linux-owned blocks (ETH_RST_N 32,
/// ETH_IRQ_N 37, USB VBUS 42/43, PWR LED 44, fan, camera regulator) and stay Linux's, as in the
/// stock DT.
pub const BOARD_GPIO_RESERVED: std::ops::RangeInclusive<u8> = 2..=27;
/// Stock-enabled RP1 nodes that need the stock RP1 firmware's mailbox service (`rp1_firmware`),
/// which rp1-hal does not implement (disabled when RP1 is the clock authority).
pub const BOARD_NEEDS_STOCK_FIRMWARE: &[&str] = &["rp1_pio"];
/// Consumers of RP1 clocks and the SCMI clock IDs (rp1_hal::scmi::id) for each `clocks` cell, in
/// the stock order (DOC: bcm2712-rpi-cm5 DT / rp1.dtsi). Nodes whose clocks rp1-hal does not serve
/// (csi, dsi, vec, dpi, mmc: video PLL, MIPI and SDIO clocks) keep `&rp1_clocks` and cannot probe.
pub const BOARD_SCMI_CONSUMERS: &[(&str, &[u32])] = &[
    // pclk, hclk = clk_sys; tsu_clk = clk_eth_tsu; tx_clk = clk_eth.
    ("rp1_eth", &[2, 2, 6, 5]),
    // core-clk = clk_dma, cfgr-clk = clk_sys.
    ("rp1_dma", &[4, 2]),
    ("rp1_adc", &[7]),
    ("rp1_pwm0", &[8]),
    ("rp1_pwm1", &[9]),
    ("rp1_audio_out", &[10]),
    ("rp1_i2s0", &[11]),
    ("rp1_i2s1", &[11]),
    ("rp1_i2s2", &[11]),
    ("rp1_i2c0", &[2]),
    ("rp1_i2c1", &[2]),
    ("rp1_i2c2", &[2]),
    ("rp1_i2c3", &[2]),
    ("rp1_i2c4", &[2]),
    ("rp1_i2c5", &[2]),
    ("rp1_i2c6", &[2]),
    ("rp1_spi0", &[2]),
    ("rp1_spi1", &[2]),
    ("rp1_spi2", &[2]),
    ("rp1_spi3", &[2]),
    ("rp1_spi4", &[2]),
    ("rp1_spi5", &[2]),
    ("rp1_spi6", &[2]),
    ("rp1_spi7", &[2]),
    ("rp1_spi8", &[2]),
];
/// Consumers whose stock `assigned-clocks` name an RP1 clock (pwm, audio out): the SCMI ID.
/// Their `assigned-clock-parents` (audio out: `<&rp1_clocks 10>`, PLL_AUDIO) would still point
/// at the disabled clk-rp1 and defer the probe; an overlay cannot delete a property, so it is
/// replaced by one null phandle, which Linux skips (`of_clk_set_defaults`, clk-conf.c). SCMI
/// clocks have no parent Linux can select.
pub const BOARD_SCMI_ASSIGNED: &[(&str, u32)] =
    &[("rp1_pwm0", 8), ("rp1_pwm1", 9), ("rp1_audio_out", 10)];

pub fn dts(d: &Declaration, note: &[u8]) -> Result<String> {
    let regions = d.regions()?;
    let ddr_end = d.ddr_size()?;
    let phys = rp1_proto::SHM_PHYS;
    let bytes = note
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join(" ");
    // BCM2712 DTBs use #address-cells = <2>, #size-cells = <1> at "/" and "/reserved-memory",
    // so every reg below is <addr_hi addr_lo size> (3 cells).
    let mut s = String::from("/dts-v1/;\n/plugin/;\n/ {\n");
    let mut n = 0;
    let mut fragment = |target: &str, content: &str| {
        s.push_str(&format!(
            "fragment@{n} {{ {target}; __overlay__ {{ {content} }}; }};\n"
        ));
        n += 1;
    };
    // gpio-reserved-ranges is replaced, not merged, by an overlay. Emit the union of the board
    // profile's reservation and the declared pins so Linux never gets more pins than the base DT
    // gives it (`cargo rp1 check --dtb` verifies this against the actual DTB).
    let mut pins: Vec<_> = d.proc0.pins.iter().chain(&d.proc1.pins).copied().collect();
    if d.log.is_some() {
        pins.extend([14, 15]);
    }
    pins.extend(BOARD_GPIO_RESERVED);
    pins.sort();
    pins.dedup();
    let ranges = pins
        .iter()
        .map(|p| format!("{p} 1"))
        .collect::<Vec<_>>()
        .join(" ");
    if !pins.is_empty() {
        fragment(
            "target = <&rp1_gpio>",
            &format!("gpio-reserved-ranges = <{ranges}>;"),
        );
    }
    let mut uarts: Vec<_> = d
        .proc0
        .peripherals
        .iter()
        .chain(&d.proc1.peripherals)
        .filter(|p| p.starts_with("UART"))
        .map(|p| p[4..].parse::<u8>().unwrap())
        .collect();
    if d.log.is_some() {
        uarts.push(0);
    }
    for uart in uarts {
        fragment(
            &format!("target = <&rp1_uart{uart}>"),
            "status = \"disabled\";",
        );
    }
    if d.proc0
        .peripherals
        .iter()
        .chain(&d.proc1.peripherals)
        .any(|p| p == "ADC")
    {
        fragment("target = <&rp1_adc>", "status = \"disabled\";");
    }
    for i2c in d
        .proc0
        .peripherals
        .iter()
        .chain(&d.proc1.peripherals)
        .filter_map(|p| p.strip_prefix("I2C"))
    {
        fragment(
            &format!("target = <&rp1_i2c{i2c}>"),
            "status = \"disabled\";",
        );
    }
    fragment(
        "target-path = \"/\"",
        &format!(
            r#"
        rp1-firmware {{ rp1,note = [{bytes}]; rp1,abi = <2>; rp1,static-only; }};
        rp1_link: rp1-link@1f0040c000 {{ compatible = "generic-uio"; linux,uio-name = "rp1:link";
            reg = <0x1f 0x0040c000 0x4000>; status = "okay"; }};
        rp1-doorbell@1f000d0000 {{ compatible = "generic-uio"; linux,uio-name = "rp1:doorbell";
            reg = <0x1f 0x000d0000 0x4000>; status = "okay"; }};
        rp1-header@{phys:x} {{ compatible = "generic-uio"; linux,uio-name = "rp1:header";
            reg = <0 0x{phys:x} 0x10000>; status = "okay"; }};
    "#
        ),
    );
    // Independent of any other reservation (`final_conflicts` rejects overlaps).
    fragment(
        "target-path = \"/reserved-memory\"",
        &format!("rp1-shm@{phys:x} {{ no-map; reg = <0 0x{phys:x} 0x{ddr_end:x}>; }};"),
    );
    for r in regions {
        let addr = phys + u64::from(r.ddr_offset);
        fragment(
            "target-path = \"/\"",
            &format!(
                "rp1-{}@{addr:x} {{ compatible = \"generic-uio\"; linux,uio-name = \"rp1:{}\"; reg = <0 0x{addr:x} 0x{:x}>; status = \"okay\"; }};",
                r.spec.name, r.spec.name, r.ddr_len
            ),
        );
    }
    // rp1-mailbox keeps HOST_EVENTS (hwirq 58) and stays SCMI's transport; UIO cannot share that
    // IRQ (HW 2026-10-28: -EBUSY). v0.1 has no proc1 -> Linux interrupt: the UIO nodes above carry
    // no interrupts and Linux polls the link page. The PoC OpenAMP remoteproc (mailbox channels
    // 1..3) has no label; the stock DTB has no such node, so this leaves an empty disabled
    // `/openamp-address-space` there (inert: no compatible) and disables it in a PoC DT.
    fragment(
        "target-path = \"/\"",
        "openamp-address-space { status = \"disabled\"; };",
    );
    if d.clock.authority == "rp1" {
        // design §6/§9: RP1 owns its clocks. Linux's clk-rp1 is disabled (otherwise
        // clk_disable_unused stops clocks of RP1-only blocks) and the stock DT gets the SCMI
        // transport rp1_hal::scmi serves: mailbox channel 0 + shmem in shared SRAM.
        let shmem = rp1_proto::LINK_START as usize - 0x2000_0000 + crate::metadata::SCMI_OFFSET;
        fragment("target = <&rp1_clocks>", "status = \"disabled\";");
        // The stock RP1 firmware client uses the same mailbox channel 0 (and the shmem@ff00
        // page); rp1-hal does not implement that protocol.
        fragment("target = <&rp1_firmware>", "status = \"disabled\";");
        fragment("target = <&rp1_fw_shmem>", "status = \"disabled\";");
        fragment(
            "target = <&sram>",
            &format!(
                "rp1_scmi_shmem: scmi@{shmem:x} {{ compatible = \"arm,scmi-shmem\"; reg = <0x{shmem:x} 0x100>; }};"
            ),
        );
        // /firmware is populated by Linux (of_platform_default_populate_init).
        fragment(
            "target-path = \"/\"",
            r#"
        firmware {
            rp1_scmi: scmi { compatible = "arm,scmi"; mboxes = <&rp1_mbox 0>; shmem = <&rp1_scmi_shmem>;
                #address-cells = <1>; #size-cells = <0>;
                rp1_scmi_clocks: protocol@14 { reg = <0x14>; #clock-cells = <1>; };
            };
        };
    "#,
        );
        // Clock consumers: the SCMI table (rp1_hal::scmi) serves uart_apb (0) and uart (1),
        // so every RP1 UART is switched (Linux may enable any of them, e.g. dtoverlay=uart1-pi5).
        for uart in 0..=5 {
            fragment(
                &format!("target = <&rp1_uart{uart}>"),
                "clocks = <&rp1_scmi_clocks 1>, <&rp1_scmi_clocks 0>; clock-names = \"uartclk\", \"apb_pclk\";",
            );
        }
        // Every other RP1 clock consumer (rp1_hal::scmi serves the clocks of the blocks Linux
        // owns; rp1_hal::linux_blocks releases their resets). A consumer the image declares is
        // already disabled above; retargeting it too is harmless.
        for (label, ids) in BOARD_SCMI_CONSUMERS {
            let cells = ids
                .iter()
                .map(|i| format!("<&rp1_scmi_clocks {i}>"))
                .collect::<Vec<_>>()
                .join(", ");
            fragment(
                &format!("target = <&{label}>"),
                &format!("clocks = {cells};"),
            );
        }
        for (label, id) in BOARD_SCMI_ASSIGNED {
            fragment(
                &format!("target = <&{label}>"),
                &format!(
                    "assigned-clocks = <&rp1_scmi_clocks {id}>; assigned-clock-parents = <0>;"
                ),
            )
        }
        for label in BOARD_NEEDS_STOCK_FIRMWARE {
            fragment(&format!("target = <&{label}>"), "status = \"disabled\";");
        }
        // PoC lab DT (PCIe handoff study, HW 2026-10-08): hold PERST 20 ms with the refclk
        // running so the RP1 endpoint reconfiguration (rp1_hal::cold) is reliable.
        fragment("target = <&pcie2>", "brcm,tperst-clk-ms = <20>;");
    }
    s.push_str("};\n");
    Ok(s)
}

/// Final-DTB ownership check (declaration-and-note.md §3.2 step 3, design §9): every conflict
/// between the note's OWNERSHIP/SHM records and what Linux would see in `dtb`.
/// The same code runs in the loader (`rp1_dt::check`).
pub fn final_conflicts(dtb: &[u8], note: &[u8]) -> Result<Vec<String>> {
    Ok(rp1_dt::check::final_conflicts(
        &rp1_dt::fdt::Fdt::parse(dtb)?,
        note,
    )?)
}
