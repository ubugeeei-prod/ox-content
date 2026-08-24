use napi_derive::napi;
use ox_content_transform::BadgeOptions;

/// Opt-in `{badge:variant}` inline badges.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsBadgeOptions {
    /// Enable `{badge:variant}text{/badge}` expansion.
    ///
    /// Default: `false`.
    pub enabled: Option<bool>,
}

impl From<JsBadgeOptions> for BadgeOptions {
    fn from(value: JsBadgeOptions) -> Self {
        Self { enabled: value.enabled }
    }
}
