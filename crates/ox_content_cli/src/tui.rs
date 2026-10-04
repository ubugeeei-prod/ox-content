mod render;
mod screen;
mod session;
mod style;
mod terminal;
#[cfg(test)]
mod tests;

use crate::{Result, files};
use clap::Parser;
use std::{
    io::{IsTerminal, Write},
    path::Path,
};

#[derive(Parser)]
#[command(name = "oxct tui", disable_version_flag = true)]
struct Options {
    path: Option<String>,
    #[arg(long, default_value = "nord", value_parser = clap::builder::PossibleValuesParser::new(style::THEMES))]
    theme: String,
    #[arg(long)]
    print: bool,
    #[arg(long, conflicts_with = "path")]
    stdin: bool,
    #[arg(long, value_parser = clap::value_parser!(u16).range(20..=240))]
    width: Option<u16>,
    #[arg(long)]
    no_watch: bool,
    #[arg(long)]
    no_color: bool,
}

pub fn run(args: &[String]) -> Result<i32> {
    let options = Options::try_parse_from(
        std::iter::once("oxct tui".to_string()).chain(args.iter().cloned()),
    )?;
    let columns = usize::from(
        options
            .width
            .unwrap_or_else(|| crossterm::terminal::size().map_or(80, |(columns, _)| columns)),
    )
    .clamp(20, 240);
    let color = !options.no_color
        && std::io::stdout().is_terminal()
        && std::env::var_os("NO_COLOR").is_none();
    if options.stdin {
        print(&files::read_bounded(std::io::stdin())?, columns, &options.theme, color)?;
        return Ok(0);
    }
    let target = options.path.as_deref().unwrap_or(".");
    let target_path = files::absolute(Path::new(target))?;
    let paths = if target_path.is_file() {
        vec![target_path]
    } else {
        files::discover(&[target.to_string()], &[])?
    };
    if paths.is_empty() {
        return Err("No Markdown files matched the viewer input".into());
    }
    if paths.len() > 5000 {
        return Err("Select a narrower directory or glob (viewer limit: 5000 files)".into());
    }
    if options.print || !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
        print(&files::read_document(&paths[0])?, columns, &options.theme, color)?;
    } else {
        session::run(paths, options.theme, color, !options.no_watch)?;
    }
    Ok(0)
}

fn print(source: &str, columns: usize, theme: &str, color: bool) -> Result<()> {
    let document = render::render(source, columns);
    let mut stdout = std::io::stdout().lock();
    for line in document.lines {
        writeln!(stdout, "{}", style::paint(&line, theme, color, false))?;
    }
    Ok(())
}
