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
fn bucket_labels_cover_zero_and_wide_classes() {
    assert_eq!(SizeHistogram::bucket_label(0), "0");
    assert_eq!(SizeHistogram::bucket_label(1), "1");
    assert_eq!(SizeHistogram::bucket_label(2), "2..4");
    assert_eq!(SizeHistogram::bucket_label(6), "32..64");
    assert_eq!(SizeHistogram::bucket_label(63), format!("{}..{}", 1u64 << 62, 1u64 << 63));
    assert_eq!(SizeHistogram::bucket_label(usize::MAX), format!("{}..{}", 1u64 << 63, 1u64 << 63));
}

#[test]
fn delta_handles_no_change() {
    let snap = AllocSnapshot::capture();
    let delta = snap.delta_from(&snap);
    assert_eq!(delta.allocations, 0);
    assert_eq!(delta.bytes_allocated, 0);
    assert_eq!(delta.net_growth(), 0);
}
