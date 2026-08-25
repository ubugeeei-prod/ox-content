use std::collections::HashMap;

use napi_derive::napi;
use ox_content_transform::FileTreeOptions;

/// Opt-in static `file-tree` fences.
#[napi(object)]
#[derive(Default, Clone)]
#[allow(clippy::disallowed_types)]
pub struct JsFileTreeOptions {
    /// Enable `file-tree` fences.
    ///
    /// Default: `false`.
    pub enabled: Option<bool>,

    /// Open directory `<details>` by default.
    ///
    /// Default: `true`.
    pub default_open: Option<bool>,

    /// Render folder and file icons.
    ///
    /// Default: `true`.
    pub icons: Option<bool>,

    /// Trusted SVG markup or class tokens for collapsed folders.
    pub icon_folder: Option<String>,

    /// Trusted SVG markup or class tokens for open folders.
    pub icon_folder_open: Option<String>,

    /// Trusted SVG markup or class tokens for files.
    pub icon_file: Option<String>,

    /// Trusted SVG markup or class tokens keyed by file extension.
    pub icon_files: Option<HashMap<String, String>>,
}

impl From<JsFileTreeOptions> for FileTreeOptions {
    fn from(value: JsFileTreeOptions) -> Self {
        Self {
            enabled: value.enabled,
            default_open: value.default_open,
            icons: value.icons,
            icon_folder: value.icon_folder,
            icon_folder_open: value.icon_folder_open,
            icon_file: value.icon_file,
            icon_files: value.icon_files.map(|values| values.into_iter().collect()),
        }
    }
}
