use super::resolve;
use crate::features::{TransformFeatureOptions, preprocess_markdown};
use crate::transformer::MarkdownTransformer;
use crate::{ContainerOptions, StepsOptions, TransformOptions};

fn steps_on() -> TransformOptions {
    TransformOptions {
        gfm: Some(true),
        steps: Some(StepsOptions { enabled: Some(true) }),
        ..Default::default()
    }
}

fn both_on() -> TransformOptions {
    TransformOptions {
        gfm: Some(true),
        containers: Some(ContainerOptions { enabled: Some(true), types: None }),
        steps: Some(StepsOptions { enabled: Some(true) }),
        ..Default::default()
    }
}

fn transform_html(source: &str, options: TransformOptions) -> String {
    MarkdownTransformer::from_options(&options).transform(source).html
}

fn preprocess(source: &str, options: &TransformOptions) -> String {
    preprocess_markdown(source, &TransformFeatureOptions::from_options(options)).source.into_owned()
}

const TWO_STEPS: &str = "::: steps\n\
1. Install the CLI\n\
\n\
   ```sh\n\
   npm i -g ox-content\n\
   ```\n\
2. Run **build**\n\
:::\n";

#[test]
fn resolve_is_none_when_option_is_omitted() {
    assert!(resolve(None).is_none());
}

#[test]
fn resolve_is_none_when_explicitly_disabled() {
    assert!(resolve(Some(&StepsOptions { enabled: Some(false) })).is_none());
}

#[test]
fn resolve_is_some_when_enabled_is_none_but_object_is_present() {
    assert!(resolve(Some(&StepsOptions { enabled: None })).is_some());
}

#[test]
fn disabled_by_default() {
    let html = transform_html(TWO_STEPS, TransformOptions::default());
    assert!(!html.contains("ox-steps"), "default transform must not emit steps:\n{html}");
    assert!(html.contains("::: steps") || html.contains("Install the CLI"), "{html}");
}

#[test]
fn disabled_object_leaves_steps_literal() {
    let html = transform_html(
        TWO_STEPS,
        TransformOptions {
            steps: Some(StepsOptions { enabled: Some(false) }),
            ..Default::default()
        },
    );
    assert!(!html.contains("ox-steps"), "{html}");
}

#[test]
fn happy_path_two_steps() {
    let html = transform_html(TWO_STEPS, steps_on());
    assert!(html.contains(r#"<div class="ox-steps">"#), "{html}");
    assert!(html.contains(r#"<ol class="ox-steps__list">"#), "{html}");
    assert!(html.contains(r#"<li class="ox-steps__item">"#), "{html}");
    assert!(html.matches(r#"<li class="ox-steps__item">"#).count() >= 2, "{html}");
    assert!(html.contains("Install the CLI"), "{html}");
    assert!(html.contains("npm i -g ox-content"), "{html}");
    assert!(html.contains("<strong>build</strong>"), "{html}");
    assert!(!html.contains(":::"), "markers must be consumed:\n{html}");
}

#[test]
fn nested_fence_inside_step() {
    let html = transform_html(TWO_STEPS, steps_on());
    assert!(html.contains("<pre") || html.contains("<code"), "fence must render:\n{html}");
    assert!(html.contains("npm i -g ox-content"), "{html}");
    assert!(!html.contains(":::"), "{html}");
}

#[test]
fn nested_markdown_emphasis() {
    let html = transform_html("::: steps\n1. Run **build**\n:::\n", steps_on());
    assert!(html.contains("<strong>build</strong>"), "{html}");
    assert!(html.contains("ox-steps__item"), "{html}");
}

#[test]
fn unclosed_stays_literal() {
    let source = "::: steps\n1. Install the CLI\n\n# Rest of the file\n";
    let html = transform_html(source, steps_on());
    assert!(!html.contains("ox-steps"), "unclosed opener must not wrap the file:\n{html}");
    assert!(html.contains("::: steps") || html.contains("Install the CLI"), "{html}");
    assert!(html.contains("Rest of the file"), "{html}");
}

#[test]
fn hostile_text_escaped() {
    let html = transform_html("::: steps\n1. <script>alert(1)</script>\n:::\n", steps_on());
    assert!(!html.contains("<script>"), "raw script must not be emitted:\n{html}");
    assert!(
        html.contains("&lt;script&gt;") || html.contains("&#x3C;script") || html.contains("&#60;"),
        "script text must be escaped:\n{html}"
    );
    assert!(html.contains("ox-steps"), "{html}");
}

#[test]
fn preamble_before_first_marker_is_preserved() {
    let html =
        transform_html("::: steps\nFollow these steps:\n\n1. First\n2. Second\n:::\n", steps_on());
    assert!(html.contains("Follow these steps:"), "leading prose must stay:\n{html}");
    assert!(html.contains("ox-steps"), "{html}");
    assert_eq!(html.matches(r#"<li class="ox-steps__item">"#).count(), 2, "{html}");
    assert!(html.contains("First"), "{html}");
    assert!(html.contains("Second"), "{html}");
    let preamble_at = html.find("Follow these steps:").unwrap_or_else(|| panic!("{html}"));
    let list_at = html.find(r#"<ol class="ox-steps__list">"#).unwrap_or_else(|| panic!("{html}"));
    assert!(preamble_at < list_at, "preamble must appear before the list:\n{html}");
}

#[test]
fn ordinary_ordered_list_unchanged() {
    let html = transform_html("1. foo\n2. bar\n", steps_on());
    assert!(!html.contains("ox-steps"), "{html}");
    assert!(html.contains("<ol>") || html.contains("<li>"), "{html}");
    assert!(html.contains("foo"), "{html}");
}

#[test]
fn extra_colons_and_indent_are_recognized() {
    let html = transform_html("   :::: steps\n1. One\n   ::::\n", steps_on());
    assert!(html.contains("ox-steps"), "{html}");
    assert!(html.contains("One"), "{html}");
}

#[test]
fn four_space_indent_is_not_a_steps_opener() {
    let html = transform_html("    ::: steps\n    1. One\n    :::\n", steps_on());
    assert!(!html.contains("ox-steps"), "{html}");
}

#[test]
fn fenced_code_is_not_rewritten() {
    let html = transform_html("```md\n::: steps\n1. One\n:::\n```\n", steps_on());
    assert!(!html.contains("ox-steps"), "{html}");
    assert!(html.contains("::: steps"), "{html}");
}

#[test]
fn nested_list_stays_inside_the_step() {
    let html = transform_html(
        "::: steps\n1. First\n   - nested\n   - items\n2. Second\n:::\n",
        steps_on(),
    );
    assert_eq!(html.matches(r#"<li class="ox-steps__item">"#).count(), 2, "{html}");
    assert!(html.contains("<ul>"), "{html}");
    assert!(html.contains("nested"), "{html}");
}

#[test]
fn steps_wins_when_containers_are_also_on() {
    let html = transform_html("::: steps\n1. Only a step\n:::\n", both_on());
    assert!(html.contains("ox-steps"), "{html}");
    assert!(!html.contains("ox-container"), "{html}");
}

#[test]
fn containers_alone_leave_steps_literal() {
    let html = transform_html(
        "::: steps\n1. Not a container\n:::\n",
        TransformOptions {
            gfm: Some(true),
            containers: Some(ContainerOptions { enabled: Some(true), types: None }),
            ..Default::default()
        },
    );
    assert!(!html.contains("ox-steps"), "{html}");
    assert!(!html.contains("ox-container--steps"), "{html}");
}

#[test]
fn later_closed_block_is_not_swallowed_by_an_unclosed_opener() {
    let source = "::: steps\n1. Incomplete\n\n::: steps\n1. Complete\n:::\n";
    let html = transform_html(source, steps_on());
    assert!(html.contains("ox-steps"), "{html}");
    assert!(html.contains("Complete"), "{html}");
    assert!(html.contains("Incomplete"), "{html}");
}

#[test]
fn preprocess_emits_blank_lines_so_inner_markdown_parses() {
    let source = preprocess("::: steps\n1. Run **build**\n:::\n", &steps_on());
    assert!(source.contains("<div class=\"ox-steps\">"), "{source}");
    assert!(source.contains("<li class=\"ox-steps__item\">"), "{source}");
    assert!(source.contains("Run **build**"), "{source}");
    assert!(!source.contains("1. Run"), "{source}");
}
