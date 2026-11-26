//! Proc0 declaration: `uart1 = { tx = 0, rx = 1 }`, with `linux.keep = {}`.
//! Cross GPIO0/1 TX/RX with a 3.3 V UART adapter and share ground. At 115200 8N1,
//! send eight nonce bytes after UART0 logs READY, verify the echo, then send ACK (0x06).

use crate::rp1; // log! resolves rp1::log in this scope.
use rp1_hal::{
    clocks::Clocks,
    fugit::{ExtU32, RateExtU32},
    log,
    pac::pl011::int,
    timer::Timer,
    uart::Parts,
};

pub fn run(uart: Parts<1, 0, 1>, clocks: &Clocks) -> Result<(), &'static str> {
    let (mut tx, mut rx) = uart
        .init(clocks, 115_200.Hz())
        .map_err(|_| "UART1 init failed")?
        .split();

    // Discard startup bytes so the nonce starts at the READY boundary.
    for _ in 0..32 {
        if matches!(rx.read_byte(), Ok(None)) {
            break;
        }
    }
    rx.clear_interrupt(int::FE | int::PE | int::BE | int::OE);

    let timer = Timer::new();
    // Use one deadline for the nonce and acknowledgement so an absent peer cannot stall boot.
    let deadline = timer.now_32() + 15.secs();
    let mut receive = || loop {
        if timer.now_32() >= deadline {
            return Err("UART1 peer timeout");
        }
        if let Some(byte) = rx.read_byte().map_err(|_| "UART1 receive error")? {
            return Ok(byte);
        }
    };

    let _ = log!("UART1 READY\n");
    let mut nonce = [0; 8];
    for byte in &mut nonce {
        *byte = receive()?;
    }
    embedded_io::Write::write_all(&mut tx, &nonce).map_err(|_| "UART1 transmit error")?;
    tx.flush().map_err(|_| "UART1 flush error")?;
    if receive()? != 0x06 {
        return Err("UART1 echo not acknowledged");
    }
    Ok(())
}
