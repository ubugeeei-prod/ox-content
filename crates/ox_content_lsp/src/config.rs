use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

const DEFAULT_CONFIG_NAMES: &[&str] = &[".ox-content.json", "ox-content.json"];

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct InitializationOptions {
    #[serde(rename = "configPath")]
    pub config_path: Option<String>,
    #[serde(rename = "frontmatterSchema")]
    pub frontmatter_schema: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
struct WorkspaceConfigFile {
    frontmatter: FrontmatterConfigFile,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
struct FrontmatterConfigFile {
    schema: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct ResolvedConfig {
    pub frontmatter_schema: Option<PathBuf>,
}

impl ResolvedConfig {
    #[must_use]
    pub fn load(root: Option<&Path>, init: &InitializationOptions) -> Self {
        let root = root.map(Path::to_path_buf);
        let frontmatter_schema = init
            .frontmatter_schema
            .as_ref()
            .map(|value| resolve_path(root.as_deref(), value))
            .or_else(|| {
                load_workspace_file(root.as_deref(), init.config_path.as_deref()).and_then(
                    |(path, config)| {
                        config.frontmatter.schema.map(|value| resolve_path(path.parent(), &value))
                    },
                )
            })
            .or_else(|| {
                env::var("OX_CONTENT_FRONTMATTER_SCHEMA")
                    .ok()
                    .map(|value| resolve_path(root.as_deref(), &value))
            });

        Self { frontmatter_schema }
    }
}

fn load_workspace_file(
    root: Option<&Path>,
    config_override: Option<&str>,
) -> Option<(PathBuf, WorkspaceConfigFile)> {
    let config_path = if let Some(config_override) = config_override {
        Some(resolve_path(root, config_override))
    } else {
        root.and_then(|root| {
            DEFAULT_CONFIG_NAMES
                .iter()
                .map(|name| root.join(name))
                .find(|candidate| candidate.exists())
        })
    }?;

    let content = fs::read_to_string(&config_path).ok()?;
    let config = serde_json::from_str::<WorkspaceConfigFile>(&content).ok()?;
    Some((config_path, config))
}

fn resolve_path(base: Option<&Path>, value: &str) -> PathBuf {
    let candidate = PathBuf::from(value);
    if candidate.is_absolute() {
        candidate
    } else if let Some(base) = base {
        base.join(candidate)
    } else {
        candidate
    }
}
