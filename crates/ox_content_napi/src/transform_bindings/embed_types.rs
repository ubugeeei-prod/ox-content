use napi_derive::napi;

/// Options for [`super::transform_youtube_embeds`]; all optional, matching the
/// TS `YouTubeOptions` defaults when omitted.
#[napi(object)]
pub struct JsYouTubeOptions {
    /// Use privacy-enhanced mode (`youtube-nocookie.com`).
    ///
    /// Default: `true`.
    pub privacy_enhanced: Option<bool>,

    /// Default iframe aspect ratio.
    ///
    /// Default: `"16/9"`.
    pub aspect_ratio: Option<String>,

    /// Allow fullscreen playback.
    ///
    /// Default: `true`.
    pub allow_fullscreen: Option<bool>,

    /// Lazy-load the iframe.
    ///
    /// Default: `true`.
    pub lazy_load: Option<bool>,
}

/// Result of [`super::transform_tabs_embeds`].
#[napi(object)]
pub struct JsTabsTransformResult {
    /// HTML with every `<tabs>` block expanded.
    pub html: String,

    /// Number of tab groups expanded; the caller advances its group counter by
    /// this amount so generated CSS covers exactly the emitted groups.
    pub group_count: u32,
}

/// Options for [`super::transform_pm_embeds`].
#[napi(object)]
pub struct JsPmOptions {
    /// Enable opt-in synced package-manager tab groups. When `true`, a
    /// `data-ox-tab-group="pkg-manager"` attribute is emitted so the client
    /// runtime keeps every pm tab group on the page in sync via `localStorage`.
    /// Off by default; when omitted/`false` the output has no group attribute
    /// and behaves exactly like a standalone tab group.
    pub sync: Option<bool>,
}

/// Result of [`super::transform_pm_embeds`].
#[napi(object)]
pub struct JsPmTransformResult {
    /// HTML with every `<pm>` block expanded into a package-manager tab widget.
    pub html: String,

    /// Number of tab groups expanded; the caller advances its shared tab-group
    /// counter by this amount.
    pub group_count: u32,
}
