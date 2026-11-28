use super::{LIMIT_US, LabResult, dma_enables, now, reg};
use rp1_hal::{
    pio::{Buffers, PIO, PIOBuilder, Program, UninitStateMachine, pio_asm},
    timer::{Duration, Timer},
};

#[inline(never)]
fn configuration(mode: usize) -> (Program<32>, u8, Buffers) {
    match mode {
        0 => (
            pio_asm!(
                "set x, 7",
                "again:",
                "pull block",
                "mov isr, osr",
                "push block",
                "jmp x-- again",
                "park:",
                "jmp park"
            )
            .program,
            8,
            Buffers::RxTx,
        ),
        1 => (
            pio_asm!(
                "set x, 15",
                "again:",
                "pull block",
                "jmp x-- again",
                "park:",
                "jmp park"
            )
            .program,
            16,
            Buffers::OnlyTx,
        ),
        _ => (
            pio_asm!(
                "set x, 15",
                "again:",
                "mov isr, x",
                "push block",
                "jmp x-- again",
                "park:",
                "jmp park"
            )
            .program,
            16,
            Buffers::OnlyRx,
        ),
    }
}

/// Run the echo from the HAL's PIO quickstart on the bench.
#[inline(never)]
pub fn echo_example(
    pio: &PIO,
    slot: &mut Option<UninitStateMachine<0>>,
    timer: &Timer,
) -> LabResult<()> {
    let echo = pio_asm!(
        "pull block",
        "mov isr, osr",
        "push block",
        "park:",
        "jmp park"
    );
    let program = pio.install(&echo.program).map_err(|_| "echo install")?;
    let (sm, mut rx, mut tx) = PIOBuilder::from_installed_program(&program)
        .build(slot.take().ok_or("echo sm")?)
        .map_err(|_| "echo build")?;
    let running = sm.start().map_err(|_| "echo start")?;
    let deadline = timer.now() + Duration::from_ticks(1_000);
    let sent = tx.polling().write_until(21, deadline);
    let word = rx.polling().read_until(deadline);
    *slot = Some(running.stop().uninit(rx, tx).0);
    program.uninstall().map_err(|_| "echo uninstall")?;
    if sent.is_err() || word != Ok(21) {
        return Err("echo data");
    }
    log!("PIO_B_ECHO word=21\n").unwrap();
    Ok(())
}

#[inline(never)]
pub fn lifecycle<const N: u8>(
    pio: &PIO,
    slot: &mut Option<UninitStateMachine<N>>,
    timer: &Timer,
) -> LabResult<()> {
    let mut sm = slot.take().ok_or("missing fifo sm")?;
    let start = now(timer);
    let mut modes = [0; 3];
    let mut tx_join = 0;
    let mut rx_join = 0;
    for i in 0..100 {
        let (program, capacity, mode) = configuration(i % 3);
        let p = pio.install(&program).map_err(|_| "fifo install")?;
        let park = p.offset() + program.wrap.source;
        let (stopped, mut rx, mut tx) = PIOBuilder::from_installed_program(&p)
            .buffers(mode)
            .build(sm)
            .map_err(|_| "fifo build")?;
        if rx.level() != 0
            || tx.level() != 0
            || rx.read().is_some()
            || rx.capacity() != if mode == Buffers::OnlyTx { 0 } else { capacity }
            || tx.capacity() != if mode == Buffers::OnlyRx { 0 } else { capacity }
        {
            return Err("fifo reset");
        }
        for word in 0..tx.capacity() {
            if tx.level() != word || !tx.write((u32::from(N) << 16) | u32::from(word)) {
                return Err("tx levels");
            }
        }
        if tx.write(0) || tx.level() != tx.capacity() || !tx.is_full() {
            return Err("tx full or disabled");
        }
        tx_join += u32::from(tx.level() == 16);
        let running = stopped.start().map_err(|_| "fifo start")?;
        let beginning = now(timer);
        while running.instruction_address() != park {
            if now(timer).wrapping_sub(beginning) >= 1_000 {
                return Err("fifo park timeout");
            }
        }
        let stopped = running.stop();
        if tx.level() != 0 || !tx.is_empty() || rx.level() != rx.capacity() || !rx.is_full() {
            return Err("fifo stopped levels");
        }
        rx_join += u32::from(rx.level() == 16);
        for index in 0..rx.capacity() {
            let expected = match mode {
                Buffers::RxTx => (u32::from(N) << 16) | u32::from(index),
                Buffers::OnlyRx => 15 - u32::from(index),
                Buffers::OnlyTx => unreachable!(),
            };
            if rx.level() != rx.capacity() - index || rx.read() != Some(expected) {
                return Err("rx levels or data");
            }
        }
        if rx.read().is_some()
            || !rx.is_empty()
            || dma_enables() != 0
            || reg(0xf000_0008) & (0x0101_0101 << N) != 0
        {
            return Err("fifo empty, stall or dreq enabled");
        }
        (sm, _) = stopped.uninit(rx, tx);
        p.uninstall().map_err(|_| "fifo uninstall")?;
        modes[i % 3] += 1;
        if now(timer).wrapping_sub(start) >= LIMIT_US {
            return Err("lifecycle timeout");
        }
    }
    *slot = Some(sm);
    log!("PIO_B_FIFO sm={N} split={} tx={} rx={} tx_16={} rx_16={} start={start} end={} dreq={:#x}\n",
        modes[0], modes[1], modes[2], tx_join, rx_join, now(timer), dma_enables()).unwrap();
    Ok(())
}
