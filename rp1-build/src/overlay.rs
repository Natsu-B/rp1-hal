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
pub fn dts(d: &Declaration, note: &[u8]) -> Result<String> {
    let regions = d.regions()?;
    let ddr_end = regions.last().map_or(65536, |r| r.ddr_offset + r.ddr_len);
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
    let mut pins: Vec<_> = d.proc0.pins.iter().chain(&d.proc1.pins).copied().collect();
    if d.log.is_some() {
        pins.extend([14, 15]);
    }
    pins.sort();
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
        rp1-header@21000000 {{ compatible = "generic-uio"; linux,uio-name = "rp1:header";
            reg = <0 0x21000000 0x10000>; status = "okay"; }};
    "#
        ),
    );
    // Merge the existing scratch node rather than overlapping it with a second reservation.
    fragment(
        "target-path = \"/reserved-memory\"",
        &format!("rp1-ddr-scratch@21000000 {{ no-map; reg = <0 0x21000000 0x{ddr_end:x}>; }};"),
    );
    for r in regions {
        let addr = rp1_proto::DDR_PHYS + u64::from(r.ddr_offset);
        fragment(
            "target-path = \"/\"",
            &format!(
                "rp1-{}@{addr:x} {{ compatible = \"generic-uio\"; linux,uio-name = \"rp1:{}\"; reg = <0 0x{addr:x} 0x{:x}>; status = \"okay\"; }};",
                r.spec.name, r.spec.name, r.ddr_len
            ),
        );
    }
    // HOST_EVENTS (RP1 hwirq 58) is requested exclusively by rp1-mailbox, and uio_pdrv_genirq
    // cannot share it (HW 2026-10-28: -EBUSY). Re-bind the mailbox node itself as the UIO
    // "rp1:events" device so it keeps its reg, interrupts and interrupt-parent.
    fragment(
        "target = <&rp1_mbox>",
        r#"compatible = "generic-uio"; linux,uio-name = "rp1:events"; status = "okay";"#,
    );
    // Everything that consumed the mailbox goes away with it: the OpenAMP remoteproc (lab
    // scmi_linux profile, no label, so by path) and the SCMI agent (its transport is this mailbox).
    fragment(
        "target-path = \"/openamp-address-space\"",
        "status = \"disabled\";",
    );
    fragment("target = <&rp1_scmi>", "status = \"disabled\";");
    if d.clock.authority == "rp1" {
        // Linux must not drive the RP1 clock generator (clk_disable_unused). Consumers rewritten to
        // SCMI clocks by the scmi_linux profile stay deferred until SCMI has a non-mailbox doorbell.
        fragment("target = <&rp1_clocks>", "status = \"disabled\";");
    }
    s.push_str("};\n");
    Ok(s)
}
