#![allow(clippy::literal_string_with_formatting_args)]

use super::resolve;
use crate::transformer::MarkdownTransformer;
use crate::{KeyboardKeysOptions, TransformOptions};
use rustc_hash::FxHashMap;

fn transform_html(source: &str, options: TransformOptions) -> String {
    MarkdownTransformer::from_options(&options).transform(source).html
}

fn keys_on() -> TransformOptions {
    keys_options(KeyboardKeysOptions { enabled: Some(true), ..Default::default() })
}

fn keys_options(options: KeyboardKeysOptions) -> TransformOptions {
    TransformOptions { gfm: Some(true), keyboard_keys: Some(options), ..Default::default() }
}

fn key(label: &str) -> String {
    format!(r#"<kbd class="ox-kbd__key">{label}</kbd>"#)
}

fn combo(labels: &[&str]) -> String {
    let inner: String = labels.iter().map(|label| key(label)).collect();
    format!(r#"<kbd class="ox-kbd ox-kbd--combo">{inner}</kbd>"#)
}

fn single(label: &str) -> String {
    format!(r#"<kbd class="ox-kbd">{}</kbd>"#, key(label))
}

#[test]
fn resolve_is_none_when_omitted_or_disabled() {
    assert!(resolve(None).is_none());
    assert!(
        resolve(Some(&KeyboardKeysOptions { enabled: Some(false), ..Default::default() }))
            .is_none()
    );
}

#[test]
fn resolve_is_some_when_object_is_present() {
    assert!(resolve(Some(&KeyboardKeysOptions::default())).is_some());
    assert!(
        resolve(Some(&KeyboardKeysOptions { enabled: Some(true), ..Default::default() })).is_some()
    );
}

#[test]
fn disabled_by_default_leaves_source_literal() {
    let source = "{kbd:Ctrl+K}";
    let html = transform_html(source, TransformOptions::default());
    assert!(!html.contains("ox-kbd"), "{html}");
    assert!(html.contains(source), "{html}");
}

#[test]
fn disabled_object_leaves_source_literal() {
    let source = "{kbd:Ctrl+K}";
    let html = transform_html(
        source,
        keys_options(KeyboardKeysOptions { enabled: Some(false), ..Default::default() }),
    );
    assert!(!html.contains("ox-kbd"), "{html}");
    assert!(html.contains(source), "{html}");
}

#[test]
fn feature_off_matches_feature_on_when_markup_is_absent() {
    let source = "Hello **world** and a [link](https://example.com).\n";
    let off = transform_html(source, TransformOptions { gfm: Some(true), ..Default::default() });
    let on = transform_html(source, keys_on());
    assert_eq!(off, on);
}

#[test]
fn renders_single_key() {
    let html = transform_html("{kbd:Esc}", keys_on());
    assert!(html.contains(&single("Esc")), "{html}");
    assert!(!html.contains("{kbd:"), "{html}");
}

#[test]
fn renders_plus_combination() {
    let html = transform_html("{kbd:Ctrl+K}", keys_on());
    assert!(html.contains(&combo(&["Ctrl", "K"])), "{html}");
    assert!(!html.contains("{kbd:"), "{html}");
}

#[test]
fn renders_space_separated_combination() {
    let html = transform_html("{kbd:Cmd Shift P}", keys_on());
    assert!(html.contains(&combo(&["Command", "Shift", "P"])), "{html}");
}

#[test]
fn normalizes_aliases_in_words_style() {
    let html = transform_html("{kbd:cmd+shift+p}", keys_on());
    assert!(html.contains(&combo(&["Command", "Shift", "p"])), "{html}");
}

#[test]
fn normalizes_aliases_in_symbols_style() {
    let html = transform_html(
        "{kbd:cmd+shift+p}",
        keys_options(KeyboardKeysOptions {
            enabled: Some(true),
            style: Some("symbols".to_string()),
            ..Default::default()
        }),
    );
    assert!(html.contains(&combo(&["⌘", "⇧", "p"])), "{html}");
}

#[test]
fn custom_aliases_override_builtins() {
    let html = transform_html(
        "{kbd:cmd+K}",
        keys_options(KeyboardKeysOptions {
            enabled: Some(true),
            aliases: Some(FxHashMap::from_iter([("cmd".to_string(), "Cmd".to_string())])),
            ..Default::default()
        }),
    );
    assert!(html.contains(&combo(&["Cmd", "K"])), "{html}");
}

#[test]
fn punctuation_keys_stay_visible() {
    let html = transform_html("{kbd:.} {kbd:,} {kbd:/} {kbd:+}", keys_on());
    assert!(html.contains(&single(".")), "{html}");
    assert!(html.contains(&single(",")), "{html}");
    assert!(html.contains(&single("/")), "{html}");
    assert!(html.contains(&single("+")), "{html}");
}

#[test]
fn escaped_literal_stays_visible() {
    let html = transform_html(r"\{kbd:Ctrl+K}", keys_on());
    assert!(!html.contains("ox-kbd"), "{html}");
    assert!(html.contains("{kbd:Ctrl+K}"), "{html}");
    assert!(!html.contains(r"\{kbd:"), "{html}");
}

#[test]
fn unclosed_and_empty_stay_literal() {
    for source in ["{kbd:Ctrl+K", "{kbd:}", "{kbd:   }"] {
        let html = transform_html(source, keys_on());
        assert!(!html.contains("ox-kbd"), "{source} => {html}");
        assert!(html.contains(source), "{html}");
    }
}

#[test]
fn skips_fenced_code() {
    let source = "{kbd:Ctrl+K}";
    let html = transform_html(&format!("```md\n{source}\n```\n"), keys_on());
    assert!(!html.contains(r#"class="ox-kbd""#), "{html}");
    assert!(html.contains(source), "{html}");
}

#[test]
fn skips_inline_code() {
    let source = "{kbd:Ctrl+K}";
    let html = transform_html(&format!("Use `{source}` in docs.\n"), keys_on());
    assert!(!html.contains(r#"class="ox-kbd""#), "{html}");
    assert!(html.contains(source), "{html}");
}

#[test]
fn skips_indented_code() {
    let source = "{kbd:Ctrl+K}";
    let html = transform_html(&format!("    {source}\n"), keys_on());
    assert!(!html.contains(r#"class="ox-kbd""#), "{html}");
    assert!(html.contains(source), "{html}");
}

#[test]
fn skips_html_comments() {
    let source = "<!-- {kbd:Ctrl+K} -->";
    let html = transform_html(source, keys_on());
    assert!(!html.contains("ox-kbd"), "{html}");
    assert!(html.contains("{kbd:Ctrl+K}"), "{html}");
}

#[test]
fn skips_raw_code_and_scripts() {
    for wrapper in ["code", "pre", "script", "style"] {
        let source = format!("<{wrapper}>{{kbd:Ctrl+K}}</{wrapper}>");
        let html = transform_html(&source, keys_on());
        assert!(!html.contains("ox-kbd"), "{wrapper} => {html}");
        assert!(html.contains("{kbd:Ctrl+K}"), "{html}");
    }
}

#[test]
fn rewrites_inside_ordinary_html() {
    let html = transform_html("<div>Press {kbd:Ctrl+K}</div>", keys_on());
    assert!(html.contains(&combo(&["Ctrl", "K"])), "{html}");
}

#[test]
fn hostile_key_text_is_escaped() {
    let html = transform_html("{kbd:<script>alert(1)</script>}", keys_on());
    assert!(html.contains(r#"<kbd class="ox-kbd">"#), "{html}");
    assert!(html.contains("&lt;script&gt;"), "{html}");
    assert!(!html.contains("<script>"), "{html}");
}

#[test]
fn snapshot_accessible_combo_markup() {
    let html = transform_html("Press {kbd:Ctrl+K} to search.", keys_on());
    let markup = combo(&["Ctrl", "K"]);
    assert!(html.contains(&markup), "themeable combo markup missing:\n{html}");
    assert!(html.contains("<kbd class=\"ox-kbd ox-kbd--combo\">"), "{html}");
    assert!(html.contains("<kbd class=\"ox-kbd__key\">Ctrl</kbd>"), "{html}");
}
