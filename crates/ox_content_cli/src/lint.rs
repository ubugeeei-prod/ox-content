mod config;
mod report;
mod worker;

use crate::{Result, files};
use clap::Parser;
use ox_content_markdown_lint::{MarkdownLinter, MarkdownlintConfig};
use rayon::prelude::*;
use std::{
    io::{BufWriter, IsTerminal, Read, Write},
    path::Path,
    time::Instant,
};

#[derive(Parser)]
#[command(name = "oxct lint", disable_version_flag = true)]
struct Options {
    paths: Vec<String>,
    #[arg(long)]
    config: Option<String>,
    /// Enable the native markdownlint profile alongside an Ox Content config.
    #[arg(long, conflicts_with = "no_markdownlint")]
    markdownlint: bool,
    #[arg(long)]
    no_markdownlint: bool,
    #[arg(long)]
    list_rules: bool,
    #[arg(long)]
    no_inline_config: bool,
    #[arg(long)]
    ignore: Vec<String>,
    #[arg(long, default_value = "text", value_parser = ["text", "json"])]
    format: String,
    #[arg(long, conflicts_with = "paths")]
    stdin: bool,
    #[arg(long, default_value = "stdin.md")]
    stdin_filepath: String,
    #[arg(long)]
    spellcheck: bool,
    #[arg(long)]
    strict: bool,
    #[arg(long, conflicts_with = "stdin")]
    fix: bool,
    #[arg(long, value_parser = clap::value_parser!(u16).range(1..=256))]
    threads: Option<u16>,
    #[arg(long, default_value_t = 0, value_parser = clap::value_parser!(u64).range(0..=9_007_199_254_740_991))]
    max_warnings: u64,
    #[arg(long)]
    no_color: bool,
}

pub fn run(args: &[String]) -> Result<i32> {
    let started = Instant::now();
    let mut options = Options::try_parse_from(
        std::iter::once("oxct lint".to_string()).chain(args.iter().cloned()),
    )?;
    if options.list_rules {
        return report::rules(&options.format);
    }
    let discovered = [
        ".oxlint.json",
        "oxlint.json",
        ".markdownlint.json",
        ".markdownlint.jsonc",
        ".markdownlint.yaml",
        ".markdownlint.yml",
    ]
    .into_iter()
    .find(|path| Path::new(path).is_file());
    let mut config = options
        .config
        .as_deref()
        .or(discovered)
        .map(config::Config::read)
        .transpose()?
        .unwrap_or_default();
    config.inline_disabled |= options.no_inline_config;
    if options.markdownlint {
        if config.markdownlint.as_ref().is_none_or(|value| value.0 == false) {
            config.markdownlint = Some(MarkdownlintConfig::default());
        }
    } else if options.no_markdownlint {
        config.markdownlint = None;
    }
    if options.paths.is_empty() {
        options.paths.clone_from(&config.include);
    }
    options.ignore.extend(config.ignore.iter().cloned());
    let paths = if options.stdin {
        vec![options.stdin_filepath.clone()]
    } else {
        files::discover_with_ignore_file(
            &options.paths,
            &options.ignore,
            Path::new(".markdownlintignore"),
        )?
        .iter()
        .map(|path| files::slash(path))
        .collect()
    };
    if paths.is_empty() {
        return Err("No Markdown files matched. Check the paths and ignore patterns.".into());
    }
    let mut stdin = String::new();
    if options.stdin {
        std::io::stdin().lock().read_to_string(&mut stdin)?;
    }
    let cwd = std::env::current_dir()?;
    let labels: Vec<_> = paths
        .iter()
        .map(|path| {
            if options.stdin {
                path.clone()
            } else {
                files::slash(&files::relative(Path::new(path), &cwd))
            }
        })
        .collect();
    let mut diagnostics = Vec::new();
    let mut errors = 0;
    let mut warnings = 0;
    let mut fixed_count = 0;
    let markdown =
        MarkdownLinter::new(Some(config.native(options.spellcheck, false, options.strict)));
    let mdx = paths
        .iter()
        .any(|path| Path::new(path).extension().is_some_and(|ext| ext.eq_ignore_ascii_case("mdx")))
        .then(|| {
            MarkdownLinter::new(Some(config.native(options.spellcheck, true, options.strict)))
        });
    let pool = if paths.len() > 1 && options.threads != Some(1) {
        let mut builder = rayon::ThreadPoolBuilder::new();
        if let Some(threads) = options.threads {
            builder = builder.num_threads(usize::from(threads));
        }
        Some(builder.build()?)
    } else {
        None
    };
    let check = |path: &String| {
        let linter =
            if Path::new(path).extension().is_some_and(|ext| ext.eq_ignore_ascii_case("mdx")) {
                mdx.as_ref().unwrap_or(&markdown)
            } else {
                &markdown
            };
        worker::check(path, options.stdin.then_some(stdin.as_str()), linter, options.fix)
            .map_err(|error| error.to_string())
    };
    let mut results = Vec::new();
    // Sources live only inside workers; keep diagnostic memory bounded by batches.
    for (batch_index, batch) in paths.chunks(128).enumerate() {
        if let Some(pool) = &pool {
            pool.install(|| batch.par_iter().map(check).collect_into_vec(&mut results));
        } else {
            results.extend(batch.iter().map(check));
        }
        #[allow(clippy::iter_with_drain, reason = "Reuse result capacity across worker batches.")]
        for (index, (path, checked)) in batch.iter().zip(results.drain(..)).enumerate() {
            let (result, fixed) = checked.map_err(|error| format!("{path}: {error}"))?;
            errors += result.error_count;
            warnings += result.warning_count;
            fixed_count += fixed;
            for diagnostic in result.diagnostics {
                diagnostics.push((batch_index * 128 + index, diagnostic));
            }
        }
    }
    diagnostics.sort_by(|(a_file, a), (b_file, b)| {
        (&labels[*a_file], a.line, a.column, &a.rule_id).cmp(&(
            &labels[*b_file],
            b.line,
            b.column,
            &b.rule_id,
        ))
    });
    let duration = (started.elapsed().as_secs_f64() * 100_000.0).round() / 100.0;
    if options.format == "json" {
        let report = report::Report {
            checked_file_count: paths.len(),
            error_count: errors,
            warning_count: warnings,
            fixed_count,
            duration_ms: duration,
            diagnostics: report::Diagnostics { entries: &diagnostics, labels: &labels },
        };
        let mut output = BufWriter::with_capacity(64 * 1024, std::io::stdout().lock());
        serde_json::to_writer_pretty(&mut output, &report)?;
        writeln!(output)?;
        output.flush()?;
    } else {
        let color = !options.no_color
            && std::io::stdout().is_terminal()
            && std::env::var_os("NO_COLOR").is_none();
        let mut output = BufWriter::with_capacity(64 * 1024, std::io::stdout().lock());
        writeln!(output, "{}", paint(color, "1;36", "◆ Ox Content · Markdown lint"))?;
        for (index, diagnostic) in diagnostics {
            let file = &labels[index];
            let severity = paint(
                color,
                if diagnostic.severity == "error" { "31" } else { "33" },
                &diagnostic.severity,
            );
            writeln!(
                output,
                "{file}:{}:{} {severity} {} ({})",
                diagnostic.line, diagnostic.column, diagnostic.message, diagnostic.rule_id
            )?;
        }
        writeln!(
            output,
            "\n{} files · {errors} errors · {warnings} warnings · {fixed_count} fixes · {duration}ms",
            paths.len()
        )?;
        output.flush()?;
    }
    Ok(i32::from(errors > 0 || u64::from(warnings) > options.max_warnings))
}

fn paint<'a>(color: bool, code: &str, text: &'a str) -> std::borrow::Cow<'a, str> {
    if color {
        std::borrow::Cow::Owned(format!("\x1b[{code}m{text}\x1b[0m"))
    } else {
        std::borrow::Cow::Borrowed(text)
    }
}
