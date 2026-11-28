use super::{Client, Result};
use std::io;
use std::time::Instant;

pub fn lock_memory() -> io::Result<()> {
    // SAFETY: mlockall takes only flags and applies to the current process.
    if unsafe { libc::mlockall(libc::MCL_CURRENT | libc::MCL_FUTURE) } != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

fn clock(id: libc::clockid_t) -> io::Result<u64> {
    let mut stamp = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // SAFETY: stamp is writable storage for one timespec.
    if unsafe { libc::clock_gettime(id, &mut stamp) } != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(stamp.tv_sec as u64 * 1_000_000_000 + stamp.tv_nsec as u64)
}

#[derive(Clone, Copy, Debug, Default)]
struct Thread {
    cpu_ns: u64,
    voluntary: libc::c_long,
    involuntary: libc::c_long,
}

impl Thread {
    fn sample() -> io::Result<Self> {
        // SAFETY: the zero representation is valid for rusage (integer fields).
        let mut usage: libc::rusage = unsafe { std::mem::zeroed() };
        // SAFETY: usage is writable storage for one rusage.
        if unsafe { libc::getrusage(libc::RUSAGE_THREAD, &mut usage) } != 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(Self {
            cpu_ns: clock(libc::CLOCK_THREAD_CPUTIME_ID)?,
            voluntary: usage.ru_nvcsw,
            involuntary: usage.ru_nivcsw,
        })
    }

    fn since(self, before: Self) -> Self {
        Self {
            cpu_ns: self.cpu_ns - before.cpu_ns,
            voluntary: self.voluntary - before.voluntary,
            involuntary: self.involuntary - before.involuntary,
        }
    }
}

pub struct Before {
    sleeps: u64,
    woken: u64,
    thread: Option<Thread>,
}

struct Slow {
    seq: u32,
    start: Instant,
    ns: u128,
    sleeps: u64,
    woken: u64,
    thread: Option<Thread>,
}

pub struct Trace {
    fallback: bool,
    anchor: Instant,
    mono_ns: u64,
    anchor_span_ns: u64,
    thread: Thread,
    slow: Vec<Slow>,
}

impl Trace {
    pub fn from_env(rounds: u32) -> Result<Option<Self>> {
        let fallback = match std::env::var("RP1_RTT_TRACE").as_deref() {
            Err(std::env::VarError::NotPresent) => return Ok(None),
            Ok("tracefs") => false,
            Ok("cpu") => true,
            _ => return Err("RP1_RTT_TRACE must be tracefs or cpu".into()),
        };
        let slow = Vec::with_capacity(rounds as usize);
        let pid = std::process::id();
        let status = std::fs::read_to_string("/proc/thread-self/status")?;
        for line in status
            .lines()
            .filter(|line| line.starts_with("Cpus_allowed_list:") || line.starts_with("VmLck:"))
        {
            println!("RTT_STATUS {line}");
        }
        // Use Linux's one-int sched_param ABI. The musl sched_getparam wrapper
        // may return ENOSYS instead of issuing the Linux syscall.
        let mut priority: libc::c_int = 0;
        // SAFETY: zero selects the calling thread and priority is writable.
        let policy = unsafe { libc::syscall(libc::SYS_sched_getscheduler, 0) };
        if policy < 0 || unsafe { libc::syscall(libc::SYS_sched_getparam, 0, &mut priority) } != 0 {
            return Err(io::Error::last_os_error().into());
        }
        println!("RTT_TASK pid={pid} policy={policy} priority={priority}");
        // Linux's Instant and tracefs's mono clock both use CLOCK_MONOTONIC.
        // Bracket the anchor to bound timestamp alignment error.
        let lower = clock(libc::CLOCK_MONOTONIC)?;
        let anchor = Instant::now();
        let upper = clock(libc::CLOCK_MONOTONIC)?;
        Ok(Some(Self {
            fallback,
            anchor,
            mono_ns: lower + (upper - lower) / 2,
            anchor_span_ns: upper - lower,
            thread: Thread::sample()?,
            slow,
        }))
    }

    pub fn begin(&self, client: &Client<u32, u32>) -> Before {
        let stats = client.wait_stats();
        Before {
            sleeps: stats.sleeps,
            woken: stats.woken,
            thread: self
                .fallback
                .then(|| Thread::sample().expect("thread counters")),
        }
    }

    pub fn end(
        &mut self,
        client: &Client<u32, u32>,
        before: Before,
        seq: u32,
        start: Instant,
        ns: u128,
    ) {
        if ns <= 25_000 {
            return;
        }
        let thread = before
            .thread
            .map(|before| Thread::sample().expect("thread counters").since(before));
        let stats = client.wait_stats();
        self.slow.push(Slow {
            seq,
            start,
            ns,
            sleeps: stats.sleeps - before.sleeps,
            woken: stats.woken - before.woken,
            thread,
        });
    }

    pub fn report(self) -> Result<()> {
        let thread = Thread::sample()?.since(self.thread);
        println!(
            "RTT_OBSERVE mode={} anchor_span_ns={} voluntary={} involuntary={} slow={}",
            if self.fallback { "cpu" } else { "tracefs" },
            self.anchor_span_ns,
            thread.voluntary,
            thread.involuntary,
            self.slow.len()
        );
        for slow in self.slow {
            let start_ns = u128::from(self.mono_ns) + (slow.start - self.anchor).as_nanos();
            let end_ns = start_ns + slow.ns;
            print!(
                "RTT_SLOW seq={} start_ns={start_ns} end_ns={end_ns} ns={} sleeps={} woken={}",
                slow.seq, slow.ns, slow.sleeps, slow.woken
            );
            if let Some(thread) = slow.thread {
                print!(
                    " cpu_ns={} voluntary={} involuntary={}",
                    thread.cpu_ns, thread.voluntary, thread.involuntary
                );
            }
            println!();
        }
        Ok(())
    }
}
