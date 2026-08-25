#![allow(clippy::literal_string_with_formatting_args)]

use super::{resolve, transform};
use crate::transformer::MarkdownTransformer;
use crate::{MagicLinkAlias, MagicLinkImageOverride, MagicLinkOptions, TransformOptions};

fn transform_html(source: &str, options: TransformOptions) -> String {
    MarkdownTransformer::from_options(&options).transform(source).html
}

fn magic_on() -> TransformOptions {
    TransformOptions {
        gfm: Some(true),
        autolink_urls: Some(false),
        magic_links: Some(MagicLinkOptions {
            enabled: Some(true),
            aliases: Some(
                std::iter::once((
                    "Oxc".to_string(),
                    MagicLinkAlias {
                        href: "https://oxc.rs".to_string(),
                        label: Some("Oxc".to_string()),
                        image: Some("https://github.com/oxc-project.png".to_string()),
                    },
                ))
                .collect(),
            ),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn magic_options(options: MagicLinkOptions) -> TransformOptions {
    TransformOptions {
        gfm: Some(true),
        autolink_urls: Some(false),
        magic_links: Some(options),
        ..Default::default()
    }
}

#[test]
fn resolve_is_none_when_omitted_or_disabled() {
    assert!(resolve(None).is_none());
    assert!(
        resolve(Some(&MagicLinkOptions { enabled: Some(false), ..Default::default() })).is_none()
    );
}

#[test]
fn resolve_is_some_when_object_is_present() {
    assert!(resolve(Some(&MagicLinkOptions::default())).is_some());
    assert!(
        resolve(Some(&MagicLinkOptions { enabled: Some(true), ..Default::default() })).is_some()
    );
}

#[test]
fn disabled_by_default_leaves_source_literal() {
    let source = "{link:@ryoppippi}";
    let html = transform_html(source, TransformOptions::default());
    assert!(!html.contains("ox-magic-link"), "{html}");
    assert!(html.contains(source), "{html}");
}

#[test]
fn disabled_object_leaves_source_literal() {
    let source = "{link:@ryoppippi}";
    let html = transform_html(
        source,
        magic_options(MagicLinkOptions { enabled: Some(false), ..Default::default() }),
    );
    assert!(!html.contains("ox-magic-link"), "{html}");
    assert!(html.contains(source), "{html}");
}

#[test]
fn feature_off_matches_feature_on_when_markup_is_absent() {
    let source = "Hello **world** and a [link](https://example.com).\n";
    let off = transform_html(source, TransformOptions { gfm: Some(true), ..Default::default() });
    let on = transform_html(source, magic_on());
    assert_eq!(off, on);
}

#[test]
fn renders_github_user_with_avatar() {
    let html = transform_html("{link:@ryoppippi}", magic_on());
    assert!(html.contains(r#"href="https://github.com/ryoppippi""#), "{html}");
    assert!(html.contains(r#"class="ox-magic-link ox-magic-link--github""#), "{html}");
    assert!(html.contains(r#"src="https://github.com/ryoppippi.png""#), "{html}");
    assert!(html.contains(r#"<span class="ox-magic-link__label">ryoppippi</span>"#), "{html}");
    assert!(!html.contains("{link:"), "{html}");
}

#[test]
fn renders_github_user_with_label_and_href() {
    let html = transform_html(
        "{link:@ubugeeei|ox-content|https://github.com/ubugeeei?tab=repositories}",
        magic_on(),
    );
    assert!(html.contains(r#"href="https://github.com/ubugeeei?tab=repositories""#), "{html}");
    assert!(html.contains(r#"src="https://github.com/ubugeeei.png""#), "{html}");
    assert!(html.contains(">ox-content</span>"), "{html}");
}

#[test]
fn renders_configured_alias() {
    let html = transform_html("See {link:Oxc}.", magic_on());
    assert!(html.contains(r#"href="https://oxc.rs""#), "{html}");
    assert!(html.contains(r#"class="ox-magic-link ox-magic-link--alias""#), "{html}");
    assert!(html.contains(r#"src="https://github.com/oxc-project.png""#), "{html}");
    assert!(html.contains(">Oxc</span>"), "{html}");
}

#[test]
fn missing_alias_stays_literal() {
    let source = "{link:Unknown}";
    let html = transform_html(source, magic_on());
    assert!(!html.contains("ox-magic-link"), "{html}");
    assert!(html.contains(source), "{html}");
}

#[test]
fn renders_explicit_label_and_url() {
    let html = transform_html("{link:Example|https://example.com}", magic_on());
    assert!(html.contains(r#"href="https://example.com""#), "{html}");
    assert!(html.contains(r#"class="ox-magic-link ox-magic-link--url""#), "{html}");
    assert!(html.contains(">Example</span>"), "{html}");
    assert!(!html.contains("ox-magic-link__image"), "{html}");
}

#[test]
fn favicon_resolver_is_opt_in() {
    let without = transform_html("{link:Example|https://example.com}", magic_on());
    assert!(!without.contains("favicon"), "{without}");

    let with = transform_html(
        "{link:Example|https://example.com}",
        magic_options(MagicLinkOptions {
            enabled: Some(true),
            favicon: Some(true),
            favicon_template: Some("https://icons.example/{host}.ico".to_string()),
            ..Default::default()
        }),
    );
    assert!(with.contains(r#"src="https://icons.example/example.com.ico""#), "{with}");
}

#[test]
fn image_override_wins() {
    let html = transform_html(
        "{link:Oxc}",
        magic_options(MagicLinkOptions {
            enabled: Some(true),
            aliases: Some(
                std::iter::once((
                    "Oxc".to_string(),
                    MagicLinkAlias {
                        href: "https://oxc.rs".to_string(),
                        label: None,
                        image: Some("https://github.com/oxc-project.png".to_string()),
                    },
                ))
                .collect(),
            ),
            image_overrides: Some(vec![MagicLinkImageOverride {
                href: Some("https://oxc.rs".to_string()),
                prefix: None,
                image: "https://example.com/oxc.png".to_string(),
            }]),
            ..Default::default()
        }),
    );
    assert!(html.contains(r#"src="https://example.com/oxc.png""#), "{html}");
    assert!(!html.contains("oxc-project.png"), "{html}");
}

#[test]
fn rejects_unsafe_schemes() {
    for source in [
        "{link:@ryoppippi|bad|javascript:alert(1)}",
        "{link:Bad|javascript:alert(1)}",
        "{link:Bad|data:text/html,hi}",
        "{link:Bad|file:///etc/passwd}",
    ] {
        let html = transform_html(source, magic_on());
        assert!(!html.contains("ox-magic-link"), "{source} => {html}");
        assert!(html.contains(source), "{html}");
    }
}

#[test]
fn escapes_hostile_label() {
    let html = transform_html("{link:@user|<script>alert(1)</script>}", magic_on());
    assert!(html.contains("ox-magic-link--github"), "{html}");
    assert!(html.contains("&lt;script&gt;"), "{html}");
    assert!(!html.contains("<script>"), "{html}");
}

#[test]
fn reserved_github_route_has_no_avatar() {
    let html = transform_html("{link:@issues}", magic_on());
    assert!(html.contains(r#"href="https://github.com/issues""#), "{html}");
    assert!(!html.contains("issues.png"), "{html}");
}

#[test]
fn skips_fenced_code() {
    let source = "{link:@ryoppippi}";
    let html = transform_html(&format!("```md\n{source}\n```\n"), magic_on());
    assert!(!html.contains("ox-magic-link"), "{html}");
    assert!(html.contains(source), "{html}");
}

#[test]
fn skips_inline_code() {
    let source = "{link:@ryoppippi}";
    let html = transform_html(&format!("Use `{source}` in docs.\n"), magic_on());
    assert!(!html.contains("ox-magic-link"), "{html}");
    assert!(html.contains(source), "{html}");
}

#[test]
fn skips_indented_code() {
    let source = "{link:@ryoppippi}";
    let html = transform_html(&format!("    {source}\n"), magic_on());
    assert!(!html.contains("ox-magic-link"), "{html}");
    assert!(html.contains(source), "{html}");
}

#[test]
fn skips_already_linked_text() {
    let source = "[{link:@ryoppippi}](https://example.com)";
    let html = transform_html(source, magic_on());
    assert!(!html.contains("ox-magic-link"), "{html}");
    assert!(html.contains("{link:@ryoppippi}"), "{html}");
    assert!(html.contains(r#"href="https://example.com""#), "{html}");
}

#[test]
fn skips_raw_code_and_scripts() {
    for wrapper in ["code", "pre", "script", "style"] {
        let source = format!("<{wrapper}>{{link:@ryoppippi}}</{wrapper}>");
        let html = transform_html(&source, magic_on());
        assert!(!html.contains("ox-magic-link"), "{wrapper} => {html}");
        assert!(html.contains("{link:@ryoppippi}"), "{html}");
    }
}

#[test]
fn skips_html_attribute_payloads() {
    let source = r#"<div data-x="{link:@ryoppippi}">ok</div>"#;
    let html = transform_html(source, magic_on());
    assert!(!html.contains("ox-magic-link"), "{html}");
    assert!(html.contains(r#"data-x="{link:@ryoppippi}""#), "{html}");
}

#[test]
fn rewrites_inside_ordinary_html() {
    let html = transform_html("<div>See {link:@ryoppippi}</div>", magic_on());
    assert!(html.contains("ox-magic-link--github"), "{html}");
}

#[test]
fn skips_already_linked_html() {
    let source = r#"<a href="https://example.com">{link:@ryoppippi}</a>"#;
    let html = transform_html(source, magic_on());
    assert!(!html.contains("ox-magic-link"), "{html}");
    assert!(html.contains("{link:@ryoppippi}"), "{html}");
}

#[test]
fn transform_is_none_without_marker() {
    let options = resolve(Some(&MagicLinkOptions { enabled: Some(true), ..Default::default() }))
        .expect("enabled");
    assert!(transform("no markers here", &options).is_none());
}
