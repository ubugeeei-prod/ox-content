use napi_derive::napi;
use ox_content_transform::MediaEmbedsOptions;

/// Built-in media embed transform switches.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsMediaEmbedsOptions {
    /// Render `<Spotify>` embeds.
    ///
    /// Default: `false`.
    pub spotify: Option<bool>,

    /// Render `<AppleMusic>` embeds.
    ///
    /// Default: `false`.
    pub apple_music: Option<bool>,

    /// Render `<StackBlitz>` embeds.
    ///
    /// Default: `false`.
    pub stack_blitz: Option<bool>,

    /// Render `<Tweet>` / `<XPost>` static cards.
    ///
    /// Default: `false`.
    pub twitter: Option<bool>,

    /// Render `<Bluesky>` static cards.
    ///
    /// Default: `false`.
    pub bluesky: Option<bool>,

    /// Render `<WebContainer>` lazy placeholder blocks.
    ///
    /// Default: `false`.
    pub web_container: Option<bool>,

    /// Render `<Audio>` native players.
    ///
    /// Default: `false`.
    pub audio: Option<bool>,

    /// Render `<Video>` native players.
    ///
    /// Default: `false`.
    pub video: Option<bool>,
}

impl From<JsMediaEmbedsOptions> for MediaEmbedsOptions {
    fn from(value: JsMediaEmbedsOptions) -> Self {
        Self {
            spotify: value.spotify,
            apple_music: value.apple_music,
            audio: value.audio,
            video: value.video,
            stack_blitz: value.stack_blitz,
            twitter: value.twitter,
            bluesky: value.bluesky,
            web_container: value.web_container,
        }
    }
}
