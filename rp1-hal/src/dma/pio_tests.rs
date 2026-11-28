use super::*;
use crate::dma::{
    BASE, ch, channel_base, common, int, pool,
    tests::{DisableStatus, Fake, reg, running},
};
use crate::timer::{Deadline, Instant};
use core::mem::{ManuallyDrop, forget};

fn channel() -> Channel<0, PioRequests> {
    Channel {
        state: State::Idle,
        gate: PioRequests::new(),
        _local: PhantomData,
    }
}

fn fake() -> Fake {
    let mut io = running();
    io.stuck = true;
    io
}

fn endpoint(rx: bool, sm: u8) -> Endpoint {
    Endpoint {
        sm,
        direction: if rx {
            Direction::FromPio
        } else {
            Direction::ToPio
        },
    }
}

#[test]
fn pio_registers_request_ids_width_and_single_beat_tails() {
    check_registers::<0>();
    check_registers::<1>();
    check_registers::<2>();
    check_registers::<3>();
}

fn check_registers<const SM: u8>() {
    use crate::pio::Buffers;
    let sm = SM;
    for rx in [false, true] {
        for words in [1, 7, 8, 9, 257] {
            let mut io = fake();
            let mut ch = channel();
            let pool = pool::fake_pool::<1>();
            let mut lease = pool.lease(0, words * 4).unwrap();
            let (mut tx, mut rxf) = (
                Tx::<SM>::new(Buffers::RxTx).into_dma(),
                Rx::<SM>::new(Buffers::RxTx).into_dma(),
            );
            let transfer = if rx {
                ch.start_fifo(&mut io, &mut lease, &mut rxf)
            } else {
                ch.start_fifo(&mut io, &mut lease, &mut tx)
            }
            .unwrap();
            // A rejected register mutation must unwind without touching real MMIO.
            let _transfer = ManuallyDrop::new(transfer);
            let c = channel_base(0);
            let fifo = 0x4017_8000 + u32::from(sm) * 4 + if rx { 16 } else { 0 };
            let (sar, dar, no_inc, request, flow, control, threshold) = if rx {
                (
                    fifo,
                    0x2000_2000,
                    1 << 4,
                    (57 + u32::from(sm) * 2) << 4,
                    2 | (1 << 4),
                    0xe8,
                    1,
                )
            } else {
                (
                    0x2000_2000,
                    fifo,
                    1 << 6,
                    (56 + u32::from(sm) * 2) << 11,
                    1 | (1 << 3),
                    0xe4,
                    7,
                )
            };
            let dreq = 0xf000_0000 + control + usize::from(sm) * 0x20;
            assert_eq!(reg(&io, c + ch::SAR), sar);
            assert_eq!(reg(&io, c + ch::DAR), dar);
            assert_eq!(reg(&io, c + ch::SAR + 4), 0xc0);
            assert_eq!(reg(&io, c + ch::DAR + 4), 0xc0);
            assert_eq!(reg(&io, c + ch::BLOCK_TS), words as u32 - 1);
            assert_eq!(reg(&io, c + ch::BLOCK_TS + 4), 0);
            assert_eq!(
                reg(&io, c + ch::CTL_L),
                (1 << 30) | (2 << 11) | (2 << 8) | no_inc
            );
            assert_eq!(reg(&io, c + ch::CTL_H), (1 << 15) | (1 << 6));
            assert_eq!(reg(&io, c + ch::CFG_L), request);
            assert_eq!(reg(&io, c + ch::CFG_H), flow);
            assert_eq!(reg(&io, c + ch::INTSIGNAL_ENA), 0);
            assert_eq!(io.writes[0], (dreq, 0));
            assert_eq!(io.writes[io.writes.len() - 2], (BASE + common::CHEN, 0x101));
            assert_eq!(io.writes.last(), Some(&(dreq, 0x8000_0000 | threshold)));
        }
    }
}

#[test]
fn pio_admission_validates_join_alignment_and_quarantine_before_mmio() {
    let mut io = fake();
    let mut ch = channel();
    let pool = pool::fake_pool::<1>();
    let mut lease = pool.lease(0, 16).unwrap();
    let mut held = false;
    for (offset, len, capacity) in [(1, 16, 8), (0, 15, 8), (0, 16, 0)] {
        lease.set_range(offset, len).unwrap();
        assert!(matches!(
            ch.start_pio(&mut io, &mut lease, &mut held, capacity, endpoint(false, 0)),
            Err(Error::Invalid)
        ));
        assert!(!held && !lease.is_quarantined());
    }
    held = true;
    assert!(matches!(
        ch.start_pio(&mut io, &mut lease, &mut held, 8, endpoint(false, 0)),
        Err(Error::Busy)
    ));
    assert!(io.writes.is_empty());
}

#[test]
fn pio_polling_accepts_completion_at_deadline_and_retires_request() {
    let mut io = fake();
    let mut ch = channel();
    let pool = pool::fake_pool::<1>();
    let mut lease = pool.lease(0, 36).unwrap();
    let mut held = false;
    let transfer = ch
        .start_pio(&mut io, &mut lease, &mut held, 8, endpoint(true, 3))
        .unwrap();
    let mut transfer = ManuallyDrop::new(transfer);
    let deadline = Deadline::at(Instant::from_ticks(io.now + 1));
    io.stop(0, int::DMA_TFR_DONE);
    assert_eq!(transfer.wait_accepted(&mut io, deadline), Ok(()));
    assert_eq!(reg(&io, 0xf000_0148), 0);
    transfer.release_on_drop(&mut io);
    let _ = transfer;
    assert!(!held && !lease.is_quarantined());
}

#[test]
fn pio_timeout_masks_request_before_stop_and_retains_late_outcome() {
    let mut io = fake();
    io.disable_status = DisableStatus::Late;
    let mut ch = channel();
    let pool = pool::fake_pool::<1>();
    let mut lease = pool.lease(0, 36).unwrap();
    let mut held = false;
    let transfer = ch
        .start_pio(&mut io, &mut lease, &mut held, 8, endpoint(false, 1))
        .unwrap();
    let mut transfer = ManuallyDrop::new(transfer);
    let deadline = Deadline::at(Instant::from_ticks(io.now + 8));
    let before = io.writes.len();
    assert_eq!(
        transfer.wait_accepted(&mut io, deadline),
        Err(Error::Recovering)
    );
    assert_eq!(io.writes[before], (0xf000_0104, 0));
    assert!(io.writes[before + 1..].contains(&(BASE + common::CHEN, 0x100)));
    assert_eq!(transfer.poll(&mut io), None);
    io.deliver_disable(0);
    io.stop(0, int::DMA_TFR_DONE);
    assert_eq!(transfer.poll(&mut io), Some(Err(Error::Timeout)));
    transfer.release_on_drop(&mut io);
    let _ = transfer;
    assert!(!held && !lease.is_quarantined());
}

#[test]
fn late_stop_proof_and_forget_do_not_release_claims() {
    for drop_late in [false, true] {
        let mut io = fake();
        io.disable_delay = 1_000;
        let mut ch = channel();
        let pool = pool::fake_pool::<1>();
        let mut lease = pool.lease(0, 36).unwrap();
        let mut held = false;
        let mut transfer = ch
            .start_pio(&mut io, &mut lease, &mut held, 8, endpoint(true, 0))
            .unwrap();
        if drop_late {
            transfer.release_on_drop(&mut io);
            assert_eq!(transfer.raw.terminal_status, int::CH_DISABLED);
        }
        forget(transfer);
        assert!(held && lease.is_quarantined() && lease.as_slice().is_empty());
        assert_eq!(ch.recover(&mut io), Ok(()));
        assert_eq!(reg(&io, 0xf000_00e8), 0);
        assert!(held && lease.is_quarantined());
        drop(lease);
        assert!(matches!(pool.lease(0, 36), Err(Error::Busy)));
    }
}
