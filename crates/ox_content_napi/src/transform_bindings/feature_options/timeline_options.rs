use napi_derive::napi;
use ox_content_transform::TimelineOptions;

/// Opt-in static `::: timeline` milestone lists.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsTimelineOptions {
    /// Enable static timeline blocks.
    ///
    /// Default: `false`.
    pub enabled: Option<bool>,

    /// Render timelines as ordered lists unless a block overrides it.
    ///
    /// Default: `true`.
    pub ordered: Option<bool>,

    /// Validation mode for malformed dates.
    ///
    /// Default: `"error"`.
    pub invalid_date: Option<String>,

    /// Validation mode for unsupported item metadata.
    ///
    /// Default: `"error"`.
    pub unknown_meta: Option<String>,

    /// Validation mode for timeline blocks without items.
    ///
    /// Default: `"error"`.
    pub empty: Option<String>,
}

impl From<JsTimelineOptions> for TimelineOptions {
    fn from(value: JsTimelineOptions) -> Self {
        Self {
            enabled: value.enabled,
            ordered: value.ordered,
            invalid_date: value.invalid_date,
            unknown_meta: value.unknown_meta,
            empty: value.empty,
        }
    }
}
