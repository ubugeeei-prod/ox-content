//! Timing engine shared by every competitor row: warmup, timed loops, and
//! per-run median selection, in the exact field units the JS harness reports.

use std::time::Instant;

/// Untimed calls before each timed loop, matching the JS harness.
const WARMUP_CALLS: u32 = 5;

/// One timed measurement, in the exact field units the JS harness reports.
#[derive(Clone, Copy)]
pub struct Measurement {
    pub ops_per_sec: f64,
    pub avg_ms: f64,
    pub throughput_mbs: f64,
}

/// One output row: the median measurement plus every per-run sample.
pub struct Row {
    pub name: &'static str,
    pub median: Measurement,
    pub samples: Vec<Measurement>,
}

pub fn bench(
    name: &'static str,
    mut op: impl FnMut(),
    iterations: u32,
    runs: u32,
    input_bytes: usize,
) -> Row {
    let samples: Vec<Measurement> =
        (0..runs).map(|_| measure_once(&mut op, iterations, input_bytes)).collect();
    Row { name, median: median_by_ops(&samples), samples }
}

fn measure_once(op: &mut dyn FnMut(), iterations: u32, input_bytes: usize) -> Measurement {
    for _ in 0..WARMUP_CALLS {
        op();
    }
    let start = Instant::now();
    for _ in 0..iterations {
        op();
    }
    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
    let avg_ms = elapsed_ms / f64::from(iterations);
    let ops_per_sec = 1000.0 / avg_ms;
    Measurement {
        ops_per_sec,
        avg_ms,
        throughput_mbs: (input_bytes as f64 / 1024.0 / 1024.0) * ops_per_sec,
    }
}

/// Median by ops/sec with the JS harness' index choice:
/// `sorted[Math.floor(sorted.length / 2)]`, the upper middle for even counts.
fn median_by_ops(samples: &[Measurement]) -> Measurement {
    let mut sorted: Vec<Measurement> = samples.to_vec();
    sorted.sort_by(|a, b| a.ops_per_sec.total_cmp(&b.ops_per_sec));
    sorted[sorted.len() / 2]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn median_matches_js_harness_selection() {
        let measurement =
            |ops: f64| Measurement { ops_per_sec: ops, avg_ms: 0.0, throughput_mbs: 0.0 };
        // Even count: sorted ops are [1, 2, 3, 4]; Math.floor(4 / 2) = index 2.
        let even = [measurement(4.0), measurement(1.0), measurement(3.0), measurement(2.0)];
        assert_eq!(median_by_ops(&even).ops_per_sec, 3.0);
        // Odd count: sorted ops are [1, 2, 3]; Math.floor(3 / 2) = index 1.
        let odd = [measurement(3.0), measurement(1.0), measurement(2.0)];
        assert_eq!(median_by_ops(&odd).ops_per_sec, 2.0);
    }
}
