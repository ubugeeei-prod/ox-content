use super::format::{
    fmt_bytes, fmt_bytes_f, fmt_duration, push_fmt, truncate, write_kv_dur, write_kv_str,
    write_kv_u64,
};
use super::Report;

impl Report {
    /// Render a human-readable, monospace-friendly table.
    #[must_use]
    pub fn render_table(&self) -> String {
        let mut out = String::with_capacity(4096);
        let ruler = "─".repeat(78);

        push_fmt(&mut out, format_args!("\n{ruler}\n"));
        push_fmt(&mut out, format_args!(" Profile: {}\n", self.label));
        push_fmt(
            &mut out,
            format_args!(
                " Iterations measured: {} (warmup: {})\n",
                self.timing.samples, self.config.warmup
            ),
        );
        push_fmt(&mut out, format_args!("{ruler}\n"));

        out.push_str("\n Timing\n");
        push_fmt(
            &mut out,
            format_args!(
                "   min   {}\n   p50   {}\n   p95   {}\n   p99   {}\n   max   {}\n   mean  {}\n",
                fmt_duration(self.timing.min),
                fmt_duration(self.timing.p50),
                fmt_duration(self.timing.p95),
                fmt_duration(self.timing.p99),
                fmt_duration(self.timing.max),
                fmt_duration(self.timing.mean),
            ),
        );
        if let Some(throughput) = self.timing.throughput_mb_s {
            push_fmt(&mut out, format_args!("   throughput   {throughput:>8.2} MB/s\n"));
        }

        out.push_str("\n Allocations (per iteration)\n");
        push_fmt(&mut out, format_args!(
            "   count       {:>12.1}\n   bytes       {:>12}\n   peak (max)  {:>12}\n   largest     {:>12}\n",
            self.allocs.mean_allocations,
            fmt_bytes_f(self.allocs.mean_bytes),
            fmt_bytes(self.allocs.max_peak_above_baseline),
            fmt_bytes(self.allocs.largest_single_alloc),
        ));

        if !self.spans.is_empty() {
            out.push_str("\n Spans (sorted by total inclusive time)\n");
            push_fmt(
                &mut out,
                format_args!(
                    "   {:<32} {:>8} {:>12} {:>12} {:>6}   {:>10} {:>10}\n",
                    "name", "hits", "self", "inclusive", "share", "allocs", "bytes",
                ),
            );
            push_fmt(&mut out, format_args!("   {}\n", "·".repeat(74)));
            for span in self.spans.iter().take(self.config.max_span_rows) {
                push_fmt(
                    &mut out,
                    format_args!(
                        "   {:<32} {:>8} {:>12} {:>12} {:>5.1}%   {:>10} {:>10}\n",
                        truncate(span.name, 32),
                        span.hits,
                        fmt_duration(span.total_self),
                        fmt_duration(span.total_inclusive),
                        span.share_of_total * 100.0,
                        span.total_allocs,
                        fmt_bytes(span.total_bytes),
                    ),
                );
            }
            if self.spans.len() > self.config.max_span_rows {
                push_fmt(
                    &mut out,
                    format_args!(
                        "   …and {} more spans\n",
                        self.spans.len() - self.config.max_span_rows
                    ),
                );
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
                    push_fmt(&mut out, format_args!("   {label:>10}  {count:>8}  {bar}\n"));
                }
            }
        }

        push_fmt(&mut out, format_args!("\n{ruler}\n"));
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
            push_fmt(&mut s, format_args!("\"throughput_mb_s\":{t}"));
        }
        s.push('}');
        s.push(',');
        s.push_str("\"allocs\":{");
        push_fmt(
            &mut s,
            format_args!(
                "\"mean_count\":{:.3},\"mean_bytes\":{:.3},\"max_peak\":{},\"largest\":{}",
                self.allocs.mean_allocations,
                self.allocs.mean_bytes,
                self.allocs.max_peak_above_baseline,
                self.allocs.largest_single_alloc
            ),
        );
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
