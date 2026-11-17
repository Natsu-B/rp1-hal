//! Exhaustive interleaving model of the handshake (protocol.md §2.3) on the real code.
//!
//! Each side's stores into the peer's memory are posted: the writer writes a private copy
//! of `data` and the reader a private copy of `ctrl` (`shadow`); a store reaches the peer's
//! copy (`real`) only when it lands. Landing is in order, so a path lands as a whole: either
//! spontaneously at any point, or when its own side reads back through the same window (a
//! non-posted read does not pass earlier posted writes). Every load in the ring code is a
//! scheduling point (`ld` calls [`before_load`]); the two sides run on threads that hand a
//! single token back and forth (so there is no data race), and a depth-first search over the
//! choices "run writer / run reader / land data / land ctrl" visits every schedule.
extern crate std;
use std::cell::RefCell;
use std::sync::{Arc, Condvar, Mutex};
use std::vec::Vec;

/// A posted-write path: what the writing side sees and what the peer sees.
pub struct Path {
    pub shadow: usize,
    pub real: usize,
    pub len: usize,
}
impl Path {
    fn pending(&self) -> bool {
        // SAFETY: both buffers live for the run; only the token holder touches them.
        unsafe {
            std::slice::from_raw_parts(self.shadow as *const u8, self.len)
                != std::slice::from_raw_parts(self.real as *const u8, self.len)
        }
    }
    fn land(&self) {
        unsafe {
            core::ptr::copy_nonoverlapping(self.shadow as *const u8, self.real as *mut u8, self.len)
        }
    }
}
#[derive(Default)]
struct State {
    running: Option<usize>,
    parked: [bool; 2],
    /// Asleep: not runnable until the other side is done.
    asleep: [bool; 2],
    done: [bool; 2],
}
pub struct Sched {
    state: Mutex<State>,
    cv: Condvar,
    pub paths: [Path; 2],
}
std::thread_local!(static CURRENT: RefCell<Option<(Arc<Sched>, usize)>> = const { RefCell::new(None) });

/// Called by `ld` before every load; a no-op outside a model run.
pub fn before_load(addr: *const u8) {
    let Some((s, me)) = CURRENT.with(|c| c.borrow().clone()) else {
        return;
    };
    s.park(me);
    for p in &s.paths {
        if (p.shadow..p.shadow + p.len).contains(&(addr as usize)) {
            p.land();
        }
    }
}
impl Sched {
    pub fn new(paths: [Path; 2]) -> Arc<Self> {
        Arc::new(Self {
            state: Mutex::new(State::default()),
            cv: Condvar::new(),
            paths,
        })
    }
    fn park(&self, me: usize) {
        self.park_as(me, false);
    }
    fn park_as(&self, me: usize, asleep: bool) {
        let mut g = self.state.lock().unwrap();
        g.parked[me] = true;
        g.asleep[me] = asleep;
        g.running = None;
        self.cv.notify_all();
        while g.running != Some(me) {
            g = self.cv.wait(g).unwrap();
        }
        g.parked[me] = false;
    }
    /// Sleep (inside side `me`) until the other side has finished: a notification
    /// would wake it, and none can come later.
    pub fn sleep(&self, me: usize) {
        self.park_as(me, true);
    }
    /// Run `f` as side `me` (0 = writer, 1 = reader) under the scheduler.
    pub fn side<R>(self: &Arc<Self>, me: usize, f: impl FnOnce() -> R) -> R {
        CURRENT.with(|c| *c.borrow_mut() = Some((self.clone(), me)));
        self.park(me);
        let r = f();
        CURRENT.with(|c| *c.borrow_mut() = None);
        let mut g = self.state.lock().unwrap();
        g.done[me] = true;
        g.running = None;
        self.cv.notify_all();
        r
    }
    /// Drive both sides to completion; `choose(n)` picks one of `n` enabled steps.
    pub fn drive(&self, mut choose: impl FnMut(usize) -> usize, trace: &mut Vec<usize>) {
        loop {
            let mut g = self.state.lock().unwrap();
            while g.running.is_some() || (0..2).any(|t| !g.done[t] && !g.parked[t]) {
                g = self.cv.wait(g).unwrap();
            }
            // 0..2: run that side; 2..4: land that path.
            let steps: Vec<usize> = (0..2)
                .filter(|&t| g.parked[t] && !(g.asleep[t] && !g.done[1 - t]))
                .chain((0..2).filter(|&p| self.paths[p].pending()).map(|p| p + 2))
                .collect();
            if (0..2).all(|t| g.done[t]) {
                return;
            }
            let step = steps[choose(steps.len())];
            trace.push(step);
            if step >= 2 {
                self.paths[step - 2].land();
            } else {
                g.running = Some(step);
                self.cv.notify_all();
            }
        }
    }
}

/// Depth-first search over every sequence of choices `run` makes; returns the count.
pub fn explore(mut run: impl FnMut(&mut dyn FnMut(usize) -> usize)) -> usize {
    let mut prefix: Vec<usize> = Vec::new();
    let mut runs = 0;
    loop {
        let mut taken: Vec<(usize, usize)> = Vec::new();
        run(&mut |n| {
            let c = prefix.get(taken.len()).copied().unwrap_or(0);
            taken.push((c, n));
            c
        });
        runs += 1;
        while let Some((c, n)) = taken.pop() {
            if c + 1 < n {
                taken.push((c + 1, n));
                break;
            }
        }
        if taken.is_empty() {
            return runs;
        }
        prefix = taken.into_iter().map(|(c, _)| c).collect();
    }
}
