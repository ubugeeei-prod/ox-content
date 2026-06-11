//! Counting global allocator.
//!
//! `CountingAllocator` wraps `std::alloc::System` and records every
//! allocation/deallocation through atomic counters. The counters are
//! process-global, so a scoped measurement uses a snapshot/delta pair rather
//! than try to reset shared state.
//!
//! The instrumentation can be globally disabled (e.g. while the profiler
//! itself is allocating its report) by calling [`CountingAllocator::enable`]
//! / [`CountingAllocator::disable`]. Disabled allocations still flow through
//! `System`; they just don't update the counters.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

mod counter;
mod histogram;
mod snapshot;

#[cfg(test)]
mod tests;

pub use counter::AllocCounter;
use histogram::size_class_bucket;
pub use histogram::SizeHistogram;
pub use snapshot::{AllocDelta, AllocSnapshot};

const SIZE_CLASS_BUCKETS: usize = 32;

#[allow(unsafe_code)]
struct GlobalStats {
    enabled: AtomicBool,
    allocations: AtomicU64,
    deallocations: AtomicU64,
    bytes_allocated: AtomicU64,
    bytes_deallocated: AtomicU64,
    current_live_bytes: AtomicU64,
    peak_live_bytes: AtomicU64,
    largest_single_alloc: AtomicU64,
    size_class_buckets: [AtomicU64; SIZE_CLASS_BUCKETS],
}

impl GlobalStats {
    const fn new() -> Self {
        // Manually expand the array since `[AtomicU64::new(0); N]` requires
        // `Copy` which atomics intentionally don't implement.
        Self {
            enabled: AtomicBool::new(false),
            allocations: AtomicU64::new(0),
            deallocations: AtomicU64::new(0),
            bytes_allocated: AtomicU64::new(0),
            bytes_deallocated: AtomicU64::new(0),
            current_live_bytes: AtomicU64::new(0),
            peak_live_bytes: AtomicU64::new(0),
            largest_single_alloc: AtomicU64::new(0),
            size_class_buckets: [
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
            ],
        }
    }
}

static STATS: GlobalStats = GlobalStats::new();

/// Counting wrapper around the system allocator.
///
/// Install with `#[global_allocator]`; call [`CountingAllocator::enable`] from
/// `main` before the profiled workload starts. Counters use relaxed atomics
/// because we only need monotonic visibility, not ordering with other memory.
pub struct CountingAllocator;

impl CountingAllocator {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Begin recording allocation events. Idempotent.
    pub fn enable() {
        STATS.enabled.store(true, Ordering::Release);
    }

    /// Stop recording allocation events. Existing counters keep their value.
    pub fn disable() {
        STATS.enabled.store(false, Ordering::Release);
    }

    /// Returns whether the allocator is currently recording events.
    #[must_use]
    pub fn is_enabled() -> bool {
        STATS.enabled.load(Ordering::Acquire)
    }
}

impl Default for CountingAllocator {
    fn default() -> Self {
        Self::new()
    }
}

// SAFETY: We forward every allocation/deallocation to `System` without
// modifying the returned pointer or the requested layout. The atomic
// bookkeeping is independent of allocator correctness; even if every counter
// were poisoned, allocation behavior would still be sound. Layout invariants
// (alignment, size) are preserved because we delegate verbatim.
#[allow(unsafe_code)]
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: `layout` is a caller-provided valid `Layout`; `System`
        // upholds the `GlobalAlloc` contract for it.
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() && STATS.enabled.load(Ordering::Relaxed) {
            record_alloc(layout.size() as u64);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if STATS.enabled.load(Ordering::Relaxed) {
            record_dealloc(layout.size() as u64);
        }
        // SAFETY: caller guarantees `ptr` came from this allocator with the
        // matching `layout`. We pass both through unchanged.
        unsafe { System.dealloc(ptr, layout) };
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // SAFETY: same as `alloc` above; `System` handles the zeroing.
        let ptr = unsafe { System.alloc_zeroed(layout) };
        if !ptr.is_null() && STATS.enabled.load(Ordering::Relaxed) {
            record_alloc(layout.size() as u64);
        }
        ptr
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // We model realloc as a dealloc of the old layout + alloc of the new
        // size. This double-counts a single OS-level operation but keeps the
        // accounting symmetric with explicit alloc/dealloc patterns, which is
        // what users tend to reason about.
        // SAFETY: caller guarantees `ptr`/`layout` match and `new_size` is
        // valid for the inferred new layout; `System` handles the rest.
        let new_ptr = unsafe { System.realloc(ptr, layout, new_size) };
        if !new_ptr.is_null() && STATS.enabled.load(Ordering::Relaxed) {
            record_dealloc(layout.size() as u64);
            record_alloc(new_size as u64);
        }
        new_ptr
    }
}

#[inline]
fn record_alloc(size: u64) {
    STATS.allocations.fetch_add(1, Ordering::Relaxed);
    STATS.bytes_allocated.fetch_add(size, Ordering::Relaxed);
    let live = STATS.current_live_bytes.fetch_add(size, Ordering::Relaxed) + size;

    // Peak tracking via CAS loop. Under contention we may miss the absolute
    // peak by a few bytes between concurrent threads, which is acceptable for
    // a profiler — we are not trying to compete with jemalloc's accuracy.
    let mut peak = STATS.peak_live_bytes.load(Ordering::Relaxed);
    while live > peak {
        match STATS.peak_live_bytes.compare_exchange_weak(
            peak,
            live,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => break,
            Err(observed) => peak = observed,
        }
    }

    let mut largest = STATS.largest_single_alloc.load(Ordering::Relaxed);
    while size > largest {
        match STATS.largest_single_alloc.compare_exchange_weak(
            largest,
            size,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => break,
            Err(observed) => largest = observed,
        }
    }

    let bucket = size_class_bucket(size);
    STATS.size_class_buckets[bucket].fetch_add(1, Ordering::Relaxed);
}

#[inline]
fn record_dealloc(size: u64) {
    STATS.deallocations.fetch_add(1, Ordering::Relaxed);
    STATS.bytes_deallocated.fetch_add(size, Ordering::Relaxed);
    // Live bytes can wrap if events are observed out of order between
    // threads; saturating subtract keeps the gauge non-negative.
    let mut cur = STATS.current_live_bytes.load(Ordering::Relaxed);
    loop {
        let next = cur.saturating_sub(size);
        match STATS.current_live_bytes.compare_exchange_weak(
            cur,
            next,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => break,
            Err(observed) => cur = observed,
        }
    }
}
