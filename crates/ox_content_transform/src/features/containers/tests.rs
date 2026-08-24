use super::{ResolvedContainerOptions, ResolvedContainerType, resolve, transform};
use crate::transformer::MarkdownTransformer;
use crate::{ContainerOptions, ContainerTypeOptions, TransformOptions};
use rustc_hash::FxHashMap;

fn enabled() -> ResolvedContainerOptions {
    resolve(Some(&ContainerOptions { enabled: Some(true), types: None })).expect("enabled")
}

fn transform_html(source: &str, options: TransformOptions) -> String {
    MarkdownTransformer::from_options(&options).transform(source).html
}

fn containers_on() -> TransformOptions {
    TransformOptions {
        gfm: Some(true),
        containers: Some(ContainerOptions { enabled: Some(true), types: None }),
        ..Default::default()
    }
}

#[test]
fn resolve_is_none_when_option_is_omitted() {
    assert!(resolve(None).is_none());
}

#[test]
fn resolve_is_none_when_explicitly_disabled() {
    assert!(resolve(Some(&ContainerOptions { enabled: Some(false), types: None })).is_none());
}

#[test]
fn resolve_is_some_when_enabled_is_none_but_object_is_present() {
    let resolved = resolve(Some(&ContainerOptions { enabled: None, types: None }));
    assert!(resolved.is_some());
    assert!(resolved.unwrap().types.contains_key("tip"));
}

#[test]
fn resolve_rejects_hostile_custom_type_names() {
    let mut types = FxHashMap::default();
    types.insert(
        r#"tip" onclick="alert(1)"#.to_string(),
        ContainerTypeOptions { title: Some("X".into()), tag: None },
    );
    types.insert("ok_type".into(), ContainerTypeOptions { title: Some("Ok".into()), tag: None });
    let resolved = resolve(Some(&ContainerOptions { enabled: Some(true), types: Some(types) }))
        .expect("resolved");
    assert!(!resolved.types.contains_key(r#"tip" onclick="alert(1)"#));
    assert!(resolved.types.contains_key("ok_type"));
}

#[test]
fn disabled_by_default_leaves_colon_fences_literal() {
    let html = transform_html("::: tip\nHello\n:::\n", TransformOptions::default());
    assert!(!html.contains("ox-container"), "default transform must not emit containers:\n{html}");
    assert!(
        html.contains("::: tip") || html.contains("Hello"),
        "source must remain visible:\n{html}"
    );
}

#[test]
fn disabled_object_leaves_colon_fences_literal() {
    let html = transform_html(
        "::: tip\nHello\n:::\n",
        TransformOptions {
            containers: Some(ContainerOptions { enabled: Some(false), types: None }),
            ..Default::default()
        },
    );
    assert!(!html.contains("ox-container"), "{html}");
}

#[test]
fn renders_builtin_tip_with_default_title() {
    let html = transform_html("::: tip\nHello **world**.\n:::\n", containers_on());
    assert!(html.contains(r#"<div class="ox-container ox-container--tip">"#), "{html}");
    assert!(html.contains(r#"<p class="ox-container-title">Tip</p>"#), "{html}");
    assert!(html.contains("<strong>world</strong>"), "inner markdown must parse:\n{html}");
    assert!(html.contains("</div>"), "{html}");
    assert!(!html.contains(":::"), "markers must be consumed:\n{html}");
}

#[test]
fn renders_every_builtin_type() {
    for (name, title, class_suffix) in [
        ("note", "Note", "note"),
        ("info", "Info", "info"),
        ("important", "Important", "important"),
        ("warning", "Warning", "warning"),
        ("danger", "Danger", "danger"),
        ("caution", "Caution", "caution"),
    ] {
        let source = format!("::: {name}\nBody\n:::\n");
        let html = transform_html(&source, containers_on());
        assert!(
            html.contains(&format!(r#"class="ox-container ox-container--{class_suffix}""#)),
            "{name}: {html}"
        );
        assert!(
            html.contains(&format!(r#"<p class="ox-container-title">{title}</p>"#)),
            "{name}: {html}"
        );
    }
}

#[test]
fn details_renders_as_details_and_summary() {
    let html = transform_html("::: details\nHidden\n:::\n", containers_on());
    assert!(html.contains(r#"<details class="ox-container ox-container--details">"#), "{html}");
    assert!(html.contains("<summary>Details</summary>"), "{html}");
    assert!(html.contains("Hidden"), "{html}");
    assert!(html.contains("</details>"), "{html}");
    assert!(!html.contains("<div class=\"ox-container ox-container--details\">"), "{html}");
}

#[test]
fn details_open_attribute_is_boolean() {
    let html = transform_html("::: details{open}\nHidden\n:::\n", containers_on());
    assert!(html.contains("<details class=\"ox-container ox-container--details\" open>"), "{html}");
}

#[test]
fn open_attribute_is_ignored_on_non_details() {
    let html = transform_html("::: tip{open}\nHi\n:::\n", containers_on());
    assert!(!html.contains(" open"), "{html}");
}

#[test]
fn custom_bracket_title_is_used() {
    let html = transform_html("::: tip[Did you know?]\nFact\n:::\n", containers_on());
    assert!(html.contains("<p class=\"ox-container-title\">Did you know?</p>"), "{html}");
}

#[test]
fn custom_trailing_title_is_used() {
    let html = transform_html("::: warning Watch out\nCareful\n:::\n", containers_on());
    assert!(html.contains("<p class=\"ox-container-title\">Watch out</p>"), "{html}");
}

#[test]
fn title_html_is_escaped() {
    let html = transform_html("::: tip[<script>alert(1)</script>]\nBody\n:::\n", containers_on());
    assert!(!html.contains("<script>"), "{html}");
    assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"), "{html}");
}

#[test]
fn id_and_class_attrs_are_emitted_inside_the_class_and_id_attributes() {
    let html = transform_html("::: tip{.lead #install}\nBody\n:::\n", containers_on());
    assert!(
        html.contains(r#"<div class="ox-container ox-container--tip lead" id="install">"#),
        "{html}"
    );
}

#[test]
fn hostile_attrs_are_dropped() {
    let html = transform_html("::: tip{onclick=alert(1) .ok #ok}\nBody\n:::\n", containers_on());
    assert!(!html.contains("onclick"), "{html}");
    assert!(html.contains(r#"class="ox-container ox-container--tip ok""#), "{html}");
    assert!(html.contains(r#"id="ok""#), "{html}");
}

#[test]
fn unknown_type_stays_literal() {
    let html = transform_html("::: spaceship\nBody\n:::\n", containers_on());
    assert!(!html.contains("ox-container--spaceship"), "{html}");
    assert!(html.contains("spaceship") || html.contains(":::"), "{html}");
}

#[test]
fn custom_type_can_be_registered() {
    let mut types = FxHashMap::default();
    types.insert("cli".into(), ContainerTypeOptions { title: Some("CLI".into()), tag: None });
    let html = transform_html(
        "::: cli\nRun this\n:::\n",
        TransformOptions {
            gfm: Some(true),
            containers: Some(ContainerOptions { enabled: Some(true), types: Some(types) }),
            ..Default::default()
        },
    );
    assert!(html.contains(r#"class="ox-container ox-container--cli""#), "{html}");
    assert!(html.contains("<p class=\"ox-container-title\">CLI</p>"), "{html}");
}

#[test]
fn custom_details_tag_uses_details() {
    let mut types = FxHashMap::default();
    types.insert(
        "spoiler".into(),
        ContainerTypeOptions { title: Some("Spoiler".into()), tag: Some("details".into()) },
    );
    let html = transform_html(
        "::: spoiler\nSecret\n:::\n",
        TransformOptions {
            gfm: Some(true),
            containers: Some(ContainerOptions { enabled: Some(true), types: Some(types) }),
            ..Default::default()
        },
    );
    assert!(html.contains("<details class=\"ox-container ox-container--spoiler\">"), "{html}");
    assert!(html.contains("<summary>Spoiler</summary>"), "{html}");
}

#[test]
fn fenced_code_is_not_rewritten() {
    let html = transform_html("```md\n::: tip\nHello\n:::\n```\n", containers_on());
    assert!(!html.contains("ox-container"), "{html}");
    assert!(html.contains("::: tip"), "{html}");
}

#[test]
fn inline_code_is_not_rewritten() {
    let html = transform_html("Use `::: tip` in docs.\n", containers_on());
    assert!(!html.contains("ox-container"), "{html}");
    assert!(html.contains("::: tip"), "{html}");
}

#[test]
fn indented_opener_up_to_three_spaces_is_recognized() {
    let html = transform_html("   ::: tip\nHello\n   :::\n", containers_on());
    assert!(html.contains("ox-container--tip"), "{html}");
}

#[test]
fn four_space_indent_is_not_a_container() {
    let html = transform_html("    ::: tip\n    Hello\n    :::\n", containers_on());
    assert!(!html.contains("ox-container"), "{html}");
}

#[test]
fn nested_containers_use_extra_colons() {
    let html = transform_html(
        "::: warning\nOuter\n:::: tip\nInner\n::::\nStill outer\n:::\n",
        containers_on(),
    );
    assert!(html.contains("ox-container--warning"), "{html}");
    assert!(html.contains("ox-container--tip"), "{html}");
    let warning_at = html.find("ox-container--warning").expect("warning");
    let tip_at = html.find("ox-container--tip").expect("tip");
    let tip_close = html[tip_at..].find("</div>").expect("inner close") + tip_at;
    let warning_close = html.rfind("</div>").expect("outer close");
    assert!(tip_at > warning_at, "{html}");
    assert!(tip_close < warning_close, "{html}");
    assert!(html.contains("Still outer"), "{html}");
}

#[test]
fn unclosed_container_is_still_closed() {
    let html = transform_html("::: tip\nHello\n", containers_on());
    assert!(html.contains("ox-container--tip"), "{html}");
    assert!(html.contains("</div>"), "{html}");
}

#[test]
fn extra_closer_is_left_literal() {
    let html = transform_html(":::\n", containers_on());
    assert!(!html.contains("ox-container"), "{html}");
}

#[test]
fn multiple_paragraphs_and_lists_parse_inside() {
    let html = transform_html(
        "::: note\nFirst paragraph.\n\n- item one\n- item two\n:::\n",
        containers_on(),
    );
    assert!(html.contains("<p>First paragraph.</p>"), "{html}");
    assert!(html.contains("<ul>"), "{html}");
    assert!(html.contains("<li>item one</li>"), "{html}");
}

#[test]
fn windows_newlines_work() {
    let html = transform("::: tip\r\nHello\r\n:::\r\n", &enabled());
    assert!(html.contains("ox-container--tip"), "{html}");
    assert!(html.contains("</div>"), "{html}");
}

#[test]
fn github_style_callouts_still_render_when_containers_are_on() {
    let html = transform_html("> [!WARNING]\n> Legacy callout\n", containers_on());
    assert!(html.contains("ox-callout"), "{html}");
    assert!(html.contains("ox-callout--warning"), "{html}");
    assert!(!html.contains("ox-container--warning"), "{html}");
}

#[test]
fn transformer_errors_stay_empty_on_valid_input() {
    let result =
        MarkdownTransformer::from_options(&containers_on()).transform("::: tip\nHello\n:::\n");
    assert!(result.errors.is_empty(), "{:?}", result.errors);
}

#[test]
fn empty_body_still_emits_wrapper_and_title() {
    let html = transform_html("::: tip\n:::\n", containers_on());
    assert!(html.contains("ox-container--tip"), "{html}");
    assert!(html.contains("ox-container-title"), "{html}");
}

#[test]
fn does_not_rewrite_colon_rules_that_are_not_containers() {
    let html = transform_html(":::\n\n---\n", containers_on());
    assert!(!html.contains("ox-container"), "{html}");
}

#[test]
fn transform_helper_emits_blank_line_so_inner_markdown_parses() {
    let source = transform("::: tip\nHello **world**\n:::\n", &enabled());
    assert!(
        source.contains("<p class=\"ox-container-title\">Tip</p>\n\nHello **world**\n\n</div>\n"),
        "{source}"
    );
}

#[test]
fn resolved_type_titles_follow_custom_overrides() {
    let mut types = FxHashMap::default();
    types.insert("tip".into(), ContainerTypeOptions { title: Some("Hint".into()), tag: None });
    let resolved =
        resolve(Some(&ContainerOptions { enabled: Some(true), types: Some(types) })).unwrap();
    assert_eq!(
        resolved.types.get("tip"),
        Some(&ResolvedContainerType {
            name: "tip".into(),
            title: "Hint".into(),
            kind: super::ContainerKind::Div,
        })
    );
}
