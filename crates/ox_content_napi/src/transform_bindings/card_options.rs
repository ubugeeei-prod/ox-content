use napi_derive::napi;
use ox_content_transform::CardOptions;

/// Opt-in `::: card` / `::: link-card` / `::: card-grid` blocks.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsCardOptions {
    /// Enable card, link-card, and card-grid containers.
    ///
    /// Default: `false`.
    pub enabled: Option<bool>,
}

impl From<JsCardOptions> for CardOptions {
    fn from(value: JsCardOptions) -> Self {
        Self { enabled: value.enabled }
    }
}
