mod config;
use crate::{Result, config_edit, prompts};
use clap::Parser;
use std::{path::Path, process::Command};

const IDE_NAMES: &[&str] = &["vscode", "cursor", "windsurf", "vscodium", "zed", "neovim"];

pub fn command(ide: &str) -> Option<&'static str> {
    match ide {
        "vscode" => Some("code"),
        "cursor" => Some("cursor"),
        "windsurf" => Some("windsurf"),
        "vscodium" => Some("codium"),
        _ => None,
    }
}

#[derive(Parser)]
#[command(name = "oxct ide install", disable_version_flag = true)]
struct Options {
    #[arg(long, value_parser = clap::builder::PossibleValuesParser::new(IDE_NAMES))]
    ide: Vec<String>,
    #[arg(long, conflicts_with = "extensions_only")]
    config_only: bool,
    #[arg(long, conflicts_with = "config_only")]
    extensions_only: bool,
    #[arg(long)]
    dry_run: bool,
    #[arg(long)]
    yes: bool,
}

pub fn run(args: &[String], assets: &Path) -> Result<i32> {
    if args.first().map(String::as_str) != Some("install") {
        return Err("Use oxct ide install".into());
    }
    let mut options = Options::try_parse_from(
        std::iter::once("oxct ide install".to_string()).chain(args[1..].iter().cloned()),
    )?;
    let interactive = prompts::interactive(options.yes);
    let mut workspace = !options.extensions_only;
    let mut extensions = !options.config_only;
    if interactive {
        prompts::print("◆ Ox Content · IDE setup")?;
        if options.ide.is_empty() {
            options.ide = prompts::multi("Choose your IDEs", IDE_NAMES, &[false; 6])?;
        }
        if !options.config_only && !options.extensions_only {
            let actions = prompts::multi(
                "What should be configured?",
                &["extensions", "config"],
                &[true; 2],
            )?;
            workspace = actions.iter().any(|value| value == "config");
            extensions = actions.iter().any(|value| value == "extensions");
        }
    }
    let mut ides = Vec::new();
    for ide in options.ide {
        if !ides.contains(&ide) {
            ides.push(ide);
        }
    }
    if ides.is_empty() {
        return Err("Choose an IDE interactively or pass --ide <name>.".into());
    }
    let plans = config::plans(&ides, workspace, extensions)?;
    let commands: Vec<_> =
        if extensions { ides.iter().filter_map(|ide| command(ide)).collect() } else { Vec::new() };
    let nvim = ides.iter().any(|ide| ide == "neovim");
    let nvim_target = if nvim && extensions {
        Some(
            config::home("XDG_DATA_HOME", ".local/share")?
                .join("nvim/site/pack/ox-content/start/ox-content"),
        )
    } else {
        None
    };
    if workspace {
        prompts::print("Enable frontmatter validation from trusted project Vite configuration.")?;
    }
    for plan in &plans {
        let action = if plan.original.as_deref() == Some(&plan.content) {
            "Keep"
        } else if plan.original.is_some() {
            "Merge and back up"
        } else {
            "Create"
        };
        prompts::print(&format!("{action}: {}", plan.file.display()))?;
    }
    for command in &commands {
        prompts::print(&format!("{command} --install-extension ubugeeei.vscode-ox-content"))?;
    }
    if let Some(target) = &nvim_target {
        prompts::print(&format!("Install bundled Neovim plugin: {}", target.display()))?;
    }
    if options.dry_run {
        return Ok(0);
    }
    if interactive && !prompts::confirm("Apply this setup?")? {
        return Ok(0);
    }
    for plan in &plans {
        config_edit::write(plan)?;
    }
    let mut failures = Vec::new();
    for command in commands {
        let executable = if cfg!(windows) { format!("{command}.cmd") } else { command.to_string() };
        let status = Command::new(executable)
            .args(["--install-extension", "ubugeeei.vscode-ox-content"])
            .status();
        if !status.as_ref().is_ok_and(std::process::ExitStatus::success) {
            failures.push(format!("{command}: {status:?}. Install the IDE CLI and retry."));
        }
    }
    if let Some(target) = nvim_target
        && let Err(error) = config::copy_plugin(assets, &target)
    {
        failures.push(error.to_string());
    }
    if !failures.is_empty() {
        return Err(format!(
            "Workspace setup was applied, but extension installation needs attention:\n{}",
            failures.join("\n")
        )
        .into());
    }
    prompts::print("◆ IDE setup applied")?;
    if extensions && ides.iter().any(|ide| ide == "zed") {
        prompts::print(
            "Zed: restart to apply extension auto-install (requires ox-content in the Zed registry).",
        )?;
    }
    if workspace && nvim {
        prompts::print("Neovim: :luafile .ox-content/neovim.lua (Neovim 0.11+).")?;
    }
    Ok(0)
}
