use napi_derive::napi;
use ox_content_transform::FileTreeOptions;

/// Opt-in static `file-tree` fences.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsFileTreeOptions {
    /// Enable `file-tree` fences.
    ///
    /// Default: `false`.
    pub enabled: Option<bool>,
}

impl From<JsFileTreeOptions> for FileTreeOptions {
    fn from(value: JsFileTreeOptions) -> Self {
        Self { enabled: value.enabled }
    }
}
