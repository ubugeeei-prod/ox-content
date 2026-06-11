use super::*;

// We can't safely install ourselves as the global allocator in these
// tests (the cfg is per-binary), but we can still exercise the snapshot
// arithmetic and the bucket layout.

#[test]
fn bucket_layout_covers_typical_sizes() {
    let mut hist = SizeHistogram { buckets: [0; SIZE_CLASS_BUCKETS] };
    for &size in &[0u64, 1, 2, 7, 16, 17, 64, 1024, 4096, 65_536] {
        let b = size_class_bucket(size);
        hist.buckets[b] += 1;
    }
    let count = hist.iter_nonempty().count();
    assert!(count > 0);
}

#[test]
fn delta_handles_no_change() {
    let snap = AllocSnapshot::capture();
    let delta = snap.delta_from(&snap);
    assert_eq!(delta.allocations, 0);
    assert_eq!(delta.bytes_allocated, 0);
    assert_eq!(delta.net_growth(), 0);
}
