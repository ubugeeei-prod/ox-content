use super::resolve;
use crate::transformer::MarkdownTransformer;
use crate::{AbbreviationsOptions, TransformOptions};
use rustc_hash::FxHashMap;

fn transform_html(source: &str, options: TransformOptions) -> String {
    MarkdownTransformer::from_options(&options).transform(source).html
}

fn abbr_on() -> TransformOptions {
    abbr_options(AbbreviationsOptions { enabled: Some(true), ..Default::default() })
}

fn abbr_options(options: AbbreviationsOptions) -> TransformOptions {
    TransformOptions { gfm: Some(true), abbreviations: Some(options), ..Default::default() }
}

fn abbr(term: &str, title: &str) -> String {
    format!(r#"<abbr class="ox-abbr" title="{title}">{term}</abbr>"#)
}

#[test]
fn resolve_is_none_when_omitted_or_disabled() {
    assert!(resolve(None).is_none());
    assert!(
        resolve(Some(&AbbreviationsOptions { enabled: Some(false), ..Default::default() }))
            .is_none()
    );
}

#[test]
fn resolve_is_some_when_object_is_present() {
    assert!(resolve(Some(&AbbreviationsOptions::default())).is_some());
    assert!(
        resolve(Some(&AbbreviationsOptions { enabled: Some(true), ..Default::default() }))
            .is_some()
    );
}

#[test]
fn disabled_by_default_leaves_source_literal() {
    let source = "*[LSP]: Language Server Protocol\n\nUse LSP today.\n";
    let html = transform_html(source, TransformOptions::default());
    assert!(!html.contains("ox-abbr"), "{html}");
    assert!(html.contains("*[LSP]"), "{html}");
    assert!(html.contains("LSP"), "{html}");
}

#[test]
fn disabled_object_leaves_source_literal() {
    let source = "*[LSP]: Language Server Protocol\n\nUse LSP today.\n";
    let html = transform_html(
        source,
        abbr_options(AbbreviationsOptions { enabled: Some(false), ..Default::default() }),
    );
    assert!(!html.contains("ox-abbr"), "{html}");
    assert!(html.contains("*[LSP]"), "{html}");
}

#[test]
fn feature_off_matches_feature_on_when_markup_is_absent() {
    let source = "Hello **world** and a [link](https://example.com).\n";
    let off = transform_html(source, TransformOptions { gfm: Some(true), ..Default::default() });
    let on = transform_html(source, abbr_on());
    assert_eq!(off, on);
}

#[test]
fn expands_markdown_definition() {
    let html =
        transform_html("*[LSP]: Language Server Protocol\n\nUse LSP in the editor.\n", abbr_on());
    assert!(html.contains(&abbr("LSP", "Language Server Protocol")), "{html}");
    assert!(!html.contains("*[LSP]"), "{html}");
}

#[test]
fn expands_every_occurrence_by_default() {
    let html =
        transform_html("*[API]: Application Programming Interface\n\nAPI then API.\n", abbr_on());
    assert_eq!(html.matches("ox-abbr").count(), 2, "{html}");
}

#[test]
fn first_use_only_wraps_the_first_occurrence() {
    let html = transform_html(
        "*[API]: Application Programming Interface\n\nAPI then API.\n",
        abbr_options(AbbreviationsOptions {
            enabled: Some(true),
            first_use_only: Some(true),
            ..Default::default()
        }),
    );
    assert_eq!(html.matches("ox-abbr").count(), 1, "{html}");
    assert!(html.contains("then API"), "{html}");
}

#[test]
fn config_terms_expand_without_markdown_defs() {
    let html = transform_html(
        "Talk about LSP.\n",
        abbr_options(AbbreviationsOptions {
            enabled: Some(true),
            terms: Some(FxHashMap::from_iter([(
                "LSP".to_string(),
                "Language Server Protocol".to_string(),
            )])),
            ..Default::default()
        }),
    );
    assert!(html.contains(&abbr("LSP", "Language Server Protocol")), "{html}");
}

#[test]
fn markdown_definition_overrides_config_term() {
    let html = transform_html(
        "*[LSP]: page local\n\nLSP\n",
        abbr_options(AbbreviationsOptions {
            enabled: Some(true),
            terms: Some(FxHashMap::from_iter([(
                "LSP".to_string(),
                "Language Server Protocol".to_string(),
            )])),
            ..Default::default()
        }),
    );
    assert!(html.contains(&abbr("LSP", "page local")), "{html}");
    assert!(!html.contains("Language Server Protocol"), "{html}");
}

#[test]
fn unicode_word_boundaries_skip_ascii_affixes() {
    let html = transform_html("*[LSP]: Language Server Protocol\n\nXLSPY and myLSP.\n", abbr_on());
    assert!(!html.contains("ox-abbr"), "{html}");
}

#[test]
fn unicode_latin_term_matches_as_a_word() {
    let html = transform_html("*[café]: coffee shop\n\nA café nearby.\n", abbr_on());
    assert!(html.contains(&abbr("café", "coffee shop")), "{html}");
}

#[test]
fn unicode_cjk_adjacent_latin_acronym_matches() {
    let html = transform_html("*[LSP]: Language Server Protocol\n\n日本語のLSPです。\n", abbr_on());
    assert!(html.contains(&abbr("LSP", "Language Server Protocol")), "{html}");
}

#[test]
fn unicode_cjk_term_does_not_match_inside_a_longer_word() {
    let html = transform_html("*[総務]: general affairs\n\n総務省\n", abbr_on());
    assert!(!html.contains("ox-abbr"), "{html}");
}

#[test]
fn skips_markdown_link_text() {
    let html = transform_html(
        "*[LSP]: Language Server Protocol\n\nSee [the LSP guide](https://example.com).\n",
        abbr_on(),
    );
    assert!(!html.contains("ox-abbr"), "{html}");
    assert!(html.contains("the LSP guide"), "{html}");
}

#[test]
fn skips_existing_html_links() {
    let html = transform_html(
        "*[LSP]: Language Server Protocol\n\n<a href=\"https://example.com\">LSP</a>\n",
        abbr_on(),
    );
    assert!(!html.contains("ox-abbr"), "{html}");
}

#[test]
fn rewrites_inside_ordinary_html() {
    let html =
        transform_html("*[LSP]: Language Server Protocol\n\n<div>Use LSP here</div>\n", abbr_on());
    assert!(html.contains(&abbr("LSP", "Language Server Protocol")), "{html}");
}

#[test]
fn skips_existing_abbr_html() {
    let html = transform_html(
        "*[LSP]: Language Server Protocol\n\n<abbr title=\"already\">LSP</abbr>\n",
        abbr_on(),
    );
    assert_eq!(html.matches("ox-abbr").count(), 0, "{html}");
    assert!(html.contains("<abbr title=\"already\">LSP</abbr>"), "{html}");
}

#[test]
fn skips_fenced_code() {
    let html = transform_html("*[LSP]: Language Server Protocol\n\n```md\nLSP\n```\n", abbr_on());
    assert!(!html.contains(r#"class="ox-abbr""#), "{html}");
}

#[test]
fn skips_inline_code() {
    let html = transform_html("*[LSP]: Language Server Protocol\n\nUse `LSP`.\n", abbr_on());
    assert!(!html.contains(r#"class="ox-abbr""#), "{html}");
    assert!(html.contains("LSP"), "{html}");
}

#[test]
fn skips_indented_code() {
    let html = transform_html("*[LSP]: Language Server Protocol\n\n    LSP\n", abbr_on());
    assert!(!html.contains(r#"class="ox-abbr""#), "{html}");
}

#[test]
fn skips_html_comments() {
    let html = transform_html("*[LSP]: Language Server Protocol\n\n<!-- LSP -->\n", abbr_on());
    assert!(!html.contains("ox-abbr"), "{html}");
}

#[test]
fn skips_raw_code_and_scripts() {
    for wrapper in ["code", "pre", "script", "style"] {
        let html = transform_html(
            &format!("*[LSP]: Language Server Protocol\n\n<{wrapper}>LSP</{wrapper}>\n"),
            abbr_on(),
        );
        assert!(!html.contains("ox-abbr"), "{wrapper} => {html}");
    }
}

#[test]
fn malformed_definitions_stay_visible() {
    for source in ["*[LSP] Language Server Protocol\n", "*[LSP]:\n", "*[]: empty\n"] {
        let html = transform_html(source, abbr_on());
        assert!(!html.contains("ox-abbr"), "{source} => {html}");
        assert!(html.contains("*["), "{html}");
    }
}

#[test]
fn hostile_title_and_term_are_escaped() {
    let html = transform_html("*[XSS]: <script>alert(1)</script>\n\nWatch for XSS.\n", abbr_on());
    assert!(html.contains("ox-abbr"), "{html}");
    assert!(html.contains("&lt;script&gt;"), "{html}");
    assert!(!html.contains("<script>"), "{html}");
}
