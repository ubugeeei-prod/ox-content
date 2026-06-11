use super::{transform_youtube, YouTubeEmbedOptions};

fn opts() -> YouTubeEmbedOptions {
    YouTubeEmbedOptions::default()
}

#[test]
fn wraps_bare_id_matching_characterization() {
    let html = transform_youtube(r#"<p><youtube id="dQw4w9WgXcQ"></youtube></p>"#, &opts());
    assert_eq!(
        html,
        r#"<p><div class="ox-youtube" style="aspect-ratio: 16/9;"><iframe src="https://www.youtube-nocookie.com/embed/dQw4w9WgXcQ" title="YouTube video dQw4w9WgXcQ" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen loading="lazy"></iframe></div></p>"#
    );
}

#[test]
fn extracts_from_url_and_title_ignoring_start() {
    let html = transform_youtube(
        r#"<youtube url="https://youtu.be/dQw4w9WgXcQ" title="Demo" start="30"></youtube>"#,
        &opts(),
    );
    assert_eq!(
        html,
        r#"<div class="ox-youtube" style="aspect-ratio: 16/9;"><iframe src="https://www.youtube-nocookie.com/embed/dQw4w9WgXcQ" title="Demo" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen loading="lazy"></iframe></div>"#
    );
}

#[test]
fn passes_through_when_no_element() {
    let html = r#"<p>Plain prose with a <a href="/x">link</a> and no embeds.</p>"#;
    assert_eq!(transform_youtube(html, &opts()), html);
}

#[test]
fn leaves_element_untouched_when_id_invalid() {
    let html = r#"<youtube id="not-a-valid-id"></youtube>"#;
    assert_eq!(transform_youtube(html, &opts()), html);
}

#[test]
fn does_not_match_youtuber() {
    let html = r#"<youtuber id="dQw4w9WgXcQ"></youtuber>"#;
    assert_eq!(transform_youtube(html, &opts()), html);
}

#[test]
fn handles_self_closing() {
    let html = transform_youtube(r#"<youtube id="dQw4w9WgXcQ" />"#, &opts());
    assert!(html.starts_with(r#"<div class="ox-youtube""#));
    assert!(html.ends_with("></iframe></div>"));
}

#[test]
fn non_privacy_and_no_fullscreen_no_lazy() {
    let options = YouTubeEmbedOptions {
        privacy_enhanced: false,
        aspect_ratio: "4/3".to_string(),
        allow_fullscreen: false,
        lazy_load: false,
    };
    let html = transform_youtube(r#"<youtube id="dQw4w9WgXcQ"></youtube>"#, &options);
    assert_eq!(
        html,
        r#"<div class="ox-youtube" style="aspect-ratio: 4/3;"><iframe src="https://www.youtube.com/embed/dQw4w9WgXcQ" title="YouTube video dQw4w9WgXcQ" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin"></iframe></div>"#
    );
}
