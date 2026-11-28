use rp1_linux::Firmware;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let firmware = Firmware::open()?;
    let timeout = Duration::from_secs(10);
    firmware.wait_ready(timeout)?;
    let mut ring = firmware.ring_rx::<[u32; 17]>("report")?;
    let [
        instance,
        divisor,
        bytes,
        rx,
        rt,
        tx,
        stray,
        rx_min,
        rx_max,
        rt_min,
        rt_max,
        tx_min,
        tx_max,
        timeout,
        rearm,
        clock,
        tail_max,
    ] = ring.recv(Some(timeout))?.ok_or("UART report is missing")?;
    assert_eq!(
        (instance, bytes, rx, rt, tx, stray, rearm),
        (0, 5101, 100, 100, 100, 0, 1)
    );
    assert!((1..=32).contains(&tail_max));
    println!(
        "UART_LOOPBACK instance={instance} clock={clock} divisor={divisor} bytes={bytes} rx_irq={rx} rt_irq={rt} tx_irq={tx} stray={stray} rx_us={rx_min}..{rx_max} rt_us={rt_min}..{rt_max} tx_us={tx_min}..{tx_max} timeout_us={timeout} rearm={rearm} tail_alarms_max={tail_max} result=PASS"
    );
    Ok(())
}
