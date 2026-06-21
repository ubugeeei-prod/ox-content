use super::*;

#[test]
fn removes_scripts_and_event_handlers() {
    let html = r#"<p onclick="x()">Hi<script>alert(1)</script><a href="javascript:x">x</a></p>"#;
    let sanitized = sanitize_html(html, Some(&SanitizeOptions::default()));

    insta::assert_snapshot!(sanitized);
}

#[test]
fn keeps_safe_iframe_sources() {
    let html = r#"<iframe src="https://open.spotify.com/embed/track/a" loading="lazy"></iframe>"#;
    let sanitized = sanitize_html(html, Some(&SanitizeOptions::default()));

    insta::assert_snapshot!(sanitized);
}

#[test]
fn keeps_safe_media_tags_and_attributes() {
    let html = r#"<video controls muted loop playsinline poster="/poster.jpg" width="640" height="360" preload="metadata"><source src="/demo.webm" type="video/webm"><track src="/captions.vtt" kind="captions" srclang="en" label="English" default>Fallback</video><audio controls src="./clip.mp3"></audio><picture><source media="(min-width: 800px)" srcset="/hero-large.jpg 2x, /hero.jpg 1x" sizes="100vw"><img src="/hero.jpg" alt="Hero"></picture>"#;
    let sanitized = sanitize_html(html, Some(&SanitizeOptions::default()));

    insta::assert_snapshot!(sanitized);
}

#[test]
fn removes_unsafe_media_urls() {
    let html = r#"<video poster="javascript:alert(1)"><source src="javascript:alert(2)" srcset="javascript:alert(3) 1x, /safe.jpg 2x"></video><img src="/safe.jpg" srcset="/safe.jpg 1x, javascript:alert(4) 2x">"#;
    let sanitized = sanitize_html(html, Some(&SanitizeOptions::default()));

    insta::assert_snapshot!(sanitized);
}
