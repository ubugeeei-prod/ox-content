use std::time::Duration;

use super::{AllocSummary, IterationRecord, SpanAggregate, TimingSummary};

pub(super) fn summarize_timing(
    iters: &[&IterationRecord],
    input_bytes: Option<u64>,
) -> TimingSummary {
    if iters.is_empty() {
        return TimingSummary {
            samples: 0,
            min: Duration::ZERO,
            p50: Duration::ZERO,
            p95: Duration::ZERO,
            p99: Duration::ZERO,
            max: Duration::ZERO,
            mean: Duration::ZERO,
            throughput_mb_s: None,
        };
    }
    let mut times: Vec<Duration> = iters.iter().map(|i| i.elapsed).collect();
    times.sort_unstable();
    let n = times.len();
    let sum: Duration = times.iter().sum();
    let mean = sum / (n as u32);
    let percentile = |p: f64| -> Duration {
        let idx = ((p / 100.0) * (n as f64 - 1.0)).round() as usize;
        times[idx.min(n - 1)]
    };
    let p50 = percentile(50.0);
    let p95 = percentile(95.0);
    let p99 = percentile(99.0);
    let min = times[0];
    let max = times[n - 1];

    // Throughput uses the median because it's robust to outliers and matches
    // what the existing benchmark README reports.
    let throughput_mb_s = input_bytes.and_then(|bytes| {
        let secs = p50.as_secs_f64();
        if secs > 0.0 {
            Some((bytes as f64) / secs / (1024.0 * 1024.0))
        } else {
            None
        }
    });

    TimingSummary { samples: n, min, p50, p95, p99, max, mean, throughput_mb_s }
}

pub(super) fn summarize_allocs(iters: &[&IterationRecord]) -> AllocSummary {
    if iters.is_empty() {
        return AllocSummary {
            samples: 0,
            mean_allocations: 0.0,
            mean_bytes: 0.0,
            max_peak_above_baseline: 0,
            largest_single_alloc: 0,
        };
    }
    let n = iters.len() as f64;
    let sum_allocs: u64 = iters.iter().map(|i| i.allocs.allocations).sum();
    let sum_bytes: u64 = iters.iter().map(|i| i.allocs.bytes_allocated).sum();
    let max_peak = iters.iter().map(|i| i.allocs.peak_above_baseline).max().unwrap_or(0);
    let largest = iters.iter().map(|i| i.allocs.largest_single_alloc).max().unwrap_or(0);
    AllocSummary {
        samples: iters.len(),
        mean_allocations: sum_allocs as f64 / n,
        mean_bytes: sum_bytes as f64 / n,
        max_peak_above_baseline: max_peak,
        largest_single_alloc: largest,
    }
}

pub(super) fn aggregate_spans(iters: &[&IterationRecord]) -> Vec<SpanAggregate> {
    let mut agg: Vec<SpanAggregate> = Vec::new();
    for iter in iters {
        for span in &iter.spans {
            if let Some(slot) = agg.iter_mut().find(|s| s.name == span.name) {
                slot.hits += span.hits;
                slot.total_inclusive += span.total_inclusive;
                slot.total_self += span.total_self;
                slot.total_allocs += span.total_allocs;
                slot.total_bytes += span.total_bytes;
                if span.max_peak_above_baseline > slot.max_peak_above_baseline {
                    slot.max_peak_above_baseline = span.max_peak_above_baseline;
                }
            } else {
                agg.push(SpanAggregate {
                    name: span.name,
                    hits: span.hits,
                    total_inclusive: span.total_inclusive,
                    total_self: span.total_self,
                    total_allocs: span.total_allocs,
                    total_bytes: span.total_bytes,
                    max_peak_above_baseline: span.max_peak_above_baseline,
                    share_of_total: 0.0,
                });
            }
        }
    }
    // Total reference for share-of-total: sum of all top-level spans'
    // inclusive time, falling back to the sum of self times if no clear
    // root exists. Spans with the same name across iterations have already
    // been merged into a single row at this point.
    let total: Duration = agg.iter().map(|s| s.total_self).sum();
    if !total.is_zero() {
        let total_ns = total.as_nanos() as f64;
        for s in &mut agg {
            s.share_of_total = s.total_self.as_nanos() as f64 / total_ns;
        }
    }
    agg.sort_by_key(|s| std::cmp::Reverse(s.total_inclusive));
    agg
}
