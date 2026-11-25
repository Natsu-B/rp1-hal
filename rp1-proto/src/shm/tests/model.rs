//! Exhaustive interleaving model of the handshake (protocol.md §2.3) on the real code.
//!
//! Each side's stores into the peer's memory are posted: the writer writes a private copy
//! of `data` and the reader a private copy of `ctrl` (`shadow`); a store reaches the peer's
//! copy (`real`) only when it lands. Every load in the ring code is a scheduling point (`ld`
//! calls [`before_load`]); the sides (writers and readers of one or more rings) run on
//! threads that hand a single token around (so there is no data race). When a side parks,
//! what it stored since it last parked is posted as one snapshot of its shadow. Snapshots
//! land in order: before a side loads from a path's real copy, any number of the path's
//! oldest snapshots may land (the only moment where landing can be observed), and a side
//! reading back through its own window lands all of them (a non-posted read does not pass
//! earlier posted writes). A depth-first search over these choices and over which side
//! runs visits every schedule. The doorbell models the Linux -> RP1 notification: a level
//! that a writer sets and the reader's handler clears.
extern crate std;
use std::cell::RefCell;
use std::sync::{Arc, Condvar, Mutex};
use std::vec;
use std::vec::Vec;

/// A posted-write path: what the writing side sees and what the peer sees.
pub struct Path {
    /// The side whose stores go this way.
    side: usize,
    shadow: usize,
    real: usize,
    len: usize,
    /// Snapshots of `shadow` not landed yet, oldest first.
    posted: Mutex<Vec<Vec<u8>>>,
}
impl Path {
    pub fn new(side: usize, shadow: *const u8, real: *const u8, len: usize) -> Self {
        Self {
            side,
            shadow: shadow as usize,
            real: real as usize,
            len,
            posted: Mutex::new(Vec::new()),
        }
    }
    fn holds(&self, base: usize, addr: usize) -> bool {
        (base..base + self.len).contains(&addr)
    }
    fn bytes(&self, base: usize) -> &[u8] {
        // SAFETY: both buffers live for the run; only the token holder touches them.
        unsafe { std::slice::from_raw_parts(base as *const u8, self.len) }
    }
    /// Post what the side stored since the last snapshot.
    fn post(&self) {
        let mut posted = self.posted.lock().unwrap();
        let latest = posted.last().map_or(self.bytes(self.real), |s| &s[..]);
        if latest != self.bytes(self.shadow) {
            let snapshot = self.bytes(self.shadow).to_vec();
            posted.push(snapshot);
        }
    }
    fn pending(&self) -> usize {
        self.posted.lock().unwrap().len()
    }
    /// Land the `n` oldest snapshots.
    fn land(&self, n: usize) {
        for snapshot in self.posted.lock().unwrap().drain(..n) {
            unsafe {
                core::ptr::copy_nonoverlapping(snapshot.as_ptr(), self.real as *mut u8, self.len)
            }
        }
    }
    /// Land everything stored so far.
    pub fn land_all(&self) {
        self.post();
        self.land(self.pending());
    }
}
/// Why a parked side may not run yet.
#[derive(Clone, Copy, Eq, PartialEq)]
enum Sleep {
    No,
    /// Until every other side has finished.
    UntilDone,
    /// Until the doorbell is set (or every other side has finished).
    Doorbell,
}
struct State {
    running: Option<usize>,
    parked: Vec<bool>,
    sleep: Vec<Sleep>,
    done: Vec<bool>,
    /// The address a parked side loads first when it runs.
    next_load: Vec<Option<usize>>,
    /// The doorbell force (protocol P4): a level, set by `raise` and kept until `ack`.
    doorbell: bool,
}
impl State {
    fn others_done(&self, me: usize) -> bool {
        (0..self.done.len()).all(|t| t == me || self.done[t])
    }
    fn runnable(&self, t: usize) -> bool {
        self.parked[t]
            && match self.sleep[t] {
                Sleep::No => true,
                Sleep::UntilDone => self.others_done(t),
                Sleep::Doorbell => self.doorbell || self.others_done(t),
            }
    }
}
pub struct Sched {
    state: Mutex<State>,
    cv: Condvar,
    pub paths: Vec<Path>,
}
std::thread_local!(static CURRENT: RefCell<Option<(Arc<Sched>, usize)>> = const { RefCell::new(None) });

/// Called by `ld` before every load; a no-op outside a model run.
pub fn before_load(addr: *const u8) {
    let Some((s, me)) = CURRENT.with(|c| c.borrow().clone()) else {
        return;
    };
    s.park_as(me, Sleep::No, Some(addr as usize));
    for p in &s.paths {
        if p.holds(p.shadow, addr as usize) {
            p.land_all();
        }
    }
}
impl Sched {
    pub fn new(sides: usize, paths: Vec<Path>) -> Arc<Self> {
        Arc::new(Self {
            state: Mutex::new(State {
                running: None,
                parked: vec![false; sides],
                sleep: vec![Sleep::No; sides],
                done: vec![false; sides],
                next_load: vec![None; sides],
                doorbell: false,
            }),
            cv: Condvar::new(),
            paths,
        })
    }
    fn post(&self, me: usize) {
        for p in self.paths.iter().filter(|p| p.side == me) {
            p.post();
        }
    }
    fn park(&self, me: usize) {
        self.park_as(me, Sleep::No, None);
    }
    fn park_as(&self, me: usize, sleep: Sleep, next_load: Option<usize>) {
        self.post(me);
        let mut g = self.state.lock().unwrap();
        g.parked[me] = true;
        g.sleep[me] = sleep;
        g.next_load[me] = next_load;
        g.running = None;
        self.cv.notify_all();
        while g.running != Some(me) {
            g = self.cv.wait(g).unwrap();
        }
        g.parked[me] = false;
    }
    /// Sleep (inside side `me`) until every other side has finished: a notification
    /// would wake it, and none can come later.
    pub fn sleep(&self, me: usize) {
        self.park_as(me, Sleep::UntilDone, None);
    }
    /// Sleep (inside side `me`) until the doorbell is set; `false` if every other side
    /// finished with the doorbell clear, so it can never be set again.
    pub fn wait_doorbell(&self, me: usize) -> bool {
        self.park_as(me, Sleep::Doorbell, None);
        self.state.lock().unwrap().doorbell
    }
    /// Set the doorbell (as side `me`, after a scheduling point: the store may come late).
    pub fn raise(&self, me: usize) {
        self.park(me);
        self.state.lock().unwrap().doorbell = true;
    }
    /// Clear the doorbell (as side `me`, after a scheduling point like `raise`).
    pub fn ack(&self, me: usize) {
        self.park(me);
        self.state.lock().unwrap().doorbell = false;
    }
    /// Run `f` as side `me` under the scheduler.
    pub fn side<R>(self: &Arc<Self>, me: usize, f: impl FnOnce() -> R) -> R {
        CURRENT.with(|c| *c.borrow_mut() = Some((self.clone(), me)));
        self.park(me);
        let r = f();
        CURRENT.with(|c| *c.borrow_mut() = None);
        self.post(me);
        let mut g = self.state.lock().unwrap();
        g.done[me] = true;
        g.running = None;
        self.cv.notify_all();
        r
    }
    /// Drive every side to completion; `choose(n)` picks one of `n` enabled steps. At most
    /// `preemptions` times a side is switched away from while it could still run (a bound
    /// on the search; `usize::MAX` explores every schedule).
    pub fn drive(
        &self,
        mut choose: impl FnMut(usize) -> usize,
        trace: &mut Vec<usize>,
        mut preemptions: usize,
    ) {
        let mut last = None;
        loop {
            let mut g = self.state.lock().unwrap();
            let sides = g.done.len();
            while g.running.is_some() || (0..sides).any(|t| !g.done[t] && !g.parked[t]) {
                g = self.cv.wait(g).unwrap();
            }
            if g.done.iter().all(|&d| d) {
                return;
            }
            let mut runnable: Vec<usize> = (0..sides).filter(|&t| g.runnable(t)).collect();
            let keeps_running = last.filter(|l| runnable.contains(l));
            if let (Some(l), 0) = (keeps_running, preemptions) {
                runnable = vec![l];
            }
            // A step: land some snapshots of the path the side loads from next, then run it.
            let mut steps: Vec<(usize, usize, usize)> = Vec::new();
            for &t in &runnable {
                steps.push((t, 0, 0));
                let read = g.next_load[t].and_then(|a| {
                    (0..self.paths.len()).find(|&p| self.paths[p].holds(self.paths[p].real, a))
                });
                if let Some(p) = read {
                    steps.extend((1..=self.paths[p].pending()).map(|n| (t, p, n)));
                }
            }
            let (step, p, n) = steps[choose(steps.len())];
            // sides..: landed a snapshot of that path.
            trace.extend(core::iter::repeat_n(sides + p, n));
            self.paths[p].land(n);
            trace.push(step);
            if keeps_running.is_some_and(|l| l != step) {
                preemptions -= 1;
            }
            last = Some(step);
            g.running = Some(step);
            self.cv.notify_all();
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
