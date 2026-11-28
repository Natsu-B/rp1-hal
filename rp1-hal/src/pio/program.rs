use super::{BASE, PIO};
use crate::{FreeError, write};
use ::pio::{
    InSource, InstructionOperands as Op, IrqIndexMode, JmpCondition, MovDestination, MovSource,
    OutDestination, PioVersion, Program, SetDestination, SideSet, WaitSource,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InstallError {
    InvalidProgram,
    UnsupportedInstruction,
    OriginOccupied,
    NoSpace,
    InUse,
}

#[derive(Default)]
pub(super) struct Requirements {
    pub(super) relative_in: u32,
    pub(super) absolute_in: u32,
    pub(super) jmp_pin: bool,
    pub(super) set_pins: bool,
    pub(super) out_pins: bool,
    pub(super) flags: u8,
    pub(super) relative_flags: u8,
}

/// An allocation in PIO instruction memory, shareable by multiple SMs on the owning core.
pub struct InstalledProgram<'pio> {
    pub(super) pio: &'pio PIO,
    pub(super) requirements: Requirements,
    pub(super) mask: u32,
    pub(super) offset: u8,
    pub(super) wrap_top: u8,
    pub(super) wrap_bottom: u8,
    pub(super) side_set: SideSet,
}

impl PIO {
    /// Validate and relocate a PIO v0 program before writing instruction memory.
    pub fn install(&self, program: &Program<32>) -> Result<InstalledProgram<'_>, InstallError> {
        self.install_with(program, |index, word| {
            write(BASE + 0x4c + index * 4, word.into())
        })
    }

    pub(super) fn install_with<const L: usize>(
        &self,
        program: &Program<L>,
        mut write_instruction: impl FnMut(usize, u16),
    ) -> Result<InstalledProgram<'_>, InstallError> {
        let requirements = validate(program)?;
        let len = program.code.len();
        let used = self.used.get();
        let (offset, mask) = place(used, len, program.origin)?;
        for (index, &word) in program.code.iter().enumerate() {
            write_instruction(index + usize::from(offset), relocate(word, offset));
        }
        self.used.set(used | mask);
        Ok(InstalledProgram {
            pio: self,
            requirements,
            mask,
            offset,
            wrap_top: program.wrap.source + offset,
            wrap_bottom: program.wrap.target + offset,
            side_set: program.side_set,
        })
    }
}

impl InstalledProgram<'_> {
    /// Address of the first instruction in PIO memory.
    pub const fn offset(&self) -> u8 {
        self.offset
    }

    /// Reclaim the allocation after every SM has returned both of its FIFO owners.
    pub fn uninstall(self) -> Result<(), FreeError<Self, InstallError>> {
        if self
            .pio
            .active
            .iter()
            .any(|allocation| allocation.get() & self.mask != 0)
        {
            return Err(FreeError::new(self, InstallError::InUse));
        }
        self.pio.used.set(self.pio.used.get() & !self.mask);
        Ok(())
    }
}

fn place(used: u32, len: usize, origin: Option<u8>) -> Result<(u8, u32), InstallError> {
    let mask = u32::MAX >> (32 - len);
    let end = (32 - len) as u8;
    if let Some(offset) = origin {
        if offset > end {
            return Err(InstallError::InvalidProgram);
        }
        if used & (mask << offset) != 0 {
            return Err(InstallError::OriginOccupied);
        }
        return Ok((offset, mask << offset));
    }
    (0..=end)
        .rev()
        .find_map(|offset| {
            let mask = mask << offset;
            (used & mask == 0).then_some((offset, mask))
        })
        .ok_or(InstallError::NoSpace)
}

fn relocate(word: u16, offset: u8) -> u16 {
    // JMP was already checked against the program length, even with a fixed origin.
    if word >> 13 == 0 {
        word + u16::from(offset)
    } else {
        word
    }
}

fn validate<const L: usize>(program: &Program<L>) -> Result<Requirements, InstallError> {
    use InstallError::{InvalidProgram, UnsupportedInstruction};

    let len = program.code.len();
    let side = program.side_set;
    let (top, bottom) = (program.wrap.source as usize, program.wrap.target as usize);
    if len == 0
        || len > 32
        || bottom > top
        || top >= len
        || side.bits() > 5
        || ((side.optional() || side.pindirs()) && side.bits() <= u8::from(side.optional()))
    {
        return Err(InvalidProgram);
    }
    if program.version != PioVersion::V0 {
        return Err(UnsupportedInstruction);
    }

    let mut r = Requirements::default();
    for (pc, &word) in program.code.iter().enumerate() {
        let op = Op::decode(word).ok_or(UnsupportedInstruction)?;
        // The last instruction must wrap or unconditionally branch back into this allocation.
        if pc + 1 == len
            && pc != top
            && !matches!(
                op,
                Op::JMP {
                    condition: JmpCondition::Always,
                    ..
                }
            )
        {
            return Err(InvalidProgram);
        }
        match op {
            Op::JMP { condition, address } => {
                if usize::from(address) >= len {
                    return Err(InvalidProgram);
                }
                r.jmp_pin |= condition == JmpCondition::PinHigh;
            }
            Op::WAIT { source: WaitSource::GPIO, index, .. } => r.absolute_in |= 1 << index,
            Op::WAIT { source: WaitSource::PIN, index, .. } => r.relative_in |= 1 << index,
            Op::WAIT { source: WaitSource::IRQ, index, relative, .. } if word & 8 == 0 => {
                if relative { r.relative_flags |= 1 << index; } else { r.flags |= 1 << index; }
            }
            Op::WAIT { .. }
            | Op::OUT { destination: OutDestination::EXEC | OutDestination::PC, .. }
            | Op::MOV {
                destination: MovDestination::EXEC | MovDestination::PC | MovDestination::PINDIRS,
                ..
            }
            // V0 MOV PINS reads all 32 GPIO inputs without an input count.
            | Op::MOV { source: MovSource::PINS, .. }
            | Op::MOVTORX { .. }
            | Op::MOVFROMRX { .. } => return Err(UnsupportedInstruction),
            Op::PUSH { .. } | Op::PULL { .. } if word & 15 != 0 => return Err(UnsupportedInstruction),
            Op::IRQ { index, index_mode, clear, wait } => {
                if clear && wait {
                    return Err(UnsupportedInstruction);
                }
                match index_mode {
                    IrqIndexMode::DIRECT => r.flags |= 1 << index,
                    IrqIndexMode::REL => r.relative_flags |= 1 << index,
                    _ => return Err(UnsupportedInstruction),
                }
            }
            Op::IN { source: InSource::PINS, bit_count } => r.relative_in |= u32::MAX >> (32 - bit_count),
            Op::SET { destination: SetDestination::PINS | SetDestination::PINDIRS, .. } => r.set_pins = true,
            Op::OUT { destination: OutDestination::PINS | OutDestination::PINDIRS, .. }
            | Op::MOV { destination: MovDestination::PINS, .. } => r.out_pins = true,
            _ => {}
        }
    }
    Ok(r)
}

#[cfg(test)]
#[path = "program_tests.rs"]
mod tests;
