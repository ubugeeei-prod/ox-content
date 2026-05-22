use std::fs;
use std::path::PathBuf;

use clap::{Parser, ValueEnum};

#[derive(Parser)]
#[command(name = "ox-content-mdc-check", about = "Check MDC component syntax")]
struct Cli {
    /// Files to check.
    #[arg(required = true)]
    files: Vec<PathBuf>,

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
struct FileDiagnostics {
    file: String,
    diagnostics: Vec<ox_content_mdc_checker::Diagnostic>,
}

fn main() {
    let cli = Cli::parse();
    let mut results = Vec::new();
    let mut error_count = 0usize;

    for file in &cli.files {
        match fs::read_to_string(file) {
            Ok(source) => {
                let diagnostics = ox_content_mdc_checker::check(&source);
                error_count += diagnostics.len();
                results.push(FileDiagnostics {
                    file: file.to_string_lossy().into_owned(),
                    diagnostics,
                });
            }
            Err(error) => {
                error_count += 1;
                results.push(FileDiagnostics {
                    file: file.to_string_lossy().into_owned(),
                    diagnostics: vec![ox_content_mdc_checker::Diagnostic {
                        severity: ox_content_mdc_checker::Severity::Error,
                        message: error.to_string(),
                        line: 1,
                        column: 1,
                        end_line: 1,
                        end_column: 1,
                        component: None,
                    }],
                });
            }
        }
    }

    match cli.format {
        Format::Json => {
            #[allow(clippy::print_stdout)]
            {
                println!("{}", serde_json::to_string_pretty(&results).unwrap_or_default());
            }
        }
        Format::Text => print_text(&results),
    }

    if error_count > 0 {
        std::process::exit(1);
    }
}

fn print_text(results: &[FileDiagnostics]) {
    for result in results {
        for diagnostic in &result.diagnostics {
            #[allow(clippy::print_stdout)]
            {
                println!(
                    "{}:{}:{}: {}",
                    result.file, diagnostic.line, diagnostic.column, diagnostic.message
                );
            }
        }
    }
}
