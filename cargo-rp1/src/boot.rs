//! Boot set assembly (build.md §2 ⑤, loader.md §1): `rp1-bundle.img`, `config.txt`,
//! `cmdline.txt` and `manifest.txt` for the `rp1boot` loader.
use crate::Result;
use rp1_proto::bundle::{self, ALIGN, ENTRY_LEN, HEADER_LEN};
use sha2::{Digest, Sha256};
use std::{fmt::Write as _, io::Read, path::Path};

pub const BUNDLE: &str = "rp1-bundle.img";
pub const LOADER: &str = "rp1boot.img";
/// HW 2026-10-29 (hwrun5 07-boot6): the firmware prepended 1034 B and cut /chosen/bootargs
/// at 1966 B, so cmdline.txt must stay well under ~930 B including the 28 B token.
pub const CMDLINE_MAX: usize = 880;

/// Lines every generated config.txt starts with (the firmware loads the loader as the
/// kernel and the bundle as the initramfs; loader.md §1).
pub const CONFIG_FIXED: &[&str] = &[
    "arm_64bit=1",
    "os_check=0",
    "enable_uart=1",
    "auto_initramfs=0",
    "kernel=rp1boot.img",
    "initramfs rp1-bundle.img followkernel",
];

pub struct Input<'a> {
    pub firmware: &'a [u8],
    /// arm64 `Image`, raw or gzip (decompressed here; the loader does not decompress).
    pub kernel: &'a [u8],
    pub initramfs: Option<&'a [u8]>,
    pub cmdline: &'a str,
    pub config_extra: &'a [String],
}

pub struct BootSet {
    pub bundle: Vec<u8>,
    pub id: [u8; 32],
    pub config: String,
    pub cmdline: String,
}

pub fn hex(b: &[u8]) -> String {
    b.iter().fold(String::new(), |mut s, b| {
        let _ = write!(s, "{b:02x}");
        s
    })
}

/// Decompress a gzip kernel and check the arm64 Image header magic.
pub fn kernel_image(kernel: &[u8]) -> Result<Vec<u8>> {
    let image = if kernel.starts_with(&[0x1f, 0x8b]) {
        let mut out = Vec::new();
        flate2::read::GzDecoder::new(kernel).read_to_end(&mut out)?;
        out
    } else {
        kernel.to_vec()
    };
    if image.get(56..60) != Some(b"ARM\x64") {
        return Err("kernel is not an arm64 Image (no ARM\\x64 magic at 56)".into());
    }
    Ok(image)
}

pub fn assemble(input: &Input<'_>) -> Result<BootSet> {
    if input.cmdline.contains(bundle::TOKEN) || input.cmdline.contains('\n') {
        return Err("cmdline must be one line without rp1.bundle=".into());
    }
    if input.cmdline.len() > CMDLINE_MAX {
        return Err(format!(
            "cmdline is {} B (> {CMDLINE_MAX}): the firmware prepends ~1 KiB and truncates \
             /chosen/bootargs, which would drop the rp1.bundle= token",
            input.cmdline.len()
        )
        .into());
    }
    for line in input.config_extra {
        let key = line.split(['=', ' ']).next().unwrap_or("").trim();
        if line.contains('\n')
            || [
                "kernel",
                "initramfs",
                "auto_initramfs",
                "arm_64bit",
                "cmdline",
            ]
            .contains(&key)
        {
            return Err(format!("config_extra may not set {key:?} (owned by rp1boot)").into());
        }
    }
    let kernel = kernel_image(input.kernel)?;
    let mut parts: Vec<(u32, &[u8])> = vec![
        (bundle::FIRMWARE, input.firmware),
        (bundle::KERNEL, &kernel),
    ];
    if let Some(i) = input.initramfs {
        parts.push((bundle::INITRAMFS, i));
    }
    let header_len = HEADER_LEN + parts.len() * ENTRY_LEN;
    let align = |n: usize| n.next_multiple_of(ALIGN as usize);
    let mut offset = align(header_len);
    let mut header = Vec::with_capacity(header_len);
    header.extend_from_slice(bundle::MAGIC);
    header.extend_from_slice(&bundle::VERSION.to_le_bytes());
    header.extend_from_slice(&(header_len as u32).to_le_bytes());
    header.extend_from_slice(&(parts.len() as u32).to_le_bytes());
    let total_at = header.len();
    header.extend_from_slice(&[0; 12]);
    let mut layout = Vec::new();
    for (kind, data) in &parts {
        if data.is_empty() {
            return Err("bundle elements must not be empty".into());
        }
        header.extend_from_slice(&kind.to_le_bytes());
        header.extend_from_slice(&0u32.to_le_bytes());
        header.extend_from_slice(&u32::try_from(offset)?.to_le_bytes());
        header.extend_from_slice(&u32::try_from(data.len())?.to_le_bytes());
        header.extend_from_slice(&Sha256::digest(data));
        layout.push(offset);
        offset = align(offset + data.len());
    }
    // The last element ends the file (no trailing padding).
    let total = layout.last().unwrap() + parts.last().unwrap().1.len();
    header[total_at..total_at + 4].copy_from_slice(&u32::try_from(total)?.to_le_bytes());
    let mut out = vec![0u8; total];
    out[..header_len].copy_from_slice(&header);
    for ((_, data), at) in parts.iter().zip(&layout) {
        out[*at..*at + data.len()].copy_from_slice(data);
    }
    let parsed = bundle::Bundle::parse(&out).map_err(|e| format!("bundle self-check: {e}"))?;
    let id: [u8; 32] = Sha256::digest(parsed.header()).into();
    let token = String::from_utf8(bundle::id_hex(&id).to_vec())?;
    let mut config = String::from(
        "# Generated by `cargo rp1 bundle`. rp1boot checks the bundle against cmdline.txt.\n",
    );
    for l in CONFIG_FIXED
        .iter()
        .copied()
        .chain(input.config_extra.iter().map(String::as_str))
    {
        config.push_str(l);
        config.push('\n');
    }
    let cmdline = format!("{} {}{token}\n", input.cmdline.trim(), bundle::TOKEN)
        .trim_start()
        .to_string();
    Ok(BootSet {
        bundle: out,
        id,
        config,
        cmdline,
    })
}

/// Write the boot set (and an optional prebuilt loader) into `dir`, which must not hold one.
pub fn write(dir: &Path, set: &BootSet, firmware: &[u8], loader: Option<&[u8]>) -> Result<String> {
    std::fs::create_dir_all(dir)?;
    let mut files: Vec<(&str, &[u8])> = vec![
        ("config.txt", set.config.as_bytes()),
        ("cmdline.txt", set.cmdline.as_bytes()),
        (BUNDLE, &set.bundle),
        ("rp1.elf", firmware),
    ];
    if let Some(l) = loader {
        files.push((LOADER, l));
    }
    let mut manifest = format!("bundle-id {}\n", hex(&set.id));
    for (name, data) in &files {
        std::fs::write(dir.join(name), data)?;
        let _ = writeln!(manifest, "{}  {name}", hex(&Sha256::digest(data)));
    }
    std::fs::write(dir.join("manifest.txt"), &manifest)?;
    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn image() -> Vec<u8> {
        let mut k = vec![0u8; 5000];
        k[56..60].copy_from_slice(b"ARM\x64");
        k
    }

    #[test]
    fn bundle_round_trips_and_binds_cmdline() {
        let (fw, k, init) = (vec![1u8; 300], image(), vec![3u8; 7]);
        let set = assemble(&Input {
            firmware: &fw,
            kernel: &k,
            initramfs: Some(&init),
            cmdline: "console=ttyAMA10,115200",
            config_extra: &["device_tree_address=0x20000000".into()],
        })
        .unwrap();
        let b = bundle::Bundle::parse(&set.bundle).unwrap();
        assert_eq!(b.data(&b.get(bundle::KERNEL).unwrap()), &k[..]);
        assert_eq!(b.data(&b.get(bundle::INITRAMFS).unwrap()), &init[..]);
        for e in b.entries() {
            assert_eq!(&Sha256::digest(b.data(&e))[..], &e.sha256);
        }
        let token = bundle::find_token(&set.cmdline).unwrap();
        assert_eq!(token.as_bytes(), &bundle::id_hex(&set.id));
        assert!(set.config.contains("kernel=rp1boot.img\n"));
        assert!(set.config.ends_with("device_tree_address=0x20000000\n"));
    }

    #[test]
    fn rejects_bad_inputs() {
        let (fw, k) = (vec![1u8; 3], image());
        let base = |cmdline, extra: &[String], kernel: &[u8]| {
            assemble(&Input {
                firmware: &fw,
                kernel,
                initramfs: None,
                cmdline,
                config_extra: extra,
            })
            .is_err()
        };
        assert!(base("rp1.bundle=1", &[], &k));
        assert!(base("", &["kernel=x".into()], &k));
        assert!(base("", &["initramfs foo".into()], &k));
        assert!(base("", &[], &[0u8; 100]));
        assert!(!base("", &[], &k));
    }
}
