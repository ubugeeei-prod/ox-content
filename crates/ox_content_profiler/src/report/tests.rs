use super::*;
use crate::alloc::{AllocDelta, SizeHistogram};

fn mock_iter(elapsed_us: u64, allocs: u64, bytes: u64) -> IterationRecord {
    IterationRecord {
        elapsed: Duration::from_micros(elapsed_us),
        allocs: AllocDelta {
            allocations: allocs,
            deallocations: 0,
            bytes_allocated: bytes,
            bytes_deallocated: 0,
            peak_above_baseline: bytes,
            starting_live_bytes: 0,
            ending_live_bytes: 0,
            largest_single_alloc: bytes,
            size_class_buckets: SizeHistogram { buckets: [0; 32] },
        },
        spans: vec![],
    }
}

#[test]
fn timing_percentiles_make_sense() {
    let iters = (1..=10).map(|i| mock_iter(i * 100, 1, 100)).collect::<Vec<_>>();
    let cfg = ReportConfig { input_bytes: Some(1024 * 1024), warmup: 0, max_span_rows: 8 };
    let report = Report::from_iterations("x".into(), iters, cfg);
    assert_eq!(report.timing.samples, 10);
    assert!(report.timing.min <= report.timing.p50);
    assert!(report.timing.p50 <= report.timing.p95);
    assert!(report.timing.p95 <= report.timing.max);
}

#[test]
fn warmup_skips_iterations() {
    let mut iters = vec![mock_iter(10_000, 0, 0)];
    iters.extend((0..5).map(|_| mock_iter(100, 0, 0)));
    let cfg = ReportConfig { input_bytes: None, warmup: 1, max_span_rows: 8 };
    let report = Report::from_iterations("x".into(), iters, cfg);
    assert_eq!(report.timing.samples, 5);
    // Without the warmup iteration the median should be near 100µs.
    assert!(report.timing.p50.as_micros() <= 200);
}

#[test]
fn table_render_is_non_empty() {
    let iters = vec![mock_iter(500, 3, 256)];
    let cfg = ReportConfig::default();
    let report = Report::from_iterations("smoke".into(), iters, cfg);
    let table = report.render_table();
    insta::assert_snapshot!(table);
}
