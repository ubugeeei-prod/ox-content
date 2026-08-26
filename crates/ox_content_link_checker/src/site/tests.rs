use std::fs;
use std::path::{Path, PathBuf};

use super::*;

fn fixture(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("ox-content-site-links-{name}"));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

fn write(root: &Path, relative: &str, content: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn options(root: &Path) -> SiteCheckOptions {
    SiteCheckOptions { site_dir: root.to_path_buf(), base: "/ox-content/".to_string() }
}

fn all_diagnostics(reports: &[SiteReport]) -> Vec<&Diagnostic> {
    reports.iter().flat_map(|report| &report.diagnostics).collect()
}

#[test]
fn generated_site_accepts_base_locale_version_routes_assets_and_encoded_fragments() {
    let root = fixture("valid-routes");
    write(
        &root,
        "index.html",
        r#"<a href="/ox-content/ja/guide/">ja</a>
<a href="/ox-content/2.90/api/#%E6%97%A5%E6%9C%AC%E8%AA%9E">api</a>
<img src="/ox-content/logo%20icon.svg" srcset="/ox-content/logo%20icon.svg 1x">
<a href="./relative/">relative</a>"#,
    );
    write(&root, "ja/guide/index.html", r#"<h1 id="guide">Guide</h1>"#);
    write(&root, "2.90/api/index.html", r#"<h2 id="日本語">API</h2>"#);
    write(&root, "relative/index.html", "<p>Relative</p>");
    write(&root, "logo icon.svg", "<svg/>");

    let reports = check_site(&options(&root)).unwrap();
    assert!(reports.is_empty(), "{reports:#?}");
}

#[test]
fn generated_site_reports_missing_page_and_cross_file_anchor() {
    let root = fixture("missing-targets");
    write(
        &root,
        "index.html",
        r#"<a href="/ox-content/missing/">missing</a>
<a href="/ox-content/guide/#nope">anchor</a>"#,
    );
    write(&root, "guide/index.html", r#"<h1 id="guide">Guide</h1>"#);

    let reports = check_site(&options(&root)).unwrap();
    let diagnostics = all_diagnostics(&reports);
    assert_eq!(diagnostics.len(), 2, "{reports:#?}");
    assert!(diagnostics.iter().all(|diagnostic| diagnostic.severity == Severity::Error));
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.message.contains("does not exist")));
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.message.contains("is missing")));
}

#[test]
fn generated_site_reports_links_to_redirects_but_validates_the_redirect_destination() {
    let root = fixture("redirect");
    write(&root, "index.html", r#"<a href="/ox-content/old/">old</a>"#);
    write(&root, "old/index.html", r#"<meta http-equiv="refresh" content="0; url=../new/">"#);
    write(&root, "new/index.html", "<p>New</p>");

    let reports = check_site(&options(&root)).unwrap();
    let diagnostics = all_diagnostics(&reports);
    assert_eq!(diagnostics.len(), 1, "{reports:#?}");
    assert_eq!(diagnostics[0].severity, Severity::Warning);
    assert!(diagnostics[0].message.contains("redirect page"));
}

#[test]
fn generated_site_rejects_outside_base_and_root_escape() {
    let root = fixture("containment");
    write(
        &root,
        "nested/index.html",
        r#"<a href="/other-site/page">outside</a><img src="../../../secret.png">"#,
    );

    let reports = check_site(&options(&root)).unwrap();
    let diagnostics = all_diagnostics(&reports);
    assert_eq!(diagnostics.len(), 2, "{reports:#?}");
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.message.contains("outside")));
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.message.contains("escapes")));
}

#[test]
fn generated_site_ignores_markup_like_text_inside_scripts_and_styles() {
    let root = fixture("raw-text");
    write(
        &root,
        "index.html",
        r#"<script>const fake = '<a href="/missing/">';</script>
<style>.x::after { content: '<img src="/missing.png">' }</style>"#,
    );

    let reports = check_site(&options(&root)).unwrap();
    assert!(reports.is_empty(), "{reports:#?}");
}

#[test]
fn generated_site_does_not_panic_on_hostile_markup() {
    let root = fixture("hostile-markup");
    let huge_href = format!("/ox-content/{}", "a".repeat(8_192));
    write(
        &root,
        "index.html",
        &format!(
            r#"<a href="javascript:alert(1)">js</a>
<a href="{huge_href}">huge</a>
<a href="/ox-content/missing/#"><img src="data:text/html,<script>x</script>"></a>
<a href="./#\0not-an-anchor">nul</a>
<p>unclosed <a href="/ox-content/ok/#gone""#
        ),
    );
    write(&root, "ok/index.html", r#"<h1 id="ok">Ok</h1>"#);

    let reports = check_site(&options(&root)).unwrap();
    let diagnostics = all_diagnostics(&reports);
    assert!(
        diagnostics.iter().any(|diagnostic| diagnostic.message.contains("does not exist")),
        "{reports:#?}"
    );
}
