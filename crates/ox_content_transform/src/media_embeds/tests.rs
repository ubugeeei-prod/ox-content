use super::transform_media_embeds;
use crate::MediaEmbedsOptions;

#[test]
fn renders_spotify_iframe() {
    let html = transform_media_embeds(
        r#"<Spotify url="https://open.spotify.com/track/abc123"></Spotify>"#,
        Some(&MediaEmbedsOptions { spotify: Some(true), ..Default::default() }),
    );
    insta::assert_snapshot!(html);
}

#[test]
fn renders_stackblitz_iframe_and_consumes_empty_close_tag() {
    let html = transform_media_embeds(
        r#"<StackBlitz url="https://stackblitz.com/edit/vitejs-vite"></StackBlitz><p>after</p>"#,
        Some(&MediaEmbedsOptions { stack_blitz: Some(true), ..Default::default() }),
    );
    insta::assert_snapshot!(html);
}

#[test]
fn renders_static_tweet_card() {
    let html = transform_media_embeds(
        r#"<Tweet url="https://x.com/jack/status/20" displayName="jack" handle="jack" avatar="https://pbs.twimg.com/profile_images/avatar.jpg" datetime="2006-03-21T20:50:14Z" dateLabel="Mar 21, 2006" replies="120" retweets="520" quotes="8" likes="2.4M" views="10M">just setting up my twttr</Tweet>"#,
        Some(&MediaEmbedsOptions { twitter: Some(true), ..Default::default() }),
    );
    insta::assert_snapshot!(html);
}

#[test]
fn renders_bluesky_rich_card_with_metadata() {
    let html = transform_media_embeds(
        r#"<Bluesky url="https://bsky.app/profile/bsky.app/post/3l6oveex3ii2l" displayName="Bluesky" handle="bsky.app" avatar="https://bsky.app/static/apple-touch-icon.png" datetime="2024-02-06T12:34:56Z" dateLabel="Feb 6, 2024" replies="10" reposts="20" likes="30" quotes="4">hello sky</Bluesky>"#,
        Some(&MediaEmbedsOptions { bluesky: Some(true), ..Default::default() }),
    );
    insta::assert_snapshot!(html);
}

#[test]
fn renders_apple_music_iframe_from_localized_share_url() {
    let html = transform_media_embeds(
        r#"<AppleMusic url="https://music.apple.com/gb/album/1989-taylors-version/1708308989"></AppleMusic>"#,
        Some(&MediaEmbedsOptions { apple_music: Some(true), ..Default::default() }),
    );
    insta::assert_snapshot!(html);
}

#[test]
fn renders_apple_music_playlist_and_song_selection() {
    let enabled = MediaEmbedsOptions { apple_music: Some(true), ..Default::default() };
    let playlist = transform_media_embeds(
        r#"<AppleMusic url="https://music.apple.com/us/playlist/todays-hits/pl.u-abc123"></AppleMusic>"#,
        Some(&enabled),
    );
    let song = transform_media_embeds(
        r#"<AppleMusic url="https://music.apple.com/gb/album/1989-taylors-version/1708308989?i=1708309399"></AppleMusic>"#,
        Some(&enabled),
    );
    let already_embedded = transform_media_embeds(
        r#"<AppleMusic url="https://embed.music.apple.com/us/album/folklore/1524801260"></AppleMusic>"#,
        Some(&enabled),
    );

    assert!(playlist.contains("ox-apple-music"));
    assert!(playlist.contains("https://embed.music.apple.com/us/playlist/todays-hits/pl.u-abc123"));
    assert!(song.contains("height=\"175\""));
    assert!(song.contains("?i=1708309399"));
    assert!(
        already_embedded.contains("https://embed.music.apple.com/us/album/folklore/1524801260")
    );
}

#[test]
fn leaves_apple_music_source_when_disabled_or_rejected() {
    let input = r#"<AppleMusic url="https://music.apple.com/gb/album/1989-taylors-version/1708308989"></AppleMusic>"#;
    let disabled = transform_media_embeds(input, Some(&MediaEmbedsOptions::default()));
    assert_eq!(disabled, input);

    let enabled = MediaEmbedsOptions { apple_music: Some(true), ..Default::default() };
    for rejected in [
        r#"<AppleMusic url="http://music.apple.com/us/album/folklore/1524801260"></AppleMusic>"#,
        r#"<AppleMusic url="https://music.apple.com.evil.com/us/album/folklore/1524801260"></AppleMusic>"#,
        r#"<AppleMusic url="https://user:pass@music.apple.com/us/album/folklore/1524801260"></AppleMusic>"#,
        r#"<AppleMusic url="https://music.apple.com/us/album"></AppleMusic>"#,
        r#"<AppleMusic url="https://music.apple.com/us/album/folklore/1524801260#oops"></AppleMusic>"#,
    ] {
        assert_eq!(transform_media_embeds(rejected, Some(&enabled)), rejected);
    }
}

#[test]
fn renders_native_audio_and_video_players() {
    let enabled = MediaEmbedsOptions { audio: Some(true), video: Some(true), ..Default::default() };
    let audio = transform_media_embeds(
        r#"<Audio src="https://cdn.example.com/intro.mp3" title="Intro" transcript="/intro.txt" download="/intro.mp3"></Audio>"#,
        Some(&enabled),
    );
    let video = transform_media_embeds(
        r#"<Video src="/talk.mp4" poster="/talk.jpg" captions="/talk.en.vtt" srclang="en" label="English" width="1280" height="720" title="Talk"></Video>"#,
        Some(&enabled),
    );
    let tracked = transform_media_embeds(
        r#"<Video src="https://cdn.example.com/talk.mp4"><track kind="subtitles" src="/ja.vtt" srclang="ja" label="日本語" /></Video>"#,
        Some(&enabled),
    );

    assert!(audio.contains("class=\"ox-audio\""));
    assert!(audio.contains("<audio "));
    assert!(audio.contains("controls"));
    assert!(audio.contains("aria-label=\"Intro\""));
    assert!(audio.contains("class=\"ox-av-transcript\""));
    assert!(audio.contains("class=\"ox-av-download\""));
    assert!(video.contains("class=\"ox-video\""));
    assert!(video.contains("poster=\"/talk.jpg\""));
    assert!(video.contains("width=\"1280\""));
    assert!(video.contains("height=\"720\""));
    assert!(video.contains("<track kind=\"captions\" src=\"/talk.en.vtt\""));
    assert!(tracked.contains("<track kind=\"subtitles\" src=\"/ja.vtt\""));
}

#[test]
fn leaves_native_media_when_disabled_or_rejected() {
    let audio = r#"<Audio src="https://cdn.example.com/intro.mp3"></Audio>"#;
    assert_eq!(transform_media_embeds(audio, Some(&MediaEmbedsOptions::default())), audio);

    let enabled = MediaEmbedsOptions { audio: Some(true), video: Some(true), ..Default::default() };
    for rejected in [
        r#"<Audio src="javascript:alert(1)"></Audio>"#,
        r#"<Audio src="data:audio/mp3,abc"></Audio>"#,
        r#"<Audio src="http://cdn.example.com/intro.mp3"></Audio>"#,
        r#"<Video src="//evil.example/talk.mp4"></Video>"#,
        r#"<audio src="https://cdn.example.com/intro.mp3"></audio>"#,
    ] {
        assert_eq!(transform_media_embeds(rejected, Some(&enabled)), rejected);
    }
}

#[test]
fn accepts_https_and_relative_media_sources() {
    use super::native::is_safe_media_url;
    for input in [
        "https://cdn.example.com/talk.mp4",
        "HTTPS://cdn.example.com/talk.mp4",
        "/audio/intro.mp3",
        "./clip.webm",
        "../media/talk.mp4",
        "local.mp3",
    ] {
        assert!(is_safe_media_url(input), "{input}");
    }
    for input in [
        "javascript:alert(1)",
        "data:text/html,hi",
        "http://example.com/a.mp3",
        "//evil.example/a.mp3",
        "https://user:pass@cdn.example.com/a.mp3",
        "vbscript:x",
        "blob:https://example.com/1",
    ] {
        assert!(!is_safe_media_url(input), "{input}");
    }
}
