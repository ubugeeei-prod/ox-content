use super::{ReportMode, resolve, transform};
use crate::transformer::MarkdownTransformer;
use crate::{ContainerOptions, ContainerTypeOptions, TimelineOptions, TransformOptions};

fn timeline_options(overrides: TimelineOptions) -> TransformOptions {
    TransformOptions {
        gfm: Some(true),
        timelines: Some(TimelineOptions { enabled: Some(true), ..overrides }),
        ..Default::default()
    }
}

fn html(source: &str, options: TransformOptions) -> crate::TransformResult {
    MarkdownTransformer::from_options(&options).transform(source)
}

#[test]
fn resolve_is_none_when_omitted_or_disabled() {
    assert!(resolve(None).is_none());
    assert!(
        resolve(Some(&TimelineOptions { enabled: Some(false), ..Default::default() })).is_none()
    );
}

#[test]
fn resolve_defaults_to_ordered_strict_diagnostics() {
    let resolved =
        resolve(Some(&TimelineOptions { enabled: None, ..Default::default() })).expect("enabled");
    assert!(resolved.ordered);
    assert_eq!(resolved.invalid_date, ReportMode::Error);
    assert_eq!(resolved.unknown_meta, ReportMode::Error);
    assert_eq!(resolved.empty, ReportMode::Error);
}

#[test]
fn disabled_by_default_keeps_timeline_literal() {
    let result = html("::: timeline\n- 2026-08-26 Release\n:::\n", TransformOptions::default());
    assert!(result.errors.is_empty(), "{:?}", result.errors);
    assert!(!result.html.contains("ox-timeline"), "{html}", html = result.html);
    assert!(result.html.contains("timeline"), "{html}", html = result.html);
}

#[test]
fn renders_semantic_static_timeline() {
    let result = html(
        "::: timeline title=\"Release history\"\n- 2026-08-26 RC cut {status=done label=\"RC\" href=\"/releases/rc\"}\n  Shipped **candidate** builds.\n- [2026-09] GA window {status=planned}\n:::\n",
        timeline_options(TimelineOptions::default()),
    );
    assert!(result.errors.is_empty(), "{:?}", result.errors);
    assert!(
        result.html.contains(r#"<section class="ox-timeline" aria-label="Timeline">"#),
        "{html}",
        html = result.html
    );
    assert!(
        result.html.contains(r#"<ol class="ox-timeline__items">"#),
        "{html}",
        html = result.html
    );
    assert!(
        result
            .html
            .contains(r#"<time class="ox-timeline__date" datetime="2026-08-26">2026-08-26</time>"#),
        "{html}",
        html = result.html
    );
    assert!(result.html.contains(r#"data-status="done""#), "{html}", html = result.html);
    assert!(
        result.html.contains(r#"<span class="ox-timeline__label">RC</span>"#),
        "{html}",
        html = result.html
    );
    assert!(
        result.html.contains(r#"<a href="/releases/rc">RC cut</a>"#),
        "{html}",
        html = result.html
    );
    assert!(result.html.contains("<strong>candidate</strong>"), "{html}", html = result.html);
    assert!(!result.html.contains("<script"), "{html}", html = result.html);
}

#[test]
fn opener_can_select_unordered_list() {
    let result = html(
        "::: timeline ordered=false\n- 2026-08-26 Done\n:::\n",
        timeline_options(TimelineOptions::default()),
    );
    assert!(result.errors.is_empty(), "{:?}", result.errors);
    assert!(
        result.html.contains(r#"<ul class="ox-timeline__items">"#),
        "{html}",
        html = result.html
    );
}

#[test]
fn invalid_date_errors_preserve_source() {
    let result = html(
        "::: timeline\n- 2026-02-31 Impossible\n:::\n",
        timeline_options(TimelineOptions::default()),
    );
    assert_eq!(result.errors.len(), 1, "{:?}", result.errors);
    assert!(result.errors[0].contains("invalid date"), "{:?}", result.errors);
    assert!(!result.html.contains("ox-timeline"), "{html}", html = result.html);
    assert!(result.html.contains("timeline"), "{html}", html = result.html);
}

#[test]
fn invalid_date_can_warn_and_render_without_datetime() {
    let result = html(
        "::: timeline\n- 2026-02-31 Impossible\n:::\n",
        timeline_options(TimelineOptions {
            invalid_date: Some("warn".to_string()),
            ..Default::default()
        }),
    );
    assert_eq!(result.errors.len(), 1, "{:?}", result.errors);
    assert!(
        result.html.contains(
            r#"<span class="ox-timeline__date ox-timeline__date--invalid">2026-02-31</span>"#
        ),
        "{html}",
        html = result.html
    );
}

#[test]
fn unknown_metadata_is_actionable() {
    let result = html(
        "::: timeline\n- 2026-08-26 Release {mood=happy}\n:::\n",
        timeline_options(TimelineOptions::default()),
    );
    assert_eq!(result.errors.len(), 1, "{:?}", result.errors);
    assert!(result.errors[0].contains("unsupported metadata"), "{:?}", result.errors);
    assert!(!result.html.contains("ox-timeline"), "{html}", html = result.html);
}

#[test]
fn empty_timeline_is_diagnostic() {
    let result = html("::: timeline\n\n:::\n", timeline_options(TimelineOptions::default()));
    assert_eq!(result.errors.len(), 1, "{:?}", result.errors);
    assert!(result.errors[0].contains("empty"), "{:?}", result.errors);
}

#[test]
fn skips_fenced_and_indented_timeline_markers() {
    let fenced = html(
        "```md\n::: timeline\n- 2026-08-26 Release\n:::\n```\n",
        timeline_options(TimelineOptions::default()),
    );
    assert!(!fenced.html.contains("ox-timeline"), "{html}", html = fenced.html);

    let indented = html(
        "    ::: timeline\n    - 2026-08-26 Release\n    :::\n",
        timeline_options(TimelineOptions::default()),
    );
    assert!(!indented.html.contains("ox-timeline"), "{html}", html = indented.html);
}

#[test]
fn timeline_precedes_custom_container_with_same_name() {
    let mut types = rustc_hash::FxHashMap::default();
    types.insert(
        "timeline".to_string(),
        ContainerTypeOptions { title: Some("Container".to_string()), tag: None },
    );
    let result = html(
        "::: timeline\n- 2026-08-26 Release\n:::\n",
        TransformOptions {
            gfm: Some(true),
            containers: Some(ContainerOptions { enabled: Some(true), types: Some(types) }),
            timelines: Some(TimelineOptions { enabled: Some(true), ..Default::default() }),
            ..Default::default()
        },
    );
    assert!(result.html.contains("ox-timeline"), "{html}", html = result.html);
    assert!(!result.html.contains("ox-container--timeline"), "{html}", html = result.html);
}

#[test]
fn preprocess_snapshot_html() {
    let resolved = resolve(Some(&TimelineOptions { enabled: Some(true), ..Default::default() }))
        .expect("enabled");
    let mut errors = Vec::new();
    let source = transform(
        "::: timeline \"Roadmap\"\n- 2026-08-26 Start {status=done}\n- 2026-09 Next\n:::\n",
        &resolved,
        &mut errors,
    );
    assert!(errors.is_empty(), "{errors:?}");
    insta::assert_snapshot!(source);
}
