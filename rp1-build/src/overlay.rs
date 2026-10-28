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
/// labelled `rp1_gpio`), read with the host `fdtget`.
pub fn reserved_gpios(dtb: &Path) -> Result<std::collections::BTreeSet<u32>> {
    let fdtget = |args: &[&str]| -> Result<String> {
        let out = Command::new("fdtget").arg(dtb).args(args).output()?;
        if !out.status.success() {
            return Err(format!("fdtget failed: {}", String::from_utf8_lossy(&out.stderr)).into());
        }
        Ok(String::from_utf8(out.stdout)?.trim().to_string())
    };
    let path = fdtget(&["/__symbols__", "rp1_gpio"])?;
    let cells = match fdtget(&["-t", "u", &path, "gpio-reserved-ranges"]) {
        Ok(cells) => cells,
        Err(_) => return Ok(Default::default()),
    };
    let cells: Vec<u32> = cells
        .split_whitespace()
        .map(str::parse)
        .collect::<std::result::Result<_, _>>()?;
    if !cells.len().is_multiple_of(2) {
        return Err("gpio-reserved-ranges must be <start count> pairs".into());
    }
    Ok(cells.chunks(2).flat_map(|c| c[0]..c[0] + c[1]).collect())
}

/// rpi-cm5 lab profile (scmi_linux.dtb): Linux keeps only GPIO 0..1 on the RP1 bank.
pub const BOARD_GPIO_RESERVED: std::ops::RangeInclusive<u8> = 2..=53;

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
    // Keep the PoC scratch/openamp reservations intact; use the independent board-table range.
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
    // no interrupts and Linux polls the link page. Only the OpenAMP remoteproc (mailbox channels
    // 1..3, not SCMI's channel 0) is disabled; it has no label in the lab profile, so by path.
    fragment(
        "target-path = \"/openamp-address-space\"",
        "status = \"disabled\";",
    );
    if d.clock.authority == "rp1" {
        // scmi_linux profiles already supply the clock-consumer rewrites and mailbox/shmem wiring.
        fragment("target = <&rp1_clocks>", "status = \"disabled\";");
        fragment("target = <&rp1_scmi>", "status = \"okay\";");
        fragment("target = <&rp1_scmi_clocks>", "status = \"okay\";");
    }
    s.push_str("};\n");
    Ok(s)
}
