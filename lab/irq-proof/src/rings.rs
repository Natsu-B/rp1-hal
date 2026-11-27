//! The notification proofs on shm rings: what the doorbell handlers do with the Linux -> RP1
//! rings (`down1` on proc1, `down0` on proc0), what proc0 sends to Linux (`events0` from main,
//! `tick0` from its TIMER_0 handler), and the commands that drive both from shm-linux.
//!
//! Every message is `lab/irq_message.rs`'s. The counters are ordinary statics of the core that
//! serves the ring (its private DSRAM), and a command copies them into the control block.
use crate::irq_message::{self, Message};
use crate::proof::{Shared, now, with};
use rp1_link::{RingRx, RingTx, notify::Doorbell};
use rp1_proto::shm::Progress;

/// Request word 1 of the control blocks (`control` for proc1, `control0` for proc0).
pub mod command {
    /// Reset the Linux -> RP1 ring's counters: budget per handler entry `count`, run generation
    /// `seed`. Both blocks.
    pub const DOWN_START: u32 = 20;
    /// The Linux -> RP1 ring's counters into words 8..14 ([`super::Tally::words`]). Both blocks.
    pub const DOWN_REPORT: u32 = 21;
    /// proc0: send `count` messages on `events0` from main, `seed` = generation, word 8 = batch,
    /// word 9 = gap between batches in µs. Answered when all are sent.
    pub const EVENTS: u32 = 22;
    /// proc0: start sending `count` messages on `tick0` from the TIMER_0 handler, one every word
    /// 8 µs, `seed` = generation. Answered at once.
    pub const TICKS: u32 = 23;
    /// proc0: the tick counters into words 8..11 ([`super::Ticks::words`]).
    pub const TICK_REPORT: u32 = 24;
    /// [`DOWN_START`] for the control run (j2): the handler drains without the readback of its
    /// arm that the protocol needs. Both blocks.
    pub const DOWN_START_WITHOUT_READBACK: u32 = 25;
}

/// What the handler saw on a Linux -> RP1 ring since the last `DOWN_START`.
#[derive(Clone, Copy, Default)]
pub struct Tally {
    /// Whole messages of this run, in order.
    pub received: u32,
    /// Messages of this run that are not whole or not the next in order.
    pub corrupt: u32,
    /// Whole messages of another generation: left by the writer of an earlier run.
    pub stale: u32,
    /// Handler entries with this core's doorbell set.
    pub entries: u32,
    /// Times a drain ran out of budget and raised the doorbell again.
    pub raises: u32,
    /// Drains that returned an error.
    pub errors: u32,
}
impl Tally {
    pub fn words(&self) -> [u32; 6] {
        [
            self.received,
            self.corrupt,
            self.stale,
            self.entries,
            self.raises,
            self.errors,
        ]
    }
}

/// A Linux -> RP1 ring as its doorbell handler drains it.
pub struct Reader<const PIN: u8, const CORE: u8> {
    ring: RingRx<Message, Doorbell<PIN, CORE>>,
    tally: Tally,
    /// The run's generation and the `seq` expected next.
    generation: u32,
    next: u32,
    /// Messages per handler entry before it raises the doorbell again.
    budget: u32,
    /// `false` only for the control run (j2).
    readback: bool,
}

/// The budget until a `DOWN_START` gives another one.
const DEFAULT_BUDGET: u32 = 16;

impl<const PIN: u8, const CORE: u8> Reader<PIN, CORE> {
    pub fn new(ring: RingRx<Message, Doorbell<PIN, CORE>>) -> Self {
        Self {
            ring,
            tally: Tally::default(),
            generation: 0,
            next: 0,
            budget: DEFAULT_BUDGET,
            readback: true,
        }
    }

    /// The handler's part, after it acknowledged `bell`: drain up to the budget and check every
    /// message. When the budget runs out the ring is not armed, so raise the doorbell: the
    /// handler runs again for the rest.
    pub fn on_doorbell(&mut self, bell: Doorbell<PIN, CORE>) {
        if self.drain() {
            bell.raise();
        }
    }

    /// Drain one budget after the ack, returning whether the handler needs another entry.
    pub fn drain(&mut self) -> bool {
        self.tally.entries += 1;
        let (tally, generation, next) = (&mut self.tally, self.generation, &mut self.next);
        let check = |m: Message| {
            if !irq_message::is_whole(&m) {
                tally.corrupt += 1;
            } else if m[2] != generation {
                tally.stale += 1;
            } else if m[0] != *next {
                tally.corrupt += 1;
                *next = m[0].wrapping_add(1);
            } else {
                tally.received += 1;
                *next = m[0].wrapping_add(1);
            }
        };
        let progress = if self.readback {
            self.ring.drain(self.budget, check)
        } else {
            self.ring.__drain_without_readback(self.budget, check)
        };
        match progress {
            Ok(Progress::Idle) => false,
            Ok(Progress::More) => {
                self.tally.raises += 1;
                true
            }
            Err(_) => {
                self.tally.errors += 1;
                false
            }
        }
    }

    /// `DOWN_START` / `DOWN_START_WITHOUT_READBACK`.
    pub fn start(&mut self, budget: u32, generation: u32, readback: bool) {
        self.tally = Tally::default();
        self.generation = generation;
        self.next = 0;
        self.budget = budget;
        self.readback = readback;
    }

    pub fn tally(&self) -> Tally {
        self.tally
    }
}

/// Serves `DOWN_START`, `DOWN_REPORT` and `DOWN_START_WITHOUT_READBACK` for the reader in
/// `slot` (the handler's), answering in `request`. Returns `None` for any other command, else
/// the errors: 1 while the reader is not open yet.
pub fn down_command<const PIN: u8, const CORE: u8>(
    slot: &Shared<Reader<PIN, CORE>, CORE>,
    request: &mut [u32; 16],
) -> Option<u32> {
    let (command, count, seed) = (request[1], request[2], request[3]);
    if !matches!(
        command,
        command::DOWN_START | command::DOWN_REPORT | command::DOWN_START_WITHOUT_READBACK
    ) {
        return None;
    }
    let mut errors = 1;
    with(slot, |reader| {
        errors = 0;
        match command {
            command::DOWN_START => reader.start(count, seed, true),
            command::DOWN_START_WITHOUT_READBACK => reader.start(count, seed, false),
            _ => request[8..14].copy_from_slice(&reader.tally().words()),
        }
    });
    Some(errors)
}

/// proc0's `tick0` writer, used by its TIMER_0 handler: one message per tick.
pub struct Ticks {
    ring: RingTx<Message, rp1_link::notify::HostIrq>,
    generation: u32,
    /// Messages sent in this run; the next one's `seq`.
    sent: u32,
    /// Ticks whose message found the ring full (it is not sent again).
    full: u32,
    errors: u32,
}
impl Ticks {
    pub fn new(ring: RingTx<Message, rp1_link::notify::HostIrq>) -> Self {
        Self {
            ring,
            generation: 0,
            sent: 0,
            full: 0,
            errors: 0,
        }
    }
    pub fn start(&mut self, generation: u32) {
        self.generation = generation;
        self.sent = 0;
        self.full = 0;
        self.errors = 0;
    }
    pub fn on_tick(&mut self) {
        match self
            .ring
            .send(&irq_message::message(self.sent, self.generation))
        {
            Ok(()) => self.sent += 1,
            Err(rp1_proto::Error::Full) => self.full += 1,
            Err(_) => self.errors += 1,
        }
    }
    pub fn words(&self) -> [u32; 3] {
        [self.sent, self.full, self.errors]
    }
}

/// `EVENTS`: `count` messages on `events0`, `batch` per `send_slice`, `gap_us` between batches.
/// Returns the errors (a stream that took over 180 s counts as one).
pub fn send_events(
    tx: &mut RingTx<Message, rp1_link::notify::HostIrq>,
    count: u32,
    generation: u32,
    batch: u32,
    gap_us: u32,
) -> u32 {
    const MAX_BATCH: usize = 32;
    let batch = (batch as usize).clamp(1, MAX_BATCH);
    let start = now();
    let mut buf = [[0u32; 4]; MAX_BATCH];
    let mut seq = 0;
    while seq < count {
        let k = (count - seq).min(batch as u32) as usize;
        for (j, slot) in buf[..k].iter_mut().enumerate() {
            *slot = irq_message::message(seq + j as u32, generation);
        }
        let mut done = 0;
        while done < k {
            match tx.send_slice(&buf[done..k]) {
                Ok(n) => done += n,
                Err(_) => return 1,
            }
            if now().wrapping_sub(start) > 180_000_000 {
                return 1;
            }
        }
        seq += k as u32;
        let sent_at = now();
        while now().wrapping_sub(sent_at) < gap_us {}
    }
    0
}
