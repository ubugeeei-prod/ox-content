mod config;

use crate::{Result, files, prompts};
use clap::Parser;
use ox_content_markdown_lint::{MarkdownLintDiagnostic, lint_markdown_documents};
use serde_json::{Value, json};
use std::{
    io::{IsTerminal, Read, Write},
    path::Path,
    time::Instant,
};

#[derive(Parser)]
#[command(name = "oxct lint", disable_version_flag = true)]
struct Options {
    paths: Vec<String>,
    #[arg(long)]
    config: Option<String>,
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
    let config =
        options.config.as_deref().map(config::Config::read).transpose()?.unwrap_or_default();
    if options.paths.is_empty() {
        options.paths.clone_from(&config.include);
    }
    options.ignore.extend(config.ignore.iter().cloned());
    let paths = if options.stdin {
        vec![options.stdin_filepath.clone()]
    } else {
        files::discover(&options.paths, &options.ignore)?
            .iter()
            .map(|path| files::slash(path))
            .collect()
    };
    if paths.is_empty() {
        return Err("No Markdown files matched. Check the paths and ignore patterns.".into());
    }
    let mut stdin = String::new();
    if options.stdin {
        std::io::stdin().read_to_string(&mut stdin)?;
    }
    let cwd = std::env::current_dir()?;
    let mut diagnostics = Vec::new();
    let mut errors = 0;
    let mut warnings = 0;
    for batch in paths.chunks(128) {
        let sources: Vec<String> = batch
            .iter()
            .map(
                |path| {
                    if options.stdin { Ok(stdin.clone()) } else { std::fs::read_to_string(path) }
                },
            )
            .collect::<std::io::Result<_>>()?;
        for mdx in [false, true] {
            let indices: Vec<_> = batch
                .iter()
                .enumerate()
                .filter_map(|(index, path)| {
                    (Path::new(path).extension().is_some_and(|ext| ext.eq_ignore_ascii_case("mdx"))
                        == mdx)
                        .then_some(index)
                })
                .collect();
            if indices.is_empty() {
                continue;
            }
            let selected = indices.iter().map(|index| sources[*index].clone()).collect::<Vec<_>>();
            let results =
                lint_markdown_documents(&selected, Some(config.native(options.spellcheck, mdx)));
            for (index, result) in indices.into_iter().zip(results) {
                errors += result.error_count;
                warnings += result.warning_count;
                let path = Path::new(&batch[index]);
                let file = if options.stdin {
                    batch[index].clone()
                } else {
                    files::slash(&files::relative(path, &cwd))
                };
                for diagnostic in result.diagnostics {
                    diagnostics.push((file.clone(), diagnostic));
                }
            }
        }
    }
    diagnostics.sort_by(|(a_file, a), (b_file, b)| {
        (a_file, a.line, a.column, &a.rule_id).cmp(&(b_file, b.line, b.column, &b.rule_id))
    });
    let duration = (started.elapsed().as_secs_f64() * 100_000.0).round() / 100.0;
    if options.format == "json" {
        let report = json!({"checkedFileCount": paths.len(), "errorCount": errors, "warningCount": warnings,
            "diagnostics": diagnostics.iter().map(|(file, diagnostic)| diagnostic_json(file, diagnostic)).collect::<Vec<_>>(), "durationMs": duration});
        prompts::print(&serde_json::to_string_pretty(&report)?)?;
    } else {
        let color = !options.no_color
            && std::io::stdout().is_terminal()
            && std::env::var_os("NO_COLOR").is_none();
        let paint = |code, text: &str| {
            if color { format!("\x1b[{code}m{text}\x1b[0m") } else { text.to_string() }
        };
        let mut output = std::io::stdout().lock();
        writeln!(output, "{}", paint("1;36", "◆ Ox Content · Markdown lint"))?;
        for (file, diagnostic) in diagnostics {
            let severity = paint(
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
            "\n{} files · {errors} errors · {warnings} warnings · {duration}ms",
            paths.len()
        )?;
    }
    Ok(i32::from(errors > 0 || u64::from(warnings) > options.max_warnings))
}

fn diagnostic_json(file: &str, diagnostic: &MarkdownLintDiagnostic) -> Value {
    let mut value = json!({"file": file, "ruleId": diagnostic.rule_id, "severity": diagnostic.severity,
        "message": diagnostic.message, "line": diagnostic.line, "column": diagnostic.column,
        "endLine": diagnostic.end_line, "endColumn": diagnostic.end_column});
    if let Some(language) = &diagnostic.language {
        value["language"] = json!(language);
    }
    if let Some(suggestions) = &diagnostic.suggestions {
        value["suggestions"] = json!(suggestions);
    }
    value
}
