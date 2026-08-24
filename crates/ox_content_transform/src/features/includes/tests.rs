use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::{resolve, transform};
use crate::features::{TransformFeatureOptions, preprocess_markdown};
use crate::transformer::MarkdownTransformer;
use crate::{IncludeOptions, TransformOptions};

struct TempRoot {
    path: PathBuf,
}

impl TempRoot {
    fn new(label: &str) -> Self {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let path = std::env::temp_dir()
            .join(format!("ox-content-includes-{label}-{}-{nanos}", std::process::id()));
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
        includes: Some(IncludeOptions {
            enabled: Some(true),
            root_dir: Some(root.to_string_lossy().into_owned()),
        }),
        ..Default::default()
    }
}

fn resolved(root: &Path, source_path: Option<&Path>) -> super::ResolvedIncludeOptions {
    resolve(
        Some(&IncludeOptions {
            enabled: Some(true),
            root_dir: Some(root.to_string_lossy().into_owned()),
        }),
        source_path.and_then(|path| path.to_str()),
    )
    .expect("includes should resolve when enabled")
}

fn preprocess(source: &str, options: &TransformOptions) -> (String, Vec<String>) {
    let result = preprocess_markdown(source, &TransformFeatureOptions::from_options(options));
    (result.source.into_owned(), result.errors)
}

#[test]
fn disabled_by_default() {
    let root = TempRoot::new("disabled");
    root.write("shared/warning.md", "INCLUDED_WARNING");
    let host = root.write("host.md", "Before\n\n<!-- @include: ./shared/warning.md -->\n\nAfter\n");
    let source = fs::read_to_string(&host).unwrap();
    let (out, errors) = preprocess(
        &source,
        &TransformOptions {
            source_path: Some(host.to_string_lossy().into_owned()),
            ..Default::default()
        },
    );
    assert!(errors.is_empty(), "{errors:?}");
    assert!(out.contains("<!-- @include: ./shared/warning.md -->"), "{out}");
    assert!(!out.contains("INCLUDED_WARNING"), "{out}");
}

#[test]
fn happy_path_relative() {
    let root = TempRoot::new("happy");
    root.write("shared/warning.md", "INCLUDED_WARNING");
    let host = root.write("host.md", "Before\n\n<!-- @include: ./shared/warning.md -->\n\nAfter\n");
    let source = fs::read_to_string(&host).unwrap();
    let mut errors = Vec::new();
    let out = transform(&source, &resolved(&root.path, Some(&host)), &mut errors);
    assert!(errors.is_empty(), "{errors:?}");
    assert!(out.contains("INCLUDED_WARNING"), "{out}");
    assert!(!out.contains("<!-- @include:"), "{out}");
    assert!(out.contains("Before"), "{out}");
    assert!(out.contains("After"), "{out}");

    let html = MarkdownTransformer::from_options(&enabled_options(&root.path, Some(&host)))
        .transform(&source)
        .html;
    assert!(html.contains("INCLUDED_WARNING"), "{html}");
}

#[test]
fn skips_fenced_code() {
    let root = TempRoot::new("fence");
    root.write("shared/warning.md", "INCLUDED_WARNING");
    let host = root.write("host.md", "```md\n<!-- @include: ./shared/warning.md -->\n```\n");
    let source = fs::read_to_string(&host).unwrap();
    let mut errors = Vec::new();
    let out = transform(&source, &resolved(&root.path, Some(&host)), &mut errors);
    assert!(errors.is_empty(), "{errors:?}");
    assert!(out.contains("<!-- @include: ./shared/warning.md -->"), "{out}");
    assert!(!out.contains("INCLUDED_WARNING"), "{out}");
}

#[test]
fn skips_inline_code() {
    let root = TempRoot::new("inline");
    root.write("shared/warning.md", "INCLUDED_WARNING");
    let host = root.write("host.md", "Use `<!-- @include: ./shared/warning.md -->` here.\n");
    let source = fs::read_to_string(&host).unwrap();
    let mut errors = Vec::new();
    let out = transform(&source, &resolved(&root.path, Some(&host)), &mut errors);
    assert!(errors.is_empty(), "{errors:?}");
    assert!(out.contains("`<!-- @include: ./shared/warning.md -->`"), "{out}");
    assert!(!out.contains("INCLUDED_WARNING"), "{out}");
}

#[test]
fn missing_target_is_error() {
    let root = TempRoot::new("missing");
    let host = root.write("host.md", "<!-- @include: ./missing.md -->\n");
    let source = fs::read_to_string(&host).unwrap();
    let mut errors = Vec::new();
    let out = transform(&source, &resolved(&root.path, Some(&host)), &mut errors);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(out.contains("<!-- @include: ./missing.md -->"), "{out}");
}

#[test]
fn path_escape_outside_root_is_error() {
    let root = TempRoot::new("escape");
    let outside = root.path.parent().unwrap().join(format!(
        "ox-content-includes-secret-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
    ));
    fs::write(&outside, "SECRET_INCLUDE").unwrap();
    let host =
        root.write("host.md", "<!-- @include: ../ox-content-includes-secret-placeholder.md -->\n");
    let directive =
        format!("<!-- @include: ../{} -->\n", outside.file_name().unwrap().to_string_lossy());
    fs::write(&host, &directive).unwrap();
    let mut errors = Vec::new();
    let out = transform(&directive, &resolved(&root.path, Some(&host)), &mut errors);
    let _ = fs::remove_file(&outside);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(errors[0].contains("outside root"), "{errors:?}");
    assert!(out.contains("<!-- @include:"), "{out}");
    assert!(!out.contains("SECRET_INCLUDE"), "{out}");
}

#[test]
fn quoted_path_works() {
    let root = TempRoot::new("quoted");
    root.write("shared/warning.md", "INCLUDED_QUOTED");
    let host = root.write("host.md", "<!-- @include: \"./shared/warning.md\" -->\n");
    let source = fs::read_to_string(&host).unwrap();
    let mut errors = Vec::new();
    let out = transform(&source, &resolved(&root.path, Some(&host)), &mut errors);
    assert!(errors.is_empty(), "{errors:?}");
    assert!(out.contains("INCLUDED_QUOTED"), "{out}");
    let host = root.write("host-single.md", "<!-- @include: './shared/warning.md' -->\n");
    let source = fs::read_to_string(&host).unwrap();
    let mut errors = Vec::new();
    let out = transform(&source, &resolved(&root.path, Some(&host)), &mut errors);
    assert!(errors.is_empty(), "{errors:?}");
    assert!(out.contains("INCLUDED_QUOTED"), "{out}");
}

#[test]
fn cycle_is_error() {
    let root = TempRoot::new("cycle");
    let a = root.write("a.md", "A\n\n<!-- @include: ./b.md -->\n");
    root.write("b.md", "B\n\n<!-- @include: ./a.md -->\n");
    let source = fs::read_to_string(&a).unwrap();
    let mut errors = Vec::new();
    let out = transform(&source, &resolved(&root.path, Some(&a)), &mut errors);
    assert!(!errors.is_empty(), "{errors:?}");
    assert!(errors.iter().any(|error| error.to_ascii_lowercase().contains("cycle")), "{errors:?}");
    assert!(
        out.contains("<!-- @include: ./a.md -->") || out.contains("<!-- @include: ./b.md -->"),
        "{out}"
    );
}

#[test]
fn unclosed_malformed_comment_stays_literal() {
    let root = TempRoot::new("malformed");
    root.write("x.md", "INCLUDED_MALFORMED");
    let host = root.write("host.md", "<!-- @include: ./x.md\n");
    let source = fs::read_to_string(&host).unwrap();
    let mut errors = Vec::new();
    let out = transform(&source, &resolved(&root.path, Some(&host)), &mut errors);
    assert!(errors.is_empty(), "{errors:?}");
    assert_eq!(out, source);
    assert!(!out.contains("INCLUDED_MALFORMED"), "{out}");
}

#[test]
fn vite_omitted_false_true_and_empty_object() {
    assert!(resolve(None, None).is_none(), "omitted => false");
    assert!(
        resolve(Some(&IncludeOptions { enabled: Some(false), root_dir: None }), None).is_none(),
        "false => false"
    );
    assert!(
        resolve(Some(&IncludeOptions { enabled: Some(true), root_dir: None }), None).is_some(),
        "true => true"
    );
    assert!(
        resolve(Some(&IncludeOptions { enabled: None, root_dir: None }), None).is_some(),
        "{{}} => true"
    );
}

#[test]
fn skips_indented_code() {
    let root = TempRoot::new("indent");
    root.write("shared/warning.md", "INCLUDED_WARNING");
    let host = root.write("host.md", "    <!-- @include: ./shared/warning.md -->\n");
    let source = fs::read_to_string(&host).unwrap();
    let mut errors = Vec::new();
    let out = transform(&source, &resolved(&root.path, Some(&host)), &mut errors);
    assert!(errors.is_empty(), "{errors:?}");
    assert!(out.contains("<!-- @include: ./shared/warning.md -->"), "{out}");
    assert!(!out.contains("INCLUDED_WARNING"), "{out}");
}

#[test]
fn unknown_html_comment_stays() {
    let root = TempRoot::new("unknown");
    root.write("x.md", "INCLUDED_UNKNOWN");
    let host = root.write("host.md", "<!-- TODO: @include: ./x.md -->\n<!-- @include ./x.md -->\n");
    let source = fs::read_to_string(&host).unwrap();
    let mut errors = Vec::new();
    let out = transform(&source, &resolved(&root.path, Some(&host)), &mut errors);
    assert!(errors.is_empty(), "{errors:?}");
    assert_eq!(out, source);
}

#[test]
fn nested_include_expands() {
    let root = TempRoot::new("nested");
    root.write("inner.md", "INNER_SNIPPET");
    root.write("mid.md", "MID\n\n<!-- @include: ./inner.md -->\n");
    let host = root.write("host.md", "HOST\n\n<!-- @include: ./mid.md -->\n");
    let source = fs::read_to_string(&host).unwrap();
    let mut errors = Vec::new();
    let out = transform(&source, &resolved(&root.path, Some(&host)), &mut errors);
    assert!(errors.is_empty(), "{errors:?}");
    assert!(out.contains("HOST"), "{out}");
    assert!(out.contains("MID"), "{out}");
    assert!(out.contains("INNER_SNIPPET"), "{out}");
}

#[test]
fn included_headings_parse() {
    let root = TempRoot::new("heading");
    root.write("shared/heading.md", "## Included Heading\n\n- item one\n");
    let host = root.write("host.md", "# Host\n\n<!-- @include: ./shared/heading.md -->\n");
    let source = fs::read_to_string(&host).unwrap();
    let result = MarkdownTransformer::from_options(&enabled_options(&root.path, Some(&host)))
        .transform(&source);
    assert!(result.errors.is_empty(), "{:?}", result.errors);
    assert!(result.html.contains("<h2"), "{}", result.html);
    assert!(result.html.contains("<li>") || result.html.contains("<ul>"), "{}", result.html);
}

#[test]
fn root_alias_and_absolute_paths() {
    let root = TempRoot::new("alias");
    root.write("shared/warning.md", "INCLUDED_ROOT");
    let host = root.write("host.md", "<!-- @include: @/shared/warning.md -->\n");
    let source = fs::read_to_string(&host).unwrap();
    let mut errors = Vec::new();
    let out = transform(&source, &resolved(&root.path, Some(&host)), &mut errors);
    assert!(errors.is_empty(), "{errors:?}");
    assert!(out.contains("INCLUDED_ROOT"), "{out}");
    let host = root.write("host-abs.md", "<!-- @include: /shared/warning.md -->\n");
    let source = fs::read_to_string(&host).unwrap();
    let mut errors = Vec::new();
    let out = transform(&source, &resolved(&root.path, Some(&host)), &mut errors);
    assert!(errors.is_empty(), "{errors:?}");
    assert!(out.contains("INCLUDED_ROOT"), "{out}");
}
