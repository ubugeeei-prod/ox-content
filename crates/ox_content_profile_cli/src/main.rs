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
//!
//! The `docs-*` subcommands profile the JS/TS documentation generator
//! (`ox_content_docs`) over a source directory instead of a Markdown file:
//!
//! ```text
//! cargo run --release -p ox_content_profile_cli -- docs-extract <DIR>
//! cargo run --release -p ox_content_profile_cli -- docs-render <DIR>
//! cargo run --release -p ox_content_profile_cli -- docs-pipeline <DIR>
//! ```
//!
//! These exercise the OXC parse + JSDoc parse + AST visit + normalize path
//! (`docs-extract`), the TypeDoc/pure-Markdown render path (`docs-render`,
//! extraction hoisted out of the measurement loop), or both end to end
//! (`docs-pipeline`).

mod args;
mod bridge;
mod docs;
mod markdown;
mod output;

use std::process::ExitCode;

use args::{Cli, Cmd};
use clap::Parser as _;
use ox_content_profiler::{scope, CountingAllocator};

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator::new();

fn run(cli: &Cli) -> std::io::Result<()> {
    CountingAllocator::enable();
    scope::enable();

    match &cli.cmd {
        Cmd::DocsExtract { dir } => docs::run(cli, dir, docs::DocsPhase::Extract),
        Cmd::DocsRender { dir } => docs::run(cli, dir, docs::DocsPhase::Render),
        Cmd::DocsPipeline { dir } => docs::run(cli, dir, docs::DocsPhase::Pipeline),
        Cmd::DocsEntrypoints { entries } => docs::run_entrypoints(cli, entries),
        Cmd::Parse { .. } | Cmd::Render { .. } | Cmd::Pipeline { .. } => markdown::run(cli),
    }
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
