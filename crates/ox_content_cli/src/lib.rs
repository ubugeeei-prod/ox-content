//! Native project setup, IDE configuration, Markdown lint and terminal reading.
//! Both the npm launcher (through NAPI) and the Rust binary use this implementation.

#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

mod config_edit;
mod files;
mod ide;
mod lint;
mod new_project;
mod prompts;
mod templates;
mod tui;

use std::path::Path;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Execute a native authoring command, returning its process exit code.
/// `editor_assets` locates the bundled Neovim plugin in npm installations.
pub fn run(args: &[String], editor_assets: &Path) -> Result<i32> {
    let Some((command, rest)) = args.split_first() else {
        prompts::print(
            "oxct <command> [options]\n\nCommands: new [directory], ide install, lint [files/globs], tui [file/directory]",
        )?;
        return Ok(0);
    };
    if ["--help", "-h"].contains(&command.as_str()) {
        return run(&[], editor_assets);
    }
    if command == "--version" {
        prompts::print(env!("CARGO_PKG_VERSION"))?;
        return Ok(0);
    }
    let result = match command.as_str() {
        "new" => new_project::run(rest),
        "ide" => ide::run(rest, editor_assets),
        "lint" => lint::run(rest),
        "tui" => tui::run(rest),
        _ => Err(format!("Unknown native command: {command}").into()),
    };
    match result {
        Err(error)
            if error.downcast_ref::<clap::Error>().is_some_and(|error| {
                matches!(
                    error.kind(),
                    clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
                )
            }) =>
        {
            prompts::print(&error.to_string())?;
            Ok(0)
        }
        result => result,
    }
}
