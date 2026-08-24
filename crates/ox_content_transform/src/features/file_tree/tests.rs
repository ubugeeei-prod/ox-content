use super::{resolve, transform};
use crate::transformer::MarkdownTransformer;
use crate::{ContainerOptions, FileTreeOptions, TransformOptions};

fn file_tree_on() -> TransformOptions {
    TransformOptions {
        gfm: Some(true),
        file_tree: Some(FileTreeOptions { enabled: Some(true) }),
        ..Default::default()
    }
}

fn transform_html(source: &str, options: TransformOptions) -> String {
    MarkdownTransformer::from_options(&options).transform(source).html
}

fn fence(body: &str) -> String {
    format!("```file-tree\n{body}\n```\n")
}

#[test]
fn resolve_is_none_when_omitted_or_false() {
    assert!(resolve(None).is_none());
    assert!(resolve(Some(&FileTreeOptions { enabled: Some(false) })).is_none());
}

#[test]
fn resolve_is_some_when_true_or_object() {
    assert!(resolve(Some(&FileTreeOptions { enabled: Some(true) })).is_some());
    assert!(resolve(Some(&FileTreeOptions { enabled: None })).is_some());
}

#[test]
fn disabled_by_default() {
    let html = transform_html(&fence("- src/\n  - index.ts"), TransformOptions::default());
    assert!(!html.contains("ox-file-tree"), "default transform must not emit a tree:\n{html}");
    assert!(
        html.contains("file-tree") || html.contains("<pre") || html.contains("<code"),
        "fence must stay a normal code block:\n{html}"
    );
}

#[test]
fn explicit_false_leaves_fence_literal() {
    let html = transform_html(
        &fence("- src/"),
        TransformOptions {
            file_tree: Some(FileTreeOptions { enabled: Some(false) }),
            ..Default::default()
        },
    );
    assert!(!html.contains("ox-file-tree"), "{html}");
}

#[test]
fn happy_path_nested_dirs_and_files() {
    let html = transform_html(
        &fence("- src/\n  - index.ts\n  - lib/\n    - util.ts\n- README.md"),
        file_tree_on(),
    );
    assert!(html.contains(r#"<div class="ox-file-tree">"#), "{html}");
    assert!(html.contains(r#"class="ox-file-tree__dir""#), "{html}");
    assert!(html.contains(r#"class="ox-file-tree__file""#), "{html}");
    assert!(html.contains("src/"), "{html}");
    assert!(html.contains("index.ts"), "{html}");
    assert!(html.contains("lib/"), "{html}");
    assert!(html.contains("util.ts"), "{html}");
    assert!(html.contains("README.md"), "{html}");
    assert!(html.contains("<ul>"), "{html}");
    assert!(!html.contains("```"), "fence markers must be consumed:\n{html}");
    assert!(!html.contains("language-file-tree"), "{html}");
}

#[test]
fn highlight_marker() {
    let html = transform_html(&fence("- index.ts **\n- **util.ts**\n- **src/**"), file_tree_on());
    assert!(html.contains("ox-file-tree__highlight"), "{html}");
    assert!(html.contains("index.ts"), "{html}");
    assert!(html.contains("util.ts"), "{html}");
    assert!(html.contains("src/"), "{html}");
    assert!(!html.contains("**"), "highlight markers must be stripped:\n{html}");
    assert!(html.contains(r#"class="ox-file-tree__file ox-file-tree__highlight""#), "{html}");
    assert!(html.contains(r#"class="ox-file-tree__dir ox-file-tree__highlight""#), "{html}");
}

#[test]
fn placeholder_ellipsis() {
    let html = transform_html(&fence("- …\n- ..."), file_tree_on());
    assert!(html.contains("ox-file-tree"), "{html}");
    assert!(html.contains('…') || html.contains("&hellip;"), "{html}");
    assert!(html.contains("..."), "{html}");
    assert!(!html.contains("<script"), "{html}");
}

#[test]
fn skips_indented_or_nested_fence() {
    let nested = "````md\n```file-tree\n- src/\n```\n````\n";
    let nested_html = transform_html(nested, file_tree_on());
    assert!(!nested_html.contains("ox-file-tree"), "nested fence:\n{nested_html}");
    assert!(nested_html.contains("file-tree") || nested_html.contains("src/"), "{nested_html}");

    let indented = "    ```file-tree\n    - src/\n    ```\n";
    let indented_html = transform_html(indented, file_tree_on());
    assert!(!indented_html.contains("ox-file-tree"), "indented fence:\n{indented_html}");

    let inline = "Use ` ```file-tree ` in docs.\n";
    let inline_html = transform_html(inline, file_tree_on());
    assert!(!inline_html.contains("ox-file-tree"), "inline code:\n{inline_html}");
}

#[test]
fn hostile_name_escaped() {
    let html = transform_html(&fence("- <script>alert(1)</script>\n- a&b"), file_tree_on());
    assert!(html.contains("ox-file-tree"), "{html}");
    assert!(!html.contains("<script>"), "{html}");
    assert!(!html.contains("</script>"), "{html}");
    assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"), "{html}");
    assert!(html.contains("a&amp;b"), "{html}");
}

#[test]
fn does_not_read_filesystem() {
    let result = MarkdownTransformer::from_options(&file_tree_on())
        .transform("```file-tree\n- ../../etc/passwd\n```\n");
    assert!(result.errors.is_empty(), "{:?}", result.errors);
    assert!(result.html.contains("ox-file-tree"), "{html}", html = result.html);
    assert!(result.html.contains("../../etc/passwd"), "{html}", html = result.html);
    assert!(
        !result.html.contains("root:") && !result.html.contains("/bin/"),
        "path names must stay literal text:\n{}",
        result.html
    );
}

#[test]
fn ignores_blank_lines_and_still_nests() {
    let html = transform_html(&fence("- src/\n\n  - index.ts\n\n"), file_tree_on());
    assert!(html.contains("ox-file-tree__dir"), "{html}");
    assert!(html.contains("ox-file-tree__file"), "{html}");
    assert!(html.contains("index.ts"), "{html}");
}

#[test]
fn containers_do_not_steal_file_tree_fences() {
    let source = "::: tip\n```file-tree\n- src/index.ts\n```\n:::\n";
    let html = transform_html(
        source,
        TransformOptions {
            gfm: Some(true),
            containers: Some(ContainerOptions { enabled: Some(true), types: None }),
            file_tree: Some(FileTreeOptions { enabled: Some(true) }),
            ..Default::default()
        },
    );
    assert!(html.contains("ox-container--tip"), "{html}");
    assert!(html.contains("ox-file-tree"), "{html}");
    assert!(html.contains("src/index.ts"), "{html}");
}

#[test]
fn preprocess_emits_static_tree_html() {
    let source = transform(&fence("- src/\n  - index.ts **"), super::ResolvedFileTreeOptions);
    assert!(source.contains(r#"<div class="ox-file-tree">"#), "{source}");
    assert!(source.contains(r#"class="ox-file-tree__dir""#), "{source}");
    assert!(source.contains(r#"class="ox-file-tree__file ox-file-tree__highlight""#), "{source}");
    assert!(!source.contains("```"), "{source}");
}
