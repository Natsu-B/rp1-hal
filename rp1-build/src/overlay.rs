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

/// rpi-cm5 profile: Linux keeps only GPIO 0..1 on the RP1 bank (UART1 TX/RX, e.g.
/// `dtoverlay=uart1-pi5`).
pub const BOARD_GPIO_RESERVED: std::ops::RangeInclusive<u8> = 2..=53;
/// Unlabelled RP1 GPIO hogs of the stock bcm2712-rpi-cm5*.dtb (lines 46 and 48).
pub const BOARD_GPIO_HOGS: &[&str] = &["micclk1_hog", "micdat1_hog"];
/// Stock-enabled Linux consumers of RP1 lines in the board reservation (PWR LED on GPIO44,
/// camera regulator enable on GPIO34); they would fail to probe with -EINVAL.
pub const BOARD_RESERVED_GPIO_USERS: &[&str] = &["led_pwr", "cam0_reg"];
/// Stock-enabled RP1 nodes that need clk-rp1 or the stock RP1 firmware (disabled when RP1
/// is the clock authority).
pub const BOARD_NO_SCMI_CLOCK: &[&str] = &[
    "rp1_adc", "rp1_eth", "rp1_dma", "rp1_pio", "rp1_usb0", "rp1_usb1",
];

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
    // The stock board DT gives Linux the RP1 GPIO hogs below; their lines are in the board
    // reservation, and a hog on a reserved line fails the whole RP1 gpiochip (HYP, gpiolib).
    // A board without them gets an empty disabled child (no gpio-hog: ignored).
    fragment(
        "target = <&rp1_gpio>",
        &BOARD_GPIO_HOGS
            .iter()
            .map(|h| format!("{h} {{ status = \"disabled\"; }};"))
            .collect::<Vec<_>>()
            .join(" "),
    );
    for label in BOARD_RESERVED_GPIO_USERS {
        fragment(&format!("target = <&{label}>"), "status = \"disabled\";");
    }
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
        // Enabled stock nodes whose clocks (or firmware) only clk-rp1 / the stock RP1 firmware
        // provide: rp1-hal serves neither, so Linux must not probe them (the PoC lab DT did the
        // same). USB has no clocks property but its clocks are set up by the stock RP1 firmware
        // (HYP), so it is left off as well.
        for label in BOARD_NO_SCMI_CLOCK {
            fragment(&format!("target = <&{label}>"), "status = \"disabled\";");
        }
        // PoC lab DT (reports/amp-pcie-handoff-20261008.md): hold PERST 20 ms with the refclk
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
