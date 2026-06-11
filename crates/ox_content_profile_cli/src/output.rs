use std::fmt::Write as _;

use ox_content_profiler::{scope, CountingAllocator, Recorder};

use crate::args::Cli;

/// Finish a recording window and print the report.
///
/// Instrumentation is disabled before stringifying so the report's own
/// allocations don't pollute the final picture.
pub fn emit_report(recorder: Recorder, cli: &Cli) {
    let report = recorder.finish();
    CountingAllocator::disable();
    scope::disable();
    if cli.json {
        println!("{}", report.render_json());
    } else {
        println!("{}", report.render_table());
    }
}

pub fn push_fmt(output: &mut String, args: std::fmt::Arguments<'_>) {
    if output.write_fmt(args).is_err() {
        output.push_str("[formatting failed]");
    }
}
