use super::{TeamLink, TeamMember, TeamOptions, render_team_page};
use crate::html::{
    HeadValidation, NavGroup, NavItem, PageData, ReaderChrome, SsgConfig, generate_html,
    team::TEAM_CSS,
};

fn member(
    name: &str,
    role: Option<&str>,
    avatar: Option<&str>,
    links: Option<Vec<TeamLink>>,
) -> TeamMember {
    TeamMember {
        name: name.to_string(),
        role: role.map(str::to_string),
        avatar: avatar.map(str::to_string),
        links,
    }
}

fn link(label: &str, href: &str) -> TeamLink {
    TeamLink { label: label.to_string(), href: href.to_string() }
}

fn enabled(members: Vec<TeamMember>) -> TeamOptions {
    TeamOptions { enabled: true, members }
}

fn page(content: &str) -> PageData {
    PageData {
        title: "Team".to_string(),
        description: None,
        content: content.to_string(),
        toc: vec![],
        last_updated: None,
        contributors: vec![],
        path: "team".to_string(),
        entry_page: None,
        prev: None,
        next: None,
        breadcrumbs: None,
        chrome: crate::PageChromeFlags::default(),
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
        a11y: crate::A11y::default(),
        page_chrome: false,
        json_ld: crate::JsonLd::default(),
        site_url: None,
        head_validation: HeadValidation::Off,
    }
}

fn nav() -> Vec<NavGroup> {
    vec![NavGroup {
        title: "Site".to_string(),
        items: vec![NavItem {
            title: "Team".to_string(),
            path: "team".to_string(),
            href: "/team/index.html".to_string(),
            children: vec![],
            collapsed: None,
            sticky_collapsed: None,
        }],
        collapsed: None,
        sticky_collapsed: None,
    }]
}

fn sample_members() -> Vec<TeamMember> {
    vec![
        member(
            "Ada Lovelace",
            Some("Mathematician"),
            Some("https://cdn.example.com/ada.png"),
            Some(vec![link("Website", "https://example.com/ada"), link("Profile", "/people/ada")]),
        ),
        member("Grace Hopper", None, Some("/avatars/grace.png"), None),
    ]
}

#[test]
fn disabled_by_default() {
    let team = TeamOptions::default();
    assert!(!team.enabled, "omitted team config must stay off");
    assert!(team.members.is_empty());
    assert_eq!(TeamOptions::disabled(), team);

    let body = "<p>Ordinary page</p>";
    let html = render_team_page(&team, "team", body);
    assert_eq!(html, body, "layout: team must be ignored while the option is off");
    assert!(!html.contains("ox-team"), "{html}");
    assert!(!html.contains("Ada"), "{html}");

    let enabled_empty = enabled(sample_members());
    let ordinary = render_team_page(&enabled_empty, "doc", body);
    assert_eq!(ordinary, body, "a non-team layout must stay an ordinary page");
    assert!(!ordinary.contains("ox-team"), "{ordinary}");
}

#[test]
fn happy_path_cards() {
    let body = "<p>About the maintainers</p>";
    let html = render_team_page(&enabled(sample_members()), "team", body);

    assert!(html.contains(r#"class="ox-team""#), "{html}");
    assert!(html.contains(r#"class="ox-team__card""#), "{html}");
    assert!(html.contains("Ada Lovelace"), "{html}");
    assert!(html.contains("Mathematician"), "{html}");
    assert!(html.contains(r#"src="https://cdn.example.com/ada.png""#), "{html}");
    assert!(html.contains(r#"href="https://example.com/ada""#), "{html}");
    assert!(html.contains("Website"), "{html}");
    assert!(html.contains(r#"href="/people/ada""#), "{html}");
    assert!(html.contains("Grace Hopper"), "{html}");
    assert!(html.contains(r#"src="/avatars/grace.png""#), "{html}");
    assert!(html.contains(body), "markdown body should remain around the cards: {html}");

    let page_html = generate_html(&page(&html), &nav(), &config());
    assert!(page_html.contains("ox-content:css:team"), "{page_html}");
    assert!(page_html.contains(TEAM_CSS.trim()), "{page_html}");
}

#[test]
fn javascript_avatar_rejected() {
    let html = render_team_page(
        &enabled(vec![member("Ada", None, Some("javascript:alert(1)"), None)]),
        "team",
        "",
    );

    assert!(html.contains("Ada"), "{html}");
    assert!(html.contains("ox-team"), "{html}");
    assert!(!html.contains("javascript:"), "{html}");
    assert!(!html.contains("alert(1)"), "{html}");
    assert!(!html.contains("<img"), "{html}");
}

#[test]
fn javascript_link_rejected() {
    let html = render_team_page(
        &enabled(vec![member("Ada", None, None, Some(vec![link("XSS", "javascript:alert(1)")]))]),
        "team",
        "",
    );

    assert!(html.contains("Ada"), "{html}");
    assert!(!html.contains("javascript:"), "{html}");
    assert!(!html.contains("alert(1)"), "{html}");
    assert!(!html.contains("XSS"), "{html}");
    assert!(!html.contains("<a "), "{html}");
}

#[test]
fn http_avatar_rejected() {
    let html = render_team_page(
        &enabled(vec![member("Ada", None, Some("http://evil.example/ada.png"), None)]),
        "team",
        "",
    );

    assert!(html.contains("Ada"), "{html}");
    assert!(!html.contains("http://"), "{html}");
    assert!(!html.contains("evil.example"), "{html}");
    assert!(!html.contains("<img"), "{html}");
}

#[test]
fn hostile_name_escaped() {
    let html = render_team_page(
        &enabled(vec![member(r"<img src=x onerror=alert(1)>", None, None, None)]),
        "team",
        "",
    );

    assert!(!html.contains("<img src=x"), "{html}");
    assert!(!html.contains("<img "), "{html}");
    assert!(!html.contains(r#"onerror=""#), "{html}");
    assert!(html.contains("&lt;img src=x onerror=alert(1)&gt;"), "{html}");
}

#[test]
fn data_avatar_rejected() {
    let html = render_team_page(
        &enabled(vec![member("Ada", None, Some("data:image/svg+xml,<svg>"), None)]),
        "team",
        "",
    );

    assert!(!html.contains("data:"), "{html}");
    assert!(!html.contains("<svg"), "{html}");
    assert!(!html.contains("<img"), "{html}");
}

#[test]
fn protocol_relative_avatar_and_link_rejected() {
    let html = render_team_page(
        &enabled(vec![member(
            "Ada",
            None,
            Some("//evil.example/ada.png"),
            Some(vec![link("CDN", "//evil.example/profile")]),
        )]),
        "team",
        "",
    );

    assert!(!html.contains("//evil.example"), "{html}");
    assert!(!html.contains("<img"), "{html}");
    assert!(!html.contains("CDN"), "{html}");
}

#[test]
fn http_link_rejected() {
    let html = render_team_page(
        &enabled(vec![member(
            "Ada",
            None,
            None,
            Some(vec![link("Blog", "http://blog.example/ada")]),
        )]),
        "team",
        "",
    );

    assert!(!html.contains("http://"), "{html}");
    assert!(!html.contains("blog.example"), "{html}");
    assert!(!html.contains("Blog"), "{html}");
}

#[test]
fn hostile_role_and_label_escaped() {
    let html = render_team_page(
        &enabled(vec![member(
            "Ada",
            Some("<script>alert(1)</script>"),
            None,
            Some(vec![link(r#"" onclick="alert(1)"#, "https://example.com")]),
        )]),
        "team",
        "",
    );

    assert!(!html.contains("<script>"), "{html}");
    assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"), "{html}");
    assert!(!html.contains(r#"onclick="alert(1)""#), "{html}");
    assert!(html.contains("&quot; onclick=&quot;alert(1)"), "{html}");
    assert!(html.contains(r#"href="https://example.com""#), "{html}");
}

#[test]
fn hostile_safe_path_is_attribute_escaped() {
    let html = render_team_page(
        &enabled(vec![member(
            "Ada",
            None,
            Some(r#"/avatars/ada.png" onerror="alert(1)"#),
            Some(vec![link("Local", r#"/people/ada" onclick="alert(1)"#)]),
        )]),
        "team",
        "",
    );

    assert!(!html.contains(r#"onerror="alert(1)""#), "{html}");
    assert!(!html.contains(r#"onclick="alert(1)""#), "{html}");
    assert!(html.contains("/avatars/ada.png&quot; onerror=&quot;alert(1)"), "{html}");
    assert!(html.contains("/people/ada&quot; onclick=&quot;alert(1)"), "{html}");
}

#[test]
fn empty_body_is_cards_only() {
    let html = render_team_page(&enabled(sample_members()), "team", "   ");
    assert!(html.contains(r#"class="ox-team""#), "{html}");
    assert!(!html.contains("<article"), "{html}");
    assert!(!html.trim_start().starts_with("<p>"), "{html}");
}
