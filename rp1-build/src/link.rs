//! Generate the firmware's typed endpoints from the layout also encoded in the ELF note.

use crate::{Result, metadata::ShmLayout};
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
    let count = shm.regions.len();
    let digest: [u8; 32] = Sha256::digest(note).into();
    writeln!(
        output,
        "const AREA: ShmArea = ShmArea {{ size: {}, handoff: {:#x}, sram_end: {:#x}, phys: {:#x} }};\n\
         const DIGEST: [u8; 32] = {digest:?};\n\
         const REGIONS: [ShmRegion<'static>; {count}] = [",
        area.size, area.handoff, area.sram_end, area.phys
    )?;
    for region in &shm.regions {
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
    for (index, r) in shm.regions.iter().enumerate() {
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
    for core in 0..2 {
        let owned: Vec<_> = shm
            .regions
            .iter()
            .enumerate()
            .filter(|(_, r)| r.core == core + 1)
            .collect();
        if !owned.is_empty() {
            write_core(&mut output, core, &owned)?;
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

fn write_core(output: &mut String, core: u8, regions: &[(usize, &ShmRegion<'_>)]) -> Result<()> {
    writeln!(
        output,
        "/// Attached endpoints owned by proc{core}.\npub struct Proc{core}Link {{"
    )?;
    for &(_, r) in regions {
        let field = field_name(r.name)?;
        let ty = type_path(r.type_name);
        let endpoint = match r.kind {
            ShmKind::RingToLinux => "RingTx",
            ShmKind::RingToRp1 => "RingRx",
            ShmKind::Block { writer } => {
                let owner = match writer {
                    Writer::Rp1 => "RP1",
                    Writer::Linux => "Linux",
                    Writer::Both => "the peer holding writer ownership",
                };
                writeln!(
                    output,
                    "/// Block whose writer is {owner}. See [`transport::Block::write`]."
                )?;
                "Block"
            }
        };
        writeln!(output, "pub {field}: transport::{endpoint}<{ty}>,")?;
    }
    writeln!(
        output,
        "}}\n\
         /// The owning core's one right to attach these endpoints.\n\
         pub struct Proc{core}LinkInit {{ _core: ::core::marker::PhantomData<*mut ()> }}\n\
         impl Proc{core}LinkInit {{\n\
         pub(super) const fn new() -> Self {{ Self {{ _core: ::core::marker::PhantomData }} }}\n\
         /// The token can be retried while pending. A lost window requires an RP1 reload.\n\
         pub fn try_open(self) -> Result<Proc{core}Link, OpenError<Self>> {{\n\
         unsafe {{ generated::try_open(self, || Ok(Proc{core}Link {{"
    )?;
    for &(index, r) in regions {
        let field = field_name(r.name)?;
        let attach = match r.kind {
            ShmKind::RingToLinux => {
                "transport::RingTx::attach(ddr, r.ddr_len as usize, generated::sram(r), r.sram_len as usize, r.depth, generated::overflow(r))"
            }
            ShmKind::RingToRp1 => {
                "transport::RingRx::attach(generated::sram(r), r.sram_len as usize, ddr, r.ddr_len as usize, r.depth, generated::overflow(r))"
            }
            ShmKind::Block { .. } => "transport::Block::attach(ddr, r.ddr_len as usize)",
        };
        writeln!(
            output,
            "{field}: {{ let r = REGIONS[{index}]; let ddr = generated::ddr(r); {attach}? }},"
        )?;
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
