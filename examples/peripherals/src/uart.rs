//! Declare `uart1 = { tx = 0, rx = 1 }` and `wait_timer = "ALARM1"` on proc0
//! with `linux.keep = {}`. The wait timer makes the generator own UART1’s IRQ.
//! Cross GPIO0/1 TX/RX with a 3.3 V UART adapter and share ground. At 115200 8N1,
//! send eight nonce bytes after UART0 logs READY, verify the echo, then send ACK (0x06).

use crate::rp1; // log! resolves rp1::log in this scope.
use rp1_hal::{
    clocks::Clocks,
    fugit::{ExtU32, RateExtU32},
    log,
    pac::pl011::int,
    timer::{Deadline, WaitTimer},
};

pub fn run(
    uart: rp1::Uart1Parts,
    clocks: &Clocks,
    wait: &mut WaitTimer,
) -> Result<(), &'static str> {
    let mut uart = uart
        .init(clocks, 115_200.Hz())
        .map_err(|_| "UART1 init failed")?;

    // Discard startup bytes so the nonce starts at the READY boundary.
    for _ in 0..32 {
        if matches!(uart.try_read_byte(), Ok(None)) {
            break;
        }
    }
    uart.clear_interrupt(int::FE | int::PE | int::BE | int::OE);
    let mut uart = uart.blocking(wait);

    // Share one deadline across the exchange so an absent peer cannot stall boot.
    let end = Deadline::after(15.secs()).map_err(|_| "UART1 deadline")?;
    let _ = log!("UART1 READY\n");
    let mut nonce = [0; 8];
    uart.read_exact_until(&mut nonce, end)
        .map_err(|_| "UART1 receive error")?;
    uart.write_all_until(&nonce, end)
        .map_err(|_| "UART1 transmit error")?;
    uart.flush_until(end).map_err(|_| "UART1 flush error")?;
    let mut ack = [0];
    uart.read_until(&mut ack, end)
        .map_err(|_| "UART1 acknowledgement timeout")?;
    if ack != [0x06] {
        return Err("UART1 echo not acknowledged");
    }
    Ok(())
}
