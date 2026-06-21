use super::transform_media_embeds;
use crate::MediaEmbedsOptions;

#[test]
fn renders_spotify_iframe() {
    let html = transform_media_embeds(
        r#"<Spotify url="https://open.spotify.com/track/abc123"></Spotify>"#,
        Some(&MediaEmbedsOptions {
            spotify: Some(true),
            stack_blitz: None,
            twitter: None,
            bluesky: None,
            web_container: None,
        }),
    );
    insta::assert_snapshot!(html);
}

#[test]
fn renders_static_tweet_card() {
    let html = transform_media_embeds(
        r#"<Tweet id="123">hello</Tweet>"#,
        Some(&MediaEmbedsOptions {
            spotify: None,
            stack_blitz: None,
            twitter: Some(true),
            bluesky: None,
            web_container: None,
        }),
    );
    insta::assert_snapshot!(html);
}
