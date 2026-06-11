use super::transform_media_embeds;
use crate::JsMediaEmbedsOptions;

#[test]
fn renders_spotify_iframe() {
    let html = transform_media_embeds(
        r#"<Spotify url="https://open.spotify.com/track/abc123"></Spotify>"#,
        Some(&JsMediaEmbedsOptions {
            spotify: Some(true),
            stack_blitz: None,
            twitter: None,
            bluesky: None,
            web_container: None,
        }),
    );
    assert!(html.contains("https://open.spotify.com/embed/track/abc123"));
}

#[test]
fn renders_static_tweet_card() {
    let html = transform_media_embeds(
        r#"<Tweet id="123">hello</Tweet>"#,
        Some(&JsMediaEmbedsOptions {
            spotify: None,
            stack_blitz: None,
            twitter: Some(true),
            bluesky: None,
            web_container: None,
        }),
    );
    assert!(html.contains("https://x.com/i/web/status/123"));
    assert!(html.contains("hello"));
}
