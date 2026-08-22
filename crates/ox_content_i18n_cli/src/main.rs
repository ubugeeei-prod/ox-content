use clap::{Parser, Subcommand, ValueEnum};
use ox_content_i18n_checker::diagnostic::{OutputFormat, format_diagnostics};

#[derive(Parser)]
#[command(name = "ox-content-i18n", about = "i18n tools for Ox Content")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run i18n checks on the project.
    Check {
        /// Path to the i18n dictionary directory.
        #[arg(long, default_value = "content/i18n")]
        dict_dir: String,

        /// Source directories to scan (can be specified multiple times).
        #[arg(long, default_value = "src")]
        src: Vec<String>,

        /// Output format.
        #[arg(long, value_enum, default_value_t = Format::Text)]
        format: Format,

        /// Default locale.
        #[arg(long, default_value = "en")]
        default_locale: String,
    },
    /// Validate an MF2 message string.
    Validate {
        /// The MF2 message to validate.
        message: String,

        /// Also print the AST as JSON.
        #[arg(long)]
        ast: bool,
    },
}

#[derive(Clone, ValueEnum)]
enum Format {
    Text,
    Json,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check { dict_dir, src, format, default_locale } => {
            let config = ox_content_i18n_checker::CheckConfig {
                dict_dir,
                src_dirs: src,
                default_locale: Some(default_locale),
                ..Default::default()
            };

            match ox_content_i18n_checker::check(&config) {
                Ok(result) => {
                    let output_format = match format {
                        Format::Text => OutputFormat::Text,
                        Format::Json => OutputFormat::Json,
                    };
                    let output = format_diagnostics(&result.diagnostics, output_format);
                    if !output.is_empty() {
                        #[allow(clippy::print_stdout)]
                        {
                            println!("{output}");
                        }
                    }

                    #[allow(clippy::print_stdout)]
                    {
                        println!(
                            "\n{} error(s), {} warning(s), {} key(s) used",
                            result.error_count,
                            result.warning_count,
                            result.used_keys.len()
                        );
                    }

                    if result.error_count > 0 {
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    #[allow(clippy::print_stderr)]
                    {
                        eprintln!("Error: {e}");
                    }
                    std::process::exit(1);
                }
            }
        }
        Commands::Validate { message, ast } => {
            match ox_content_i18n::mf2::parse_and_validate(&message) {
                Ok((parsed_ast, errors)) => {
                    if errors.is_empty() {
                        #[allow(clippy::print_stdout)]
                        {
                            println!("Valid MF2 message.");
                        }
                    } else {
                        #[allow(clippy::print_stdout)]
                        {
                            println!("Validation warnings:");
                        }
                        for err in &errors {
                            #[allow(clippy::print_stdout)]
                            {
                                println!("  - {err}");
                            }
                        }
                    }
                    if ast && let Ok(json) = serde_json::to_string_pretty(&parsed_ast) {
                        #[allow(clippy::print_stdout)]
                        {
                            println!("\nAST:\n{json}");
                        }
                    }
                }
                Err(e) => {
                    #[allow(clippy::print_stderr)]
                    {
                        eprintln!("Parse error: {e}");
                    }
                    std::process::exit(1);
                }
            }
        }
    }
}
