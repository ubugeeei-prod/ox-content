use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::spacing::{SpaceBetweenHalfAndFullWidth, SpacingConfig};

const DEFAULT_CONFIG_NAMES: &[&str] = &[".ox-content.json", "ox-content.json"];

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct InitializationOptions {
    #[serde(rename = "configPath")]
    pub config_path: Option<String>,
    #[serde(rename = "frontmatterSchema")]
    pub frontmatter_schema: Option<String>,
    /// Opt-in for the textlint sidecar (off by default — textlint
    /// is heavy and noisy for projects that don't use it).
    #[serde(rename = "textlintEnabled", default)]
    pub textlint_enabled: bool,
    /// Optional override command for the textlint binary. Empty
    /// falls back to `npx textlint`.
    #[serde(rename = "textlintCommand")]
    pub textlint_command: Option<String>,
    /// Path to a JSON file declaring known MDC components and their
    /// attributes. See `ox_content_mdc_checker::Registry`.
    #[serde(rename = "mdcComponents")]
    pub mdc_components: Option<String>,
    /// Built-in spacing rule for boundaries such as `Rustと日本語`.
    #[serde(rename = "spaceBetweenHalfAndFullWidth")]
    pub space_between_half_and_full_width: Option<SpaceBetweenHalfAndFullWidth>,
    /// Whether `willSaveWaitUntil` should return built-in spacing fixes.
    #[serde(rename = "spacingAutoFixOnSave", default)]
    pub spacing_auto_fix_on_save: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
struct WorkspaceConfigFile {
    frontmatter: FrontmatterConfigFile,
    textlint: TextlintConfigFile,
    mdc: MdcConfigFile,
    spacing: SpacingConfigFile,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
struct FrontmatterConfigFile {
    schema: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
struct TextlintConfigFile {
    enabled: Option<bool>,
    command: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
struct MdcConfigFile {
    components: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
struct SpacingConfigFile {
    #[serde(rename = "betweenHalfAndFullWidth")]
    between_half_and_full_width: Option<SpaceBetweenHalfAndFullWidth>,
    #[serde(rename = "autoFixOnSave")]
    auto_fix_on_save: Option<bool>,
}

#[derive(Clone, Debug, Default)]
pub struct ResolvedConfig {
    pub frontmatter_schema: Option<PathBuf>,
    pub textlint: crate::textlint::TextlintConfig,
    pub mdc_components: Option<PathBuf>,
    pub spacing: SpacingConfig,
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

        // textlint flows through the same init -> workspace file ->
        // env var pipeline so users have one consistent override
        // model. The init option wins so editors can flip it
        // dynamically without rewriting the workspace config.
        let textlint_enabled = if init.textlint_enabled {
            true
        } else if let Some((_, config)) = workspace_file.as_ref() {
            config.textlint.enabled.unwrap_or(false)
        } else {
            env::var("OX_CONTENT_TEXTLINT_ENABLED")
                .ok()
                .is_some_and(|value| matches!(value.as_str(), "1" | "true" | "yes"))
        };
        let textlint_command = init
            .textlint_command
            .clone()
            .or_else(|| {
                workspace_file.as_ref().and_then(|(_, config)| config.textlint.command.clone())
            })
            .or_else(|| env::var("OX_CONTENT_TEXTLINT_COMMAND").ok());

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

        let between_half_and_full_width = init
            .space_between_half_and_full_width
            .or_else(|| {
                workspace_file
                    .as_ref()
                    .and_then(|(_, config)| config.spacing.between_half_and_full_width)
            })
            .or_else(|| {
                env::var("OX_CONTENT_SPACE_BETWEEN_HALF_AND_FULL_WIDTH")
                    .ok()
                    .and_then(|value| SpaceBetweenHalfAndFullWidth::parse(&value))
            })
            .unwrap_or_default();
        let spacing_auto_fix_on_save = if init.spacing_auto_fix_on_save {
            true
        } else if let Some((_, config)) = workspace_file.as_ref() {
            config.spacing.auto_fix_on_save.unwrap_or(false)
        } else {
            env::var("OX_CONTENT_SPACING_AUTO_FIX_ON_SAVE")
                .ok()
                .is_some_and(|value| matches!(value.as_str(), "1" | "true" | "yes"))
        };

        Self {
            frontmatter_schema,
            textlint: crate::textlint::TextlintConfig {
                enabled: textlint_enabled,
                command: textlint_command,
            },
            mdc_components,
            spacing: SpacingConfig {
                between_half_and_full_width,
                auto_fix_on_save: spacing_auto_fix_on_save,
            },
        }
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
