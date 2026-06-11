use std::hint::black_box;
use std::path::{Path, PathBuf};

use ox_content_docs::{
    collect_source_files, extract_docs_from_directories, extract_docs_from_entry_points,
    generate_markdown, ApiDocModule, EntryPointDocsOptions, EntryPointSpec, ExtractedDocModule,
    GraphOptions, MarkdownDisplayFormat, MarkdownDocsOptions, MarkdownPathStrategy,
    MarkdownRenderStyle,
};
use ox_content_profiler::{report::ReportConfig, Recorder};

use crate::args::Cli;
use crate::bridge::to_api_module;
use crate::output::{emit_report, push_fmt};

/// Which slice of the docs pipeline a docs-* run measures.
#[derive(Clone, Copy)]
pub enum DocsPhase {
    Extract,
    Render,
    Pipeline,
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
pub fn run(cli: &Cli, dir: &Path, phase: DocsPhase) -> std::io::Result<()> {
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
pub fn run_entrypoints(cli: &Cli, entries: &[PathBuf]) -> std::io::Result<()> {
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

fn docs_error(err: impl std::fmt::Display) -> std::io::Error {
    let mut message = String::from("failed to extract docs for profiling: ");
    push_fmt(&mut message, format_args!("{err}"));
    std::io::Error::new(std::io::ErrorKind::InvalidData, message)
}
