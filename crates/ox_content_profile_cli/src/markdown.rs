use std::path::PathBuf;

use ox_content_allocator::Allocator;
use ox_content_parser::{Parser, ParserOptions};
use ox_content_profiler::{report::ReportConfig, Recorder};
use ox_content_renderer::{HtmlRenderer, HtmlRendererOptions};

use crate::args::{Cli, Cmd};
use crate::output::{emit_report, push_fmt};

/// Built-in corpus reused when the user doesn't pass a file. Kept reasonably
/// large so the per-iteration timing isn't dominated by Instant overhead.
const EMBEDDED_CORPUS: &str = include_str!("../../../benchmarks/bundle-size/content/api.md");

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

fn parse_error(err: impl std::fmt::Display) -> std::io::Error {
    let mut message = String::from("failed to parse Markdown input for profiling: ");
    push_fmt(&mut message, format_args!("{err}"));
    std::io::Error::new(std::io::ErrorKind::InvalidData, message)
}

/// Which slice of the Markdown engine a `parse`/`render`/`pipeline` run measures.
#[derive(Clone, Copy)]
enum MarkdownPhase {
    Parse,
    Render,
    Pipeline,
}

/// Profile the Markdown parse/render/pipeline path against a single input file
/// (or the embedded corpus when none is given).
pub fn run(cli: &Cli) -> std::io::Result<()> {
    let (phase, file) = match &cli.cmd {
        Cmd::Parse { file } => (MarkdownPhase::Parse, file.as_ref()),
        Cmd::Render { file } => (MarkdownPhase::Render, file.as_ref()),
        Cmd::Pipeline { file } => (MarkdownPhase::Pipeline, file.as_ref()),
        // docs-* commands are dispatched to run_docs in run().
        _ => unreachable!("non-Markdown command routed to run_markdown"),
    };
    let (label_input, source) = load_input(file)?;
    let bytes = source.len() as u64;

    let total_iters = cli.warmup + cli.iters;
    let config = ReportConfig { input_bytes: Some(bytes), warmup: cli.warmup, max_span_rows: 24 };

    let label_prefix = match phase {
        MarkdownPhase::Parse => "parse",
        MarkdownPhase::Render => "render",
        MarkdownPhase::Pipeline => "pipeline",
    };
    let mut label = String::new();
    push_fmt(&mut label, format_args!("{label_prefix} ({label_input}, {bytes} bytes)"));

    let mut recorder = Recorder::new(label).with_config(config);

    match phase {
        MarkdownPhase::Parse => {
            for _ in 0..total_iters {
                recorder.record(|| -> std::io::Result<()> {
                    let alloc = Allocator::for_source_len(source.len());
                    let parser = Parser::with_options(&alloc, &source, parser_options(cli));
                    let _ = parser.parse().map_err(parse_error)?;
                    Ok(())
                })?;
            }
        }
        MarkdownPhase::Render => {
            // Renderer needs a pre-parsed document. Allocate one for each
            // iteration to keep the AST distinct (rendering itself mutates
            // shared HtmlRenderer state across iterations otherwise).
            for _ in 0..total_iters {
                let alloc = Allocator::for_source_len(source.len());
                let parser = Parser::with_options(&alloc, &source, parser_options(cli));
                let doc = parser.parse().map_err(parse_error)?;
                recorder.record(|| {
                    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions::new());
                    let _ = renderer.render(&doc);
                });
            }
        }
        MarkdownPhase::Pipeline => {
            for _ in 0..total_iters {
                recorder.record(|| -> std::io::Result<()> {
                    let alloc = Allocator::for_source_len(source.len());
                    let parser = Parser::with_options(&alloc, &source, parser_options(cli));
                    let doc = parser.parse().map_err(parse_error)?;
                    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions::new());
                    let _ = renderer.render(&doc);
                    Ok(())
                })?;
            }
        }
    }

    emit_report(recorder, cli);
    Ok(())
}
