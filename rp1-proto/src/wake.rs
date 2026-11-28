//! Experimental link-wake ABI, shared by NOTE, link SRAM and the SHM header.

pub const ABI: u16 = 1;
pub const LATCHED_WAKE: u32 = 1 << 0;
pub const STARTUP_WAKE: u32 = 1 << 1;
pub const CREDIT_WAKE: u32 = 1 << 2;
pub const RPC_REPLY_WAKE: u32 = 1 << 3;
pub const KNOWN_CAPS: u32 = LATCHED_WAKE | STARTUP_WAKE | CREDIT_WAKE | RPC_REPLY_WAKE;
/// Capabilities offered by the IRQ profile with request-carried reply wake.
pub const IRQ_RPC_CAPS: u32 = LATCHED_WAKE | RPC_REPLY_WAKE;
pub const IRQ_STARTUP_CAPS: u32 = IRQ_RPC_CAPS | STARTUP_WAKE;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum StartupError {
    Unsupported = 0x11,
    /// Reserved for compatibility with earlier v3 firmware.
    StaleRequest = 0x12,
    InvalidRequest = 0x13,
    Timeout = 0x14,
    Initialization = 0x15,
}

impl StartupError {
    pub fn from_status(status: u32) -> Option<Self> {
        [
            Self::Unsupported,
            Self::StaleRequest,
            Self::InvalidRequest,
            Self::Timeout,
            Self::Initialization,
        ]
        .into_iter()
        .find(|&error| error as u32 == status)
    }
}

/// Link-page offsets. Every field is an aligned LE u32 with one writer.
pub mod mailbox {
    pub const START: usize = crate::status::END;
    pub const LEN: usize = 64;
    pub const END: usize = START + LEN;

    // A copy of status::EPOCH. proc0 commits the offered caps by publishing this
    // word last, so a reader can distinguish mailbox admission from status startup.
    pub const BOOT_EPOCH: usize = START;
    pub const OFFERED_CAPS: usize = START + 4;
    // Linux commits an empty slot with link::SHM_REQUEST and a stale slot with REQUEST_EPOCH.
    pub const REQUEST_EPOCH: usize = START + 8;
    pub const REQUEST_SEQ: usize = START + 12;
    pub const REQUIRED_CAPS: usize = START + 16;
    // proc0 commits these fields by publishing link::SHM_STATUS.
    pub const ACCEPTED_CAPS: usize = START + 20;
    pub const RESULT_SEQ: usize = START + 24;
    // Linux owns WAIT and ACK. The host IRQ owner owns SEQ.
    pub const WAIT: usize = START + 28;
    pub const SEQ: usize = START + 32;
    pub const ACK: usize = START + 36;
    pub const REQUEST_OPCODE: u32 = 3;
    pub const FIRST_REQUEST: u32 = 1;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionError {
    NotReady,
    Restarted,
    Exhausted,
    Unsupported,
}

/// Startup and terminal notification tickets belong to a boot even before SHM is Ready.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BootEpoch(u32);

impl BootEpoch {
    pub fn new(word: u32) -> Result<Self, SessionError> {
        if word == 0 || word & 1 != 0 {
            return Err(SessionError::NotReady);
        }
        Ok(Self(word))
    }

    pub fn word(self) -> u32 {
        self.0
    }
}

/// An attachment is scoped to a boot and a committed (even) SHM generation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Session {
    epoch: BootEpoch,
    generation: u32,
}
impl Session {
    pub fn new(epoch: u32, generation: u32) -> Result<Self, SessionError> {
        let epoch = BootEpoch::new(epoch)?;
        if generation == 0 || generation & 1 != 0 {
            return Err(SessionError::NotReady);
        }
        Ok(Self { epoch, generation })
    }

    pub fn epoch(self) -> u32 {
        self.epoch.word()
    }

    pub fn generation(self) -> u32 {
        self.generation
    }

    /// Even generation to commit after publishing its preceding odd value.
    pub fn next_generation(previous: u32) -> Result<u32, SessionError> {
        (previous | 1).checked_add(1).ok_or(SessionError::Exhausted)
    }
}

/// Fields acquired after the request commit word. Duplicate handling belongs to the
/// startup owner, which caches the committed result for this request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Request {
    pub epoch: u32,
    pub seq: u32,
    pub required_caps: u32,
}

impl Request {
    pub fn accept(self, opcode: u32, epoch: u32, offered: u32) -> Result<u32, SessionError> {
        if opcode != mailbox::REQUEST_OPCODE || self.required_caps & !(offered & KNOWN_CAPS) != 0 {
            return Err(SessionError::Unsupported);
        }
        if self.epoch != epoch {
            return Err(SessionError::Restarted);
        }
        if self.seq == 0 || epoch == 0 || epoch & 1 != 0 {
            return Err(SessionError::NotReady);
        }
        Ok(self.required_caps)
    }

    /// Match the result fields acquired after SHM_STATUS; its status is interpreted by the caller.
    pub fn check_result(
        self,
        epoch: u32,
        result_seq: u32,
        accepted_caps: u32,
        offered: u32,
    ) -> Result<(), SessionError> {
        let required = self.accept(mailbox::REQUEST_OPCODE, epoch, offered)?;
        if self.seq != result_seq {
            return Err(SessionError::NotReady);
        }
        if accepted_caps & required != required || accepted_caps & !(offered & KNOWN_CAPS) != 0 {
            return Err(SessionError::Unsupported);
        }
        Ok(())
    }
}
