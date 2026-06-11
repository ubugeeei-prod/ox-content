use std::sync::atomic::Ordering;

use super::histogram::SizeHistogram;
use super::{SIZE_CLASS_BUCKETS, STATS};

/// Instantaneous snapshot of allocator counters.
#[derive(Debug, Clone)]
pub struct AllocSnapshot {
    pub allocations: u64,
    pub deallocations: u64,
    pub bytes_allocated: u64,
    pub bytes_deallocated: u64,
    pub current_live_bytes: u64,
    pub peak_live_bytes: u64,
    pub largest_single_alloc: u64,
    pub size_class_buckets: [u64; SIZE_CLASS_BUCKETS],
}

impl AllocSnapshot {
    #[must_use]
    pub fn capture() -> Self {
        let mut buckets = [0u64; SIZE_CLASS_BUCKETS];
        for (i, slot) in STATS.size_class_buckets.iter().enumerate() {
            buckets[i] = slot.load(Ordering::Relaxed);
        }
        Self {
            allocations: STATS.allocations.load(Ordering::Relaxed),
            deallocations: STATS.deallocations.load(Ordering::Relaxed),
            bytes_allocated: STATS.bytes_allocated.load(Ordering::Relaxed),
            bytes_deallocated: STATS.bytes_deallocated.load(Ordering::Relaxed),
            current_live_bytes: STATS.current_live_bytes.load(Ordering::Relaxed),
            peak_live_bytes: STATS.peak_live_bytes.load(Ordering::Relaxed),
            largest_single_alloc: STATS.largest_single_alloc.load(Ordering::Relaxed),
            size_class_buckets: buckets,
        }
    }

    /// Compute the change between `before` and `self` (`self` is the later
    /// observation).
    #[must_use]
    pub fn delta_from(&self, before: &Self) -> AllocDelta {
        let mut buckets = [0u64; SIZE_CLASS_BUCKETS];
        for (i, slot) in buckets.iter_mut().enumerate() {
            *slot = self.size_class_buckets[i].saturating_sub(before.size_class_buckets[i]);
        }
        AllocDelta {
            allocations: self.allocations.saturating_sub(before.allocations),
            deallocations: self.deallocations.saturating_sub(before.deallocations),
            bytes_allocated: self.bytes_allocated.saturating_sub(before.bytes_allocated),
            bytes_deallocated: self.bytes_deallocated.saturating_sub(before.bytes_deallocated),
            // Peak above baseline: how much higher did the peak go vs. the
            // peak at snapshot time? This is the closest single-counter
            // approximation to "max additional live bytes during the scope"
            // without resetting shared state.
            peak_above_baseline: self.peak_live_bytes.saturating_sub(before.peak_live_bytes),
            ending_live_bytes: self.current_live_bytes,
            starting_live_bytes: before.current_live_bytes,
            largest_single_alloc: self.largest_single_alloc.max(before.largest_single_alloc),
            size_class_buckets: SizeHistogram { buckets },
        }
    }
}

/// Difference between two [`AllocSnapshot`]s.
#[derive(Debug, Clone)]
pub struct AllocDelta {
    pub allocations: u64,
    pub deallocations: u64,
    pub bytes_allocated: u64,
    pub bytes_deallocated: u64,
    /// Max additional live bytes observed during the window.
    pub peak_above_baseline: u64,
    pub starting_live_bytes: u64,
    pub ending_live_bytes: u64,
    pub largest_single_alloc: u64,
    pub size_class_buckets: SizeHistogram,
}

impl AllocDelta {
    /// Net change in live bytes (ending - starting). Can be negative-looking
    /// when more was freed than allocated; returns saturating signed-ish u64
    /// for simplicity. Use [`AllocDelta::net_growth`] for clarity.
    #[must_use]
    pub fn net_growth(&self) -> i128 {
        i128::from(self.ending_live_bytes) - i128::from(self.starting_live_bytes)
    }
}
