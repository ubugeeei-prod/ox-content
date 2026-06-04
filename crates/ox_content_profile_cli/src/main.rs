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

use std::fmt::Write as _;
use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser as ClapParser, Subcommand};
use ox_content_allocator::Allocator;
use ox_content_docs::{
    collect_source_files, extract_docs_from_directories, extract_docs_from_entry_points,
    generate_markdown, ApiDocEntry, ApiDocMember, ApiDocModule, ApiDocTag, ApiParamDoc,
    ApiReturnDoc, ApiTypeParamDoc, EntryPointDocsOptions, EntryPointSpec, ExtractedDocModule,
    GraphOptions, MarkdownDisplayFormat, MarkdownDocsOptions, MarkdownPathStrategy,
    MarkdownRenderStyle, NormalizedDocEntry, NormalizedMember, NormalizedParamDoc,
    NormalizedReturnDoc, NormalizedTypeParam,
};
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
    /// Profile JS/TS docs extraction over a directory tree: OXC parse + JSDoc
    /// parse + AST visit + normalize, for every matched source file.
    DocsExtract { dir: PathBuf },
    /// Profile the docs Markdown render path. Extraction runs once outside the
    /// measurement loop so the timing reflects rendering in isolation.
    DocsRender { dir: PathBuf },
    /// Profile the full docs pipeline: extraction + normalize + Markdown render.
    DocsPipeline { dir: PathBuf },
    /// Profile the entry-point docs path (`extractDocsFromEntryPoints`): builds
    /// the export graph and extracts normalized docs for the given entry files.
    /// This is the path published packages (e.g. gunshi) use, where the export
    /// graph and doc extraction each parse every reachable module.
    DocsEntrypoints { entries: Vec<PathBuf> },
}

/// Which slice of the docs pipeline a `docs-*` run measures.
#[derive(Clone, Copy)]
enum DocsPhase {
    Extract,
    Render,
    Pipeline,
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

fn run(cli: &Cli) -> std::io::Result<()> {
    CountingAllocator::enable();
    scope::enable();

    match &cli.cmd {
        Cmd::DocsExtract { dir } => run_docs(cli, dir, DocsPhase::Extract),
        Cmd::DocsRender { dir } => run_docs(cli, dir, DocsPhase::Render),
        Cmd::DocsPipeline { dir } => run_docs(cli, dir, DocsPhase::Pipeline),
        Cmd::DocsEntrypoints { entries } => run_docs_entrypoints(cli, entries),
        Cmd::Parse { .. } | Cmd::Render { .. } | Cmd::Pipeline { .. } => run_markdown(cli),
    }
}

/// Profile the Markdown parse/render/pipeline path against a single input file
/// (or the embedded corpus when none is given).
fn run_markdown(cli: &Cli) -> std::io::Result<()> {
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

/// Markdown render options used by the `docs-render` / `docs-pipeline` drivers.
///
/// Mirrors the modern VitePress-style production path that recent work targets:
/// the TypeDoc page layout with pure-Markdown output (rather than the legacy
/// flat/HTML defaults).
fn docs_markdown_options() -> MarkdownDocsOptions {
    // Mirror a real consumer (gunshi): TypeDoc page layout, pure-Markdown output,
    // and `table` display formats for indexes / params / members, so the driver
    // exercises the table render path published packages actually use.
    MarkdownDocsOptions {
        path_strategy: MarkdownPathStrategy::TypeDoc,
        render_style: MarkdownRenderStyle::Markdown,
        index_format: MarkdownDisplayFormat::Table,
        parameters_format: MarkdownDisplayFormat::Table,
        interface_properties_format: MarkdownDisplayFormat::Table,
        class_properties_format: MarkdownDisplayFormat::Table,
        type_alias_properties_format: MarkdownDisplayFormat::Table,
        enum_members_format: MarkdownDisplayFormat::Table,
        property_members_format: MarkdownDisplayFormat::Table,
        ..MarkdownDocsOptions::default()
    }
}

/// Glob filters matching the source files a typical docs build ingests.
fn docs_filters() -> (Vec<String>, Vec<String>) {
    let include = ["**/*.ts", "**/*.tsx", "**/*.mts", "**/*.cts"]
        .iter()
        .map(|pattern| (*pattern).to_string())
        .collect();
    let exclude = ["**/*.d.ts", "**/*.test.*", "**/*.spec.*", "node_modules"]
        .iter()
        .map(|pattern| (*pattern).to_string())
        .collect();
    (include, exclude)
}

/// Profile the JS/TS docs generator over `dir`.
fn run_docs(cli: &Cli, dir: &Path, phase: DocsPhase) -> std::io::Result<()> {
    if !dir.is_dir() {
        let mut message = String::from("docs profiling target is not a directory: ");
        push_fmt(&mut message, format_args!("{}", dir.display()));
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, message));
    }

    let dir_str = dir.display().to_string();
    let (include, exclude) = docs_filters();
    let src_dirs = [dir_str.clone()];

    // Total source bytes processed, so the report's MB/s reflects throughput
    // over the real input rather than a single synthetic buffer.
    let files = collect_source_files(&dir_str, &include, &exclude);
    let file_count = files.len();
    let bytes: u64 =
        files.iter().filter_map(|file| std::fs::metadata(file).ok()).map(|meta| meta.len()).sum();

    let total_iters = cli.warmup + cli.iters;
    let config = ReportConfig { input_bytes: Some(bytes), warmup: cli.warmup, max_span_rows: 32 };

    let label_prefix = match phase {
        DocsPhase::Extract => "docs-extract",
        DocsPhase::Render => "docs-render",
        DocsPhase::Pipeline => "docs-pipeline",
    };
    let mut label = String::new();
    push_fmt(
        &mut label,
        format_args!("{label_prefix} ({dir_str}, {file_count} files, {bytes} bytes)"),
    );
    let mut recorder = Recorder::new(label).with_config(config);

    match phase {
        DocsPhase::Extract => {
            for _ in 0..total_iters {
                recorder.record(|| -> std::io::Result<()> {
                    let modules = extract_docs(&src_dirs, &include, &exclude)?;
                    black_box(modules);
                    Ok(())
                })?;
            }
        }
        DocsPhase::Render => {
            // Extraction is hoisted out of the loop so the timing reflects the
            // Markdown render path in isolation.
            let api_modules = extract_api_modules(&src_dirs, &include, &exclude)?;
            let options = docs_markdown_options();
            for _ in 0..total_iters {
                recorder.record(|| {
                    let pages = generate_markdown(&api_modules, &options);
                    black_box(pages);
                });
            }
        }
        DocsPhase::Pipeline => {
            let options = docs_markdown_options();
            for _ in 0..total_iters {
                recorder.record(|| -> std::io::Result<()> {
                    let api_modules = extract_api_modules(&src_dirs, &include, &exclude)?;
                    let pages = generate_markdown(&api_modules, &options);
                    black_box(pages);
                    Ok(())
                })?;
            }
        }
    }

    emit_report(recorder, cli);
    Ok(())
}

/// Profile `extract_docs_from_entry_points` over the given entry files — the
/// published-package path. Today it parses every reachable module twice (once
/// to build the export graph, once to extract docs), visible as the
/// `docs::graph_oxc_parse` and `docs::oxc_parse` spans.
fn run_docs_entrypoints(cli: &Cli, entries: &[PathBuf]) -> std::io::Result<()> {
    if entries.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "docs-entrypoints needs at least one entry file",
        ));
    }

    // Canonicalize entry paths so resolution doesn't depend on the graph root.
    let mut specs = Vec::with_capacity(entries.len());
    let mut label = String::from("docs-entrypoints (");
    for (index, entry) in entries.iter().enumerate() {
        let path = std::fs::canonicalize(entry)?;
        if index > 0 {
            label.push_str(", ");
        }
        push_fmt(&mut label, format_args!("{}", entry.display()));
        specs.push(EntryPointSpec { path, name: None });
    }
    label.push(')');

    let options = EntryPointDocsOptions {
        graph: GraphOptions::default(),
        include_private: false,
        include_internal: false,
        type_parameters: true,
    };

    let total_iters = cli.warmup + cli.iters;
    let config = ReportConfig { input_bytes: None, warmup: cli.warmup, max_span_rows: 32 };
    let mut recorder = Recorder::new(label).with_config(config);

    for _ in 0..total_iters {
        recorder.record(|| -> std::io::Result<()> {
            let modules = extract_docs_from_entry_points(&specs, &options).map_err(docs_error)?;
            black_box(modules);
            Ok(())
        })?;
    }

    emit_report(recorder, cli);
    Ok(())
}

/// Run extraction + normalization over `src_dirs`, mapping the error into the
/// CLI's `io::Error` channel.
fn extract_docs(
    src_dirs: &[String],
    include: &[String],
    exclude: &[String],
) -> std::io::Result<Vec<ExtractedDocModule>> {
    extract_docs_from_directories(src_dirs, include, exclude, false, false, true)
        .map_err(docs_error)
}

/// Extract and convert into the render IR consumed by `generate_markdown`.
fn extract_api_modules(
    src_dirs: &[String],
    include: &[String],
    exclude: &[String],
) -> std::io::Result<Vec<ApiDocModule>> {
    Ok(extract_docs(src_dirs, include, exclude)?.into_iter().map(to_api_module).collect())
}

/// Finish a recording window and print the report.
///
/// Instrumentation is disabled before stringifying so the report's own
/// allocations don't pollute the final picture.
fn emit_report(recorder: Recorder, cli: &Cli) {
    let report = recorder.finish();
    CountingAllocator::disable();
    scope::disable();
    if cli.json {
        println!("{}", report.render_json());
    } else {
        println!("{}", report.render_table());
    }
}

fn docs_error(err: impl std::fmt::Display) -> std::io::Error {
    let mut message = String::from("failed to extract docs for profiling: ");
    push_fmt(&mut message, format_args!("{err}"));
    std::io::Error::new(std::io::ErrorKind::InvalidData, message)
}

// Bridge the normalized extraction output into the `ApiDocModule` render IR.
// `generate_markdown` consumes the IR that the NAPI layer reconstructs in JS
// between the `extractDocsFrom*` and `generateDocsMarkdown` calls; this mirrors
// that conversion so the full pipeline can be profiled in-process. The mapping
// tracks `ox_content_napi`'s `convert_markdown_*` helpers.

fn to_api_module(module: ExtractedDocModule) -> ApiDocModule {
    ApiDocModule {
        file: module.file,
        description: String::new(),
        source_path: String::new(),
        examples: Vec::new(),
        tags: Vec::new(),
        entries: module.entries.into_iter().map(to_api_entry).collect(),
    }
}

fn to_api_entry(entry: NormalizedDocEntry) -> ApiDocEntry {
    ApiDocEntry {
        name: entry.name,
        kind: entry.kind.as_str().to_string(),
        description: entry.description,
        params: entry.params.into_iter().map(to_api_param).collect(),
        returns: entry.returns.map(to_api_return),
        examples: entry.examples,
        tags: entry.tags.into_iter().map(|(tag, value)| ApiDocTag { tag, value }).collect(),
        private: entry.private,
        file: entry.file,
        line: entry.line,
        end_line: entry.end_line,
        signature: entry.signature,
        extends: entry.extends,
        implements: entry.implements,
        has_body: entry.has_body,
        members: entry.members.into_iter().map(to_api_member).collect(),
        type_parameters: entry.type_parameters.into_iter().map(to_api_type_param).collect(),
    }
}

fn to_api_member(member: NormalizedMember) -> ApiDocMember {
    ApiDocMember {
        name: member.name,
        kind: member.kind.as_str().to_string(),
        description: member.description,
        signature: member.signature,
        type_annotation: member.type_annotation,
        params: member.params.into_iter().map(to_api_param).collect(),
        returns: member.returns.map(to_api_return),
        optional: member.optional,
        readonly: member.readonly,
        r#static: member.r#static,
        private: member.private,
        tags: member.tags.into_iter().map(|(tag, value)| ApiDocTag { tag, value }).collect(),
        implementation_of: Vec::new(),
        line: member.line,
        end_line: member.end_line,
    }
}

fn to_api_param(param: NormalizedParamDoc) -> ApiParamDoc {
    ApiParamDoc {
        name: param.name,
        type_annotation: param.type_annotation,
        description: param.description,
        optional: param.optional,
        default_value: param.default_value,
    }
}

fn to_api_return(return_doc: NormalizedReturnDoc) -> ApiReturnDoc {
    ApiReturnDoc {
        type_annotation: return_doc.type_annotation,
        description: return_doc.description,
        members: return_doc.members.into_iter().map(to_api_member).collect(),
    }
}

fn to_api_type_param(type_param: NormalizedTypeParam) -> ApiTypeParamDoc {
    ApiTypeParamDoc {
        name: type_param.name,
        constraint: type_param.constraint,
        default: type_param.default,
        description: type_param.description,
    }
}

fn push_fmt(output: &mut String, args: std::fmt::Arguments<'_>) {
    if output.write_fmt(args).is_err() {
        output.push_str("[formatting failed]");
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
