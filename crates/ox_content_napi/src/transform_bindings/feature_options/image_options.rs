use napi_derive::napi;
use ox_content_transform::ImageOptions;

/// Opt-in figures, captions, and lazy images.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsImageOptions {
    /// Enable figure captions, lazy loading, and safe dimensions.
    ///
    /// Default: `false`.
    pub enabled: Option<bool>,

    /// Add `loading="lazy"` to transformed images.
    ///
    /// Default: `true`.
    pub lazy: Option<bool>,
}

impl From<JsImageOptions> for ImageOptions {
    fn from(value: JsImageOptions) -> Self {
        Self { enabled: value.enabled, lazy: value.lazy }
    }
}
