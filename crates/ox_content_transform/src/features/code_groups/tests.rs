use super::resolve;
use crate::features::{TransformFeatureOptions, preprocess_markdown};
use crate::tabs::transform_tabs;
use crate::transformer::MarkdownTransformer;
use crate::{CodeGroupOptions, ContainerOptions, TransformOptions};

fn groups_on() -> TransformOptions {
    TransformOptions {
        gfm: Some(true),
        code_groups: Some(CodeGroupOptions { enabled: Some(true) }),
        ..Default::default()
    }
}

fn transform_html(source: &str, options: TransformOptions) -> String {
    MarkdownTransformer::from_options(&options).transform(source).html
}

fn preprocess(source: &str, options: &TransformOptions) -> String {
    preprocess_markdown(source, &TransformFeatureOptions::from_options(options)).source.into_owned()
}

fn preprocess_result<'a>(
    source: &'a str,
    options: &TransformOptions,
) -> crate::features::PreprocessResult<'a> {
    preprocess_markdown(source, &TransformFeatureOptions::from_options(options))
}

const GROUPED: &str = "::: code-group\n\n```js [config.js]\nexport default {}\n```\n\n```ts [config.ts]\nexport default {}\n```\n\n:::\n";

#[test]
fn disabled_by_default() {
    let html = transform_html(GROUPED, TransformOptions::default());
    assert!(!html.contains("<tabs"), "default transform must not rewrite groups:\n{html}");
    assert!(!html.contains("ox-tabs"), "{html}");
    assert!(html.contains("::: code-group") || html.contains("export default"), "{html}");

    assert!(resolve(None).is_none());
    assert!(resolve(Some(&CodeGroupOptions { enabled: Some(false) })).is_none());
    assert!(resolve(Some(&CodeGroupOptions { enabled: None })).is_some());
    assert!(resolve(Some(&CodeGroupOptions { enabled: Some(true) })).is_some());
}

#[test]
fn enabled_rewrites_to_tabs_markup() {
    let source = preprocess(GROUPED, &groups_on());
    assert!(source.contains("<tabs>"), "{source}");
    assert!(source.contains(r#"<tab label="config.js">"#), "{source}");
    assert!(source.contains(r#"<tab label="config.ts">"#), "{source}");
    assert!(source.contains("```js [config.js]"), "{source}");
    assert!(!source.contains(":::"), "{source}");
}

#[test]
fn language_label_when_title_is_omitted() {
    let source = preprocess(
        "::: code-group\n```js\nconst a = 1\n```\n```ts\nconst a = 1\n```\n:::\n",
        &groups_on(),
    );
    assert!(source.contains(r#"<tab label="js">"#), "{source}");
    assert!(source.contains(r#"<tab label="ts">"#), "{source}");
}

#[test]
fn fence_meta_title_is_used() {
    let source = preprocess(
        "::: code-group\n```js title=\"app.js\"\nmodule.exports = {}\n```\n:::\n",
        &groups_on(),
    );
    assert!(source.contains(r#"<tab label="app.js">"#), "{source}");
}

#[test]
fn rendered_tabs_use_existing_widget_without_script() {
    let html = transform_html(GROUPED, groups_on());
    let widget = transform_tabs(&html, 0);
    assert!(widget.group_count >= 1, "{html}");
    assert!(widget.html.contains("ox-tabs"), "{html}\n{}", widget.html);
    assert!(widget.html.contains("<noscript>"), "{}", widget.html);
    assert!(!widget.html.contains("<script"), "must not ship extra JS:\n{}", widget.html);
    assert!(widget.html.contains("config.js"), "{}", widget.html);
    assert!(widget.html.contains("config.ts"), "{}", widget.html);
}

#[test]
fn skips_fenced_and_indented_code() {
    let fenced = transform_html("```md\n::: code-group\n```js\nx\n```\n:::\n```\n", groups_on());
    assert!(!fenced.contains("<tabs"), "{fenced}");
    assert!(fenced.contains("::: code-group"), "{fenced}");

    let indented =
        transform_html("    ::: code-group\n    ```js\n    x\n    ```\n    :::\n", groups_on());
    assert!(!indented.contains("<tabs"), "{indented}");
}

#[test]
fn unclosed_stays_literal() {
    let source = "::: code-group\n```js\nx\n```\n# Later heading\n";
    let html = transform_html(source, groups_on());
    assert!(!html.contains("<tabs"), "unclosed must not wrap the file:\n{html}");
    assert!(html.contains("Later heading"), "rest of file must not be swallowed:\n{html}");
}

#[test]
fn extra_content_degrades_to_ordinary_fences() {
    let result = preprocess_result("::: code-group\nHello\n```js\nx\n```\n:::\n", &groups_on());
    assert!(!result.source.contains("<tabs"), "{}", result.source);
    assert!(result.source.contains("```js"), "{}", result.source);
    assert!(!result.source.contains(":::"), "{}", result.source);
    assert!(result.errors.iter().any(|error| error.contains("non-fence")), "{:?}", result.errors);
}

#[test]
fn malformed_title_warns_and_falls_back() {
    let result = preprocess_result("::: code-group\n```js [config.js\nx\n```\n:::\n", &groups_on());
    assert!(result.source.contains("<tabs>"), "{}", result.source);
    assert!(result.source.contains(r#"<tab label="js">"#), "{}", result.source);
    assert!(result.errors.iter().any(|error| error.contains("malformed")), "{:?}", result.errors);
}

#[test]
fn hostile_label_is_escaped() {
    let source = preprocess(
        "::: code-group\n```js [<script>alert(1)</script>]\nx\n```\n:::\n",
        &groups_on(),
    );
    assert!(source.contains(r#"<tab label="&lt;script&gt;alert(1)&lt;/script&gt;">"#), "{source}");
    assert!(!source.contains(r#"<tab label="<script>"#), "{source}");
}

#[test]
fn disabled_object_leaves_markers_literal() {
    let html = transform_html(
        GROUPED,
        TransformOptions {
            code_groups: Some(CodeGroupOptions { enabled: Some(false) }),
            ..Default::default()
        },
    );
    assert!(!html.contains("<tabs"), "{html}");
}

#[test]
fn reserves_code_group_from_generic_containers() {
    let source = "::: code-group\n```js [a.js]\nx\n```\n:::\n::: tip\nHello\n:::\n";
    let options = TransformOptions {
        gfm: Some(true),
        code_groups: Some(CodeGroupOptions { enabled: Some(true) }),
        containers: Some(ContainerOptions { enabled: Some(true), types: None }),
        ..Default::default()
    };
    let html = transform_html(source, options);
    assert!(html.contains("<tabs") || html.contains("ox-tabs"), "{html}");
    assert!(html.contains("ox-container--tip"), "{html}");
    assert!(!html.contains("ox-container--code-group"), "{html}");
}

/// The pass as it was, with every opener searching for its own closer.
fn reference_transform(source: &str, errors: &mut Vec<String>) -> String {
    use super::parse::{
        parse_any_opener, parse_closer, parse_opener, split_ending, trim_container_indent,
    };
    use super::{collect_inner, emit_group};
    use crate::features::colon_blocks::tests::reference_closer;
    use crate::features::segments::{is_closing_fence, parse_opening_fence};

    let lines: Vec<(&str, &str)> = source.split_inclusive('\n').map(split_ending).collect();
    let mut out = String::new();
    let mut index = 0usize;
    let mut fence: Option<(u8, usize)> = None;
    while index < lines.len() {
        let (line, ending) = lines[index];
        if let Some((fence_char, fence_len)) = fence {
            out.push_str(line);
            out.push_str(ending);
            if is_closing_fence(line, fence_char, fence_len) {
                fence = None;
            }
            index += 1;
            continue;
        }
        if let Some(open) = parse_opening_fence(line) {
            fence = Some((open.fence_char, open.fence_len));
            out.push_str(line);
            out.push_str(ending);
            index += 1;
            continue;
        }
        let trimmed = trim_container_indent(line);
        if let Some(colon_count) = parse_opener(trimmed)
            && let Some(end) =
                reference_closer(&lines, index, colon_count, parse_any_opener, parse_closer)
        {
            emit_group(&mut out, &collect_inner(&lines, index + 1, end), errors);
            index = end + 1;
            continue;
        }
        out.push_str(line);
        out.push_str(ending);
        index += 1;
    }
    out
}

#[test]
fn rewrites_as_the_per_opener_search_did() {
    let mut random = crate::features::colon_blocks::tests::Random(0x2545_F491_4F6C_DD1D);
    for _ in 0..4000 {
        let source = random.document();
        let (mut errors, mut expected_errors) = (Vec::new(), Vec::new());
        let actual = super::transform(&source, &mut errors);
        let expected = reference_transform(&source, &mut expected_errors);
        assert_eq!(actual, expected, "for {source:?}");
        assert_eq!(errors, expected_errors, "for {source:?}");
    }
}
