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

    /// Render `<SpeakerDeck>` embeds.
    ///
    /// Default: `false`.
    pub speaker_deck: Option<bool>,

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

    /// Render `<GoogleMaps>` static place cards.
    ///
    /// Default: `false`.
    pub google_maps: Option<bool>,

    /// Render `<Qiita>` static article cards.
    ///
    /// Default: `false`.
    pub qiita: Option<bool>,

    /// Render `<Zenn>` static article cards.
    ///
    /// Default: `false`.
    pub zenn: Option<bool>,

    /// Render `<Discord>` static invite/message cards.
    ///
    /// Default: `false`.
    pub discord: Option<bool>,

    /// Render `<Fediverse>`, `<Mastodon>`, `<Misskey>`, and `<Mixi2>` static post cards.
    ///
    /// Default: `false`.
    pub fediverse: Option<bool>,

    /// Render `<Facebook>` static post cards.
    ///
    /// Default: `false`.
    pub facebook: Option<bool>,

    /// Render `<Threads>` static post cards.
    ///
    /// Default: `false`.
    pub threads: Option<bool>,

    /// Render `<Instagram>` static post cards.
    ///
    /// Default: `false`.
    pub instagram: Option<bool>,

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
            speaker_deck: value.speaker_deck,
            audio: value.audio,
            video: value.video,
            stack_blitz: value.stack_blitz,
            twitter: value.twitter,
            bluesky: value.bluesky,
            google_maps: value.google_maps,
            qiita: value.qiita,
            zenn: value.zenn,
            discord: value.discord,
            fediverse: value.fediverse,
            facebook: value.facebook,
            threads: value.threads,
            instagram: value.instagram,
            web_container: value.web_container,
        }
    }
}
