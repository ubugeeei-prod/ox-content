use super::super::*;

fn page(contributors: Vec<Contributor>) -> PageData {
    PageData {
        title: "Guide".to_string(),
        description: None,
        content: "<p>Article</p>".to_string(),
        toc: vec![],
        last_updated: None,
        contributors,
        path: "guide".to_string(),
        entry_page: None,
        prev: None,
        next: None,
        breadcrumbs: None,
        chrome: PageChromeFlags::default(),
        robots: None,
        canonical: None,
        markdown_source: None,
    }
}

fn config() -> SsgConfig {
    SsgConfig {
        site_name: "Docs".to_string(),
        base: "/".to_string(),
        breadcrumb_root_href: None,
        og_image: None,
        theme: None,
        locale: None,
        available_locales: None,
        pagination: false,
        breadcrumbs: false,
        reader_chrome: ReaderChrome::default(),
        locale_switcher: false,
        locale_paths: vec![],
        a11y: A11y::default(),
        page_chrome: false,
        json_ld: JsonLd::default(),
        site_url: None,
        head_validation: HeadValidation::Off,
    }
}

fn render(contributors: Vec<Contributor>) -> String {
    generate_html(&page(contributors), &[], &config())
}

fn contributor(name: &str, avatar: Option<&str>) -> Contributor {
    Contributor { name: name.to_string(), avatar: avatar.map(str::to_string) }
}

#[test]
fn contributors_disabled_by_default() {
    let html = render(vec![]);
    assert!(
        !html.contains("class=\"contributors\""),
        "contributor chrome must stay off unless authors are passed: {html}"
    );
}

#[test]
fn contributors_empty_when_no_contributors() {
    let html = render(vec![contributor("", None), contributor("   ", None)]);
    assert!(
        !html.contains("class=\"contributors\""),
        "blank names must not emit the contributor list: {html}"
    );
}

#[test]
fn contributors_happy_path_unique_authors() {
    let html = render(vec![
        contributor("Ada Lovelace", None),
        contributor("ada lovelace", None),
        contributor("Grace Hopper", None),
    ]);

    assert!(html.contains("<ul class=\"contributors\" aria-label=\"Contributors\">"));
    assert_eq!(html.matches("class=\"contributor-name\"").count(), 2);
    assert!(html.contains("<span class=\"contributor-name\">Ada Lovelace</span>"));
    assert!(html.contains("<span class=\"contributor-name\">Grace Hopper</span>"));
    assert!(!html.contains("ada lovelace"));
}

#[test]
fn contributors_hostile_name_escaped() {
    let html = render(vec![contributor("<img src=x onerror=alert(1)>", None)]);

    assert!(
        html.contains("<span class=\"contributor-name\">&lt;img src=x onerror=alert(1)&gt;</span>")
    );
    assert!(!html.contains("<img src=x onerror=alert(1)>"));
}

#[test]
fn contributors_reject_unsafe_avatar_urls() {
    let html = render(vec![contributor("Ada", Some("javascript:alert(1)"))]);

    assert!(html.contains("<span class=\"contributor-name\">Ada</span>"));
    assert!(!html.contains("javascript:alert(1)"));
    assert!(!html.contains("<img class=\"contributor-avatar\""));
}

#[test]
fn contributors_render_https_avatars() {
    let html = render(vec![contributor("Ada", Some("https://www.gravatar.com/avatar/abc"))]);

    assert!(html.contains(
        "<img class=\"contributor-avatar\" src=\"https://www.gravatar.com/avatar/abc\" alt=\"\" width=\"24\" height=\"24\">"
    ));
}
