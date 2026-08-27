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
fn spotify_iframes_carry_an_accessible_name() {
    let options = MediaEmbedsOptions { spotify: Some(true), ..Default::default() };

    // Every kind names itself, so a screen reader announces more than "frame".
    for kind in ["track", "album", "playlist", "episode", "show", "artist"] {
        let html = transform_media_embeds(
            &format!(r#"<Spotify url="https://open.spotify.com/{kind}/abc123"></Spotify>"#),
            Some(&options),
        );
        assert!(html.contains(&format!(r#"title="Spotify {kind}""#)), "{kind}: {html}");
    }

    // A pre-built embed URL still names its kind.
    let embed = transform_media_embeds(
        r#"<Spotify url="https://open.spotify.com/embed/album/abc123"></Spotify>"#,
        Some(&options),
    );
    assert!(embed.contains(r#"title="Spotify album""#), "{embed}");

    // An author-supplied title wins.
    let titled = transform_media_embeds(
        r#"<Spotify url="https://open.spotify.com/track/abc123" title="Rick Astley set"></Spotify>"#,
        Some(&options),
    );
    assert!(titled.contains(r#"title="Rick Astley set""#), "{titled}");

    // And cannot break out of the attribute it is written into.
    let hostile = transform_media_embeds(
        r#"<Spotify url="https://open.spotify.com/track/abc123" title="a<b>c"></Spotify>"#,
        Some(&options),
    );
    assert!(hostile.contains(r#"title="a&lt;b&gt;c""#), "{hostile}");
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
fn renders_provider_grade_static_cards() {
    let enabled = MediaEmbedsOptions {
        google_maps: Some(true),
        qiita: Some(true),
        zenn: Some(true),
        discord: Some(true),
        fediverse: Some(true),
        facebook: Some(true),
        threads: Some(true),
        instagram: Some(true),
        ..Default::default()
    };
    let html = transform_media_embeds(
        r#"<GoogleMaps url="https://www.google.com/maps/place/Tokyo+Station/" place="Tokyo Station" address="1 Chome Marunouchi, Chiyoda City" embed="https://www.google.com/maps/embed?pb=!1m18"></GoogleMaps>
<Qiita url="https://qiita.com/ubugeeei/items/abcdef123456" title="Rust docs pipeline" author="ubugeeei" tags="Rust, Markdown" likes="42">Static cards keep builds predictable.</Qiita>
<Zenn url="https://zenn.dev/ubugeeei/articles/ox-content" title="Ox Content notes" author="ubugeeei" date="2026-08-26" likes="12"></Zenn>
<Discord url="https://discord.gg/abc123" server="Ox Content" channel="announcements">Join the release channel.</Discord>
<Mastodon url="https://mastodon.social/@docs/111" author="@docs@mastodon.social" replies="3" reposts="5" likes="8">Fediverse release note.</Mastodon>
<Facebook url="https://www.facebook.com/example/posts/123" title="Launch note" author="Example"></Facebook>
<Threads url="https://www.threads.net/@example/post/abc" author="@example">Thread copy.</Threads>
<Instagram url="https://www.instagram.com/p/abc123/" author="@example" image="https://cdn.example.com/photo.jpg">Caption text.</Instagram>"#,
        Some(&enabled),
    );

    insta::assert_snapshot!(html);
}

#[test]
fn renders_package_registry_cards() {
    let enabled = MediaEmbedsOptions { package_registry: Some(true), ..Default::default() };
    let html = transform_media_embeds(
        r#"<NpmPackage url="https://www.npmjs.com/package/@vitejs/plugin-vue/v/6.0.0" title="@vitejs/plugin-vue" license="MIT" repository="https://github.com/vitejs/vite-plugin-vue" dateTime="2026-08-26T00:00:00Z" dateLabel="2026-08-26">Vue support for Vite.</NpmPackage>
<CratesIo url="https://crates.io/crates/serde/1.0.0" description="Serialization framework" license="MIT OR Apache-2.0" downloads="123456"></CratesIo>
<PyPI url="https://pypi.org/project/requests/2.32.0" repository="https://github.com/psf/requests">Python HTTP for Humans.</PyPI>
<DockerHub url="https://hub.docker.com/_/nginx/tags?name=mainline" downloads="987654" stars="321" dateLabel="2026-08-23"></DockerHub>"#,
        Some(&enabled),
    );

    insta::assert_snapshot!(html);
}

#[test]
fn renders_playground_cards() {
    let enabled = MediaEmbedsOptions { playgrounds: Some(true), ..Default::default() };
    let html = transform_media_embeds(
        r#"<CodePen url="https://codepen.io/ubugeeei/pen/abc123" title="Card demo" author="ubugeeei" image="https://shots.codepen.io/card.png">Static preview.</CodePen>
<JSFiddle url="https://jsfiddle.net/ubugeeei/abc123/2/" title="Fiddle demo" embed="https://jsfiddle.net/ubugeeei/abc123/2/embedded/result/"></JSFiddle>
<Observable url="https://observablehq.com/@d3/bar-chart" author="@d3" runtime="Observable Runtime"></Observable>"#,
        Some(&enabled),
    );

    insta::assert_snapshot!(html);
}

#[test]
fn renders_code_sandbox_cards_from_every_url_form() {
    let options = MediaEmbedsOptions { playgrounds: Some(true), ..Default::default() };

    // A sandbox is named four ways and they all mean the same sandbox.
    for url in [
        "https://codesandbox.io/s/vite-react-demo",
        "https://codesandbox.io/p/sandbox/vite-react-demo",
        "https://codesandbox.io/p/devbox/vite-react-demo",
        "https://codesandbox.io/embed/vite-react-demo",
    ] {
        let html = transform_media_embeds(
            &format!(r#"<CodeSandbox url="{url}"></CodeSandbox>"#),
            Some(&options),
        );
        assert!(html.contains("ox-provider-card--codesandbox"), "{url}: {html}");
        assert!(html.contains("CodeSandbox"), "{url}: {html}");
        // The slug becomes the title, with separators read as spaces.
        assert!(html.contains("vite react demo"), "{url}: {html}");
    }

    // Only an /embed/ URL is accepted as an iframe source.
    let framed = transform_media_embeds(
        r#"<CodeSandbox url="https://codesandbox.io/s/demo" embed="https://codesandbox.io/embed/demo"></CodeSandbox>"#,
        Some(&options),
    );
    assert!(framed.contains("<iframe"), "{framed}");

    let unframed = transform_media_embeds(
        r#"<CodeSandbox url="https://codesandbox.io/s/demo" embed="https://evil.example/embed/demo"></CodeSandbox>"#,
        Some(&options),
    );
    assert!(!unframed.contains("<iframe"), "{unframed}");
}

#[test]
fn renders_video_provider_cards() {
    let enabled =
        MediaEmbedsOptions { vimeo: Some(true), twitch: Some(true), ..Default::default() };
    let html = transform_media_embeds(
        r#"<Vimeo url="https://vimeo.com/123456789" title="Vimeo demo" author="Vimeo Staff" duration="1:30" image="https://i.vimeocdn.com/video/123.jpg" embed="https://player.vimeo.com/video/123456789?dnt=1">A video fallback.</Vimeo>
<Twitch url="https://www.twitch.tv/videos/40464143" title="Twitch VOD" channel="twitchdev" duration="2:00:00" embed="https://player.twitch.tv/?video=v40464143&parent=docs.example.com&autoplay=false"></Twitch>
<Twitch url="https://clips.twitch.tv/FriendlySlug" status="Clip"></Twitch>
<Twitch url="https://www.twitch.tv/twitchdev" live="offline"></Twitch>"#,
        Some(&enabled),
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
fn renders_resolved_speaker_deck_card() {
    let html = transform_media_embeds(
        r#"<SpeakerDeck url="https://speakerdeck.com/player/abcdef1234567890" title="My Talk" author="Jane Doe"></SpeakerDeck>"#,
        Some(&MediaEmbedsOptions { speaker_deck: Some(true), ..Default::default() }),
    );
    insta::assert_snapshot!(html);
}

#[test]
fn renders_speaker_deck_fallback_link_card() {
    let html = transform_media_embeds(
        r#"<SpeakerDeck url="https://speakerdeck.com/jane/my-cool-talk" preview="https://files.speakerdeck.com/presentations/abcdef1234567890/slide.jpg"></SpeakerDeck>"#,
        Some(&MediaEmbedsOptions { speaker_deck: Some(true), ..Default::default() }),
    );
    insta::assert_snapshot!(html);
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

// --- Catalog additions (#1100) -------------------------------------------

fn catalog_options() -> crate::MediaEmbedsOptions {
    crate::MediaEmbedsOptions {
        loom: Some(true),
        asciinema: Some(true),
        figma: Some(true),
        note: Some(true),
        google_slides: Some(true),
        playgrounds: Some(true),
        ..Default::default()
    }
}

fn render_catalog(html: &str) -> String {
    transform_media_embeds(html, Some(&catalog_options()))
}

#[test]
fn note_article_renders_a_card() {
    let out = render_catalog(r#"<note url="https://note.com/someone/n/nabc123"></note>"#);
    assert!(out.contains("ox-provider-card--note"), "{out}");
    assert!(out.contains("note"), "{out}");
}

#[test]
fn note_magazine_renders_a_card() {
    let out = render_catalog(r#"<note url="https://note.com/someone/m/mabc123"></note>"#);
    assert!(out.contains("ox-provider-card--note"), "{out}");
}

#[test]
fn a_note_profile_is_not_an_article() {
    // No `/n/` or `/m/` segment, so there is nothing to card.
    let out = render_catalog(r#"<note url="https://note.com/someone"></note>"#);
    assert!(!out.contains("ox-provider-card--note"), "{out}");
}

#[test]
fn loom_share_link_renders_a_card() {
    let out = render_catalog(r#"<loom url="https://www.loom.com/share/abc123def"></loom>"#);
    assert!(out.contains("ox-provider-card--loom"), "{out}");
    assert!(out.contains("Watch on Loom"), "{out}");
}

#[test]
fn a_loom_folder_is_not_a_recording() {
    let out = render_catalog(r#"<loom url="https://www.loom.com/share/folder/abc123"></loom>"#);
    assert!(!out.contains("ox-provider-card--loom"), "{out}");
}

#[test]
fn asciinema_cast_renders_a_card() {
    let out = render_catalog(r#"<asciinema url="https://asciinema.org/a/569727"></asciinema>"#);
    assert!(out.contains("ox-provider-card--asciinema"), "{out}");
    assert!(out.contains("Play recording"), "{out}");
}

#[test]
fn figma_file_design_and_prototype_all_render() {
    for url in [
        "https://www.figma.com/file/AbC123/My-Design",
        "https://www.figma.com/design/AbC123/My-Design",
        "https://www.figma.com/proto/AbC123/My-Design",
        "https://www.figma.com/board/AbC123/My-Board",
    ] {
        let out = render_catalog(&format!(r#"<figma url="{url}"></figma>"#));
        assert!(out.contains("ox-provider-card--figma"), "{url}: {out}");
    }
}

#[test]
fn a_figma_url_with_no_file_key_does_not_render() {
    let out = render_catalog(r#"<figma url="https://www.figma.com/file/"></figma>"#);
    assert!(!out.contains("ox-provider-card--figma"), "{out}");
}

#[test]
fn google_slides_renders_both_the_file_and_published_forms() {
    for url in [
        "https://docs.google.com/presentation/d/1AbC_123/edit",
        "https://docs.google.com/presentation/d/e/2PACX-tok3n/pub",
    ] {
        let out = render_catalog(&format!(r#"<googleslides url="{url}"></googleslides>"#));
        assert!(out.contains("ox-provider-card--googleslides"), "{url}: {out}");
    }
}

#[test]
fn a_google_doc_is_not_a_deck() {
    let out = render_catalog(
        r#"<googleslides url="https://docs.google.com/document/d/1AbC/edit"></googleslides>"#,
    );
    assert!(!out.contains("ox-provider-card--googleslides"), "{out}");
}

#[test]
fn replit_renders_under_the_shared_playgrounds_option() {
    let out = render_catalog(r#"<replit url="https://replit.com/@someone/my-repl"></replit>"#);
    assert!(out.contains("ox-provider-card--replit"), "{out}");
}

#[test]
fn a_replit_url_without_an_owner_handle_does_not_render() {
    // The owner segment must carry the `@`; a bare path is some other page.
    let out = render_catalog(r#"<replit url="https://replit.com/someone/my-repl"></replit>"#);
    assert!(!out.contains("ox-provider-card--replit"), "{out}");
}

#[test]
fn every_catalog_addition_rejects_a_look_alike_host() {
    for (tag, url) in [
        ("note", "https://note.com.evil.example/someone/n/nabc"),
        ("loom", "https://loom.com.evil.example/share/abc"),
        ("asciinema", "https://asciinema.org.evil.example/a/1"),
        ("figma", "https://figma.com.evil.example/file/AbC/x"),
        ("googleslides", "https://docs.google.com.evil.example/presentation/d/1AbC/edit"),
        ("replit", "https://replit.com.evil.example/@someone/repl"),
    ] {
        let out = render_catalog(&format!(r#"<{tag} url="{url}"></{tag}>"#));
        assert!(!out.contains("ox-provider-card--"), "{tag} accepted a look-alike host: {out}");
        // It still becomes a plain link rather than staying as markup.
        assert!(out.contains("ox-embed-fallback"), "{tag}: {out}");
    }
}

#[test]
fn a_catalog_provider_stays_off_until_its_option_is_set() {
    let out = transform_media_embeds(
        r#"<figma url="https://www.figma.com/file/AbC123/My-Design"></figma>"#,
        Some(&crate::MediaEmbedsOptions::default()),
    );
    assert!(!out.contains("ox-provider-card"), "{out}");
}
