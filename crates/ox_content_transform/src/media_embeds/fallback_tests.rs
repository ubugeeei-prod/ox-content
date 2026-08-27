//! What an enabled provider does with a tag it will not embed.
//!
//! Two outcomes, and which one applies is decided by the URL: a tag carrying a
//! link-safe URL degrades to a neutral link, and one that does not keeps its
//! markup because there is nothing better to say.

use super::{EmbedFallback, transform_media_embeds, transform_media_embeds_with_diagnostics};
use crate::MediaEmbedsOptions;

/// A rejected tag either degrades to a neutral link or keeps its markup,
/// depending on whether it carries a URL safe enough to link to.
fn assert_rejected(rendered: &str, source: &str) {
    if rendered == source {
        return;
    }
    assert!(rendered.starts_with(r#"<a class="ox-embed-fallback" href=""#), "{rendered}");
    // The fallback never names the provider, so a look-alike host cannot
    // borrow its styling.
    assert!(!rendered.contains("ox-embed-fallback--"), "{rendered}");
    assert!(!rendered.contains("data-ox-embed"), "{rendered}");
}

#[test]
fn leaves_provider_cards_when_disabled_or_rejected() {
    let input = r#"<Qiita url="https://qiita.com/ubugeeei/items/abcdef123456"></Qiita>"#;
    assert_eq!(transform_media_embeds(input, Some(&MediaEmbedsOptions::default())), input);

    let enabled = MediaEmbedsOptions {
        google_maps: Some(true),
        qiita: Some(true),
        zenn: Some(true),
        package_registry: Some(true),
        playgrounds: Some(true),
        vimeo: Some(true),
        twitch: Some(true),
        discord: Some(true),
        fediverse: Some(true),
        facebook: Some(true),
        threads: Some(true),
        instagram: Some(true),
        ..Default::default()
    };
    for rejected in [
        r#"<GoogleMaps url="https://google.com.evil.example/maps/place/Tokyo"></GoogleMaps>"#,
        r#"<Qiita url="https://qiita.com/ubugeeei"></Qiita>"#,
        r#"<Zenn url="https://zenn.dev/ubugeeei"></Zenn>"#,
        r#"<NpmPackage url="https://user:pass@npmjs.com/package/vite"></NpmPackage>"#,
        r#"<CratesIo url="http://crates.io/crates/serde"></CratesIo>"#,
        r#"<PyPI url="https://pypi.org/user/requests"></PyPI>"#,
        r#"<DockerHub url="https://hub.docker.com.evil/r/library/nginx"></DockerHub>"#,
        r#"<CodePen url="https://codepen.io.evil/ubugeeei/pen/abc123"></CodePen>"#,
        r#"<JSFiddle url="http://jsfiddle.net/ubugeeei/abc123"></JSFiddle>"#,
        r#"<Observable url="https://observablehq.com/docs"></Observable>"#,
        r#"<Vimeo url="https://vimeo.com.evil/123456789"></Vimeo>"#,
        r#"<Twitch url="http://www.twitch.tv/videos/40464143"></Twitch>"#,
        r#"<Twitch url="https://www.twitch.tv/directory"></Twitch>"#,
        r#"<Discord url="https://evil.example/channels/1"></Discord>"#,
        r#"<Facebook url="https://facebook.com.evil.example/post"></Facebook>"#,
        r#"<Threads url="http://threads.net/@example/post/abc"></Threads>"#,
        r#"<Instagram url="https://user:pass@instagram.com/p/abc123/"></Instagram>"#,
    ] {
        assert_rejected(&transform_media_embeds(rejected, Some(&enabled)), rejected);
    }
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
        assert_rejected(&transform_media_embeds(rejected, Some(&enabled)), rejected);
    }
}

#[test]
fn leaves_speaker_deck_when_disabled_or_rejected() {
    let input = r#"<SpeakerDeck url="https://speakerdeck.com/jane/my-cool-talk"></SpeakerDeck>"#;
    assert_eq!(transform_media_embeds(input, Some(&MediaEmbedsOptions::default())), input);

    let enabled = MediaEmbedsOptions { speaker_deck: Some(true), ..Default::default() };
    for rejected in [
        r#"<SpeakerDeck url="javascript:alert(1)"></SpeakerDeck>"#,
        r#"<SpeakerDeck url="data:text/html,hi"></SpeakerDeck>"#,
        r#"<SpeakerDeck url="http://speakerdeck.com/jane/talk"></SpeakerDeck>"#,
        r#"<SpeakerDeck url="https://speakerdeck.com.evil.com/jane/talk"></SpeakerDeck>"#,
        r#"<SpeakerDeck url="https://user:pass@speakerdeck.com/jane/talk"></SpeakerDeck>"#,
    ] {
        assert_rejected(&transform_media_embeds(rejected, Some(&enabled)), rejected);
    }
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
        assert_rejected(&transform_media_embeds(rejected, Some(&enabled)), rejected);
    }
}

#[test]
fn an_unrecognised_url_still_reaches_the_reader() {
    let options = MediaEmbedsOptions { qiita: Some(true), ..Default::default() };

    // A real Qiita URL that is not an item. The card cannot be built, but the
    // link the author wrote must not vanish from the page.
    let rendered = transform_media_embeds(
        r#"<Qiita url="https://qiita.com/ubugeeei"></Qiita>"#,
        Some(&options),
    );
    assert_eq!(
        rendered,
        r#"<a class="ox-embed-fallback" href="https://qiita.com/ubugeeei" target="_blank" rel="noopener noreferrer">https://qiita.com/ubugeeei</a>"#
    );
}

#[test]
fn the_fallback_keeps_what_the_author_wrote() {
    let options = MediaEmbedsOptions { zenn: Some(true), ..Default::default() };

    let labelled = transform_media_embeds(
        r#"<Zenn url="https://zenn.dev/ubugeeei">ubugeeei on Zenn</Zenn>"#,
        Some(&options),
    );
    assert!(labelled.ends_with(">ubugeeei on Zenn</a>"), "{labelled}");

    // And cannot break out of the text or the href.
    let hostile = transform_media_embeds(
        r#"<Zenn url="https://zenn.dev/x"><script>alert(1)</script></Zenn>"#,
        Some(&options),
    );
    assert!(!hostile.contains("<script>"), "{hostile}");
    assert!(hostile.contains("&lt;script&gt;"), "{hostile}");
}

#[test]
fn a_disabled_provider_is_left_completely_alone() {
    let source = r#"<Qiita url="https://qiita.com/ubugeeei"></Qiita>"#;
    let rendered = transform_media_embeds(source, Some(&MediaEmbedsOptions::default()));
    assert_eq!(rendered, source);
}

#[test]
fn a_refused_tag_is_reported_with_its_url_and_line() {
    let options = MediaEmbedsOptions { qiita: Some(true), ..Default::default() };
    let source = concat!(
        "<p>intro</p>\n",
        "<p>more</p>\n",
        r#"<Qiita url="https://qiita.com/ubugeeei"></Qiita>"#,
    );

    let (_, diagnostics) = transform_media_embeds_with_diagnostics(source, Some(&options));

    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    let diagnostic = &diagnostics[0];
    assert_eq!(diagnostic.provider, "qiita");
    assert_eq!(diagnostic.url.as_deref(), Some("https://qiita.com/ubugeeei"));
    assert_eq!(diagnostic.line, 3);
    assert_eq!(diagnostic.fallback, EmbedFallback::Linked);
}

#[test]
fn a_tag_with_no_safe_url_reports_that_its_markup_was_kept() {
    let options = MediaEmbedsOptions { speaker_deck: Some(true), ..Default::default() };
    let (html, diagnostics) = transform_media_embeds_with_diagnostics(
        r#"<SpeakerDeck url="javascript:alert(1)"></SpeakerDeck>"#,
        Some(&options),
    );

    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!(diagnostics[0].fallback, EmbedFallback::Kept);
    assert_eq!(diagnostics[0].url, None);
    assert!(html.contains("<SpeakerDeck"), "{html}");
}

#[test]
fn a_provider_that_resolves_reports_nothing() {
    let options = MediaEmbedsOptions { qiita: Some(true), ..Default::default() };
    let (_, diagnostics) = transform_media_embeds_with_diagnostics(
        r#"<Qiita url="https://qiita.com/ubugeeei/items/abcdef123456"></Qiita>"#,
        Some(&options),
    );
    assert!(diagnostics.is_empty(), "{diagnostics:?}");

    // And a disabled provider is not the transform's business to report.
    let (_, none) = transform_media_embeds_with_diagnostics(
        r#"<Qiita url="https://qiita.com/ubugeeei"></Qiita>"#,
        Some(&MediaEmbedsOptions::default()),
    );
    assert!(none.is_empty(), "{none:?}");
}
