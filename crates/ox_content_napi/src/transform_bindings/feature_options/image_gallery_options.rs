use napi_derive::napi;
use ox_content_transform::ImageGalleryOptions;

/// Opt-in static `::: gallery` image groups.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsImageGalleryOptions {
    /// Enable static image gallery blocks.
    ///
    /// Default: `false`.
    pub enabled: Option<bool>,

    /// Add `loading="lazy"` to images inside galleries.
    ///
    /// Default: follows `images.lazy`, or `true` when `images` is disabled.
    pub lazy: Option<bool>,

    /// Validation mode for images with empty alt text.
    ///
    /// Default: `"error"`.
    pub missing_alt: Option<String>,

    /// Validation mode for galleries without image items.
    ///
    /// Default: `"error"`.
    pub empty: Option<String>,
}

impl From<JsImageGalleryOptions> for ImageGalleryOptions {
    fn from(value: JsImageGalleryOptions) -> Self {
        Self {
            enabled: value.enabled,
            lazy: value.lazy,
            missing_alt: value.missing_alt,
            empty: value.empty,
        }
    }
}
