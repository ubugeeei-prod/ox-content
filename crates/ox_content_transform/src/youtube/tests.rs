use super::{YouTubeEmbedOptions, extract_video_id, transform_youtube};

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
fn extracts_from_url_and_honours_title_and_start() {
    let html = transform_youtube(
        r#"<youtube url="https://youtu.be/dQw4w9WgXcQ" title="Demo" start="30"></youtube>"#,
        &opts(),
    );
    assert_eq!(
        html,
        r#"<div class="ox-youtube" style="aspect-ratio: 16/9;"><iframe src="https://www.youtube-nocookie.com/embed/dQw4w9WgXcQ?start=30" title="Demo" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen loading="lazy"></iframe></div>"#
    );
}

#[test]
fn honours_unquoted_zero_start() {
    let html = transform_youtube(r#"<youtube id="dQw4w9WgXcQ" start=0></youtube>"#, &opts());
    assert!(html.contains(r#"src="https://www.youtube-nocookie.com/embed/dQw4w9WgXcQ?start=0""#));
}

#[test]
fn ignores_invalid_negative_fractional_overflow_and_hostile_start() {
    for start in [
        r#"start="-1""#,
        r#"start="30.5""#,
        r#"start="+30""#,
        r#"start="4294967296""#,
        r#"start="1e3""#,
        r#"start="javascript:alert(1)""#,
        r#"start="""#,
    ] {
        let input = format!(r#"<youtube id="dQw4w9WgXcQ" {start}></youtube>"#);
        let html = transform_youtube(&input, &opts());
        assert!(
            html.contains(r#"src="https://www.youtube-nocookie.com/embed/dQw4w9WgXcQ""#),
            "{start} must drop the query string, got {html}"
        );
        assert!(!html.contains("start="), "{start} leaked into {html}");
    }
}

#[test]
fn first_start_wins_even_when_invalid() {
    let html =
        transform_youtube(r#"<youtube id="dQw4w9WgXcQ" start="-1" start="30"></youtube>"#, &opts());
    assert!(html.contains(r#"src="https://www.youtube-nocookie.com/embed/dQw4w9WgXcQ""#));
    assert!(!html.contains("start="));
}

#[test]
fn regular_host_keeps_start_query() {
    let options = YouTubeEmbedOptions {
        privacy_enhanced: false,
        aspect_ratio: "16/9".to_string(),
        allow_fullscreen: true,
        lazy_load: true,
    };
    let html = transform_youtube(r#"<youtube id="dQw4w9WgXcQ" start="4190"></youtube>"#, &options);
    assert!(html.contains(r#"src="https://www.youtube.com/embed/dQw4w9WgXcQ?start=4190""#));
}

#[test]
fn passes_through_when_no_element() {
    let html = r#"<p>Plain prose with a <a href="/x">link</a> and no embeds.</p>"#;
    assert_eq!(transform_youtube(html, &opts()), html);
}

#[test]
fn extract_video_id_accepts_bare_ids_and_url_shapes() {
    assert_eq!(extract_video_id("dQw4w9WgXcQ").as_deref(), Some("dQw4w9WgXcQ"));
    assert_eq!(
        extract_video_id("https://www.youtube.com/watch?v=dQw4w9WgXcQ&t=30").as_deref(),
        Some("dQw4w9WgXcQ")
    );
    assert_eq!(extract_video_id("https://youtu.be/dQw4w9WgXcQ").as_deref(), Some("dQw4w9WgXcQ"));
    assert_eq!(
        extract_video_id("https://www.youtube.com/embed/dQw4w9WgXcQ").as_deref(),
        Some("dQw4w9WgXcQ")
    );
    assert_eq!(
        extract_video_id("https://www.youtube.com/v/dQw4w9WgXcQ").as_deref(),
        Some("dQw4w9WgXcQ")
    );
    assert_eq!(
        extract_video_id("https://www.youtube.com/shorts/dQw4w9WgXcQ").as_deref(),
        Some("dQw4w9WgXcQ")
    );
}

#[test]
fn extract_video_id_rejects_hostile_and_partial_inputs() {
    assert_eq!(extract_video_id(""), None);
    assert_eq!(extract_video_id("not-a-valid-id"), None);
    assert_eq!(extract_video_id("dQw4w9WgXc"), None);
    assert_eq!(extract_video_id("dQw4w9WgXcQQ"), None);
    assert_eq!(extract_video_id("javascript:alert(1)"), None);
    assert_eq!(extract_video_id("https://example.com/watch?v=dQw4w9WgXcQ"), None);
    assert_eq!(extract_video_id("https://YOUTUBE.COM/watch?v=dQw4w9WgXcQ"), None);
    assert_eq!(extract_video_id("<script>dQw4w9WgXcQ</script>"), None);
    assert_eq!(
        extract_video_id("https://youtube.com/shorts/abcdefghijk https://youtu.be/dQw4w9WgXcQ")
            .as_deref(),
        Some("dQw4w9WgXcQ")
    );
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
