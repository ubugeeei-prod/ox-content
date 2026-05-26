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
    /// Path to a JSON file declaring known MDC components and their
    /// attributes. See `ox_content_mdc_checker::Registry`.
    #[serde(rename = "mdcComponents")]
    pub mdc_components: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
struct WorkspaceConfigFile {
    frontmatter: FrontmatterConfigFile,
    mdc: MdcConfigFile,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
struct FrontmatterConfigFile {
    schema: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
struct MdcConfigFile {
    components: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct ResolvedConfig {
    pub frontmatter_schema: Option<PathBuf>,
    pub mdc_components: Option<PathBuf>,
}

impl ResolvedConfig {
    #[must_use]
    pub fn load(root: Option<&Path>, init: &InitializationOptions) -> Self {
        let root = root.map(Path::to_path_buf);
        let workspace_file = load_workspace_file(root.as_deref(), init.config_path.as_deref());

        let frontmatter_schema = init
            .frontmatter_schema
            .as_ref()
            .map(|value| resolve_path(root.as_deref(), value))
            .or_else(|| {
                workspace_file.as_ref().and_then(|(path, config)| {
                    config
                        .frontmatter
                        .schema
                        .as_ref()
                        .map(|value| resolve_path(path.parent(), value))
                })
            })
            .or_else(|| {
                env::var("OX_CONTENT_FRONTMATTER_SCHEMA")
                    .ok()
                    .map(|value| resolve_path(root.as_deref(), &value))
            });

        let mdc_components = init
            .mdc_components
            .as_ref()
            .map(|value| resolve_path(root.as_deref(), value))
            .or_else(|| {
                workspace_file.as_ref().and_then(|(path, config)| {
                    config.mdc.components.as_ref().map(|value| resolve_path(path.parent(), value))
                })
            })
            .or_else(|| {
                env::var("OX_CONTENT_MDC_COMPONENTS")
                    .ok()
                    .map(|value| resolve_path(root.as_deref(), &value))
            });

        Self { frontmatter_schema, mdc_components }
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
