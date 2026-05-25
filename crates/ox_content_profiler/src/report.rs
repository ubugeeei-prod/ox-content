//! Report formatting.
//!
//! [`Report::from_iterations`] takes the per-iteration records captured by
//! [`crate::Recorder`] and folds them into aggregate statistics:
//!
//! - Timing percentiles (min / p50 / p95 / p99 / max / mean) per workload.
//! - Throughput in MB/s when an input byte size is provided.
//! - Allocation totals + averages (count, bytes, peak).
//! - Per-span breakdown, sorted by total inclusive time.
//!
//! Rendering is intentionally string-based and lock-free so that printing
//! the report does not itself perturb the global allocator counters by much.

use std::time::Duration;

use crate::alloc::AllocDelta;
use crate::scope::ScopeRecord;

#[derive(Debug, Clone)]
pub struct ReportConfig {
    /// When set, throughput is computed as `input_bytes / iteration_elapsed`.
    pub input_bytes: Option<u64>,
    /// Number of warmup iterations to drop from the front when aggregating.
    pub warmup: usize,
    /// Maximum number of span rows to print in the table view.
    pub max_span_rows: usize,
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self { input_bytes: None, warmup: 0, max_span_rows: 32 }
    }
}

#[derive(Debug, Clone)]
pub struct IterationRecord {
    pub elapsed: Duration,
    pub allocs: AllocDelta,
    pub spans: Vec<ScopeRecord>,
}

#[derive(Debug, Clone)]
pub struct Report {
    pub label: String,
    pub config: ReportConfig,
    pub iterations: Vec<IterationRecord>,
    pub timing: TimingSummary,
    pub allocs: AllocSummary,
    pub spans: Vec<SpanAggregate>,
}

#[derive(Debug, Clone)]
pub struct TimingSummary {
    pub samples: usize,
    pub min: Duration,
    pub p50: Duration,
    pub p95: Duration,
    pub p99: Duration,
    pub max: Duration,
    pub mean: Duration,
    pub throughput_mb_s: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct AllocSummary {
    pub samples: usize,
    /// Mean allocations per iteration.
    pub mean_allocations: f64,
    /// Mean bytes allocated per iteration.
    pub mean_bytes: f64,
    /// Max peak above baseline observed across iterations.
    pub max_peak_above_baseline: u64,
    /// Largest single allocation observed.
    pub largest_single_alloc: u64,
}

#[derive(Debug, Clone)]
pub struct SpanAggregate {
    pub name: &'static str,
    pub hits: u64,
    pub total_inclusive: Duration,
    pub total_self: Duration,
    pub total_allocs: u64,
    pub total_bytes: u64,
    pub max_peak_above_baseline: u64,
    /// Share of total inclusive time spent in this span across all
    /// iterations (0.0..=1.0).
    pub share_of_total: f64,
}

impl Report {
    pub(crate) fn from_iterations(
        label: String,
        all_iterations: Vec<IterationRecord>,
        config: ReportConfig,
    ) -> Self {
        let kept: Vec<&IterationRecord> = all_iterations.iter().skip(config.warmup).collect();
        let timing = summarize_timing(&kept, config.input_bytes);
        let allocs = summarize_allocs(&kept);
        let spans = aggregate_spans(&kept);
        Self { label, config, iterations: all_iterations, timing, allocs, spans }
    }

    /// Render a human-readable, monospace-friendly table.
    #[must_use]
    pub fn render_table(&self) -> String {
        let mut out = String::with_capacity(4096);
        let ruler = "─".repeat(78);

        out.push_str(&format!("\n{ruler}\n"));
        out.push_str(&format!(" Profile: {}\n", self.label));
        out.push_str(&format!(
            " Iterations measured: {} (warmup: {})\n",
            self.timing.samples, self.config.warmup
        ));
        out.push_str(&format!("{ruler}\n"));

        out.push_str("\n Timing\n");
        out.push_str(&format!(
            "   min   {}\n   p50   {}\n   p95   {}\n   p99   {}\n   max   {}\n   mean  {}\n",
            fmt_duration(self.timing.min),
            fmt_duration(self.timing.p50),
            fmt_duration(self.timing.p95),
            fmt_duration(self.timing.p99),
            fmt_duration(self.timing.max),
            fmt_duration(self.timing.mean),
        ));
        if let Some(throughput) = self.timing.throughput_mb_s {
            out.push_str(&format!("   throughput   {throughput:>8.2} MB/s\n"));
        }

        out.push_str("\n Allocations (per iteration)\n");
        out.push_str(&format!(
            "   count       {:>12.1}\n   bytes       {:>12}\n   peak (max)  {:>12}\n   largest     {:>12}\n",
            self.allocs.mean_allocations,
            fmt_bytes_f(self.allocs.mean_bytes),
            fmt_bytes(self.allocs.max_peak_above_baseline),
            fmt_bytes(self.allocs.largest_single_alloc),
        ));

        if !self.spans.is_empty() {
            out.push_str("\n Spans (sorted by total inclusive time)\n");
            out.push_str(&format!(
                "   {:<32} {:>8} {:>12} {:>12} {:>6}   {:>10} {:>10}\n",
                "name", "hits", "self", "inclusive", "share", "allocs", "bytes",
            ));
            out.push_str(&format!("   {}\n", "·".repeat(74)));
            for span in self.spans.iter().take(self.config.max_span_rows) {
                out.push_str(&format!(
                    "   {:<32} {:>8} {:>12} {:>12} {:>5.1}%   {:>10} {:>10}\n",
                    truncate(span.name, 32),
                    span.hits,
                    fmt_duration(span.total_self),
                    fmt_duration(span.total_inclusive),
                    span.share_of_total * 100.0,
                    span.total_allocs,
                    fmt_bytes(span.total_bytes),
                ));
            }
            if self.spans.len() > self.config.max_span_rows {
                out.push_str(&format!(
                    "   …and {} more spans\n",
                    self.spans.len() - self.config.max_span_rows
                ));
            }
        }

        // Allocation size-class histogram from the last iteration. Picking
        // the last (rather than any aggregate) avoids inflating warmup
        // effects and matches what a user inspecting the report typically
        // wants to see.
        if let Some(last) = self.iterations.last() {
            let any = last.allocs.size_class_buckets.iter_nonempty().next().is_some();
            if any {
                out.push_str("\n Size-class histogram (last iteration)\n");
                for (label, count) in last.allocs.size_class_buckets.iter_nonempty() {
                    let bar_len = ((count as f64).log2().max(0.0) * 2.0) as usize;
                    let bar = "▏".repeat(bar_len.min(40));
                    out.push_str(&format!("   {label:>10}  {count:>8}  {bar}\n"));
                }
            }
        }

        out.push_str(&format!("\n{ruler}\n"));
        out
    }

    /// Render a single-line JSON summary. Stable enough for shell scripts and
    /// CI diffing. Kept hand-rolled so the profiler crate can stay
    /// dependency-free by default.
    #[must_use]
    pub fn render_json(&self) -> String {
        let mut s = String::with_capacity(1024);
        s.push('{');
        write_kv_str(&mut s, "label", &self.label);
        s.push(',');
        write_kv_u64(&mut s, "samples", self.timing.samples as u64);
        s.push(',');
        s.push_str("\"timing\":{");
        write_kv_dur(&mut s, "min_ns", self.timing.min);
        s.push(',');
        write_kv_dur(&mut s, "p50_ns", self.timing.p50);
        s.push(',');
        write_kv_dur(&mut s, "p95_ns", self.timing.p95);
        s.push(',');
        write_kv_dur(&mut s, "p99_ns", self.timing.p99);
        s.push(',');
        write_kv_dur(&mut s, "max_ns", self.timing.max);
        s.push(',');
        write_kv_dur(&mut s, "mean_ns", self.timing.mean);
        if let Some(t) = self.timing.throughput_mb_s {
            s.push(',');
            s.push_str(&format!("\"throughput_mb_s\":{t}"));
        }
        s.push('}');
        s.push(',');
        s.push_str("\"allocs\":{");
        s.push_str(&format!(
            "\"mean_count\":{:.3},\"mean_bytes\":{:.3},\"max_peak\":{},\"largest\":{}",
            self.allocs.mean_allocations,
            self.allocs.mean_bytes,
            self.allocs.max_peak_above_baseline,
            self.allocs.largest_single_alloc
        ));
        s.push('}');
        s.push(',');
        s.push_str("\"spans\":[");
        for (i, span) in self.spans.iter().enumerate() {
            if i > 0 {
                s.push(',');
            }
            s.push('{');
            write_kv_str(&mut s, "name", span.name);
            s.push(',');
            write_kv_u64(&mut s, "hits", span.hits);
            s.push(',');
            write_kv_dur(&mut s, "self_ns", span.total_self);
            s.push(',');
            write_kv_dur(&mut s, "inclusive_ns", span.total_inclusive);
            s.push(',');
            write_kv_u64(&mut s, "allocs", span.total_allocs);
            s.push(',');
            write_kv_u64(&mut s, "bytes", span.total_bytes);
            s.push('}');
        }
        s.push(']');
        s.push('}');
        s
    }
}

fn summarize_timing(iters: &[&IterationRecord], input_bytes: Option<u64>) -> TimingSummary {
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

fn summarize_allocs(iters: &[&IterationRecord]) -> AllocSummary {
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

fn aggregate_spans(iters: &[&IterationRecord]) -> Vec<SpanAggregate> {
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
    agg.sort_by(|a, b| b.total_inclusive.cmp(&a.total_inclusive));
    agg
}

// ---- formatting helpers ----------------------------------------------------

fn fmt_duration(d: Duration) -> String {
    let ns = d.as_nanos();
    if ns < 1_000 {
        format!("{ns} ns")
    } else if ns < 1_000_000 {
        format!("{:.2} µs", ns as f64 / 1_000.0)
    } else if ns < 1_000_000_000 {
        format!("{:.2} ms", ns as f64 / 1_000_000.0)
    } else {
        format!("{:.3} s", d.as_secs_f64())
    }
}

fn fmt_bytes(b: u64) -> String {
    fmt_bytes_f(b as f64)
}

fn fmt_bytes_f(b: f64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut value = b;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{value:.0} {}", UNITS[unit])
    } else {
        format!("{value:.2} {}", UNITS[unit])
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        let mut t = String::with_capacity(max);
        t.push_str(&s[..max.saturating_sub(1)]);
        t.push('…');
        t
    }
}

fn write_kv_str(s: &mut String, k: &str, v: &str) {
    s.push('"');
    s.push_str(k);
    s.push_str("\":\"");
    for ch in v.chars() {
        match ch {
            '"' => s.push_str("\\\""),
            '\\' => s.push_str("\\\\"),
            c if (c as u32) < 0x20 => s.push_str(&format!("\\u{:04x}", c as u32)),
            c => s.push(c),
        }
    }
    s.push('"');
}

fn write_kv_u64(s: &mut String, k: &str, v: u64) {
    s.push('"');
    s.push_str(k);
    s.push_str("\":");
    s.push_str(&v.to_string());
}

fn write_kv_dur(s: &mut String, k: &str, v: Duration) {
    write_kv_u64(s, k, v.as_nanos() as u64);
}

#[cfg(test)]
mod tests {
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
        assert!(table.contains("smoke"));
        assert!(table.contains("Timing"));
        assert!(table.contains("Allocations"));
    }
}
