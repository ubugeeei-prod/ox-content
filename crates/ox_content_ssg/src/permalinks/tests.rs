use serde_json::{Map, Value};

use super::{CascadeOptions, PermalinksOptions, RoutePage, escape_attribute, resolve_page_routes};

fn on() -> PermalinksOptions {
    PermalinksOptions { enabled: true }
}

fn cascade_on() -> CascadeOptions {
    CascadeOptions { enabled: true }
}

fn off_permalinks() -> PermalinksOptions {
    PermalinksOptions { enabled: false }
}

fn off_cascade() -> CascadeOptions {
    CascadeOptions { enabled: false }
}

fn page(source: &str, file_url: &str, pairs: &[(&str, &str)]) -> RoutePage {
    let mut frontmatter = Map::new();
    for (key, value) in pairs {
        frontmatter.insert((*key).to_string(), Value::String((*value).to_string()));
    }
    RoutePage { source: source.to_string(), file_url: file_url.to_string(), frontmatter }
}

fn url_of(output: &super::RouteResolveOutput, source: &str) -> String {
    output
        .pages
        .iter()
        .find(|page| page.source == source)
        .map(|page| page.url_path.clone())
        .unwrap_or_default()
}

fn field<'a>(output: &'a super::RouteResolveOutput, source: &str, key: &str) -> Option<&'a str> {
    output
        .pages
        .iter()
        .find(|page| page.source == source)
        .and_then(|page| page.frontmatter.get(key).and_then(Value::as_str))
}

#[test]
fn disabled_by_default_keeps_file_tree_urls() {
    let pages = vec![
        page("guide/intro.md", "guide/intro", &[("permalink", "/custom"), ("slug", "other")]),
        page("guide/_index.md", "guide/_index", &[("sidebar", "Guide")]),
        page("guide/child.md", "guide/child", &[]),
    ];

    let output = resolve_page_routes(&pages, &off_permalinks(), &off_cascade());

    assert_eq!(url_of(&output, "guide/intro.md"), "guide/intro");
    assert_eq!(url_of(&output, "guide/child.md"), "guide/child");
    assert_eq!(field(&output, "guide/child.md", "sidebar"), None);
    assert!(output.errors.is_empty(), "{output:?}");
}

#[test]
fn permalink_replaces_the_file_tree_url() {
    let pages = vec![page("guide/intro.md", "guide/intro", &[("permalink", "/getting-started")])];

    let output = resolve_page_routes(&pages, &on(), &off_cascade());

    assert_eq!(url_of(&output, "guide/intro.md"), "getting-started");
    assert!(output.errors.is_empty(), "{output:?}");
}

#[test]
fn slug_replaces_the_last_segment() {
    let pages = vec![
        page("guide/intro.md", "guide/intro", &[("slug", "hello")]),
        page("index.md", "/", &[("slug", "home")]),
    ];

    let output = resolve_page_routes(&pages, &on(), &off_cascade());

    assert_eq!(url_of(&output, "guide/intro.md"), "guide/hello");
    assert_eq!(url_of(&output, "index.md"), "home");
}

#[test]
fn permalink_wins_over_slug() {
    let pages = vec![page("a.md", "a", &[("permalink", "/custom"), ("slug", "ignored")])];

    let output = resolve_page_routes(&pages, &on(), &off_cascade());

    assert_eq!(url_of(&output, "a.md"), "custom");
}

#[test]
fn cascade_inherits_and_child_overrides() {
    let pages = vec![
        page("guide/_index.md", "guide/_index", &[("sidebar", "Guide"), ("title", "Section")]),
        page("guide/child.md", "guide/child", &[("title", "Child")]),
        page("guide/plain.md", "guide/plain", &[]),
        page("other.md", "other", &[]),
    ];

    let output = resolve_page_routes(&pages, &off_permalinks(), &cascade_on());

    assert_eq!(field(&output, "guide/child.md", "title"), Some("Child"));
    assert_eq!(field(&output, "guide/child.md", "sidebar"), Some("Guide"));
    assert_eq!(field(&output, "guide/plain.md", "sidebar"), Some("Guide"));
    assert_eq!(field(&output, "guide/plain.md", "title"), Some("Section"));
    assert_eq!(field(&output, "other.md", "sidebar"), None);
    assert_eq!(field(&output, "guide/_index.md", "title"), Some("Section"));
}

#[test]
fn cascade_does_not_inherit_permalink_or_slug() {
    let pages = vec![
        page("guide/_index.md", "guide/_index", &[("permalink", "/section"), ("slug", "section")]),
        page("guide/child.md", "guide/child", &[]),
    ];

    let output = resolve_page_routes(&pages, &on(), &cascade_on());

    assert_eq!(url_of(&output, "guide/child.md"), "guide/child");
    assert_eq!(url_of(&output, "guide/_index.md"), "section");
}

#[test]
fn path_escape_is_rejected() {
    let pages = vec![
        page("a.md", "a", &[("permalink", "../etc/passwd")]),
        page("b.md", "b", &[("permalink", "/../secret")]),
        page("c.md", "c", &[("permalink", "javascript:alert(1)")]),
        page("d.md", "d", &[("permalink", "//evil.example")]),
        page("e.md", "e", &[("permalink", r"C:\Windows")]),
        page("f.md", "f", &[("slug", "..")]),
    ];

    let output = resolve_page_routes(&pages, &on(), &off_cascade());

    assert_eq!(url_of(&output, "a.md"), "a");
    assert_eq!(url_of(&output, "b.md"), "b");
    assert_eq!(url_of(&output, "c.md"), "c");
    assert_eq!(url_of(&output, "d.md"), "d");
    assert_eq!(url_of(&output, "e.md"), "e");
    assert_eq!(url_of(&output, "f.md"), "f");
    assert_eq!(output.errors.len(), 6, "{output:?}");
    assert!(output.errors.iter().all(|error| error.contains("rejected")), "{output:?}");
}

#[test]
fn url_collision_errors_and_keeps_the_first_page() {
    let pages = vec![
        page("first.md", "first", &[("permalink", "/guide")]),
        page("second.md", "second", &[("permalink", "/guide/")]),
        page("third.md", "third", &[]),
    ];

    let output = resolve_page_routes(&pages, &on(), &off_cascade());

    assert_eq!(output.pages.len(), 2, "{output:?}");
    assert_eq!(url_of(&output, "first.md"), "guide");
    assert_eq!(url_of(&output, "third.md"), "third");
    assert!(url_of(&output, "second.md").is_empty(), "{output:?}");
    assert_eq!(output.errors.len(), 1, "{output:?}");
    assert!(output.errors[0].contains("collision"), "{output:?}");
    assert!(output.errors[0].contains("first.md"), "{output:?}");
    assert!(output.errors[0].contains("second.md"), "{output:?}");
}

#[test]
fn permalink_is_escaped_for_html_attributes() {
    let escaped = escape_attribute(r#"/foo" onclick="alert(1)"#);

    assert!(!escaped.contains(r#"" onclick="#), "{escaped}");
    assert!(escaped.contains("&quot;"), "{escaped}");
    assert_eq!(escape_attribute("a&b<c>'"), "a&amp;b&lt;c&gt;&#39;");
}

#[test]
fn hostile_input_does_not_panic() {
    let mut weird = Map::new();
    weird.insert("permalink".to_string(), Value::Array(vec![Value::Null]));
    weird.insert("slug".to_string(), Value::Bool(true));
    let pages = vec![
        page("a.md", "a", &[("permalink", ""), ("slug", "   ")]),
        RoutePage { source: "b.md".to_string(), file_url: "b".to_string(), frontmatter: weird },
        page("c.md", "c", &[("permalink", "foo\nbar")]),
        page("d.md", "d", &[("permalink", "\0js")]),
        page("_index.md", "/", &[("title", "Root")]),
    ];

    let output = resolve_page_routes(&pages, &on(), &cascade_on());

    assert_eq!(output.pages.len(), 5, "{output:?}");
    assert_eq!(url_of(&output, "a.md"), "a");
    assert_eq!(url_of(&output, "b.md"), "b");
    assert_eq!(url_of(&output, "c.md"), "c");
}
