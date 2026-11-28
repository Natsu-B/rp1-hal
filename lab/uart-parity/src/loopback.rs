use super::*;

const TRIALS: usize = 100;
const TIMEOUT_US: u32 = 10_000;
const TX_SAMPLE: usize = 34;
pub(crate) type Report = [u32; 16];

#[derive(Clone, Copy)]
struct Range(u32, u32);

impl Range {
    fn record(&mut self, us: u32, bounds: core::ops::RangeInclusive<u32>, source: u32) {
        assert!(
            bounds.contains(&us),
            "IRQ {source:#x}: {us} us outside {bounds:?}"
        );
        self.0 = self.0.min(us);
        self.1 = self.1.max(us);
    }
}

fn irq_latency(source: u32, start: u32) -> u32 {
    let entry = PROBE.try_with(|(_, entry)| *entry).unwrap();
    assert_eq!(entry.serviced, source);
    assert_ne!(entry.ris & source, 0);
    let fifo_unavailable = if source == int::TX { 1 << 5 } else { 1 << 4 };
    assert_eq!(entry.fr & fifo_unavailable, 0);
    entry.at.wrapping_sub(start)
}

fn run(uart: &mut Driver, timer: &mut WaitTimer, clock: u32) -> Report {
    // 8N1 is ten bits per frame. The scaled divisor gives the actual baud.
    let divisor = registers().ibrd.read().bits() * 64 + registers().fbrd.read().bits();
    assert_eq!(divisor, 1736);
    let bit_floor =
        |bits: u32| (u64::from(bits) * u64::from(divisor) * 250_000 / u64::from(clock)) as u32;
    let bit_ceil = |bits: u32| {
        (u64::from(bits) * u64::from(divisor) * 250_000).div_ceil(u64::from(clock)) as u32
    };
    let bounds = |lo, hi| bit_floor(lo).saturating_sub(1)..=bit_ceil(hi) + 6;
    let mut rx_time = Range(u32::MAX, 0);
    let mut rt_time = rx_time;
    let mut tx_time = rx_time;
    let mut bytes = 0;

    for trial in 0..TRIALS {
        let data: [u8; TX_SAMPLE] = core::array::from_fn(|i| (37 * i + 19 * trial) as u8);
        for (len, source) in [(WATERMARK, int::RX), (1, int::RT)] {
            let before = counts();
            let end = Deadline::after(TIMEOUT_US.micros()).unwrap();
            let mut received = [0; WATERMARK];
            let mut io = uart.blocking(timer);
            let start = now();
            io.write_all_until(&data[..len], end).unwrap();
            io.read_exact_until(&mut received[..len], end).unwrap();
            let us = irq_latency(source, start);
            io.flush_until(end).unwrap();
            assert_eq!(received[..len], data[..len]);
            let delta = counts().since(before);
            if source == int::RX {
                assert_eq!(
                    delta,
                    Counts {
                        rx: 1,
                        ..Counts::default()
                    }
                );
                rx_time.record(us, bounds(158, 161), source);
            } else {
                assert_eq!(
                    delta,
                    Counts {
                        rt: 1,
                        ..Counts::default()
                    }
                );
                rt_time.record(us, bounds(40, 43), source);
            }
            bytes += len as u32;
        }

        let before = counts();
        let end = Deadline::after(TIMEOUT_US.micros()).unwrap();
        let start = now();
        uart.blocking(timer)
            .write_all_until(&data[..32], end)
            .unwrap();
        let first_blocked = (32..TX_SAMPLE)
            .find(|&i| !uart.try_write_byte(data[i]).unwrap())
            .expect("TX FIFO did not fill");
        let mut io = uart.blocking(timer);
        io.write_all_until(&data[first_blocked..], end).unwrap();
        let us = irq_latency(int::TX, start);
        let frames = (first_blocked - WATERMARK) as u32;
        tx_time.record(us, bounds(10 * (frames - 2), 10 * (frames + 1)), int::TX);
        let delta = counts().since(before);
        assert_eq!(
            delta,
            Counts {
                tx: 1,
                ..Counts::default()
            }
        );
        let mut received = [0; TX_SAMPLE];
        // Make room in RX while the remaining frames are still shifting.
        io.read_exact_until(&mut received[..8], end).unwrap();
        io.flush_until(end).unwrap();
        io.read_exact_until(&mut received[8..], end).unwrap();
        assert_eq!(counts().since(before), delta);
        assert_eq!(received, data);
        bytes += received.len() as u32;
    }

    let before = counts();
    let start = Timer::new().now();
    let end = Deadline::at(start + TIMEOUT_US.micros());
    assert_eq!(
        uart.blocking(timer).read_until(&mut [0], end),
        Err(uart::Error::Timeout)
    );
    let timeout = (Timer::new().now() - start).ticks() as u32;
    assert!((TIMEOUT_US..=TIMEOUT_US + 4).contains(&timeout));
    assert_eq!(counts(), before);
    let end = Deadline::after(TIMEOUT_US.micros()).unwrap();
    let mut io = uart.blocking(timer);
    io.write_all_until(&[0xa5], end).unwrap();
    let mut received = [0];
    io.read_exact_until(&mut received, end).unwrap();
    assert_eq!(received, [0xa5]);
    assert_eq!(
        counts().since(before),
        Counts {
            rt: 1,
            ..Counts::default()
        }
    );
    [
        u32::from(N),
        divisor,
        bytes + 1,
        before.rx,
        before.rt,
        before.tx,
        before.other,
        rx_time.0,
        rx_time.1,
        rt_time.0,
        rt_time.1,
        tx_time.0,
        tx_time.1,
        timeout,
        1,
        clock,
    ]
}

#[entry(proc0)]
fn main(mut core: Proc0, mut p: rp1::Proc0Resources) -> ! {
    #[allow(unused_mut)]
    let mut sys = rp1::platform::init(&mut core, p.platform).unwrap();
    let mut uart = parts!(p).init_loopback(&sys.clocks, BAUD.Hz()).unwrap();
    let report = run(&mut uart, &mut p.wait_timer, sys.clocks.uart().raw());
    #[cfg(feature = "uart0")]
    {
        rp1::link::enable(&mut sys.pcie, &mut p.TIMER_ALARM0, &mut p.irq.TIMER_0).unwrap();
        p.link.open().unwrap().report.send(&report).unwrap();
    }
    #[cfg(not(feature = "uart0"))]
    {
        let [
            _,
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
        ] = report;
        rp1_hal::log!(
            "UART_LOOPBACK instance={} clock={} divisor={} bytes={} rx_irq={} rt_irq={} tx_irq={} stray={} rx_us={}..{} rt_us={}..{} tx_us={}..{} timeout_us={} rearm={} result=PASS\n",
            N, clock, divisor, bytes, rx, rt, tx, stray, rx_min, rx_max, rt_min, rt_max, tx_min, tx_max, timeout, rearm
        ).unwrap();
    }
    park()
}

#[entry(proc1)]
fn parked(_core: Proc1, _p: rp1::Proc1Resources) -> ! {
    park()
}

fn park() -> ! {
    loop {
        wait_for_event();
    }
}
