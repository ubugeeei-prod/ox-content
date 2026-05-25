# Profiling Mode

Ox Content ships a built-in profiler — `ox_content_profiler` — for chasing down
allocations and time in the Markdown engine. It is intentionally tuned for
"how much is this code actually doing?" rather than wall-clock benchmarking,
which is what `criterion` and the JS benchmark harness in `benchmarks/` cover.

Use the profiler when you want to answer questions like:

- How many allocations does parsing this file produce, and from which spans?
- Which block-level parser function dominates time for documents in our corpus?
- Did this change actually reduce allocations, or just trade them around?

## What it measures

There are three independent layers, all exposed through one CLI:

1. **Counting global allocator** (`ox_content_profiler::CountingAllocator`)
   wraps `std::alloc::System` and atomically records every allocation,
   deallocation, byte counter, peak live bytes, and a power-of-two
   size-class histogram. Installed as `#[global_allocator]` in the
   `ox-content-profile` binary, so the counts include _everything_ the
   process does during a measurement window.

2. **Hierarchical timing spans** (`ox_content_profiler::scope`) maintain a
   thread-local span stack with self / inclusive time aggregation, plus
   per-span allocation deltas. The parser and renderer crates each have a
   `profile` Cargo feature that swaps in real `profile_span!` guards at
   their hot block-level entry points (`parse_block`, `parse_html_block`,
   `visit_heading`, `write_escaped`, etc.). With the feature disabled
   (the default), `profile_span!` expands to a zero-sized binding the
   optimizer drops.

3. **Report formatting** (`ox_content_profiler::Report`) folds per-iteration
   records into percentile timings, allocation summaries, span breakdown,
   and a histogram. Renders as a monospace table or a single-line JSON
   document for CI consumption.

## CLI quick start

The CLI lives in `crates/ox_content_profile_cli` and builds the `profile`
features of both `ox_content_parser` and `ox_content_renderer`:

```bash
# Pipeline (parse + render) over the embedded corpus
cargo run --release -p ox_content_profile_cli -- pipeline

# Profile a specific file, GFM-enabled, with 200 measured iterations
cargo run --release -p ox_content_profile_cli -- \
    pipeline --gfm --iters 200 --warmup 20 \
    docs/content/api/types.md

# Parse only — useful for isolating parser work
cargo run --release -p ox_content_profile_cli -- parse path/to/file.md

# Render only — input is parsed once outside the measurement loop
cargo run --release -p ox_content_profile_cli -- render path/to/file.md

# Machine-readable output for diffing in CI
cargo run --release -p ox_content_profile_cli -- pipeline --json path/to/file.md
```

Always build `--release`: the macro-expanded `profile_span!` guards are
cheap, but in a debug build they dominate the actual work.

## Reading the report

```text
 Timing
   min   15.50 µs
   p50   35.83 µs
   p95   38.38 µs
   ...
   throughput     680.30 MB/s

 Allocations (per iteration)
   count               46.0
   bytes           57.01 KB
   peak (max)      46.25 KB
   largest         90.00 KB

 Spans (sorted by total inclusive time)
   name                          hits      self  inclusive  share   allocs  bytes
   parser::parse_html_block      7600   3.56 ms    3.56 ms  55.4%      0     0 B
   ...
```

- **Timing percentiles** are computed over `--iters` iterations after
  dropping the first `--warmup` to discard cold-cache effects.
- **Allocations per iteration** are the mean count + bytes over those
  iterations, plus the maximum peak live bytes any single iteration
  reached above its starting baseline.
- **Spans** are aggregated across all measured iterations. `self` is
  inclusive minus child-span inclusive time, so it's the time the function
  spends in its own body. `share` is the span's self time as a fraction of
  the total self time across all spans, which is a quick proxy for
  "fraction of CPU spent here."
- **Size-class histogram** shows the last iteration's allocations bucketed
  by power-of-two size. Useful for spotting spikes in small short-lived
  allocations.

## Profile-feature anatomy

The instrumentation hooks are gated on a Cargo feature per crate. To
profile a different consumer of the parser/renderer:

```toml
[dependencies]
ox_content_parser    = { workspace = true, features = ["profile"] }
ox_content_renderer  = { workspace = true, features = ["profile"] }
ox_content_profiler  = { workspace = true }
```

Inside your binary, install the global allocator and enable both layers
before the workload, then drain the report:

```rust
use ox_content_profiler::{CountingAllocator, Recorder, scope};

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator::new();

fn main() {
    CountingAllocator::enable();
    scope::enable();

    let mut recorder = Recorder::new("my-workload");
    for _ in 0..100 {
        recorder.record(|| {
            // ...exercise parser + renderer...
        });
    }
    let report = recorder.finish();
    println!("{}", report.render_table());
}
```

## Suggested workflow for performance work

1. Run the profiler against a representative corpus _before_ changing
   anything. Save the table or JSON.
2. Find the highest-`share` span that is not obviously memory-bandwidth
   bound. Look at its `allocs` and `bytes` columns — span-level
   allocations are usually low-hanging fruit.
3. Make a change that targets that span.
4. Re-run the profiler with the same flags and compare span counts,
   allocations, and tail percentiles.
5. Run `cargo bench -p ox_content_parser` to confirm the synthetic
   benchmarks haven't regressed.

This was the loop used to land [issue #159](https://github.com/ubugeeei/ox-content/issues/159):
the first run on `docs/content/api/types.md` showed `parse_html_block`
consuming 86.9% of pipeline time with `to_ascii_lowercase()` allocating
per line; replacing that with a byte-level case-insensitive search and
inlining `consume_line`'s newline scan moved the same file from
240 MB/s → 803 MB/s end-to-end while cutting per-iteration allocations
from 122 → 32.
