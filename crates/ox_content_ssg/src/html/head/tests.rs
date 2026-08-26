use super::{HeadAlternate, HeadInput, HeadJsonLd, HeadLink, HeadMeta, SiteHead, render_head};
use crate::html::page::HeadValidation;
use crate::html::urls::{is_safe_href, page_absolute_url, safe_http_url};

#[test]
fn disabled_social_emits_only_title() {
    let html = render_head(&HeadInput {
        title: Some("Guide".into()),
        social: false,
        ..HeadInput::default()
    })
    .html;
    assert_eq!(html, "  <title>Guide</title>");
}

#[test]
fn default_themed_social_matches_built_in_order() {
    let rendered = render_head(&HeadInput {
        site: SiteHead { name: Some("Docs".into()), ..SiteHead::default() },
        title: Some("Guide".into()),
        description: Some("How it works".into()),
        trusted: true,
        ..HeadInput::default()
    });
    assert!(rendered.diagnostics.is_empty());
    assert_eq!(
        rendered.html,
        concat!(
            "  <title>Guide - Docs</title>\n",
            "  <meta name=\"description\" content=\"How it works\">\n",
            "  <meta property=\"og:description\" content=\"How it works\">\n",
            "  <meta name=\"twitter:description\" content=\"How it works\">\n",
            "  <meta property=\"og:type\" content=\"website\">\n",
            "  <meta property=\"og:title\" content=\"Guide - Docs\">\n",
            "  <meta name=\"twitter:card\" content=\"summary_large_image\">\n",
            "  <meta name=\"twitter:title\" content=\"Guide - Docs\">",
        )
    );
}

#[test]
fn later_descriptor_wins_and_keeps_first_position() {
    let html = render_head(&HeadInput {
        title: Some("Guide".into()),
        social: false,
        metas: vec![
            HeadMeta {
                name: Some("description".into()),
                content: "first".into(),
                ..HeadMeta::default()
            },
            HeadMeta {
                name: Some("description".into()),
                content: "second".into(),
                ..HeadMeta::default()
            },
        ],
        ..HeadInput::default()
    })
    .html;
    assert!(html.contains("content=\"second\""));
    assert!(!html.contains("content=\"first\""));
    assert_eq!(html.matches("name=\"description\"").count(), 1);
}

#[test]
fn seo_conflicts_replace_built_ins_without_duplicate_tags() {
    let rendered = render_head(&HeadInput {
        site: SiteHead { name: Some("Docs".into()), ..SiteHead::default() },
        title: Some("Guide".into()),
        description: Some("Built-in summary".into()),
        canonical: Some("https://example.com/guide/".into()),
        robots: Some("noindex".into()),
        validation: HeadValidation::Strict,
        metas: vec![
            HeadMeta {
                name: Some("description".into()),
                content: "Custom summary".into(),
                ..HeadMeta::default()
            },
            HeadMeta {
                name: Some("robots".into()),
                content: "index, follow".into(),
                ..HeadMeta::default()
            },
            HeadMeta {
                property: Some("og:url".into()),
                content: "https://example.com/social/".into(),
                ..HeadMeta::default()
            },
        ],
        alternates: vec![
            HeadAlternate { lang: "en".into(), href: "https://example.com/en/guide/".into() },
            HeadAlternate { lang: "EN".into(), href: "https://example.com/en-us/guide/".into() },
        ],
        links: vec![HeadLink {
            rel: "canonical".into(),
            href: "https://example.com/custom/".into(),
            ..HeadLink::default()
        }],
        ..HeadInput::default()
    });

    assert_eq!(rendered.html.matches("name=\"description\"").count(), 1);
    assert_eq!(rendered.html.matches("name=\"robots\"").count(), 1);
    assert_eq!(rendered.html.matches("property=\"og:url\"").count(), 1);
    assert_eq!(rendered.html.matches("rel=\"canonical\"").count(), 1);
    assert_eq!(rendered.html.matches("rel=\"alternate\"").count(), 1);
    assert!(rendered.html.contains("content=\"Custom summary\""), "{}", rendered.html);
    assert!(rendered.html.contains("content=\"index, follow\""), "{}", rendered.html);
    assert!(rendered.html.contains("content=\"https://example.com/social/\""), "{}", rendered.html);
    assert!(rendered.html.contains("href=\"https://example.com/custom/\""), "{}", rendered.html);
    assert!(
        rendered.html.contains("href=\"https://example.com/en-us/guide/\""),
        "{}",
        rendered.html
    );
    assert!(rendered.diagnostics.iter().all(|diagnostic| !diagnostic.strict));
    assert!(
        rendered
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("description meta overrides")),
        "{:?}",
        rendered.diagnostics
    );
    assert!(
        rendered
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("canonical link overrides")),
        "{:?}",
        rendered.diagnostics
    );
    assert!(
        rendered
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("hreflang \"EN\" alternate overrides")),
        "{:?}",
        rendered.diagnostics
    );
    assert!(
        rendered
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("canonical link and og:url disagree")),
        "{:?}",
        rendered.diagnostics
    );
}

#[test]
fn seo_conflict_diagnostics_stay_silent_when_validation_is_off() {
    let rendered = render_head(&HeadInput {
        description: Some("Built-in summary".into()),
        metas: vec![HeadMeta {
            name: Some("description".into()),
            content: "Custom summary".into(),
            ..HeadMeta::default()
        }],
        ..HeadInput::default()
    });

    assert!(rendered.diagnostics.is_empty());
    assert_eq!(rendered.html.matches("name=\"description\"").count(), 1);
    assert!(rendered.html.contains("content=\"Custom summary\""), "{}", rendered.html);
}

#[test]
fn unsafe_urls_are_dropped_and_escaped_text_cannot_break_out() {
    let rendered = render_head(&HeadInput {
        title: Some("<script>alert(1)</script>".into()),
        canonical: Some("javascript:alert(1)".into()),
        social: false,
        validation: HeadValidation::Strict,
        links: vec![HeadLink {
            rel: "stylesheet".into(),
            href: "data:text/css,body{}".into(),
            ..HeadLink::default()
        }],
        json_ld: vec![HeadJsonLd {
            key: Some("extra".into()),
            json: r#"{"@type":"BlogPosting","headline":"</script><script>"}"#.into(),
        }],
        ..HeadInput::default()
    });
    assert!(rendered.html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    assert!(!rendered.html.contains("javascript:"));
    assert!(!rendered.html.contains("data:text/css"));
    assert!(rendered.html.contains(r"\u003c/script\u003e"));
    assert!(rendered.diagnostics.iter().any(|d| d.strict && d.message.contains("canonical")));
}

#[test]
fn page_url_and_href_safety() {
    assert_eq!(
        page_absolute_url("https://example.com", "/docs/", "guide").as_deref(),
        Some("https://example.com/docs/guide/")
    );
    assert_eq!(
        page_absolute_url("https://example.com", "/docs/", "index").as_deref(),
        Some("https://example.com/docs/")
    );
    assert!(safe_http_url("https://example.com/x").is_some());
    assert!(safe_http_url("javascript:alert(1)").is_none());
    assert!(!is_safe_href("//evil.example"));
    assert!(!is_safe_href("vbscript:x"));
}
