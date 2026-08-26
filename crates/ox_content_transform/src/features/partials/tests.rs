use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::{resolve, transform};
use crate::features::{TransformFeatureOptions, preprocess_markdown};
use crate::transformer::MarkdownTransformer;
use crate::{PartialsOptions, TransformOptions};

struct TempRoot {
    path: PathBuf,
}

impl TempRoot {
    fn new(label: &str) -> Self {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let path = std::env::temp_dir()
            .join(format!("ox-content-partials-{label}-{}-{nanos}", std::process::id()));
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

fn enabled_options(root: &Path, source_path: Option<&Path>) -> TransformOptions {
    TransformOptions {
        gfm: Some(true),
        source_path: source_path.map(|path| path.to_string_lossy().into_owned()),
        partials: Some(PartialsOptions {
            enabled: Some(true),
            root_dir: Some(root.to_string_lossy().into_owned()),
            root: None,
            missing: None,
        }),
        ..Default::default()
    }
}

fn resolved(root: &Path, source_path: Option<&Path>) -> super::ResolvedPartials {
    resolve(
        Some(&PartialsOptions {
            enabled: Some(true),
            root_dir: Some(root.to_string_lossy().into_owned()),
            root: None,
            missing: None,
        }),
        source_path.and_then(|path| path.to_str()),
    )
    .expect("partials should resolve when enabled")
}

fn preprocess(source: &str, options: &TransformOptions) -> (String, Vec<String>) {
    let result = preprocess_markdown(source, &TransformFeatureOptions::from_options(options));
    (result.source.into_owned(), result.errors)
}

#[test]
fn disabled_by_default() {
    let root = TempRoot::new("disabled");
    root.write("_partials/install.md", "Install {{ package }}");
    let host = root.write(
        "host.md",
        "Before\n\n<!-- @partial: ./_partials/install.md package=\"ox-content\" -->\n\nAfter\n",
    );
    let source = fs::read_to_string(&host).unwrap();
    let (out, errors) = preprocess(
        &source,
        &TransformOptions {
            source_path: Some(host.to_string_lossy().into_owned()),
            ..Default::default()
        },
    );
    assert!(errors.is_empty(), "{errors:?}");
    assert!(out.contains("<!-- @partial:"), "{out}");
    assert!(!out.contains("Install ox-content"), "{out}");
}

#[test]
fn happy_path_relative() {
    let root = TempRoot::new("happy");
    root.write("_partials/install.md", "Install {{ package }} with {{ manager }}.\n");
    let host = root.write(
        "host.md",
        "Before\n\n<!-- @partial: ./_partials/install.md package=\"ox-content\" manager=\"pnpm\" -->\n",
    );
    let source = fs::read_to_string(&host).unwrap();
    let mut errors = Vec::new();
    let out = transform(&source, &resolved(&root.path, Some(&host)), &mut errors);
    assert!(errors.is_empty(), "{errors:?}");
    assert!(out.contains("Install ox-content with pnpm."), "{out}");
    assert!(!out.contains("<!-- @partial:"), "{out}");
}

#[test]
fn substitutions_are_html_escaped() {
    let root = TempRoot::new("escape");
    root.write("_partials/warn.md", "Value: {{ payload }}\n");
    let host = root.write(
        "host.md",
        "<!-- @partial: ./_partials/warn.md payload=\"<script>alert(1)</script>\" -->\n",
    );
    let source = fs::read_to_string(&host).unwrap();
    let mut errors = Vec::new();
    let out = transform(&source, &resolved(&root.path, Some(&host)), &mut errors);
    assert!(errors.is_empty(), "{errors:?}");
    assert!(out.contains("Value: &lt;script&gt;alert(1)&lt;/script&gt;"), "{out}");
    assert!(!out.contains("<script>"), "{out}");
}

#[test]
fn missing_params_stay_literal() {
    let root = TempRoot::new("missing-literal");
    root.write("_partials/install.md", "Install {{ package }} with {{ manager }}.\n");
    let host =
        root.write("host.md", "<!-- @partial: ./_partials/install.md package=\"ox-content\" -->\n");
    let source = fs::read_to_string(&host).unwrap();
    let mut errors = Vec::new();
    let out = transform(&source, &resolved(&root.path, Some(&host)), &mut errors);
    assert!(errors.is_empty(), "{errors:?}");
    assert!(out.contains("Install ox-content with {{ manager }}."), "{out}");
}

#[test]
fn missing_params_can_report() {
    let root = TempRoot::new("missing-error");
    root.write("_partials/install.md", "Install {{ package }}\n");
    let host = root.write("host.md", "<!-- @partial: install.md -->\n");
    let source = fs::read_to_string(&host).unwrap();
    let options = resolve(
        Some(&PartialsOptions {
            enabled: Some(true),
            root_dir: Some(root.path.to_string_lossy().into_owned()),
            root: None,
            missing: Some("error".to_string()),
        }),
        host.to_str(),
    )
    .unwrap();
    let mut errors = Vec::new();
    let out = transform(&source, &options, &mut errors);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(errors[0].contains("`package`"), "{errors:?}");
    assert!(out.contains("Install {{ package }}"), "{out}");
}

#[test]
fn recursion_is_an_error() {
    let root = TempRoot::new("cycle");
    let a = root.write("a.md", "A\n\n<!-- @partial: ./b.md -->\n");
    root.write("b.md", "B\n\n<!-- @partial: ./a.md -->\n");
    let source = fs::read_to_string(&a).unwrap();
    let mut errors = Vec::new();
    let out = transform(&source, &resolved(&root.path, Some(&a)), &mut errors);
    assert!(errors.iter().any(|error| error.to_ascii_lowercase().contains("cycle")), "{errors:?}");
    assert!(errors.iter().any(|error| error.contains(":3")), "{errors:?}");
    assert!(out.contains("<!-- @partial:"), "{out}");
}

#[test]
fn path_escape_outside_root_is_error() {
    let root = TempRoot::new("escape-path");
    let outside = root.path.parent().unwrap().join(format!(
        "ox-content-partials-secret-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
    ));
    fs::write(&outside, "SECRET_PARTIAL").unwrap();
    let host = root.write("host.md", "placeholder\n");
    let directive =
        format!("<!-- @partial: ../{} -->\n", outside.file_name().unwrap().to_string_lossy());
    fs::write(&host, &directive).unwrap();
    let mut errors = Vec::new();
    let out = transform(&directive, &resolved(&root.path, Some(&host)), &mut errors);
    let _ = fs::remove_file(&outside);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(errors[0].contains("outside root"), "{errors:?}");
    assert!(out.contains("<!-- @partial:"), "{out}");
    assert!(!out.contains("SECRET_PARTIAL"), "{out}");
}

#[test]
fn skips_fenced_and_inline_code() {
    let root = TempRoot::new("code");
    root.write("_partials/install.md", "INCLUDED_PARTIAL");
    let host = root.write(
        "host.md",
        "```md\n<!-- @partial: ./_partials/install.md -->\n```\nUse `<!-- @partial: ./_partials/install.md -->`.\n",
    );
    let source = fs::read_to_string(&host).unwrap();
    let mut errors = Vec::new();
    let out = transform(&source, &resolved(&root.path, Some(&host)), &mut errors);
    assert!(errors.is_empty(), "{errors:?}");
    assert!(out.contains("<!-- @partial: ./_partials/install.md -->"), "{out}");
    assert!(!out.contains("INCLUDED_PARTIAL"), "{out}");
}

#[test]
fn include_directive_is_unchanged() {
    let root = TempRoot::new("include");
    root.write("_partials/install.md", "PARTIAL_OK");
    root.write("shared.md", "INCLUDE_OK");
    let host = root.write(
        "host.md",
        "<!-- @include: ./shared.md -->\n<!-- @partial: ./_partials/install.md -->\n",
    );
    let source = fs::read_to_string(&host).unwrap();
    let mut errors = Vec::new();
    let out = transform(&source, &resolved(&root.path, Some(&host)), &mut errors);
    assert!(errors.is_empty(), "{errors:?}");
    assert!(out.contains("<!-- @include: ./shared.md -->"), "{out}");
    assert!(out.contains("PARTIAL_OK"), "{out}");
}

#[test]
fn bare_name_uses_default_root() {
    let root = TempRoot::new("bare");
    root.write("_partials/install.md", "bare {{ package }}\n");
    let host = root.write("host.md", "<!-- @partial: install.md package=\"ox-content\" -->\n");
    let source = fs::read_to_string(&host).unwrap();
    let mut errors = Vec::new();
    let out = transform(&source, &resolved(&root.path, Some(&host)), &mut errors);
    assert!(errors.is_empty(), "{errors:?}");
    assert!(out.contains("bare ox-content"), "{out}");
}

#[test]
fn vite_omitted_false_true_and_empty_object() {
    assert!(resolve(None, None).is_none());
    assert!(
        resolve(Some(&PartialsOptions { enabled: Some(false), ..Default::default() }), None)
            .is_none()
    );
    assert!(
        resolve(Some(&PartialsOptions { enabled: Some(true), ..Default::default() }), None)
            .is_some()
    );
    assert!(
        resolve(Some(&PartialsOptions { enabled: None, ..Default::default() }), None).is_some()
    );
}

#[test]
fn partial_headings_parse_and_line_maps() {
    let root = TempRoot::new("heading");
    root.write("_partials/heading.md", "## Included Heading\n\n- item one\n");
    let host = root.write("host.md", "# Host\n\n<!-- @partial: ./_partials/heading.md -->\n");
    let source = fs::read_to_string(&host).unwrap();
    let result = MarkdownTransformer::from_options(&enabled_options(&root.path, Some(&host)))
        .transform(&source);
    assert!(result.errors.is_empty(), "{:?}", result.errors);
    assert!(result.html.contains("<h2"), "{}", result.html);
    assert!(result.html.contains("Included Heading"), "{}", result.html);
    assert!(result.html.contains("<li>") || result.html.contains("<ul>"), "{}", result.html);
}
