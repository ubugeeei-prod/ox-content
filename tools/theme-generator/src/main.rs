#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::literal_string_with_formatting_args,
    clippy::redundant_pub_crate,
    clippy::unused_self
)]

mod case;
mod color_math;
mod color_readme;
mod color_tokens;
mod colors;
mod css;
mod fs;
mod package;
mod skin_readme;
mod skins;
mod ts;

use std::env;
use std::error::Error;
use std::process::ExitCode;

pub(crate) type BoxError = Box<dyn Error + Send + Sync + 'static>;
pub(crate) type Result<T> = std::result::Result<T, BoxError>;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            report_error(error.as_ref());
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let command = Command::parse(env::args().skip(1))?;
    if matches!(command, Command::Help) {
        print_help();
        return Ok(());
    }

    let workspace = fs::Workspace::discover()?;

    match command {
        Command::All => {
            colors::generate(&workspace)?;
            skins::generate(&workspace)?;
        }
        Command::Colors => colors::generate(&workspace)?,
        Command::Skins => skins::generate(&workspace)?,
        Command::Help => unreachable!("handled before workspace discovery"),
    }

    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
enum Command {
    All,
    Colors,
    Skins,
    Help,
}

impl Command {
    fn parse(mut args: impl Iterator<Item = String>) -> Result<Self> {
        match args.next().as_deref() {
            None | Some("all") => Ok(Self::All),
            Some("colors") => Ok(Self::Colors),
            Some("skins") => Ok(Self::Skins),
            Some("-h" | "--help") => Ok(Self::Help),
            Some(value) => Err(format!("unknown theme generator command: {value}").into()),
        }
    }
}

fn print_help() {
    print_line("Usage: cargo run -p ox_content_theme_generator -- [all|colors|skins]");
}

pub(crate) fn print_line(message: &str) {
    #[allow(clippy::print_stdout)]
    {
        println!("{message}");
    }
}

fn report_error(error: &(dyn Error + 'static)) {
    #[allow(clippy::print_stderr)]
    {
        eprintln!("theme generator failed: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::{Command, Result};

    fn parse(args: &[&str]) -> Result<Command> {
        Command::parse(args.iter().map(|arg| (*arg).to_string()))
    }

    #[test]
    fn parses_help_without_selecting_generation_target() -> Result<()> {
        assert_eq!(parse(&["--help"])?, Command::Help);
        assert_eq!(parse(&["-h"])?, Command::Help);
        Ok(())
    }
}
