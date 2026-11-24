//! What the platform costs in one image (PLAT_SIZE, contract R12): the memory budgets and the
//! bytes of each section kind by owner.
//!
//! Ownership comes from symbol names, so it is approximate (STATIC): LTO inlines platform code
//! into the user's functions (for example `platform::init` into `main`), which counts as user
//! code, and anonymous constants and literal pools have no owner. The platform figure is
//! therefore a lower bound and the user figure an upper bound.

use object::{Object, ObjectSection, ObjectSymbol};
use std::collections::HashSet;

/// Memory map of `rp1-rt/link.x` (origin, length); a host test compares it with the script.
const SRAM: (u64, u32) = (0x2000_0000, 48 * 1024);
const ISRAM: (u64, u32) = (0x1000_0000, 8 * 1024);
const DSRAM: (u64, u32) = (0x1000_2000, 8 * 1024);
/// The C library functions that compiler-builtins provides.
const MEM_FUNCTIONS: [&str; 4] = ["memcpy", "memmove", "memset", "memcmp"];

/// Crates whose code is the platform (rp1-hal and its runtime, protocol and register crates).
const PLATFORM_CRATES: [&str; 4] = ["rp1_rt", "rp1_hal", "rp1_proto", "rp1_pac"];
/// Crates of the Rust toolchain.
const RUNTIME_CRATES: [&str; 3] = ["core", "compiler_builtins", "alloc"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Owner {
    Platform,
    Runtime,
    User,
    /// No name to go by (anonymous constants, literal pools, padding).
    Unattributed,
}
const OWNERS: [(Owner, &str); 4] = [
    (Owner::Platform, "platform"),
    (Owner::Runtime, "runtime"),
    (Owner::User, "user"),
    (Owner::Unattributed, "unattributed"),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Code,
    Data,
    Bss,
}

/// The owner of a symbol, from its (mangled) name.
pub fn owner(name: &str) -> Owner {
    if name.is_empty() || name.starts_with(".L") || name.starts_with('$') {
        return Owner::Unattributed;
    }
    if name.starts_with("__rp1_") {
        return Owner::Platform;
    }
    let path = if let Some(v0) = name.strip_prefix("_R") {
        v0_crate(v0)
    } else if let Some(legacy) = name.strip_prefix("_ZN") {
        ident(legacy)
    } else if name.starts_with("__") || MEM_FUNCTIONS.contains(&name) {
        // compiler-builtins: __aeabi_*, memcpy, memset, ...
        return Owner::Runtime;
    } else {
        // An unmangled (`#[unsafe(no_mangle)]`) function of the firmware.
        return Owner::User;
    };
    match path {
        Some((krate, _)) if PLATFORM_CRATES.contains(&krate) => Owner::Platform,
        Some((krate, _)) if RUNTIME_CRATES.contains(&krate) => Owner::Runtime,
        // The `rp1` module that rp1-build generates into the firmware crate (note, log sink).
        Some((_, rest))
            if rest.starts_with("3rp1") && !rest[4..].starts_with(char::is_alphabetic) =>
        {
            Owner::Platform
        }
        _ => Owner::User,
    }
}

/// `<decimal length><identifier>` at the start of `s`, and what follows it.
fn ident(s: &str) -> Option<(&str, &str)> {
    let digits = s.find(|c: char| !c.is_ascii_digit())?;
    let len: usize = s.get(..digits)?.parse().ok()?;
    let rest = s.get(digits..)?;
    // v0 separates an identifier that starts with a digit or `_` by one `_`.
    let rest = rest.strip_prefix('_').unwrap_or(rest);
    Some((rest.get(..len)?, rest.get(len..)?))
}

/// The first crate root of a v0 symbol (`C [s<disambiguator>_] <identifier>`).
fn v0_crate(s: &str) -> Option<(&str, &str)> {
    let mut from = 0;
    while let Some(k) = s[from..].find('C') {
        let at = from + k + 1;
        let tail = &s[at..];
        let tail = match tail.strip_prefix('s') {
            Some(t) => t.split_once('_').map_or(t, |(_, r)| r),
            None => tail,
        };
        if let Some(found) = ident(tail)
            && !found.0.is_empty()
        {
            return Some(found);
        }
        from = at;
    }
    None
}

/// Bytes of one section kind by owner.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Bytes {
    pub code: u64,
    pub data: u64,
    pub bss: u64,
}
impl Bytes {
    fn add(&mut self, kind: Kind, n: u64) {
        match kind {
            Kind::Code => self.code += n,
            Kind::Data => self.data += n,
            Kind::Bss => self.bss += n,
        }
    }
}

/// The size report of one image.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Size {
    /// Shared-SRAM image (code, read-only data, ISRAM/DSRAM load copies, `.shared`).
    pub image: u64,
    pub isram: u64,
    /// `.data` + `.bss` in each core's DSRAM.
    pub dsram_static: u64,
    pub stack0: u64,
    pub stack1: u64,
    /// The platform's part of `stack0` (`__stack_margin0`).
    pub margin: u64,
    /// By owner, in the order of [`OWNERS`].
    pub owners: [Bytes; 4],
}

impl Size {
    pub fn of(file: &object::File<'_>) -> Result<Self, String> {
        let symbol = |name| symbol(file, name);
        let image = symbol("__image_end")?
            .checked_sub(SRAM.0)
            .ok_or("__image_end below SRAM")?;
        let dsram_static = symbol("__ebss")?
            .checked_sub(DSRAM.0)
            .ok_or("__ebss below DSRAM")?;
        let (stack0, stack1) = (symbol("__stack_size0")?, symbol("__stack_size1")?);
        let margin = symbol("__stack_margin0")?;

        let mut owners = [Bytes::default(); 4];
        let mut isram = 0;
        for section in file.sections() {
            let name = section.name().map_err(|e| e.to_string())?;
            let kind = match name {
                ".vectors" | ".boot" | ".isram" | ".ARM.exidx" => Kind::Code,
                _ if name.starts_with(".text") => Kind::Code,
                ".data" | ".shared" => Kind::Data,
                ".bss" => Kind::Bss,
                _ => continue,
            };
            if name == ".isram" {
                isram = section.size();
            }
            // The vector tables and the reset code are rp1-rt's, whatever their symbols say.
            if matches!(name, ".vectors" | ".boot") {
                owners[0].add(kind, section.size());
                continue;
            }
            let mut attributed = 0;
            let mut seen = HashSet::new();
            for s in file.symbols() {
                if s.section_index() != Some(section.index())
                    || s.size() == 0
                    || s.kind() == object::SymbolKind::Section
                    || !seen.insert(s.address() & !1)
                {
                    continue;
                }
                let o = owner(s.name().unwrap_or(""));
                if o == Owner::Unattributed {
                    continue;
                }
                let index = OWNERS.iter().position(|(x, _)| *x == o).unwrap();
                owners[index].add(kind, s.size());
                attributed += s.size();
            }
            let rest = section.size().saturating_sub(attributed);
            owners[3].add(kind, rest);
        }
        Ok(Size {
            image,
            isram,
            dsram_static,
            stack0,
            stack1,
            margin,
            owners,
        })
    }

    /// One `size budget` line and one `size owner=` line per owner.
    pub fn lines(&self) -> Vec<String> {
        let free = |len: u32, used: u64| i64::from(len) - used as i64;
        let mut lines = vec![format!(
            "size budget image={}/{} sram_free={} isram={}/{} isram_free={} dsram_static={} \
             stack0={} stack1={} margin={} dsram_free0={} dsram_free1={}",
            self.image,
            SRAM.1,
            free(SRAM.1, self.image),
            self.isram,
            ISRAM.1,
            free(ISRAM.1, self.isram),
            self.dsram_static,
            self.stack0,
            self.stack1,
            self.margin,
            free(DSRAM.1, self.dsram_static + self.stack0),
            free(DSRAM.1, self.dsram_static + self.stack1),
        )];
        for ((_, name), b) in OWNERS.iter().zip(&self.owners) {
            lines.push(format!(
                "size owner={name} code={} data={} bss={}",
                b.code, b.data, b.bss
            ));
        }
        lines
    }

    /// Every budget has room left (the linker asserts the same; this is a cross-check).
    pub fn fits(&self) -> bool {
        self.image <= u64::from(SRAM.1)
            && self.isram <= u64::from(ISRAM.1)
            && self.dsram_static + self.stack0.max(self.stack1) <= u64::from(DSRAM.1)
    }
}

/// The value of an ELF symbol (an address, or a linker-script constant).
pub fn symbol(file: &object::File<'_>, name: &str) -> Result<u64, String> {
    file.symbols()
        .find(|s| s.name() == Ok(name))
        .map(|s| s.address())
        .ok_or_else(|| format!("missing symbol {name}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owners_from_symbol_names() {
        for (name, expected) in [
            (
                "_RNvMs_NtCshGUIoPiw6HO_7rp1_hal7loggingNtB4_6Logger5write",
                Owner::Platform,
            ),
            ("_RNvNtCs1a2b_6rp1_rt6status11record_fault", Owner::Platform),
            ("__rp1_default", Owner::Platform),
            (
                "_RNvNtCskAvI1dPvZKc_9irq_proof3rp13log5write",
                Owner::Platform,
            ),
            ("_RNvNtCshyNPUZkqTc0_4core3fmt5write", Owner::Runtime),
            ("_RNvNtNtCs0_17compiler_builtins3mem6memcpy", Owner::Runtime),
            ("__aeabi_memcpy", Owner::Runtime),
            ("memset", Owner::Runtime),
            ("_RNvCskAvI1dPvZKc_9irq_proof5main0", Owner::User),
            ("_RNvNtCskAvI1dPvZKc_9irq_proof5proc14rp1x", Owner::User),
            ("_RNvNtCskAvI1dPvZKc_9irq_proof4rp1x3run", Owner::User),
            // The panic handler is the firmware's (`#[panic_handler]`).
            (
                "_RNvCs5fGHjzWXRav_7___rustc17rust_begin_unwind",
                Owner::User,
            ),
            (
                "_ZN7rp1_hal4scmi5serve17h0123456789abcdefE",
                Owner::Platform,
            ),
            ("_ZN4core3fmt5write17h0123456789abcdefE", Owner::Runtime),
            ("my_no_mangle_fn", Owner::User),
            ("memory_test", Owner::User),
            ("memcmp", Owner::Runtime),
            (
                ".Lanon.9e35279fbf81762a22508ca34cf25b38.14",
                Owner::Unattributed,
            ),
            ("", Owner::Unattributed),
        ] {
            assert_eq!(owner(name), expected, "{name}");
        }
    }

    #[test]
    fn memory_map_matches_the_linker_script() {
        let script = include_str!("../../../rp1-rt/link.x");
        let region = |name: &str| -> (u64, u32) {
            let line = script
                .lines()
                .find(|l| l.trim_start().starts_with(&format!("{name} ")))
                .unwrap_or_else(|| panic!("no {name} in link.x"));
            let value = |key: &str| {
                let v = line
                    .split(key)
                    .nth(1)
                    .unwrap()
                    .trim_start_matches([' ', '=']);
                let v = v.split([',', ' ']).next().unwrap();
                match (v.strip_prefix("0x"), v.strip_suffix('K')) {
                    (Some(h), _) => u64::from_str_radix(h, 16).unwrap(),
                    (_, Some(k)) => k.parse::<u64>().unwrap() * 1024,
                    _ => v.parse().unwrap(),
                }
            };
            (value("ORIGIN"), value("LENGTH") as u32)
        };
        assert_eq!(region("SRAM"), SRAM);
        assert_eq!(region("ISRAM"), ISRAM);
        assert_eq!(region("DSRAM"), DSRAM);
    }

    #[test]
    fn budget_line_counts_free_space() {
        let size = Size {
            image: 43536,
            isram: 3876,
            dsram_static: 272,
            stack0: 2816,
            stack1: 2048,
            margin: 768,
            owners: [Bytes {
                code: 1,
                data: 2,
                bss: 3,
            }; 4],
        };
        let lines = size.lines();
        assert_eq!(
            lines[0],
            "size budget image=43536/49152 sram_free=5616 isram=3876/8192 isram_free=4316 \
             dsram_static=272 stack0=2816 stack1=2048 margin=768 dsram_free0=5104 \
             dsram_free1=5872"
        );
        assert_eq!(lines[1], "size owner=platform code=1 data=2 bss=3");
        assert_eq!(lines.len(), 5);
        assert!(size.fits());
        assert!(
            !Size {
                stack0: 8192,
                ..size
            }
            .fits()
        );
    }
}
