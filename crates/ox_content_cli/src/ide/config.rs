use crate::{
    Result,
    config_edit::{self, Plan},
};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

pub fn home(variable: &str, fallback: &str) -> Result<PathBuf> {
    if let Some(value) = std::env::var_os(variable) {
        return Ok(PathBuf::from(value));
    }
    let user = std::env::home_dir().ok_or("Could not locate the user home directory")?;
    Ok(user.join(fallback))
}

pub fn plans(ides: &[String], workspace: bool, extensions: bool) -> Result<Vec<Plan>> {
    let root = std::env::current_dir()?;
    let mut plans = Vec::new();
    if workspace && ides.iter().any(|ide| super::command(ide).is_some()) {
        plans.push(config_edit::plan(
            &root.join(".vscode/settings.json"),
            &[
                (&["files.associations", "*.mdc"], json!("markdown")),
                (&["oxContent.frontmatter.projectValidation"], json!(true)),
            ],
        )?);
        let file = root.join(".vscode/extensions.json");
        let existing = config_edit::parse(config_edit::read_optional(&file)?.as_deref())?;
        let recommendations =
            merged_strings(&existing["recommendations"], &["ubugeeei.vscode-ox-content"])?;
        plans.push(config_edit::plan(&file, &[(&["recommendations"], recommendations)])?);
    }
    if ides.iter().any(|ide| ide == "zed") {
        if workspace {
            let file = root.join(".zed/settings.json");
            let existing = config_edit::parse(config_edit::read_optional(&file)?.as_deref())?;
            plans.push(config_edit::plan(
                &file,
                &[
                    (
                        &["file_types", "Markdown"],
                        merged_strings(
                            &existing["file_types"]["Markdown"],
                            &["md", "markdown", "mdc", "mdx"],
                        )?,
                    ),
                    (&["lsp", "ox-content-lsp", "binary", "path"], json!("vpx")),
                    (
                        &["lsp", "ox-content-lsp", "binary", "arguments"],
                        json!(["oxct", "lsp", "--project"]),
                    ),
                ],
            )?);
        }
        if extensions {
            let config = home("XDG_CONFIG_HOME", ".config")?;
            let file = if cfg!(windows) {
                std::env::var_os("APPDATA").map_or(config, PathBuf::from).join("Zed/settings.json")
            } else {
                config.join("zed/settings.json")
            };
            plans.push(config_edit::plan(
                &file,
                &[(&["auto_install_extensions", "ox-content"], json!(true))],
            )?);
        }
    }
    if workspace && ides.iter().any(|ide| ide == "neovim") {
        let file = root.join(".ox-content/neovim.lua");
        let original = config_edit::read_optional(&file)?;
        let content = include_str!("neovim.lua").to_string();
        if original.as_ref().is_some_and(|original| *original != content) {
            return Err(format!(
                "Preserving existing {}. Move it before generating a replacement.",
                file.display()
            )
            .into());
        }
        plans.push(Plan { file, original, content });
    }
    Ok(plans)
}

fn merged_strings(existing: &Value, additions: &[&str]) -> Result<Value> {
    let mut values = if existing.is_null() {
        Vec::new()
    } else {
        existing.as_array().ok_or("Invalid configuration: expected an array of strings")?.clone()
    };
    if values.iter().any(|value| !value.is_string()) {
        return Err("Invalid configuration: expected an array of strings".into());
    }
    for value in additions {
        let value = json!(value);
        if !values.contains(&value) {
            values.push(value);
        }
    }
    Ok(Value::Array(values))
}

pub fn copy_plugin(source: &Path, target: &Path) -> Result<()> {
    if !source.is_dir() {
        return Err(format!("Bundled Neovim plugin is missing: {}", source.display()).into());
    }
    // Never overwrite an existing plugin, including concurrent creations.
    if target.exists() {
        return Err(format!(
            "Plugin already exists at {}; preserving it. Update it through your plugin manager.",
            target.display()
        )
        .into());
    }
    std::fs::create_dir_all(target.parent().ok_or("Missing plugin parent")?)?;
    std::fs::create_dir(target)?;
    copy_directory(source, target)
}

fn copy_directory(source: &Path, target: &Path) -> Result<()> {
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let target = target.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            std::fs::create_dir(&target)?;
            copy_directory(&entry.path(), &target)?;
        } else if entry.file_type()?.is_file() {
            use std::io::Write;
            let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(target)?;
            file.write_all(&std::fs::read(entry.path())?)?;
        }
    }
    Ok(())
}
