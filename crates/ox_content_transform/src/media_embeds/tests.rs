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
fn renders_stackblitz_iframe_and_consumes_empty_close_tag() {
    let html = transform_media_embeds(
        r#"<StackBlitz url="https://stackblitz.com/edit/vitejs-vite"></StackBlitz><p>after</p>"#,
        Some(&MediaEmbedsOptions {
            spotify: None,
            stack_blitz: Some(true),
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
        r#"<Tweet url="https://x.com/jack/status/20" displayName="jack" handle="jack" avatar="https://pbs.twimg.com/profile_images/avatar.jpg" datetime="2006-03-21T20:50:14Z" dateLabel="Mar 21, 2006" replies="120" retweets="520" quotes="8" likes="2.4M" views="10M">just setting up my twttr</Tweet>"#,
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

#[test]
fn renders_bluesky_rich_card_with_metadata() {
    let html = transform_media_embeds(
        r#"<Bluesky url="https://bsky.app/profile/bsky.app/post/3l6oveex3ii2l" displayName="Bluesky" handle="bsky.app" avatar="https://bsky.app/static/apple-touch-icon.png" datetime="2024-02-06T12:34:56Z" dateLabel="Feb 6, 2024" replies="10" reposts="20" likes="30" quotes="4">hello sky</Bluesky>"#,
        Some(&MediaEmbedsOptions {
            spotify: None,
            stack_blitz: None,
            twitter: None,
            bluesky: Some(true),
            web_container: None,
        }),
    );
    insta::assert_snapshot!(html);
}
