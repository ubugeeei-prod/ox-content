use std::path::Path;

use crate::EditThisPageOptions;

use super::super::option_resolve::resolve_edit_this_page;
use super::edit_this_page_href;

fn options(root_dir: Option<&str>, src_dir: Option<&str>) -> EditThisPageOptions {
    EditThisPageOptions {
        enabled: Some(true),
        repo_url: Some("https://example.com/owner/repo".to_string()),
        branch: Some("main".to_string()),
        root_dir: root_dir.map(ToOwned::to_owned),
        src_dir: src_dir.map(ToOwned::to_owned),
        provider: None,
        url_pattern: None,
        label: None,
    }
}

fn href(root_dir: Option<&str>, src_dir: Option<&str>, source_path: &str) -> String {
    let resolved = resolve_edit_this_page(Some(&options(root_dir, src_dir)), source_path)
        .expect("edit link should resolve");
    edit_this_page_href(&resolved)
}

/// An absolute path the way a build hands one to the transform.
fn under_cwd(relative: &str) -> String {
    let cwd = std::env::current_dir().expect("cwd");
    cwd.join(Path::new(relative)).to_string_lossy().into_owned()
}

#[test]
fn without_a_root_dir_the_path_is_relative_to_the_working_directory() {
    assert_eq!(
        href(None, None, &under_cwd("docs/guide/nested.md")),
        "https://example.com/owner/repo/edit/main/docs/guide/nested.md"
    );
}

#[test]
fn a_root_dir_prefixes_the_path_inside_the_source_root() {
    let src_dir = under_cwd("docs");

    assert_eq!(
        href(Some("docs"), Some(&src_dir), &under_cwd("docs/guide/nested.md")),
        "https://example.com/owner/repo/edit/main/docs/guide/nested.md"
    );
    assert_eq!(
        href(Some("packages/site/docs"), Some(&src_dir), &under_cwd("docs/index.md")),
        "https://example.com/owner/repo/edit/main/packages/site/docs/index.md"
    );
}

#[test]
fn a_root_dir_never_leaks_the_build_machines_path() {
    // Every value used to switch the page path from repository-relative to
    // the absolute path of the checkout, which then shipped in the HTML.
    let src_dir = under_cwd("docs");
    let source = under_cwd("docs/guide/nested.md");

    let checkout = std::env::current_dir().expect("cwd").to_string_lossy().into_owned();
    for root_dir in ["docs", "sub", ".", "packages/site", "/docs/", "", "/nowhere/on/disk"] {
        let href = href(Some(root_dir), Some(&src_dir), &source);
        assert!(!href.contains(&checkout), "rootDir {root_dir:?} leaked the checkout: {href}");
        assert!(!href.contains("/edit/main//"), "rootDir {root_dir:?} doubled a slash: {href}");
    }
}

#[test]
fn a_blank_or_dot_root_dir_behaves_like_an_omitted_one() {
    let source = under_cwd("docs/guide/nested.md");
    let expected = "https://example.com/owner/repo/edit/main/docs/guide/nested.md";

    assert_eq!(href(Some(""), None, &source), expected);
    assert_eq!(href(Some("."), None, &source), expected);
    assert_eq!(href(Some("   "), None, &source), expected);
}

#[test]
fn surrounding_slashes_in_root_dir_do_not_reach_the_url() {
    let src_dir = under_cwd("docs");

    assert_eq!(
        href(Some("/packages/site/docs/"), Some(&src_dir), &under_cwd("docs/index.md")),
        "https://example.com/owner/repo/edit/main/packages/site/docs/index.md"
    );
}

#[test]
fn a_root_dir_without_a_source_root_still_prefixes_the_working_directory_path() {
    // `renderMarkdown` and the dev server hand over no source root; the page
    // path stays relative to the working directory and `rootDir` goes in
    // front of it, which is what a build inside a package needs.
    assert_eq!(
        href(Some("packages/site"), None, &under_cwd("docs/guide/nested.md")),
        "https://example.com/owner/repo/edit/main/packages/site/docs/guide/nested.md"
    );
}

#[test]
fn an_absolute_root_dir_containing_the_page_is_a_directory_on_disk() {
    // The long-standing shape: point at the repository root from a build
    // that runs somewhere below it.
    let repo_root = std::env::current_dir().expect("cwd").to_string_lossy().into_owned();

    assert_eq!(
        href(Some(&repo_root), None, &under_cwd("docs/guide/nested.md")),
        "https://example.com/owner/repo/edit/main/docs/guide/nested.md"
    );
}

#[test]
fn a_leading_slash_is_read_as_a_repository_path_not_a_disk_path() {
    // `/packages/site/docs` is how a repository path gets written by hand.
    // It cannot be a directory on this machine holding the page, so it is
    // a prefix rather than something to measure from.
    let src_dir = under_cwd("docs");

    assert_eq!(
        href(Some("/packages/site/docs"), Some(&src_dir), &under_cwd("docs/guide/nested.md")),
        "https://example.com/owner/repo/edit/main/packages/site/docs/guide/nested.md"
    );
}

#[test]
fn a_relative_source_path_is_resolved_before_it_is_relativized() {
    assert_eq!(
        href(None, None, "docs/guide/nested.md"),
        "https://example.com/owner/repo/edit/main/docs/guide/nested.md"
    );
}
