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

fn enabled() -> super::ResolvedKeyboardKeys {
    resolve(Some(&KeyboardKeysOptions { enabled: Some(true), ..Default::default() }))
        .expect("enabled")
}

#[test]
fn remembered_scans_agree_with_scanning_afresh() {
    const PIECES: [&str; 18] = [
        "{kbd:a}", "{kbd:", "}", "\\", "<code>", "</code>", "<pre>", "</pre>", "<", ">", "\"", "'",
        "<!--", "-->", "<a>", "a", " ", "\n",
    ];
    let options = enabled();
    let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
    for round in 0..6000 {
        let pieces = if round < 5000 { 1 + round % 30 } else { 100 + round % 300 };
        let mut segment = String::new();
        for _ in 0..pieces {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            segment.push_str(PIECES[(state % PIECES.len() as u64) as usize]);
        }
        let (mut remembered, mut afresh) = (String::new(), String::new());
        super::replace_with(&segment, &options, &mut remembered, true);
        super::replace_with(&segment, &options, &mut afresh, false);
        assert_eq!(remembered, afresh, "for {segment:?}");
    }
}

#[test]
fn keys_and_unclosed_tags_cost_linear_time() {
    let options = enabled();
    let time = |segment: &str| {
        (0..3)
            .map(|_| {
                let started = std::time::Instant::now();
                let mut out = String::new();
                super::replace(segment, &options, &mut out);
                std::hint::black_box(out.len());
                started.elapsed()
            })
            .min()
            .unwrap_or(std::time::Duration::MAX)
    };
    for unit in ["{kbd:a} ", "{kbd:a} < ", "{kbd:a} <!-- ", "{kbd:a} < \"\""] {
        let (small, large) =
            (unit.repeat(16 * 1024 / unit.len()), unit.repeat(64 * 1024 / unit.len()));
        let (small_time, large_time) = (time(&small), time(&large));
        // 4x the input: linear is about x4, quadratic about x16.
        assert!(
            large_time < small_time * 8 + std::time::Duration::from_millis(20),
            "{unit:?}: 64 KiB took {large_time:?} against {small_time:?} for 16 KiB"
        );
    }
}
