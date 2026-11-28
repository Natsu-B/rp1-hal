use super::{CALL_TIMEOUT, Result, commands};
use rp1_linux::Client;
use std::{
    hint::black_box,
    time::{Duration, Instant},
};

const WIDTH: usize = 640;
const HEIGHT: usize = 480;
const CHANNELS: usize = 3;
const FPS: u64 = 30;
const FRAMES: usize = 60 * FPS as usize;

pub fn run(client: &mut Client<u32, u32>) -> Result<()> {
    let mut p99 = [0; 2];
    for (index, mode) in [commands::CONTROL_NO_IPC, commands::CONTROL_IPC]
        .into_iter()
        .enumerate()
    {
        let start = Instant::now() + Duration::from_millis(200);
        let (mut samples, calls) = std::thread::scope(|threads| -> Result<_> {
            let image = threads.spawn(move || image_worker(start));
            sleep_until(start);
            assert_eq!(client.call(&mode, CALL_TIMEOUT)?, mode);
            let mut calls = 0;
            let end = start + Duration::from_secs(60);
            if mode == commands::CONTROL_IPC {
                let mut next = start;
                while Instant::now() < end {
                    assert_eq!(client.call(&calls, CALL_TIMEOUT)?, calls);
                    calls += 1;
                    next += Duration::from_millis(1);
                    sleep_until(next);
                }
            } else {
                sleep_until(end);
            }
            let samples = image
                .join()
                .map_err(|_| "image worker panicked")?
                .map_err(std::io::Error::from_raw_os_error)?;
            Ok((samples, calls))
        })?;
        for (frame, ns) in samples.iter().enumerate() {
            println!("LWC_FRAME mode={mode} frame={frame} ns={ns}");
        }
        samples.sort_unstable();
        p99[index] = samples[(samples.len() * 99).div_ceil(100) - 1];
        println!(
            "LWC_FRAME_SUMMARY mode={mode} n={} p99_ns={} calls={calls}",
            samples.len(),
            p99[index]
        );
        assert_eq!(samples.len(), FRAMES);
        // Leave time for RP1 to finish the last interrupt and print its summary.
        std::thread::sleep(Duration::from_millis(500));
    }
    assert!(p99[1] * 100 <= p99[0] * 105, "frame p99 {p99:?}");
    Ok(())
}

fn sleep_until(end: Instant) {
    std::thread::sleep(end.saturating_duration_since(Instant::now()));
}

fn image_worker(start: Instant) -> std::result::Result<Vec<u128>, i32> {
    let param: libc::sched_param = unsafe { std::mem::zeroed() };
    let errno =
        unsafe { libc::pthread_setschedparam(libc::pthread_self(), libc::SCHED_OTHER, &param) };
    if errno != 0 {
        return Err(errno);
    }
    let mut input = vec![0u8; WIDTH * HEIGHT * CHANNELS];
    let mut output = vec![0i16; WIDTH * HEIGHT * CHANNELS];
    let mut samples = Vec::with_capacity(FRAMES);
    let mut seed = 1u32;
    for frame in 0..FRAMES {
        let release = start + Duration::from_nanos(frame as u64 * 1_000_000_000 / FPS);
        sleep_until(release);
        for byte in &mut input {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            *byte = (seed >> 24) as u8;
        }
        filter(&input, &mut output);
        black_box(&output);
        samples.push(release.elapsed().as_nanos());
    }
    Ok(samples)
}

#[inline(never)]
fn filter(input: &[u8], output: &mut [i16]) {
    const STRIDE: usize = WIDTH * CHANNELS;
    for (row, out) in (1..HEIGHT - 1).zip(output.as_chunks_mut::<STRIDE>().0.iter_mut().skip(1)) {
        let above = &input[(row - 1) * STRIDE..row * STRIDE];
        let middle = &input[row * STRIDE..(row + 1) * STRIDE];
        let below = &input[(row + 1) * STRIDE..(row + 2) * STRIDE];
        for i in CHANNELS..STRIDE - CHANNELS {
            // Three independent 3x3 integer feature planes.
            let j = CHANNELS;
            let edge =
                i16::from(above[i - j]) + 2 * i16::from(middle[i - j]) + i16::from(below[i - j])
                    - i16::from(above[i + j])
                    - 2 * i16::from(middle[i + j])
                    - i16::from(below[i + j]);
            let smooth = i16::from(above[i]) + 2 * i16::from(middle[i]) + i16::from(below[i]);
            out[i] = edge + smooth;
        }
    }
}
