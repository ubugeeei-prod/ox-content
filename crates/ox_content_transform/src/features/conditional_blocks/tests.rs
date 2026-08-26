use rustc_hash::FxHashMap;
use serde_json::{Value, json};

use super::{resolve, transform};
use crate::transformer::MarkdownTransformer;
use crate::{ConditionalBlockOptions, TransformOptions};

fn conditional_options(values: FxHashMap<String, Value>) -> TransformOptions {
    TransformOptions {
        gfm: Some(true),
        conditional_blocks: Some(ConditionalBlockOptions {
            enabled: Some(true),
            values: Some(values),
        }),
        ..Default::default()
    }
}

fn map(entries: &[(&str, Value)]) -> FxHashMap<String, Value> {
    entries.iter().map(|(key, value)| ((*key).to_string(), value.clone())).collect()
}

fn html(source: &str, options: TransformOptions) -> crate::TransformResult {
    MarkdownTransformer::from_options(&options).transform(source)
}

#[test]
fn resolve_is_none_when_omitted_or_disabled() {
    assert!(resolve(None).is_none());
    assert!(
        resolve(Some(&ConditionalBlockOptions { enabled: Some(false), values: None })).is_none()
    );
}

#[test]
fn disabled_by_default_keeps_blocks_literal() {
    let result = html(
        "::: if runtime == \"node\"\nNode\n::: else\nBrowser\n:::\n",
        TransformOptions::default(),
    );
    assert!(result.errors.is_empty(), "{:?}", result.errors);
    assert!(result.html.contains("runtime"), "{html}", html = result.html);
    assert!(result.html.contains("Node"), "{html}", html = result.html);
    assert!(result.html.contains("Browser"), "{html}", html = result.html);
}

#[test]
fn selected_branch_is_the_only_rendered_and_indexed_content() {
    let result = html(
        "# Always\n\n::: if runtime == \"node\"\n## Node only\nSelected.\n::: else\n## Browser only\nHidden.\n:::\n",
        conditional_options(map(&[("runtime", json!("node"))])),
    );
    assert!(result.errors.is_empty(), "{:?}", result.errors);
    assert!(result.html.contains("Node only"), "{html}", html = result.html);
    assert!(!result.html.contains("Browser only"), "{html}", html = result.html);
    assert!(toc_contains(&result.toc, "Always"));
    assert!(toc_contains(&result.toc, "Node only"));
    assert!(!toc_contains(&result.toc, "Browser only"));
}

#[test]
fn frontmatter_values_override_bare_config_identifiers() {
    let result = html(
        "---\nruntime: browser\n---\n::: if runtime == \"browser\"\nFrontmatter wins.\n::: else if config.runtime == \"node\"\nConfig path wins.\n:::\n",
        conditional_options(map(&[("runtime", json!("node"))])),
    );
    assert!(result.errors.is_empty(), "{:?}", result.errors);
    assert!(result.html.contains("Frontmatter wins."), "{html}", html = result.html);
    assert!(!result.html.contains("Config path wins."), "{html}", html = result.html);
}

#[test]
fn evaluates_in_and_boolean_operators() {
    let result = html(
        "::: if runtime in [\"node\", \"deno\"] and experimental == true\nRunnable.\n::: else\nSkipped.\n:::\n",
        conditional_options(map(&[("runtime", json!("deno")), ("experimental", json!(true))])),
    );
    assert!(result.errors.is_empty(), "{:?}", result.errors);
    assert!(result.html.contains("Runnable."), "{html}", html = result.html);
    assert!(!result.html.contains("Skipped."), "{html}", html = result.html);
}

#[test]
fn nested_conditionals_only_evaluate_selected_outer_branch() {
    let result = html(
        "::: if runtime == \"node\"\nOuter.\n::: if tier == \"edge\"\nEdge.\n::: else\nServer.\n:::\n::: else\nBrowser.\n:::\n",
        conditional_options(map(&[("runtime", json!("node")), ("tier", json!("server"))])),
    );
    assert!(result.errors.is_empty(), "{:?}", result.errors);
    assert!(result.html.contains("Outer."), "{html}", html = result.html);
    assert!(result.html.contains("Server."), "{html}", html = result.html);
    assert!(!result.html.contains("Edge."), "{html}", html = result.html);
    assert!(!result.html.contains("Browser."), "{html}", html = result.html);
}

#[test]
fn malformed_conditions_report_diagnostics_and_can_fall_through_to_else() {
    let result = html(
        "::: if runtime = \"node\"\nBroken.\n::: else\nFallback.\n:::\n",
        conditional_options(map(&[("runtime", json!("node"))])),
    );
    assert_eq!(result.errors.len(), 1, "{:?}", result.errors);
    assert!(result.errors[0].contains("could not be evaluated"), "{:?}", result.errors);
    assert!(result.html.contains("Fallback."), "{html}", html = result.html);
    assert!(!result.html.contains("Broken."), "{html}", html = result.html);
}

#[test]
fn skips_fenced_conditional_markers() {
    let result = html(
        "```md\n::: if runtime == \"node\"\nStill literal.\n:::\n```\n",
        conditional_options(map(&[("runtime", json!("node"))])),
    );
    assert!(result.errors.is_empty(), "{:?}", result.errors);
    assert!(result.html.contains("::: if runtime"), "{html}", html = result.html);
    assert!(result.html.contains("Still literal."), "{html}", html = result.html);
}

#[test]
fn unclosed_blocks_stay_literal_with_a_diagnostic() {
    let resolved = resolve(Some(&ConditionalBlockOptions {
        enabled: Some(true),
        values: Some(map(&[("runtime", json!("node"))])),
    }))
    .expect("enabled");
    let mut errors = Vec::new();
    let source = transform(
        "::: if runtime == \"node\"\nUnclosed.\n",
        &resolved,
        &FxHashMap::default(),
        &mut errors,
    );
    assert_eq!(errors, vec!["Conditional block is missing a closing ::: fence."]);
    assert!(source.contains("::: if runtime"), "{source}");
    assert!(source.contains("Unclosed."), "{source}");
}

#[test]
fn selected_branch_preserves_crlf_line_endings() {
    let resolved = resolve(Some(&ConditionalBlockOptions {
        enabled: Some(true),
        values: Some(map(&[("runtime", json!("node"))])),
    }))
    .expect("enabled");
    let mut errors = Vec::new();
    let source = transform(
        "::: if runtime == \"node\"\r\nSelected.\r\n:::\r\n",
        &resolved,
        &FxHashMap::default(),
        &mut errors,
    );
    assert!(errors.is_empty(), "{errors:?}");
    assert_eq!(source, "Selected.\r\n");
}

fn toc_contains(entries: &[crate::TocEntry], text: &str) -> bool {
    entries.iter().any(|entry| entry.text == text || toc_contains(&entry.children, text))
}
