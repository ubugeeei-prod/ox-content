//! JSON serialization of the suite results into the exact row shape the JS
//! benchmark tables consume: `{"parse": {<size>: [row, ...]}, "render": ...}`.

use std::fmt::Write as _;

use crate::bench::{Measurement, Row};

/// Per-suite results as `(size name, rows)` in harness size order.
pub struct SuiteResults {
    pub parse: Vec<(&'static str, Vec<Row>)>,
    pub render: Vec<(&'static str, Vec<Row>)>,
}

pub fn render_json(results: &SuiteResults) -> String {
    let mut out = String::new();
    out.push('{');
    for (suite_index, (suite_name, sizes)) in
        [("parse", &results.parse), ("render", &results.render)].into_iter().enumerate()
    {
        if suite_index > 0 {
            out.push(',');
        }
        let _ = write!(out, "\"{suite_name}\":{{");
        for (size_index, (size_name, rows)) in sizes.iter().enumerate() {
            if size_index > 0 {
                out.push(',');
            }
            let _ = write!(out, "\"{size_name}\":[");
            for (row_index, row) in rows.iter().enumerate() {
                if row_index > 0 {
                    out.push(',');
                }
                push_json_row(&mut out, row);
            }
            out.push(']');
        }
        out.push('}');
    }
    out.push('}');
    out
}

fn push_json_row(out: &mut String, row: &Row) {
    // Row names are fixed ASCII constants without `"` or `\`, so they embed
    // into JSON without an escaping pass.
    debug_assert!(!row.name.contains(['"', '\\']));
    out.push_str("{\"name\":\"");
    out.push_str(row.name);
    out.push_str("\",");
    push_json_measurement_fields(out, &row.median);
    out.push_str(",\"samples\":[");
    for (index, sample) in row.samples.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push('{');
        push_json_measurement_fields(out, sample);
        out.push('}');
    }
    out.push_str("]}");
}

fn push_json_measurement_fields(out: &mut String, measurement: &Measurement) {
    out.push_str("\"opsPerSec\":");
    push_json_number(out, measurement.ops_per_sec);
    out.push_str(",\"avgMs\":");
    push_json_number(out, measurement.avg_ms);
    out.push_str(",\"throughputMBs\":");
    push_json_number(out, measurement.throughput_mbs);
}

fn push_json_number(out: &mut String, value: f64) {
    // Rust's shortest-roundtrip float formatting is a valid JSON number for
    // finite values (never scientific notation, no NaN/inf spellings). The
    // non-finite arm is unreachable for real measurements but keeps the
    // output parseable no matter what.
    if value.is_finite() {
        let _ = write!(out, "{value}");
    } else {
        out.push('0');
    }
}
