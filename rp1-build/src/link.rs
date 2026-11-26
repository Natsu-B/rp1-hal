//! Generate the firmware's typed endpoints from the layout also encoded in the ELF note.

use crate::{
    Result,
    metadata::{Endpoint, Pair, ShmLayout},
};
use rp1_proto::note::{ShmKind, ShmRegion, Writer};
use sha2::{Digest, Sha256};
use std::fmt::Write;

pub(crate) fn generate(shm: &ShmLayout<'_>, note: &[u8]) -> Result<String> {
    let mut output = String::from(
        "use ::rp1_hal::platform::Pcie;\n\
         use ::rp1_link::{self as transport, EnableError, Message, OpenError};\n\
         use ::rp1_link::generated::{self, ShmArea, ShmKind, ShmRegion};\n",
    );
    let area = shm.area;
    let regions = shm
        .regions
        .iter()
        .map(|r| r.wire())
        .collect::<Result<Vec<_>>>()?;
    let count = regions.len();
    let digest: [u8; 32] = Sha256::digest(note).into();
    writeln!(
        output,
        "const AREA: ShmArea = ShmArea {{ size: {}, handoff: {:#x}, sram_end: {:#x}, phys: {:#x} }};\n\
         const DIGEST: [u8; 32] = {digest:?};\n\
         const REGIONS: [ShmRegion<'static>; {count}] = [",
        area.size, area.handoff, area.sram_end, area.phys
    )?;
    for region in &regions {
        write_region(&mut output, region)?;
    }
    output.push_str("];\n");
    // Only these pairs and the digest need to occupy firmware memory.
    writeln!(
        output,
        "const SRAM_RANGES: [(u16, u16); {count}] = {{\n\
         let mut ranges = [(0, 0); {count}];\n\
         let mut i = 0;\n\
         while i < {count} {{\n\
             ranges[i] = (REGIONS[i].sram_offset, REGIONS[i].sram_len);\n\
             i += 1;\n\
         }}\n\
         ranges\n}};"
    )?;
    output.push_str(
        "/// Map and publish the declared reservation in response to Linux's request.\n\
         pub fn try_enable(pcie: &mut Pcie) -> Result<u32, EnableError> {\n\
             unsafe { generated::try_enable(pcie, AREA, &SRAM_RANGES, &DIGEST) }\n\
         }\n\
         #[doc(hidden)]\n\
         pub fn __map_for_probe(pcie: &mut Pcie) -> Result<u32, EnableError> {\n\
             unsafe { generated::map_probe(pcie, AREA) }\n\
         }\n",
    );
    for (index, r) in regions.iter().enumerate() {
        let ty = type_path(r.type_name);
        writeln!(
            output,
            "const _: () = assert!(::core::mem::size_of::<{ty}>() == REGIONS[{index}].elem_size as usize, {:?});\n\
             const _: () = assert!(<{ty} as Message>::TYPE_ID == REGIONS[{index}].type_id, {:?});",
            format!(
                "SHM region {} has a Rust type size different from its registration",
                r.name
            ),
            format!(
                "SHM region {} has a Rust TYPE_ID different from its registration",
                r.name
            )
        )?;
    }
    let aliases = aliases(&shm.endpoints)?;
    for (endpoint, alias) in shm.endpoints.iter().zip(&aliases) {
        let ty = match endpoint.pair {
            Some(pair) => pair_type(pair, &regions[endpoint.first], &regions[endpoint.first + 1]),
            None => endpoint_type(&regions[endpoint.first]),
        };
        writeln!(output, "pub type {alias} = {ty};")?;
    }
    for core in 0..2 {
        let owned: Vec<_> = shm
            .endpoints
            .iter()
            .zip(&aliases)
            .filter(|(endpoint, _)| regions[endpoint.first].core == core + 1)
            .collect();
        if !owned.is_empty() {
            write_core(&mut output, core, &regions, &owned)?;
        }
    }
    Ok(output)
}

fn write_region(output: &mut String, r: &ShmRegion<'_>) -> Result<()> {
    let kind = match r.kind {
        ShmKind::RingToLinux => "ShmKind::RingToLinux".to_owned(),
        ShmKind::RingToRp1 => "ShmKind::RingToRp1".to_owned(),
        ShmKind::Block { writer } => {
            format!("ShmKind::Block {{ writer: generated::Writer::{writer:?} }}")
        }
    };
    writeln!(
        output,
        "ShmRegion {{ kind: {kind}, core: {}, notify_irq: {}, overwrite: {},\n\
         ddr_offset: {:#x}, ddr_len: {:#x}, sram_offset: {:#x}, sram_len: {:#x},\n\
         elem_size: {}, depth: {}, type_id: {:#x}, name: {:?}, type_name: {:?} }},",
        r.core,
        r.notify_irq,
        r.overwrite,
        r.ddr_offset,
        r.ddr_len,
        r.sram_offset,
        r.sram_len,
        r.elem_size,
        r.depth,
        r.type_id,
        r.name,
        r.type_name
    )?;
    Ok(())
}

/// The endpoint type of `r`, with its notifier: what the field of `ProcNLink` holds.
fn endpoint_type(r: &ShmRegion<'_>) -> String {
    let ty = type_path(r.type_name);
    let core = r.core - 1;
    match (r.kind, r.notify_irq) {
        (ShmKind::RingToLinux, false) => format!("transport::RingTx<{ty}>"),
        (ShmKind::RingToLinux, true) => {
            format!("transport::RingTx<{ty}, transport::notify::HostIrq>")
        }
        (ShmKind::RingToRp1, false) => format!("transport::RingRx<{ty}>"),
        (ShmKind::RingToRp1, true) => format!("transport::RingRx<{ty}, super::Doorbell{core}>"),
        (ShmKind::Block { .. }, _) => format!("transport::Block<{ty}>"),
    }
}

fn pair_type(pair: Pair, down: &ShmRegion<'_>, up: &ShmRegion<'_>) -> String {
    let (kind, first, second) = match pair {
        Pair::Channel => ("Channel", up, down),
        Pair::Rpc => ("Server", down, up),
    };
    let ty = type_path(first.type_name);
    let peer = type_path(second.type_name);
    let mut args = format!("{ty}, {peer}");
    if up.notify_irq {
        args.push_str(", transport::notify::HostIrq");
    } else if down.notify_irq {
        args.push_str(", transport::notify::Poll");
    }
    if down.notify_irq {
        let core = down.core - 1;
        args.push_str(&format!(", super::Doorbell{core}"));
    }
    format!("transport::{kind}<{args}>")
}

/// Names the generated link module already uses, which an alias must not take.
const RESERVED_ALIASES: [&str; 11] = [
    "Pcie",
    "EnableError",
    "Message",
    "OpenError",
    "ShmArea",
    "ShmKind",
    "ShmRegion",
    "Proc0Link",
    "Proc0LinkInit",
    "Proc1Link",
    "Proc1LinkInit",
];

/// The type alias of each region: its name in UpperCamelCase (`wide_64` -> `Wide64`).
fn aliases(regions: &[Endpoint<'_>]) -> Result<Vec<String>> {
    let mut aliases: Vec<String> = Vec::new();
    for r in regions {
        let alias: String = r
            .name
            .split('_')
            .map(|word| {
                let mut chars = word.chars();
                chars.next().map_or_else(String::new, |first| {
                    first.to_ascii_uppercase().to_string() + chars.as_str()
                })
            })
            .collect();
        if RESERVED_ALIASES.contains(&alias.as_str()) {
            return Err(format!(
                "SHM name {:?} gives the type alias {alias}, which the generated code uses; \
                 rename this region",
                r.name
            )
            .into());
        }
        if let Some(other) = aliases.iter().position(|a| *a == alias) {
            return Err(format!(
                "SHM names {:?} and {:?} both give the type alias {alias}; rename one",
                regions[other].name, r.name
            )
            .into());
        }
        aliases.push(alias);
    }
    Ok(aliases)
}

fn write_core(
    output: &mut String,
    core: u8,
    regions: &[ShmRegion<'_>],
    endpoints: &[(&Endpoint<'_>, &String)],
) -> Result<()> {
    writeln!(
        output,
        "/// Attached endpoints owned by proc{core}.\npub struct Proc{core}Link {{"
    )?;
    for &(endpoint, alias) in endpoints {
        if let ShmKind::Block { writer } = regions[endpoint.first].kind {
            let owner = match writer {
                Writer::Rp1 => "RP1",
                Writer::Linux => "Linux",
                Writer::Both => "the peer holding writer ownership",
            };
            writeln!(
                output,
                "/// Block whose writer is {owner}. See [`transport::Block::write`]."
            )?;
        }
        writeln!(output, "pub {}: {alias},", field_name(endpoint.name)?)?;
    }
    writeln!(
        output,
        "}}\n\
         /// The owning core's one right to attach these endpoints.\n\
         pub struct Proc{core}LinkInit {{ _core: ::core::marker::PhantomData<*mut ()> }}\n\
         impl Proc{core}LinkInit {{\n\
         pub(super) const fn new() -> Self {{ Self {{ _core: ::core::marker::PhantomData }} }}\n\
         /// The token can be retried while pending. A lost window requires an RP1 reload.\n\
         /// A doorbell ring is armed when this returns: give it to its handler, then raise the doorbell once ([`transport::RingRx`]).\n\
         pub fn try_open(self) -> Result<Proc{core}Link, OpenError<Self>> {{\n\
         unsafe {{ generated::try_open(self, || Ok(Proc{core}Link {{"
    )?;
    for &(endpoint, _) in endpoints {
        let index = endpoint.first;
        let field = field_name(endpoint.name)?;
        match endpoint.pair {
            Some(pair) => {
                let constructor = match pair {
                    Pair::Channel => "channel",
                    Pair::Rpc => "server",
                };
                let up = index + 1;
                writeln!(
                    output,
                    "{field}: generated::{constructor}(generated::ring_tx(REGIONS[{up}])?, generated::ring_rx(REGIONS[{index}])?),"
                )?;
            }
            None => {
                let attach = match regions[index].kind {
                    ShmKind::RingToLinux => "ring_tx",
                    ShmKind::RingToRp1 => "ring_rx",
                    ShmKind::Block { .. } => "block",
                };
                writeln!(output, "{field}: generated::{attach}(REGIONS[{index}])?,")?;
            }
        }
    }
    output.push_str("})) }\n}\n}\n");
    Ok(())
}

fn field_name(name: &str) -> Result<String> {
    let field = match name {
        "crate" | "self" | "super" => {
            return Err(
                format!("SHM name {name:?} cannot be a Rust field; rename this region").into(),
            );
        }
        "abstract" | "as" | "async" | "await" | "become" | "box" | "break" | "const"
        | "continue" | "do" | "dyn" | "else" | "enum" | "extern" | "false" | "final" | "fn"
        | "for" | "gen" | "if" | "impl" | "in" | "let" | "loop" | "macro" | "match" | "mod"
        | "move" | "mut" | "override" | "priv" | "pub" | "ref" | "return" | "static" | "struct"
        | "trait" | "true" | "try" | "type" | "typeof" | "unsafe" | "unsized" | "use"
        | "virtual" | "where" | "while" | "yield" => format!("r#{name}"),
        _ => name.to_owned(),
    };
    Ok(field)
}

fn type_path(name: &str) -> String {
    if name.starts_with("crate::") || name.starts_with("::") {
        name.to_owned()
    } else if name.contains("::") {
        format!("::{name}")
    } else {
        format!("crate::{name}")
    }
}

#[cfg(test)]
mod tests;
