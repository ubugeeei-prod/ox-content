// This is a CLI binary — writing a report to stdout / errors to stderr is
// the entire point, so the workspace's `print_stdout` / `print_stderr`
// lints are inappropriate here.
#![allow(clippy::print_stdout, clippy::print_stderr)]

//! Profile-mode CLI for Ox Content.
//!
//! Drives the full parse + render pipeline against a Markdown input, with
//! the counting global allocator installed and span recording on. Prints a
//! report combining timing percentiles, per-iteration allocation counts,
//! and a per-span breakdown so you can see which phases of the engine are
//! actually doing the work.
//!
//! Default invocations:
//!
//! ```text
//! cargo run --release -p ox_content_profile_cli -- parse <FILE>
//! cargo run --release -p ox_content_profile_cli -- render <FILE>
//! cargo run --release -p ox_content_profile_cli -- pipeline <FILE>
//! ```
//!
//! Without a `<FILE>` the embedded corpus is used so the binary stays
//! useful in CI / first-time setup.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser as ClapParser, Subcommand};
use ox_content_allocator::Allocator;
use ox_content_parser::{Parser, ParserOptions};
use ox_content_profiler::{report::ReportConfig, scope, CountingAllocator, Recorder};
use ox_content_renderer::{HtmlRenderer, HtmlRendererOptions};

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator::new();

/// Built-in corpus reused when the user doesn't pass a file. Kept reasonably
/// large so the per-iteration timing isn't dominated by Instant overhead.
const EMBEDDED_CORPUS: &str = include_str!("../../../benchmarks/bundle-size/content/api.md");

#[derive(ClapParser, Debug)]
#[command(name = "ox-content-profile", about = "Profiling driver for Ox Content")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,

    /// Iterations to warm up before recording (defaults to 5).
    #[arg(long, global = true, default_value_t = 5)]
    warmup: usize,

    /// Iterations to record after warmup (defaults to 50).
    #[arg(long, global = true, default_value_t = 50)]
    iters: usize,

    /// Emit machine-readable JSON instead of the table view.
    #[arg(long, global = true)]
    json: bool,

    /// Use GFM-enabled parser options for the run.
    #[arg(long, global = true)]
    gfm: bool,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Profile the parser only (allocator + AST construction).
    Parse {
        /// Markdown file to read. If omitted, the embedded corpus is used.
        file: Option<PathBuf>,
    },
    /// Profile the renderer only. The input is parsed once outside the
    /// measurement loop so the timing reflects the HTML pass in isolation.
    Render { file: Option<PathBuf> },
    /// Profile the full pipeline: allocator + parse + render.
    Pipeline { file: Option<PathBuf> },
}

fn load_input(file: Option<&PathBuf>) -> std::io::Result<(String, String)> {
    match file {
        Some(p) => {
            let bytes = std::fs::read_to_string(p)?;
            Ok((p.display().to_string(), bytes))
        }
        None => Ok(("<embedded corpus>".to_string(), EMBEDDED_CORPUS.to_string())),
    }
}

fn parser_options(cli: &Cli) -> ParserOptions {
    if cli.gfm {
        ParserOptions::gfm()
    } else {
        ParserOptions::default()
    }
}

fn run(cli: &Cli) -> std::io::Result<()> {
    CountingAllocator::enable();
    scope::enable();

    let (label_input, source) = match &cli.cmd {
        Cmd::Parse { file } | Cmd::Render { file } | Cmd::Pipeline { file } => {
            load_input(file.as_ref())?
        }
    };
    let bytes = source.len() as u64;

    let total_iters = cli.warmup + cli.iters;
    let config = ReportConfig { input_bytes: Some(bytes), warmup: cli.warmup, max_span_rows: 24 };

    let label = match &cli.cmd {
        Cmd::Parse { .. } => format!("parse ({label_input}, {bytes} bytes)"),
        Cmd::Render { .. } => format!("render ({label_input}, {bytes} bytes)"),
        Cmd::Pipeline { .. } => format!("pipeline ({label_input}, {bytes} bytes)"),
    };

    let mut recorder = Recorder::new(label).with_config(config);

    match &cli.cmd {
        Cmd::Parse { .. } => {
            for _ in 0..total_iters {
                recorder.record(|| {
                    let alloc = Allocator::new();
                    let parser = Parser::with_options(&alloc, &source, parser_options(cli));
                    let _ = parser.parse();
                });
            }
        }
        Cmd::Render { .. } => {
            // Renderer needs a pre-parsed document. Allocate one for each
            // iteration to keep the AST distinct (rendering itself mutates
            // shared HtmlRenderer state across iterations otherwise).
            for _ in 0..total_iters {
                let alloc = Allocator::new();
                let parser = Parser::with_options(&alloc, &source, parser_options(cli));
                let doc = parser.parse().expect("parser failure during profile");
                recorder.record(|| {
                    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions::new());
                    let _ = renderer.render(&doc);
                });
            }
        }
        Cmd::Pipeline { .. } => {
            for _ in 0..total_iters {
                recorder.record(|| {
                    let alloc = Allocator::new();
                    let parser = Parser::with_options(&alloc, &source, parser_options(cli));
                    let doc = parser.parse().expect("parser failure during profile");
                    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions::new());
                    let _ = renderer.render(&doc);
                });
            }
        }
    }

    let report = recorder.finish();

    // We disable instrumentation before stringifying so the report's own
    // allocations don't pollute the final picture if a user re-uses
    // `AllocSnapshot::capture` after this.
    CountingAllocator::disable();
    scope::disable();

    if cli.json {
        println!("{}", report.render_json());
    } else {
        println!("{}", report.render_table());
    }
    Ok(())
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(&cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("ox-content-profile: {err}");
            ExitCode::FAILURE
        }
    }
}
