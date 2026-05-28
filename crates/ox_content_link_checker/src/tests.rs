use std::fs;
use std::path::PathBuf;

use super::*;

fn run(source: &str, file_path: PathBuf) -> Vec<Diagnostic> {
    check_source(source, &CheckOptions::for_file(file_path))
}

fn tmp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ox-content-link-checker-{name}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

#[test]
fn external_links_are_never_reported() {
    let diagnostics = run(
        "[home](https://example.com) and [http](http://example.com)\n",
        PathBuf::from("/tmp/doc.md"),
    );
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn mailto_and_other_schemes_pass_through() {
    let diagnostics =
        run("[email](mailto:dev@example.com) [tel](tel:+1)\n", PathBuf::from("/tmp/doc.md"));
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn self_anchor_is_validated_against_headings() {
    let source = concat!(
        "# Top Heading\n",
        "\n",
        "## Has Sub-Headings!\n",
        "\n",
        "[good](#top-heading)\n",
        "[bad](#nope)\n",
        "[punctuated](#has-sub-headings)\n",
    );
    let diagnostics = run(source, PathBuf::from("/tmp/doc.md"));
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!(diagnostics[0].target, "#nope");
    assert_eq!(diagnostics[0].kind, LinkKind::Anchor);
}

#[test]
fn missing_relative_file_is_reported_with_resolved_path() {
    let dir = tmp_dir("missing-rel");
    let doc = dir.join("doc.md");
    let diagnostics = run("[lost](./missing.md)\n", doc);
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert!(diagnostics[0].message.contains("missing.md"), "{}", diagnostics[0].message);
    assert_eq!(diagnostics[0].kind, LinkKind::File);
}

#[test]
fn existing_relative_file_is_silent() {
    let dir = tmp_dir("existing-rel");
    fs::write(dir.join("sibling.md"), "ok").unwrap();
    let diagnostics = run("[ok](./sibling.md)\n", dir.join("doc.md"));
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn absolute_paths_resolve_under_src_dir_when_configured() {
    let dir = tmp_dir("src-dir-abs");
    fs::create_dir_all(dir.join("nested")).unwrap();
    fs::write(dir.join("nested/leaf.md"), "ok").unwrap();

    let opts = CheckOptions {
        file_path: dir.join("docs/doc.md"),
        src_dir: Some(dir.clone()),
        ignore_patterns: Vec::new(),
    };
    let diagnostics = check_source("[abs](/nested/leaf.md) [miss](/nope.md)\n", &opts);
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!(diagnostics[0].target, "/nope.md");
}

#[test]
fn cross_file_anchor_emits_warning_until_followup_lands() {
    let dir = tmp_dir("cross-file-anchor");
    fs::write(dir.join("other.md"), "# other\n").unwrap();
    let diagnostics = run("[hop](./other.md#section)\n", dir.join("doc.md"));
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!(diagnostics[0].severity, Severity::Warning);
    assert!(diagnostics[0].message.contains("not validated yet"), "{}", diagnostics[0].message);
}

#[test]
fn reference_links_are_skipped_for_now() {
    // The parser doesn't currently expand reference links into Link
    // nodes — they survive as plain Text. The checker therefore can't
    // see them. Document the current behavior so a future parser
    // change that does expand them lights up this test as a TODO.
    let dir = tmp_dir("ref-link");
    fs::write(dir.join("target.md"), "ok").unwrap();
    let source = concat!("[ok][ref]\n", "\n", "[ref]: ./target.md\n");
    let diagnostics = run(source, dir.join("doc.md"));
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn ignore_patterns_suppress_diagnostics() {
    let opts = CheckOptions {
        file_path: PathBuf::from("/tmp/doc.md"),
        src_dir: None,
        ignore_patterns: vec!["intentionally-broken".into()],
    };
    let diagnostics =
        check_source("[skipped](./intentionally-broken-link.md) [reported](./other.md)\n", &opts);
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert!(diagnostics[0].target.contains("other.md"));
}

#[test]
fn image_target_is_resolved_like_links() {
    let dir = tmp_dir("image");
    let diagnostics = run("![alt](./missing.png)\n", dir.join("doc.md"));
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert!(diagnostics[0].message.contains("image"), "{}", diagnostics[0].message);
}

#[test]
fn line_index_points_at_the_link_start() {
    let dir = tmp_dir("line-index");
    let source = "line one\nline two has [broken](./nope.md) target\n";
    let diagnostics = run(source, dir.join("doc.md"));
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!(diagnostics[0].line, 2);
    assert!(diagnostics[0].column > 1);
}
