//! UART1 IRQ, native try, timeout/re-arm and physical flush hardware candidate.
#![no_std]
#![no_main]

use core::fmt;
use embedded_io::{Read, Write};
use rp1_hal::{
    fugit::{ExtU32, RateExtU32},
    log,
    pac::pl011::int,
    timer::{Deadline, Timer, WaitTimer},
    uart::{self, Irq, Rx, Tx},
};
use rp1_rt::{Proc0, Proc1, entry, interrupt::Slot, wait_for_event};

mod probe;

rp1_rt::bind!();

use probe::{now, uart as registers};

const TRIALS: u32 = 1_000;
const STREAM: usize = 100_000;
const BAUD: u32 = 115_200;
const WATERMARK: usize = 16;
const TX_SAMPLE: usize = 34;
const STREAM_CHUNK: usize = 128;
const STREAM_TX_IRQ_MAX: u32 = STREAM.div_ceil(WATERMARK) as u32;
const STREAM_RX_IRQ_MAX: u32 = STREAM_TX_IRQ_MAX + (STREAM / STREAM_CHUNK) as u32;

type Tx1 = Tx<1, 0, Irq<0>>;
type Rx1 = Rx<1, 1, Irq<0>>;

#[derive(Clone, Copy, Default, Debug, PartialEq)]
struct Counts {
    rx: u32,
    rt: u32,
    tx: u32,
    other: u32,
}

impl Counts {
    fn since(self, earlier: Self) -> Self {
        Self {
            rx: self.rx - earlier.rx,
            rt: self.rt - earlier.rt,
            tx: self.tx - earlier.tx,
            other: self.other - earlier.other,
        }
    }
}

#[derive(Clone, Copy, Default)]
struct Entry {
    at: u32,
    ris: u32,
    fr: u32,
    serviced: u32,
}

static PROBE: Slot<(Counts, Entry), 0, 4> = Slot::new((
    Counts {
        rx: 0,
        rt: 0,
        tx: 0,
        other: 0,
    },
    Entry {
        at: 0,
        ris: 0,
        fr: 0,
        serviced: 0,
    },
));

// Wrap the generated dispatcher while preserving its vector and normal service.
#[inline(always)]
fn observe_uart1(service: impl FnOnce() -> u32) {
    let at = now();
    let ris = registers().ris.read().bits();
    let fr = registers().fr.read().bits();
    let serviced = service();
    PROBE.try_with(|(counts, entry)| {
        *entry = Entry {
            at,
            ris,
            fr,
            serviced,
        };
        match serviced {
            bits if bits == int::RX || bits == int::RX | int::RT => counts.rx += 1,
            int::RT => counts.rt += 1,
            int::TX => counts.tx += 1,
            _ => counts.other += 1,
        }
    });
}

fn counts() -> Counts {
    PROBE.try_with(|(counts, _)| *counts).unwrap()
}

fn trial(run: &str, kind: &str, i: u32, edges: [u32; 6], flush: [u32; 3]) {
    let [low, high, empty_lo, empty_hi, busy_lo, busy_hi] = edges;
    let [begin, end, end_fr] = flush;
    let Entry {
        at,
        ris,
        fr,
        serviced,
    } = PROBE.try_with(|(_, entry)| *entry).unwrap();
    let _ = log!(
        "UART_PARITY_TRIAL run={run} kind={kind} i={i} low={low} high={high} entry={at} ris={ris} fr={fr} serviced={serviced} empty_lo={empty_lo} empty_hi={empty_hi} busy_lo={busy_lo} busy_hi={busy_hi} begin={begin} end={end} end_fr={end_fr}\n"
    );
}

fn byte(index: usize, run: &str) -> u8 {
    ((index * 73) ^ (index >> 8)) as u8 ^ run.as_bytes()[index % 8]
}

struct Samples([u8; TRIALS as usize]);

impl Samples {
    fn record(&mut self, index: u32, ticks: u32) {
        // One tick covers the quantization of the two timer reads.
        self.0[index as usize] = u8::try_from(ticks.saturating_add(1)).unwrap_or(u8::MAX);
    }

    fn report(&self, run: &str, kind: &str) -> u8 {
        let _ = log!("UART_PARITY_NB_SAMPLES run={run} kind={kind} us={self}\n");
        self.0.iter().copied().max().unwrap()
    }
}

impl fmt::Display for Samples {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        for sample in &self.0 {
            write!(out, "{sample:02x}")?;
        }
        Ok(())
    }
}

struct Bench<'a> {
    wait: &'a mut WaitTimer,
}

impl Bench<'_> {
    fn send(&mut self, tx: &mut Tx1, bytes: &[u8]) {
        let mut tx = tx.blocking(self.wait);
        tx.write_all(bytes).unwrap();
        tx.flush().unwrap();
    }

    fn rx_trials(&mut self, tx: &mut Tx1, rx: &mut Rx1, watermark: bool, run: &str) -> u8 {
        let (command, len) = if watermark {
            (b'R', WATERMARK)
        } else {
            (b'S', 1)
        };
        let timer = Timer::new();
        let mut samples = Samples([0; TRIALS as usize]);
        for i in 0..TRIALS {
            if watermark {
                let started = timer.now_32();
                let empty = rx.try_read_byte();
                let elapsed = timer.now_32().ticks().wrapping_sub(started.ticks());
                samples.record(i, elapsed);
                assert_eq!(empty, Ok(None));
            }

            self.send(tx, &[command]);
            let before = counts();
            let (source, kind) = if watermark {
                (int::RX, "rx")
            } else {
                (int::RT, "rt")
            };
            let mut data = [0; WATERMARK];
            probe::arm(source);
            rx.blocking(self.wait).read_exact(&mut data[..len]).unwrap();
            let edges = probe::finish();
            let mut expected = before;
            let serviced = PROBE.try_with(|(_, entry)| entry.serviced).unwrap();
            assert_ne!(serviced & source, 0);
            if serviced & int::RX != 0 {
                expected.rx += 1;
            } else {
                expected.rt += 1;
            }
            assert_eq!(counts(), expected);
            for (j, &got) in data[..len].iter().enumerate() {
                assert_eq!(got, byte(i as usize * len + j, run));
            }
            trial(run, kind, i, edges, [0; 3]);
        }
        if watermark {
            samples.report(run, "empty")
        } else {
            0
        }
    }

    fn tx_trials(&mut self, tx: &mut Tx1, run: &str) -> u8 {
        let timer = Timer::new();
        let mut samples = Samples([0; TRIALS as usize]);
        for i in 0..TRIALS {
            self.send(tx, b"T");
            let mut data = [0; TX_SAMPLE];
            for (j, value) in data.iter_mut().enumerate() {
                *value = byte(i as usize * TX_SAMPLE + j, run);
            }
            // Queue the initial FIFO in one burst. Per-byte accounting across
            // the entire prefill can consume the first frame and reopen space
            // between the full-FIFO try and the start of the wait.
            let prefill_start = now();
            tx.blocking(self.wait).write_all(&data[..32]).unwrap();
            let filled = (32..TX_SAMPLE)
                .find(|&i| !tx.try_write_byte(data[i]).unwrap())
                .unwrap();

            let started = timer.now_32();
            let full = tx.try_write_byte(data[filled]);
            let elapsed = timer.now_32().ticks().wrapping_sub(started.ticks());
            samples.record(i, elapsed);
            assert_eq!(full, Ok(false));

            let before = counts();
            probe::arm(int::TX);
            let prefill_us = now().wrapping_sub(prefill_start);
            let before_write = registers().fr.read().bits();
            let mut output = tx.blocking(self.wait);
            output.write_all(&data[filled..]).unwrap();
            let delta = counts().since(before);
            if delta
                != (Counts {
                    tx: 1,
                    ..Counts::default()
                })
            {
                let _ = log!(
                    "UART_PARITY_TX_FAIL run={run} i={i} delta={delta:?} filled={filled} prefill_us={prefill_us} fr={before_write}\n"
                );
                panic!("expected one TX IRQ");
            }
            let begin = now();
            let result = output.flush();
            let end = now();
            if let Err(error) = result {
                let _ = log!("UART_PARITY_TX_FAIL run={run} i={i} flush={error:?}\n");
                panic!("TX flush failed");
            }
            let fr = registers().fr.read().bits();
            let edges = probe::finish();
            self.send(tx, &[filled as u8]);
            trial(run, "tx", i, edges, [begin, end, fr]);
        }
        samples.report(run, "full")
    }

    fn stream(&mut self, tx: &mut Tx1, rx: &mut Rx1, run: &str) -> (Counts, Counts) {
        self.send(tx, b"I");
        let before_rx = counts();
        let end = Deadline::after(25.secs()).unwrap();
        let mut buf = [0; STREAM_CHUNK];
        let mut input = rx.blocking(self.wait);
        for offset in (0..STREAM).step_by(buf.len()) {
            let len = (STREAM - offset).min(buf.len());
            input.read_exact_until(&mut buf[..len], end).unwrap();
            for (i, &got) in buf[..len].iter().enumerate() {
                assert_eq!(got, byte(offset + i, run));
            }
        }
        let rx_counts = counts().since(before_rx);
        self.send(tx, b"O");
        let before_tx = counts();
        let end = Deadline::after(25.secs()).unwrap();
        let mut output = tx.blocking(self.wait);
        for offset in (0..STREAM).step_by(buf.len()) {
            let len = (STREAM - offset).min(buf.len());
            for (i, value) in buf[..len].iter_mut().enumerate() {
                *value = byte(offset + i, run);
            }
            output.write_all_until(&buf[..len], end).unwrap();
        }
        output.flush_until(end).unwrap();
        (rx_counts, counts().since(before_tx))
    }
}

#[entry(proc0)]
fn main(mut core: Proc0, mut p: rp1::Proc0Resources) -> ! {
    let clocks = &rp1::platform::init(&mut core, p.platform).unwrap().clocks;
    let mut uart = p.uart1.init(clocks, BAUD.Hz()).unwrap();
    p.proc1.launch(&mut core).unwrap();
    let mut bench = Bench {
        wait: &mut p.wait_timer,
    };

    for _ in 0..32 {
        if uart.try_read_byte() == Ok(None) {
            break;
        }
    }
    let divisor = registers().ibrd.read().bits() * 64 + registers().fbrd.read().bits();
    let _ = log!(
        "UART_PARITY_CLOCK timer_hz=1000000 uart_hz={} divisor={divisor}\n",
        clocks.uart().raw()
    );
    let _ = log!("UART_PARITY_READY\n");
    let mut io = uart.blocking(bench.wait).with_timeout(30.secs());
    let mut nonce = [0; 8];
    io.read_exact(&mut nonce).unwrap();
    let run = core::str::from_utf8(&nonce).unwrap();
    assert!(nonce.iter().all(u8::is_ascii_hexdigit));
    io.write_all(&nonce).unwrap();
    io.flush().unwrap();

    let (mut tx, mut rx) = uart.split();
    let before = counts();
    let empty_us = bench.rx_trials(&mut tx, &mut rx, true, run);
    bench.rx_trials(&mut tx, &mut rx, false, run);
    let full_us = bench.tx_trials(&mut tx, run);
    let after = counts();
    let trial_counts = after.since(before);
    assert_eq!(trial_counts.rx + trial_counts.rt, 2 * TRIALS);
    assert_eq!(trial_counts.tx, TRIALS);
    assert_eq!(trial_counts.other, 0);
    assert!(empty_us <= 10 && full_us <= 10);
    let _ = log!(
        "PARITY_UART_NB_END run={} full={} empty={} full_max_us={} empty_max_us={} false_ready=0 result=PASS\n",
        run,
        TRIALS,
        TRIALS,
        full_us,
        empty_us
    );
    let (rx_stream, tx_stream) = bench.stream(&mut tx, &mut rx, run);
    let false_wake = rx_stream.tx + rx_stream.other + tx_stream.rx + tx_stream.rt + tx_stream.other;
    assert_eq!(false_wake, 0);
    assert_eq!(counts().other, after.other);
    assert!((1..=STREAM_RX_IRQ_MAX).contains(&(rx_stream.rx + rx_stream.rt)));
    assert!((1..=STREAM_TX_IRQ_MAX).contains(&tx_stream.tx));

    // The host sends no data until A, so this tests the actual deadline and
    // retirement. Then reset with reunited halves, reacquire the same pins and re-arm.
    let mut uart = rp1::Uart1::join(tx, rx);
    let before_timeout = counts();
    let timeout_start = now();
    let end = Deadline::after(10.millis()).unwrap();
    assert_eq!(
        uart.blocking(bench.wait).read_until(&mut [0], end),
        Err(uart::Error::Timeout)
    );
    let timeout_us = now().wrapping_sub(timeout_start);
    assert_eq!(counts(), before_timeout);
    let _ = uart.try_cancel();
    let end = Deadline::after(10.millis()).unwrap();
    let parts = loop {
        match uart.try_free() {
            Ok(parts) => break parts,
            Err(error) => uart = error.into_driver(),
        }
        assert!(
            Timer::new().now() < end.instant(),
            "UART reset did not complete"
        );
        bench.wait.delay(10.micros()).unwrap();
    };
    let (mut tx, mut rx) = parts.init(clocks, BAUD.Hz()).unwrap().split();
    bench.send(&mut tx, b"A");
    let rearm = counts();
    let mut value = [0];
    rx.blocking(bench.wait).read_exact(&mut value).unwrap();
    assert_eq!(value, [nonce[0]]);
    assert_eq!(
        counts(),
        Counts {
            rt: rearm.rt + 1,
            ..rearm
        }
    );

    let _ = log!(
        "UART_PARITY_REARM run={run} timeout_us={timeout_us} timeout_irq=0 rearm_rt=1 byte={}\n",
        value[0]
    );

    // Identical peer turnaround and payload for the IRQ and polling adapters.
    bench.send(&mut tx, b"Q");
    let irq_count = counts();
    let irq_start = now();
    let mut comparison = [0; 8];
    rx.blocking(bench.wait).read_exact(&mut comparison).unwrap();
    assert_eq!(comparison, nonce);
    bench.send(&mut tx, &comparison);
    let irq_us = now().wrapping_sub(irq_start);
    let irq = counts().since(irq_count);
    assert_eq!(
        irq,
        Counts {
            rt: 1,
            ..Counts::default()
        }
    );

    bench.send(&mut tx, b"P");
    let poll_count = counts();
    let poll_start = now();
    rx.polling().read_exact(&mut comparison).unwrap();
    assert_eq!(comparison, nonce);
    let mut tx = tx.polling();
    tx.write_all(&comparison).unwrap();
    tx.flush().unwrap();
    let poll_us = now().wrapping_sub(poll_start);
    assert_eq!(counts(), poll_count);
    let _ = log!(
        "UART_PARITY_COMPARE run={run} irq_us={irq_us} irq_rt=1 poll_us={poll_us} poll_irq=0 bytes=8\n"
    );

    let _ = log!(
        "UART_PARITY_FW_END run={} rx_watermark={} rx_timeout={} tx={} rx_bytes={} tx_bytes={} rx_stream_rx={} rx_stream_rt={} rx_stream_tx={} rx_stream_other={} tx_stream_rx={} tx_stream_rt={} tx_stream_tx={} tx_stream_other={} timeout=1 rearm=1 polling=8 flush={} false_wake={} result=PASS\n",
        run,
        TRIALS,
        TRIALS,
        TRIALS,
        STREAM,
        STREAM,
        rx_stream.rx,
        rx_stream.rt,
        rx_stream.tx,
        rx_stream.other,
        tx_stream.rx,
        tx_stream.rt,
        tx_stream.tx,
        tx_stream.other,
        TRIALS,
        false_wake
    );
    loop {
        wait_for_event();
    }
}

#[entry(proc1)]
fn sample(_core: Proc1, _p: rp1::Proc1Resources) -> ! {
    probe::sample()
}
