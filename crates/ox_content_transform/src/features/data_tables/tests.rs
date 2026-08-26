use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::{MissingMode, resolve, transform};
use crate::transformer::MarkdownTransformer;
use crate::{ContainerOptions, DataTableOptions, TransformOptions};

struct TempRoot {
    path: PathBuf,
}

impl TempRoot {
    fn new(label: &str) -> Self {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let path = std::env::temp_dir()
            .join(format!("ox-content-data-tables-{label}-{}-{nanos}", std::process::id()));
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    fn write(&self, rel: &str, contents: &str) -> PathBuf {
        let path = self.path.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&path, contents).unwrap();
        path
    }
}

impl Drop for TempRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn options(root: Option<&Path>, missing: Option<&str>) -> TransformOptions {
    TransformOptions {
        gfm: Some(true),
        source_path: root.map(|path| path.join("page.md").to_string_lossy().into_owned()),
        data_tables: Some(DataTableOptions {
            enabled: Some(true),
            root_dir: root.map(|path| path.to_string_lossy().into_owned()),
            missing: missing.map(ToString::to_string),
        }),
        ..Default::default()
    }
}

fn html(source: &str, opts: TransformOptions) -> String {
    MarkdownTransformer::from_options(&opts).transform(source).html
}

fn csv_fence(title: &str, body: &str) -> String {
    format!("```csv-table title=\"{title}\"\n{body}\n```\n")
}

#[test]
fn resolve_is_none_when_omitted_or_false() {
    assert!(resolve(None, None).is_none());
    assert!(
        resolve(Some(&DataTableOptions { enabled: Some(false), ..Default::default() }), None)
            .is_none()
    );
}

#[test]
fn resolve_is_some_when_true_or_object() {
    assert!(
        resolve(Some(&DataTableOptions { enabled: Some(true), ..Default::default() }), None)
            .is_some()
    );
    assert!(
        resolve(Some(&DataTableOptions { enabled: None, ..Default::default() }), None).is_some()
    );
}

#[test]
fn disabled_by_default() {
    let source = csv_fence("Options", "Option,Type,Default\nhighlight,boolean,false");
    let result = MarkdownTransformer::from_options(&TransformOptions::default()).transform(&source);
    assert!(result.errors.is_empty(), "{:?}", result.errors);
    assert!(!result.html.contains("ox-data-table"), "{html}", html = result.html);
    assert!(
        result.html.contains("csv-table")
            || result.html.contains("<pre")
            || result.html.contains("<code"),
        "{html}",
        html = result.html
    );
}

#[test]
fn explicit_false_leaves_fence() {
    let html = html(
        &csv_fence("Options", "Option,Type\nhighlight,boolean"),
        TransformOptions {
            data_tables: Some(DataTableOptions { enabled: Some(false), ..Default::default() }),
            ..Default::default()
        },
    );
    assert!(!html.contains("ox-data-table"), "{html}");
}

#[test]
fn inline_csv_renders_semantic_table() {
    let html = html(
        &csv_fence("Options", "Option,Type,Default\nhighlight,boolean,false"),
        options(None, None),
    );
    assert!(html.contains(r#"<div class="ox-data-table">"#), "{html}");
    assert!(html.contains(r#"class="ox-data-table__scroll""#), "{html}");
    assert!(html.contains("<table"), "{html}");
    assert!(html.contains("<caption"), "{html}");
    assert!(html.contains("Options"), "{html}");
    assert!(html.contains("<th"), "{html}");
    assert!(html.contains("highlight"), "{html}");
    assert!(!html.contains("```"), "{html}");
    assert!(!html.contains("language-csv-table"), "{html}");
}

#[test]
fn inline_json_objects_and_headers_rows() {
    let objects =
        "```json-table title=\"Objects\"\n[{\"Option\":\"highlight\",\"Type\":\"boolean\"}]\n```\n";
    let objects_html = html(objects, options(None, None));
    assert!(objects_html.contains("ox-data-table"), "{objects_html}");
    assert!(objects_html.contains("highlight"), "{objects_html}");
    assert!(objects_html.contains("boolean"), "{objects_html}");

    let shaped = "```json-table\n{\"headers\":[\"A\",\"B\"],\"rows\":[[\"1\",\"2\"]]}\n```\n";
    let shaped_html = html(shaped, options(None, None));
    assert!(shaped_html.contains("<th"), "{shaped_html}");
    assert!(shaped_html.contains(">1<"), "{shaped_html}");
    assert!(shaped_html.contains(">2<"), "{shaped_html}");
}

#[test]
fn hostile_cells_are_escaped() {
    let html =
        html(&csv_fence("X", "Name,Note\n<script>alert(1)</script>,a&b"), options(None, None));
    assert!(html.contains("ox-data-table"), "{html}");
    assert!(!html.contains("<script>"), "{html}");
    assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"), "{html}");
    assert!(html.contains("a&amp;b"), "{html}");
}

#[test]
fn skips_nested_or_indented_fence() {
    let nested = "````md\n```csv-table\nA,B\n1,2\n```\n````\n";
    let nested_html = html(nested, options(None, None));
    assert!(!nested_html.contains("ox-data-table"), "{nested_html}");

    let indented = "    ```csv-table\n    A,B\n    1,2\n    ```\n";
    let indented_html = html(indented, options(None, None));
    assert!(!indented_html.contains("ox-data-table"), "{indented_html}");
}

#[test]
fn malformed_csv_and_json_are_actionable() {
    let csv = MarkdownTransformer::from_options(&options(None, None))
        .transform("```csv-table\n\"unclosed\n```\n");
    assert_eq!(csv.errors.len(), 1, "{:?}", csv.errors);
    assert!(csv.errors[0].contains("unclosed quote"), "{:?}", csv.errors);
    assert!(!csv.html.contains("ox-data-table"), "{html}", html = csv.html);

    let json = MarkdownTransformer::from_options(&options(None, None))
        .transform("```json-table\n{not json}\n```\n");
    assert_eq!(json.errors.len(), 1, "{:?}", json.errors);
    assert!(json.errors[0].contains("not valid JSON"), "{:?}", json.errors);
}

#[test]
fn parent_dir_escape_is_rejected() {
    let root = TempRoot::new("escape");
    root.write("page.md", "");
    let result = MarkdownTransformer::from_options(&options(Some(&root.path), None))
        .transform("```csv-table\n../../etc/passwd\n```\n");
    assert_eq!(result.errors.len(), 1, "{:?}", result.errors);
    assert!(result.errors[0].contains(".."), "{:?}", result.errors);
    assert!(!result.html.contains("ox-data-table"), "{html}", html = result.html);
}

#[test]
fn missing_external_file_error_and_warn() {
    let root = TempRoot::new("missing");
    root.write("page.md", "");
    let source = "```csv-table src=\"./gone.csv\"\n```\n";
    let error = MarkdownTransformer::from_options(&options(Some(&root.path), Some("error")))
        .transform(source);
    assert_eq!(error.errors.len(), 1, "{:?}", error.errors);
    assert!(error.errors[0].contains("could not be resolved"), "{:?}", error.errors);

    let warn = MarkdownTransformer::from_options(&options(Some(&root.path), Some("warn")))
        .transform(source);
    assert!(warn.errors.is_empty(), "{:?}", warn.errors);
    assert!(!warn.html.contains("ox-data-table"), "{html}", html = warn.html);
}

#[test]
fn imports_csv_and_json_files() {
    let root = TempRoot::new("import");
    root.write("data/options.csv", "Option,Type,Default\nhighlight,boolean,false\n");
    root.write("options.json", "[{\"Name\":\"alpha\",\"Count\":2}]\n");
    root.write("page.md", "");
    let csv = html(
        "```csv-table src=\"@/data/options.csv\" title=\"From CSV\"\n```\n",
        options(Some(&root.path), None),
    );
    assert!(csv.contains("From CSV"), "{csv}");
    assert!(csv.contains("highlight"), "{csv}");
    assert!(csv.contains("boolean"), "{csv}");

    let json = html("```json-table\n./options.json\n```\n", options(Some(&root.path), None));
    assert!(json.contains("alpha"), "{json}");
    assert!(json.contains(">2<"), "{json}");
}

#[test]
fn containers_do_not_steal_data_table_fences() {
    let source = "::: tip\n```csv-table\nA,B\n1,2\n```\n:::\n";
    let html = html(
        source,
        TransformOptions {
            gfm: Some(true),
            containers: Some(ContainerOptions { enabled: Some(true), types: None }),
            data_tables: Some(DataTableOptions { enabled: Some(true), ..Default::default() }),
            ..Default::default()
        },
    );
    assert!(html.contains("ox-container--tip"), "{html}");
    assert!(html.contains("ox-data-table"), "{html}");
    assert!(html.contains(">1<"), "{html}");
}

#[test]
fn preprocess_snapshot_html() {
    let resolved =
        resolve(Some(&DataTableOptions { enabled: Some(true), ..Default::default() }), None)
            .expect("enabled");
    assert_eq!(resolved.missing, MissingMode::Error);
    let mut errors = Vec::new();
    let source = transform(
        &csv_fence("Options", "Option,Type,Default\nhighlight,boolean,false"),
        &resolved,
        &mut errors,
    );
    assert!(errors.is_empty(), "{errors:?}");
    insta::assert_snapshot!(source);
}

#[test]
fn quoted_csv_fields_keep_commas() {
    let html = html(&csv_fence("Q", "Name,Note\n\"a,b\",\"say \"\"hi\"\"\""), options(None, None));
    assert!(html.contains("a,b"), "{html}");
    assert!(html.contains("say &quot;hi&quot;") || html.contains("say \"hi\""), "{html}");
}
