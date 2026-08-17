//! Thread-local hierarchical timing spans.
//!
//! The span machinery is driven exclusively through [`ScopeGuard::enter`]:
//! call it to push a frame, drop the guard to pop. Each frame remembers its
//! parent's accumulated child-time so that on close we can split inclusive
//! time (wall clock between enter and drop) from self time (inclusive minus
//! the sum of child inclusive times).
//!
//! All state lives in a `thread_local!` cell so spans never need a lock on
//! the hot path. Pull the per-iteration tree out with [`take_thread_spans`]
//! between iterations; flush with [`reset_thread_spans`] before starting one.
//!
//! The runtime gate ([`enable`] / [`disable`]) is a single relaxed `AtomicBool`
//! lookup per `enter`/`drop`, which is cheap enough to leave on the hot path
//! even at default-disabled. When disabled, [`ScopeGuard`] becomes a
//! zero-state marker.

use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use crate::alloc::{AllocCounter, AllocDelta};

static ENABLED: AtomicBool = AtomicBool::new(false);

/// Second gate for micro-spans ("detail" tracing).
///
/// Phase-level spans keep the default profile readable and cheap; detail
/// spans sit on per-node / per-line hot paths where the guard's own cost is
/// comparable to the work being measured. They only record when BOTH gates
/// are open, so a default run keeps its phase-level numbers undistorted and
/// an explicit `--detail` run trades accuracy for the full trace.
static DETAIL: AtomicBool = AtomicBool::new(false);

/// Globally enable span recording. Spans entered while disabled are no-ops.
pub fn enable() {
    ENABLED.store(true, Ordering::Release);
}

/// Globally disable span recording. Already-open guards keep working but
/// record nothing on drop.
pub fn disable() {
    ENABLED.store(false, Ordering::Release);
}

/// Whether the span subsystem is currently recording.
#[must_use]
pub fn is_enabled() -> bool {
    ENABLED.load(Ordering::Acquire)
}

/// Enable detail (micro) spans. Has no effect unless [`enable`] is also on.
pub fn enable_detail() {
    DETAIL.store(true, Ordering::Release);
}

/// Disable detail (micro) spans; phase-level spans keep recording.
pub fn disable_detail() {
    DETAIL.store(false, Ordering::Release);
}

/// Whether detail spans are currently recording (both gates open).
#[must_use]
pub fn is_detail_enabled() -> bool {
    is_enabled() && DETAIL.load(Ordering::Acquire)
}

thread_local! {
    static STATE: RefCell<SpanState> = const { RefCell::new(SpanState::new()) };
}

/// Per-thread span state. Open frames live on `stack`; closed frames are
/// merged into `records` keyed by the span name.
struct SpanState {
    stack: Vec<Frame>,
    records: Vec<ScopeRecord>,
}

impl SpanState {
    const fn new() -> Self {
        Self { stack: Vec::new(), records: Vec::new() }
    }
}

struct Frame {
    name: &'static str,
    start: Instant,
    child_time: Duration,
    alloc_baseline: AllocCounter,
}

/// RAII guard for a timing span. Construct via [`ScopeGuard::enter`].
pub struct ScopeGuard {
    /// `true` if we pushed a frame; `false` when the subsystem was disabled
    /// at `enter` time and we should be a no-op on drop.
    active: bool,
}

impl ScopeGuard {
    /// Push a new frame onto the thread-local stack.
    ///
    /// The label MUST be `'static` so the registry can use it as a key
    /// without copying.
    #[inline]
    pub fn enter(name: &'static str) -> Self {
        if !is_enabled() {
            return Self { active: false };
        }
        STATE.with(|cell| {
            // Borrow may already be held if a Drop impl re-enters us; in that
            // case we silently degrade rather than panic.
            let Ok(mut state) = cell.try_borrow_mut() else {
                return;
            };
            state.stack.push(Frame {
                name,
                start: Instant::now(),
                child_time: Duration::ZERO,
                alloc_baseline: AllocCounter::start(),
            });
        });
        Self { active: true }
    }

    /// Push a detail (micro) span frame. No-op unless both the span
    /// subsystem and the detail gate are enabled — see [`enable_detail`].
    #[inline]
    pub fn enter_detail(name: &'static str) -> Self {
        if !is_detail_enabled() {
            return Self { active: false };
        }
        Self::enter(name)
    }

    /// Whether this guard is recording. False if the subsystem was disabled
    /// when this guard was created (the common no-op case).
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active
    }
}

impl Drop for ScopeGuard {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        STATE.with(|cell| {
            let Ok(mut state) = cell.try_borrow_mut() else {
                return;
            };
            let Some(frame) = state.stack.pop() else {
                return;
            };
            let inclusive = frame.start.elapsed();
            let self_time = inclusive.saturating_sub(frame.child_time);
            let alloc_delta = frame.alloc_baseline.delta();

            // Bubble inclusive time up to the parent's child accumulator so
            // when the parent closes, its self_time excludes this child.
            if let Some(parent) = state.stack.last_mut() {
                parent.child_time += inclusive;
            }

            merge_record(&mut state.records, frame.name, inclusive, self_time, &alloc_delta);
        });
    }
}

fn merge_record(
    records: &mut Vec<ScopeRecord>,
    name: &'static str,
    inclusive: Duration,
    self_time: Duration,
    alloc: &AllocDelta,
) {
    if let Some(slot) = records.iter_mut().find(|r| r.name == name) {
        slot.hits += 1;
        slot.total_inclusive += inclusive;
        slot.total_self += self_time;
        slot.total_allocs += alloc.allocations;
        slot.total_bytes += alloc.bytes_allocated;
        if alloc.peak_above_baseline > slot.max_peak_above_baseline {
            slot.max_peak_above_baseline = alloc.peak_above_baseline;
        }
        return;
    }
    records.push(ScopeRecord {
        name,
        hits: 1,
        total_inclusive: inclusive,
        total_self: self_time,
        total_allocs: alloc.allocations,
        total_bytes: alloc.bytes_allocated,
        max_peak_above_baseline: alloc.peak_above_baseline,
    });
}

/// Aggregated record for all hits of a named span within one iteration.
#[derive(Debug, Clone)]
pub struct ScopeRecord {
    pub name: &'static str,
    pub hits: u64,
    pub total_inclusive: Duration,
    pub total_self: Duration,
    pub total_allocs: u64,
    pub total_bytes: u64,
    pub max_peak_above_baseline: u64,
}

/// Drain the current thread's recorded spans, returning them.
#[must_use]
pub fn take_thread_spans() -> Vec<ScopeRecord> {
    STATE.with(|cell| {
        let Ok(mut state) = cell.try_borrow_mut() else {
            return Vec::new();
        };
        std::mem::take(&mut state.records)
    })
}

/// Clear the current thread's records without returning them.
pub fn reset_thread_spans() {
    STATE.with(|cell| {
        if let Ok(mut state) = cell.try_borrow_mut() {
            state.records.clear();
            // Leave any open frames alone; the caller is responsible for not
            // having an open span when starting a new iteration.
        }
    });
}

/// Open a span using a static label and an arbitrary block.
///
/// This is the function-style equivalent of [`crate::profile_span!`] for
/// callers that find the macro inconvenient (e.g. when returning from inside
/// the span). Returns whatever `f` returns.
pub fn span<R>(name: &'static str, f: impl FnOnce() -> R) -> R {
    let _guard = ScopeGuard::enter(name);
    f()
}

/// Estimates the per-hit cost of one enabled `enter`/`drop` guard pair, in
/// nanoseconds.
///
/// With detail tracing on, guard cost is comparable to the work inside the
/// hottest micro-spans, so reports need a yardstick to stay honest: the
/// table renderer multiplies this by each row's hit count to show how much
/// of the measured time is the measurement itself.
///
/// Requires the span subsystem to be [`enable`]d (returns 0.0 otherwise so
/// callers can pass the result straight into a report config). Runs a few
/// thousand guard pairs and takes the fastest batch to approximate the
/// steady-state cost; the calibration records are drained afterwards so
/// they never leak into a real measurement window.
#[must_use]
pub fn calibrate_overhead_ns() -> f64 {
    const BATCH: u32 = 1024;
    const ROUNDS: usize = 8;
    if !is_enabled() {
        return 0.0;
    }
    let mut best = Duration::MAX;
    for _ in 0..ROUNDS {
        let start = Instant::now();
        for _ in 0..BATCH {
            let _guard = ScopeGuard::enter("__ox_profiler_calibration");
        }
        let elapsed = start.elapsed();
        if elapsed < best {
            best = elapsed;
        }
    }
    // Drop the calibration span's records so they don't pollute the first
    // real iteration.
    let _ = take_thread_spans();
    best.as_nanos() as f64 / f64::from(BATCH)
}

/// Registry view used by the report. Owns no state itself; constructed on
/// demand from a list of per-iteration [`ScopeRecord`]s.
#[derive(Debug, Default)]
pub struct SpanRegistry {
    pub records: Vec<ScopeRecord>,
}

#[cfg(test)]
mod tests;
