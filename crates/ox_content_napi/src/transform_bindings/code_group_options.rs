use napi_derive::napi;
use ox_content_transform::CodeGroupOptions;

/// Opt-in `::: code-group` fence groups.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsCodeGroupOptions {
    /// Enable VitePress-style `::: code-group` rewriting.
    ///
    /// Default: `false`.
    pub enabled: Option<bool>,
}

impl From<JsCodeGroupOptions> for CodeGroupOptions {
    fn from(value: JsCodeGroupOptions) -> Self {
        Self { enabled: value.enabled }
    }
}
