extern crate std;
use super::*;
use std::{collections::BTreeMap, vec::Vec};

/// A fake that retains status separately from CHEN and can refuse disable indefinitely.
#[derive(Default)]
struct Fake {
    regs: BTreeMap<usize, u32>,
    writes: Vec<(usize, u32)>,
    now: u64,
    reads: usize,
    stuck: bool,
    refuse_stop: bool,
    refuse_ack: bool,
}
impl Fake {
    fn stop(&mut self, n: u8, status: u32) {
        *self.regs.entry(BASE + common::CHEN).or_default() &= !(1 << n);
        *self
            .regs
            .entry(channel_base(n) + ch::INTSTATUS)
            .or_default() |= status;
    }
}
impl Registers for Fake {
    fn read(&mut self, a: usize) -> u32 {
        self.reads += 1;
        match a {
            _ if a == BASE + common::RESET => 0,
            RESETS_DONE0 => RESET_BIT,
            0x400a_c024 => (self.now >> 32) as u32,
            0x400a_c028 => {
                self.now += 1;
                self.now as u32
            }
            _ => *self.regs.get(&a).unwrap_or(&0),
        }
    }

    fn write(&mut self, a: usize, v: u32) {
        self.writes.push((a, v));
        match a {
            _ if a == BASE + common::CHEN => {
                for n in 0..8u8 {
                    let en = 1 << n;
                    if v & (en << common::CHEN_WE_SHIFT) == 0 {
                        continue;
                    }
                    if v & en == 0 {
                        if !self.refuse_stop {
                            self.stop(n, int::CH_DISABLED);
                        }
                    } else {
                        *self.regs.entry(a).or_default() |= en;
                        if !self.stuck {
                            self.stop(n, int::DMA_TFR_DONE | int::BLOCK_TFR_DONE);
                        }
                    }
                }
            }
            _ if a >= BASE + CHANNEL_BASE && (a - BASE) % CHANNEL_STRIDE == ch::INTCLEAR => {
                if !self.refuse_ack {
                    *self
                        .regs
                        .entry(a - ch::INTCLEAR + ch::INTSTATUS)
                        .or_default() &= !v;
                }
            }
            _ => {
                self.regs.insert(a, v);
            }
        }
    }
}
fn running() -> Fake {
    let mut f = Fake::default();
    start(&mut f).unwrap();
    f.writes.clear();
    f
}
fn reg(f: &Fake, a: usize) -> u32 {
    f.writes.iter().rev().find(|w| w.0 == a).unwrap().1
}

#[test]
fn address_map_follows_the_datasheet() {
    assert_eq!(BusAddr::from_proc(0x2000_0000), Ok(BusAddr(0xc0_2000_0000)));
    assert_eq!(BusAddr::from_proc(0x2000_fffc), Ok(BusAddr(0xc0_2000_fffc)));
    assert_eq!(BusAddr::from_proc(0x4005_0060), Ok(BusAddr(0xc0_4005_0060)));
    assert_eq!(BusAddr::from_proc(0x8100_0000), Ok(BusAddr(0x80_0100_0000)));
    assert_eq!(BusAddr::host_phys(0x2200_0000), BusAddr(0x10_2200_0000));
    for local in [
        0,
        0x1000_0000,
        0x1000_2000,
        0x2001_0000,
        0x4040_0000,
        0xc000_0000,
    ] {
        assert_eq!(
            BusAddr::from_proc(local),
            Err(Error::NotDmaVisible),
            "{local:#x}"
        );
    }
}

#[test]
fn width_follows_alignment_and_caps_at_128_bits() {
    assert_eq!(width_log2(0, 0, 4096), 4);
    assert_eq!(width_log2(4, 0, 4096), 2);
    assert_eq!(width_log2(0, 0, 6), 1);
    assert_eq!(width_log2(1, 0, 16), 0);
    assert_eq!(max_block_bytes(4), MAX_BLOCK_TS << 4);
}

#[test]
fn init_starts_clock_releases_reset_and_enables_controller() {
    let mut f = Fake::default();
    start(&mut f).unwrap();
    assert!(f.writes.contains(&(RESETS_CLR0, RESET_BIT)));
    assert!(f.writes.contains(&(BASE + common::RESET, 1)));
    assert_eq!(reg(&f, BASE + common::CFG), common::CFG_DMAC_EN);
    assert_eq!(reg(&f, BASE + common::CHEN), 0xff00);
    clock_current(&mut f).unwrap();
}

#[test]
fn single_block_programs_linux_compatible_fields() {
    let mut f = running();
    let mut state = State::Idle;
    start_block(
        &mut f,
        2,
        &mut state,
        BusAddr(0xc0_2000_1000),
        BusAddr(0x80_0100_0040),
        0x400,
    )
    .unwrap();
    let c = channel_base(2);
    assert_eq!(c, 0x4018_8300);
    assert_eq!(reg(&f, c + ch::SAR), 0x2000_1000);
    assert_eq!(reg(&f, c + ch::SAR + 4), 0xc0);
    assert_eq!(reg(&f, c + ch::DAR + 4), 0x80);
    assert_eq!(reg(&f, c + ch::BLOCK_TS), 0x400 / 16 - 1);
    assert_eq!(
        reg(&f, c + ch::CTL_L),
        ctl_l::NONPOSTED_LAST_WRITE_EN | (1 << 18) | (1 << 14) | (4 << 11) | (4 << 8)
    );
    assert_eq!(
        reg(&f, c + ch::CTL_H),
        (1 << 6) | (3 << 7) | (1 << 15) | (3 << 16)
    );
    assert_eq!(reg(&f, c + ch::CFG_L), 0);
    assert_eq!(reg(&f, c + ch::CFG_H), (2 << 20) | (1 << 3) | (1 << 4));
    assert_eq!(reg(&f, BASE + common::CHEN), (1 << 2) | (1 << 10));
    assert_eq!(observe(&mut f, 2, &mut state), Some(Ok(())));
}

#[test]
fn stale_clock_is_a_precondition_error() {
    let mut f = running();
    clock_current(&mut f).unwrap();
    let ctrl = generator().ctrl;
    f.regs.insert(ctrl, 0);
    assert_eq!(clock_current(&mut f), Err(Error::Precondition));
}

fn channel<const N: u8>() -> Channel<N> {
    Channel {
        state: State::Idle,
        _local: core::marker::PhantomData,
    }
}

#[test]
fn failed_admission_preserves_both_leases_and_does_not_enable() {
    let mut io = running();
    let mut channel = channel::<2>();
    let pool = pool::fake_pool::<2>();
    let failure = channel
        .start_leased(
            &mut io,
            pool.lease(0, 64).unwrap(),
            pool.lease(1, 4).unwrap(),
        )
        .err()
        .unwrap();
    assert_eq!(failure.error, Error::Invalid);
    assert!(io.writes.is_empty());
    assert!(matches!(pool.lease(0, 64), Err(Error::Busy)));
    drop(failure.resources);
    assert!(pool.lease(0, 64).is_ok());
    assert!(pool.lease(1, 4).is_ok());
    let start = BusAddr(BusAddr::SRAM);
    for (src, dst, len) in [
        (start, start, 16),
        (start, BusAddr(start.0 + 8), 16),
        (start, BusAddr(start.0 + 80), 0),
        (
            start,
            BusAddr(start.0 + 0x100_0000),
            max_block_bytes(4) + 16,
        ),
        (start, BusAddr((1 << 40) - 8), 16),
    ] {
        assert_eq!(
            start_block(&mut io, 2, &mut channel.state, src, dst, len),
            Err(Error::Invalid)
        );
    }
    assert!(io.writes.is_empty());
}

#[test]
fn pool_excludes_remote_peripheral_stack_and_sram_crossings() {
    for (addr, len) in [
        (0x8000_0000, 64),
        (0x4005_0060, 64),
        (0x1000_1000, 64),
        (0x2000_fff0, 64),
        (usize::MAX, 16),
    ] {
        assert_eq!(pool::sram(addr, len), Err(Error::NotDmaVisible));
    }
    assert_eq!(
        pool::sram(0x2000_fff0, 16),
        Ok(BusAddr(BusAddr::SRAM + 0xfff0))
    );
}

#[test]
fn leases_are_unique_and_released_exactly_once() {
    let pool = pool::fake_pool::<2>();
    let mut first = pool.lease(0, 4096).unwrap();
    assert!(matches!(pool.lease(0, 1), Err(Error::Busy)));
    assert_eq!(first.set_len(4097), Err(Error::Invalid));
    first.set_len(4).unwrap();
    assert!(matches!(pool.lease(0, 1), Err(Error::Busy)));
    drop(first);
    let second = pool.lease(0, 1).unwrap();
    assert!(matches!(pool.lease(0, 1), Err(Error::Busy)));
    core::mem::forget(second);
    assert!(matches!(pool.lease(0, 1), Err(Error::Busy)));
    assert!(pool.lease(1, 1).is_ok());
}

#[test]
fn forgotten_transfer_keeps_slots_and_blocks_channel_until_current_generation_drains() {
    let mut io = running();
    io.stuck = true;
    io.refuse_stop = true;
    let pool = pool::fake_pool::<4>();
    let mut channel = channel::<2>();
    let transfer = channel
        .start_leased(
            &mut io,
            pool.lease(0, 64).unwrap(),
            pool.lease(1, 64).unwrap(),
        )
        .unwrap();
    core::mem::forget(transfer);
    assert!(matches!(pool.lease(0, 64), Err(Error::Busy)));
    assert!(matches!(pool.lease(1, 64), Err(Error::Busy)));
    assert_eq!(channel.recover(&mut io), Err(Error::Recovering));
    let err = channel
        .start_leased(
            &mut io,
            pool.lease(2, 64).unwrap(),
            pool.lease(3, 64).unwrap(),
        )
        .err()
        .unwrap();
    assert_eq!(err.error, Error::Recovering);
    drop(err.resources);
    io.stop(2, int::CH_DISABLED | int::DMA_TFR_DONE);
    channel.recover(&mut io).unwrap();
    assert_eq!(io.read(channel_base(2) + ch::INTSTATUS), 0);
    assert!(matches!(pool.lease(1, 64), Err(Error::Busy)));
    let mut transfer = channel
        .start_leased(
            &mut io,
            pool.lease(2, 64).unwrap(),
            pool.lease(3, 64).unwrap(),
        )
        .unwrap();
    assert_eq!(transfer.raw.poll(&mut io), None);
    io.stop(2, int::DMA_TFR_DONE);
    transfer.release_on_drop(&mut io);
    core::mem::forget(transfer);
    assert!(pool.lease(2, 64).is_ok());
    assert!(pool.lease(3, 64).is_ok());
}

#[test]
fn stuck_drop_is_bounded_and_quarantines_both_slots() {
    let mut io = running();
    io.stuck = true;
    io.refuse_stop = true;
    let pool = pool::fake_pool::<2>();
    let mut channel = channel::<7>();
    let mut transfer = channel
        .start_leased(
            &mut io,
            pool.lease(0, 4).unwrap(),
            pool.lease(1, 4).unwrap(),
        )
        .unwrap();
    let reads = io.reads;
    let writes = io.writes.len();
    transfer.release_on_drop(&mut io);
    assert!(io.reads - reads <= 5);
    assert_eq!(io.writes.len() - writes, 2);
    assert_eq!(reg(&io, BASE + common::CHEN), 1 << 15);
    core::mem::forget(transfer);
    assert_eq!(channel.state, State::Stopping);
    assert!(matches!(pool.lease(0, 4), Err(Error::Busy)));
    assert!(matches!(pool.lease(1, 4), Err(Error::Busy)));
}

#[test]
fn stale_or_late_status_requires_idle_and_ack_before_rearm() {
    let mut io = running();
    io.stuck = true;
    io.refuse_stop = true;
    let mut channel = channel::<0>();
    let src = BusAddr(BusAddr::SRAM);
    let dst = BusAddr(src.0 + 128);
    let isr = channel_base(0) + ch::INTSTATUS;
    io.regs.insert(isr, int::DMA_TFR_DONE);
    io.refuse_ack = true;
    assert_eq!(
        start_block(&mut io, 0, &mut channel.state, src, dst, 64),
        Err(Error::Recovering)
    );
    assert!(!enabled(&mut io, 0));
    io.refuse_ack = false;
    start_block(&mut io, 0, &mut channel.state, src, dst, 64).unwrap();
    assert_eq!(io.read(isr), 0);
    request_stop(&mut io, 0, &mut channel.state, Error::Cancelled).unwrap_err();
    io.regs.insert(isr, int::DMA_TFR_DONE);
    assert_eq!(observe(&mut io, 0, &mut channel.state), None);
    io.stop(0, int::CH_DISABLED);
    io.refuse_ack = true;
    assert_eq!(channel.recover(&mut io), Err(Error::Recovering));
    io.refuse_ack = false;
    channel.recover(&mut io).unwrap();
    start_block(&mut io, 0, &mut channel.state, src, dst, 64).unwrap();
    assert_eq!(observe(&mut io, 0, &mut channel.state), None);
}

#[test]
fn abort_is_not_a_drain_even_with_chen_clear() {
    let mut io = running();
    io.stuck = true;
    let mut channel = channel::<4>();
    let src = BusAddr(BusAddr::SRAM);
    let dst = BusAddr(src.0 + 128);
    start_block(&mut io, 4, &mut channel.state, src, dst, 64).unwrap();
    io.stop(4, int::CH_ABORTED | int::CH_DISABLED);
    assert_eq!(channel.recover(&mut io), Err(Error::Aborted));
    assert_eq!(
        start_block(&mut io, 4, &mut channel.state, src, dst, 64),
        Err(Error::Recovering)
    );
}

#[test]
fn polling_uses_the_full_deadline_across_low_word_wrap() {
    let mut io = running();
    io.now = u32::MAX as u64 - 3;
    let end = Deadline::after_ticks(read_instant(&mut io), 8).unwrap();
    assert_eq!(
        wait_block(&mut io, end, |_| None::<Result<(), Error>>),
        Err(Error::Timeout)
    );
    assert_eq!(io.now, end.instant().ticks());
    // Retained completion wins at the deadline.
    assert_eq!(wait_block(&mut io, end, |_| Some(Ok(()))), Ok(()));
}

#[test]
fn a_bus_error_requests_stop_then_reports_the_original_fault() {
    let mut io = running();
    io.stuck = true;
    let mut channel = channel::<3>();
    let src = BusAddr(BusAddr::SRAM);
    let dst = BusAddr(src.0 + 128);
    start_block(&mut io, 3, &mut channel.state, src, dst, 64).unwrap();
    io.regs
        .insert(channel_base(3) + ch::INTSTATUS, int::SRC_SLV_ERR);
    let end = Deadline::after_ticks(read_instant(&mut io), 50).unwrap();
    let mut transfer = RawTransfer {
        channel: &mut channel,
        result: None,
    };
    assert_eq!(
        transfer.wait_accepted(&mut io, end),
        Err(Error::Bus(int::SRC_SLV_ERR))
    );
    assert_eq!(transfer.channel.state, State::Idle);
}
