use super::{ResolvedImageOptions, resolve, transform};
use crate::transformer::MarkdownTransformer;
use crate::{ImageOptions, TransformOptions};

fn enabled() -> ResolvedImageOptions {
    resolve(Some(&ImageOptions { enabled: Some(true), lazy: None })).expect("enabled")
}

fn transform_html(source: &str, options: TransformOptions) -> String {
    MarkdownTransformer::from_options(&options).transform(source).html
}

fn images_on() -> TransformOptions {
    TransformOptions {
        gfm: Some(true),
        images: Some(ImageOptions { enabled: Some(true), lazy: None }),
        ..Default::default()
    }
}

fn images_eager() -> TransformOptions {
    TransformOptions {
        gfm: Some(true),
        images: Some(ImageOptions { enabled: Some(true), lazy: Some(false) }),
        ..Default::default()
    }
}

#[test]
fn resolve_is_none_when_option_is_omitted() {
    assert!(resolve(None).is_none());
}

#[test]
fn resolve_is_none_when_explicitly_disabled() {
    assert!(resolve(Some(&ImageOptions { enabled: Some(false), lazy: Some(true) })).is_none());
}

#[test]
fn resolve_is_some_when_object_is_present() {
    let resolved = resolve(Some(&ImageOptions { enabled: None, lazy: None })).expect("enabled");
    assert!(resolved.lazy);
}

#[test]
fn disabled_by_default() {
    let html = transform_html("![Alt](/x.png \"Cap\")\n", TransformOptions::default());
    assert!(!html.contains("ox-figure"), "default transform must not emit figures:\n{html}");
    assert!(!html.contains(r#"loading="lazy""#), "default transform must not lazy-load:\n{html}");
    assert!(html.contains("<img"), "markdown images still render:\n{html}");
}

#[test]
fn lazy_img_without_caption() {
    let html = transform_html("![Alt](/x.png)\n", images_on());
    assert!(html.contains(r#"<img src="/x.png" alt="Alt" loading="lazy">"#), "{html}");
    assert!(!html.contains("ox-figure"), "{html}");
    assert!(!html.contains("figcaption"), "{html}");
}

#[test]
fn title_becomes_figcaption() {
    let html = transform_html("![Alt](/x.png \"Caption\")\n", images_on());
    assert!(html.contains(r#"<figure class="ox-figure">"#), "{html}");
    assert!(html.contains(r#"<img src="/x.png" alt="Alt" loading="lazy">"#), "{html}");
    assert!(html.contains("<figcaption>Caption</figcaption>"), "{html}");
    assert!(!html.contains(r#"title="Caption""#), "{html}");
}

#[test]
fn skips_fenced_code() {
    let html = transform_html("```md\n![Alt](/x.png \"Cap\")\n```\n", images_on());
    assert!(!html.contains("ox-figure"), "{html}");
    assert!(!html.contains(r#"loading="lazy""#), "{html}");
    assert!(html.contains("![Alt](/x.png"), "{html}");
}

#[test]
fn skips_inline_code() {
    let html = transform_html("Use `![Alt](/x.png \"Cap\")` in docs.\n", images_on());
    assert!(!html.contains("ox-figure"), "{html}");
    assert!(!html.contains(r#"loading="lazy""#), "{html}");
    assert!(html.contains("![Alt](/x.png"), "{html}");
}

#[test]
fn skips_indented_code() {
    let html = transform_html("    ![Alt](/x.png \"Cap\")\n", images_on());
    assert!(!html.contains("ox-figure"), "{html}");
    assert!(!html.contains(r#"loading="lazy""#), "{html}");
}

#[test]
fn hostile_caption_escaped() {
    let html = transform_html(
        "![Alt](/x.png \"<script>alert(1)</script> and \\\"quotes\\\"\")\n",
        images_on(),
    );
    assert!(html.contains("<figcaption>"), "{html}");
    assert!(!html.contains("<script>"), "{html}");
    assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"), "{html}");
    assert!(html.contains("&quot;quotes&quot;") || html.contains("\"quotes\""), "{html}");
}

#[test]
fn hostile_src_rejected() {
    let html = transform_html("![xss](javascript:alert(1))\n", images_on());
    assert!(!html.contains(r#"<img src="javascript:"#), "{html}");
    assert!(!html.contains("javascript:alert"), "{html}");
}

#[test]
fn protocol_relative_and_data_src_rejected() {
    for src in ["//evil.example/x.png", "data:text/html,<h1>x</h1>", "vbscript:msgbox(1)"] {
        let html = transform_html(&format!("![xss]({src})\n"), images_on());
        assert!(!html.contains(&format!(r#"src="{src}""#)), "{src}: {html}");
        assert!(!html.contains(r#"src="//"#) || !src.starts_with("//"), "{src}: {html}");
    }
}

#[test]
fn safe_dimensions_emitted() {
    let html = transform_html("![Alt](/x.png){width=320 height=180}\n", images_on());
    assert!(html.contains(r#"width="320""#), "{html}");
    assert!(html.contains(r#"height="180""#), "{html}");
    assert!(html.contains(r#"src="/x.png""#), "{html}");
    assert!(!html.contains("{width="), "{html}");
}

#[test]
fn hostile_dimensions_rejected() {
    let html = transform_html("![Alt](/x.png){width=100 onclick=alert(1)}\n", images_on());
    assert!(!html.contains("onclick"), "{html}");
    assert!(!html.contains("alert(1)"), "{html}");
    assert!(!html.contains(r#"width="100""#), "{html}");
}

#[test]
fn quoted_hostile_width_is_rejected() {
    let html = transform_html(r#"![Alt](/x.png){width="100 onclick=alert(1)"}"#, images_on());
    assert!(!html.contains("onclick"), "{html}");
    assert!(!html.contains("alert(1)"), "{html}");
}

#[test]
fn lazy_false_omits_loading() {
    let html = transform_html("![Alt](/x.png)\n", images_eager());
    assert!(html.contains(r#"<img src="/x.png" alt="Alt">"#), "{html}");
    assert!(!html.contains("loading="), "{html}");
}

#[test]
fn alt_is_escaped() {
    let html = transform_html("![<em>\"x\"](/x.png)\n", images_on());
    assert!(!html.contains("<em>"), "{html}");
    assert!(html.contains("&lt;em&gt;"), "{html}");
    assert!(html.contains("&quot;x"), "{html}");
}

#[test]
fn transform_helper_rewrites_markdown_to_raw_html() {
    let source = transform("![Alt](/x.png \"Cap\")", &enabled());
    assert!(
        source.contains(
            r#"<figure class="ox-figure"><img src="/x.png" alt="Alt" loading="lazy"><figcaption>Cap</figcaption></figure>"#
        ),
        "{source}"
    );
}
