use napi_derive::napi;
use ox_content_transform::DefinitionListOptions;

/// Opt-in PHP Markdown Extra / mdBook-style definition lists.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsDefinitionListOptions {
    /// Enable `Term` / `: definition` expansion.
    ///
    /// Default: `false`.
    pub enabled: Option<bool>,
}

impl From<JsDefinitionListOptions> for DefinitionListOptions {
    fn from(value: JsDefinitionListOptions) -> Self {
        Self { enabled: value.enabled }
    }
}
