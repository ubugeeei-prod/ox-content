use napi_derive::napi;
use ox_content_transform::NotByAiOptions;

/// Opt-in `<NotByAI />` authorship badge.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsNotByAiOptions {
    /// Enable `<NotByAI />` expansion.
    ///
    /// Default: `false`.
    pub enabled: Option<bool>,

    /// Accessible label for the badge link.
    ///
    /// Default: `"Written by human, not by AI"`.
    pub label: Option<String>,

    /// Destination URL. Unsafe values fall back to `https://notbyai.fyi`.
    ///
    /// Default: `"https://notbyai.fyi"`.
    pub href: Option<String>,
}

impl From<JsNotByAiOptions> for NotByAiOptions {
    fn from(value: JsNotByAiOptions) -> Self {
        Self { enabled: value.enabled, label: value.label, href: value.href }
    }
}
