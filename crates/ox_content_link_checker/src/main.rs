//! `ox-content-link-check` — CLI front-end for `ox_content_link_checker`.
//!
//! Exit codes:
//!
//! * `0` — every checked file was free of error-severity diagnostics.
//! * `1` — at least one error diagnostic was emitted, or a file failed
//!   to read.
//!
//! Warning-severity diagnostics (e.g. unverifiable cross-file anchors)
//! never fail the run on their own; they appear in the output and are
//! counted in the trailing summary so CI can keep the job green while
//! still surfacing the noise.

use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, ValueEnum};
use ox_content_link_checker::{check_source, CheckOptions, Diagnostic, Severity};

#[derive(Parser)]
#[command(
    name = "ox-content-link-check",
    about = "Check Markdown links and assets for missing local targets"
)]
struct Cli {
    /// Files to check. Pass one or more `.md` / `.mdc` paths.
    #[arg(required = true)]
    files: Vec<PathBuf>,

    /// Treat paths starting with `/` as relative to this directory.
    /// Defaults to each file's parent directory.
    #[arg(long)]
    src_dir: Option<PathBuf>,

    /// Substring patterns that suppress diagnostics whose target
    /// contains the pattern. Repeatable. Plain `contains` match — the
    /// linker is intentionally simple; layer regex/glob filtering in
    /// front when you need it.
    #[arg(long = "ignore", value_name = "PATTERN")]
    ignore: Vec<String>,

    /// Output format.
    #[arg(long, value_enum, default_value_t = Format::Text)]
    format: Format,
}

#[derive(Clone, Copy, ValueEnum)]
enum Format {
    Text,
    Json,
}

#[derive(serde::Serialize)]
struct FileReport {
    file: String,
    diagnostics: Vec<Diagnostic>,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let mut reports = Vec::with_capacity(cli.files.len());
    let mut error_count = 0usize;
    let mut io_error_count = 0usize;

    for file in &cli.files {
        let display = file.to_string_lossy().into_owned();
        let source = match fs::read_to_string(file) {
            Ok(text) => text,
            Err(error) => {
                io_error_count += 1;
                reports.push(FileReport {
                    file: display,
                    diagnostics: vec![Diagnostic {
                        severity: Severity::Error,
                        message: format!("Failed to read file: {error}"),
                        line: 1,
                        column: 1,
                        end_line: 1,
                        end_column: 1,
                        kind: ox_content_link_checker::LinkKind::Unknown,
                        target: String::new(),
                    }],
                });
                continue;
            }
        };

        let opts = CheckOptions {
            file_path: file.clone(),
            src_dir: cli.src_dir.clone(),
            ignore_patterns: cli.ignore.clone(),
        };
        let diagnostics = check_source(&source, &opts);
        error_count += diagnostics.iter().filter(|d| d.severity == Severity::Error).count();
        reports.push(FileReport { file: display, diagnostics });
    }

    match cli.format {
        Format::Json => emit_json(&reports),
        Format::Text => emit_text(&reports),
    }

    if error_count + io_error_count > 0 {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

#[allow(clippy::print_stdout)]
fn emit_json(reports: &[FileReport]) {
    let json = serde_json::to_string_pretty(reports).unwrap_or_else(|_| "[]".into());
    println!("{json}");
}

#[allow(clippy::print_stdout)]
fn emit_text(reports: &[FileReport]) {
    for report in reports {
        for diagnostic in &report.diagnostics {
            println!(
                "{}:{}:{}: {} {}",
                report.file,
                diagnostic.line,
                diagnostic.column,
                severity_tag(diagnostic.severity),
                diagnostic.message,
            );
        }
    }
}

fn severity_tag(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error:",
        Severity::Warning => "warning:",
    }
}
