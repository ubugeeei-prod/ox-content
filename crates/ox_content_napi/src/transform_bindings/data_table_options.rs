use napi_derive::napi;
use ox_content_transform::DataTableOptions;

/// Opt-in static `csv-table` / `json-table` fences.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsDataTableOptions {
    /// Enable `csv-table` and `json-table` fences.
    ///
    /// Default: `false`.
    pub enabled: Option<bool>,

    /// Root directory used for `@/` and absolute data-table imports.
    ///
    /// Default: project root from the JavaScript caller.
    pub root_dir: Option<String>,

    /// What to do when an imported file is missing.
    ///
    /// Default: `"error"`.
    pub missing: Option<String>,
}

impl From<JsDataTableOptions> for DataTableOptions {
    fn from(value: JsDataTableOptions) -> Self {
        Self { enabled: value.enabled, root_dir: value.root_dir, missing: value.missing }
    }
}
