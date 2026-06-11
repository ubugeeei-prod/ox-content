//! Report formatting.
//!
//! `Report::from_iterations` takes the per-iteration records captured by
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

mod aggregate;
mod format;
mod render;

#[cfg(test)]
mod tests;

use aggregate::{aggregate_spans, summarize_allocs, summarize_timing};

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
}
