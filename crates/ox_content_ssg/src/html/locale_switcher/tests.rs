use super::super::{
    A11y, LocaleInfo, LocalePath, NavGroup, NavItem, PageChromeFlags, PageData, ReaderChrome,
    SsgConfig, generate_bare_html, generate_html,
};

fn page(path: &str) -> PageData {
    PageData {
        title: "Guide".to_string(),
        description: None,
        content: "<p>Body</p>".to_string(),
        toc: vec![],
        last_updated: None,
        path: path.to_string(),
        entry_page: None,
        prev: None,
        next: None,
        breadcrumbs: None,
        chrome: PageChromeFlags::default(),
    }
}

fn nav() -> Vec<NavGroup> {
    vec![NavGroup {
        title: "Guide".to_string(),
        items: vec![NavItem {
            title: "Guide".to_string(),
            path: "guide".to_string(),
            href: "/docs/guide/index.html".to_string(),
            children: vec![],
            collapsed: None,
            sticky_collapsed: None,
        }],
        collapsed: None,
        sticky_collapsed: None,
    }]
}

fn locale(code: &str, name: &str, dir: &str) -> LocaleInfo {
    LocaleInfo { code: code.to_string(), name: name.to_string(), dir: dir.to_string() }
}

fn locale_path(code: &str, href: Option<&str>, root: Option<&str>) -> LocalePath {
    LocalePath {
        code: code.to_string(),
        href: href.map(ToOwned::to_owned),
        root: root.map(ToOwned::to_owned),
    }
}

fn locales() -> Vec<LocaleInfo> {
    vec![locale("en", "English", "ltr"), locale("ja", "日本語", "ltr")]
}

fn config(
    locale_switcher: bool,
    current: Option<&str>,
    available: Option<Vec<LocaleInfo>>,
    locale_paths: Vec<LocalePath>,
) -> SsgConfig {
    SsgConfig {
        site_name: "Docs".to_string(),
        base: "/docs/".to_string(),
        og_image: None,
        theme: None,
        locale: current.map(ToOwned::to_owned),
        available_locales: available,
        pagination: false,
        breadcrumbs: false,
        reader_chrome: ReaderChrome::default(),
        locale_switcher,
        locale_paths,
        a11y: A11y::default(),
        page_chrome: false,
    }
}

fn render(
    locale_switcher: bool,
    current: Option<&str>,
    available: Option<Vec<LocaleInfo>>,
    locale_paths: Vec<LocalePath>,
) -> String {
    generate_html(
        &page("ja/guide"),
        &nav(),
        &config(locale_switcher, current, available, locale_paths),
    )
}

fn switcher_html(html: &str) -> Option<&str> {
    let start = html.find(r#"<nav class="ox-locale-switcher""#)?;
    let rest = &html[start..];
    let end = rest.find("</nav>")?;
    Some(&rest[..end + "</nav>".len()])
}

fn html_open_tag(html: &str) -> &str {
    let start = html.find("<html").expect("generated page must have an html tag");
    let end = html[start..].find('>').expect("html tag must close");
    &html[start..=start + end]
}

#[test]
fn disabled_by_default() {
    let html = render(
        false,
        Some("ja"),
        Some(locales()),
        vec![
            locale_path("en", Some("/docs/guide/index.html"), Some("/docs/")),
            locale_path("ja", Some("/docs/ja/guide/index.html"), Some("/docs/ja/")),
        ],
    );

    assert!(switcher_html(&html).is_none(), "omitted/false must not emit a switcher: {html}");
    assert!(!html.contains(r#"<nav class="ox-locale-switcher""#), "{html}");
    assert!(!html.contains(r#"aria-label="Language""#), "{html}");
    let open = html_open_tag(&html);
    assert!(open.contains(r#"lang="ja""#), "{open}");
    assert!(open.contains(r#"dir="ltr""#), "{open}");
}

#[test]
fn happy_path_lists_locales_and_marks_current() {
    let html = render(
        true,
        Some("ja"),
        Some(locales()),
        vec![
            locale_path("en", Some("/docs/guide/index.html"), Some("/docs/")),
            locale_path("ja", Some("/docs/ja/guide/index.html"), Some("/docs/ja/")),
        ],
    );

    let switcher = switcher_html(&html).expect("enabled switcher must render");
    assert!(
        switcher.starts_with(r#"<nav class="ox-locale-switcher" aria-label="Language">"#),
        "{switcher}"
    );
    assert!(switcher.contains(r#"href="/docs/guide/index.html""#), "{switcher}");
    assert!(switcher.contains(r#"href="/docs/ja/guide/index.html""#), "{switcher}");
    assert!(switcher.contains(">English<"), "{switcher}");
    assert!(switcher.contains(">日本語<"), "{switcher}");
    assert!(switcher.contains(r#"lang="en""#), "{switcher}");
    assert!(switcher.contains(r#"lang="ja""#), "{switcher}");
    assert!(
        switcher.contains(r#"lang="ja""#) && switcher.contains(r#"aria-current="page""#),
        "current locale must be marked: {switcher}"
    );
    let ja = switcher.split("日本語").next().expect("ja link");
    assert!(ja.contains(r#"aria-current="page""#), "{switcher}");
    let en = switcher.split("English").next().expect("en link");
    assert!(!en.contains(r#"aria-current="page""#), "{switcher}");
}

#[test]
fn missing_sibling_falls_back_to_locale_root() {
    let html = render(
        true,
        Some("en"),
        Some(locales()),
        vec![
            locale_path("en", Some("/docs/guide/index.html"), Some("/docs/")),
            locale_path("ja", None, Some("/docs/ja/")),
        ],
    );

    let switcher = switcher_html(&html).expect("enabled switcher must render");
    assert!(switcher.contains(r#"href="/docs/guide/index.html""#), "{switcher}");
    assert!(switcher.contains(r#"href="/docs/ja/""#), "{switcher}");
    assert!(!switcher.contains("/docs/ja/guide"), "{switcher}");
}

#[test]
fn rtl_dir_honored() {
    let html = render(
        true,
        Some("ar"),
        Some(vec![locale("en", "English", "ltr"), locale("ar", "العربية", "rtl")]),
        vec![
            locale_path("en", Some("/docs/guide/index.html"), Some("/docs/")),
            locale_path("ar", Some("/docs/ar/guide/index.html"), Some("/docs/ar/")),
        ],
    );

    let switcher = switcher_html(&html).expect("enabled switcher must render");
    assert!(switcher.contains(r#"dir="rtl""#), "{switcher}");
    assert!(switcher.contains(r#"lang="ar""#), "{switcher}");
    assert!(switcher.contains(">العربية<"), "{switcher}");
    let open = html_open_tag(&html);
    assert!(open.contains(r#"lang="ar""#), "{open}");
    assert!(open.contains(r#"dir="rtl""#), "{open}");
}

#[test]
fn hostile_name_escaped() {
    let html = render(
        true,
        Some("en"),
        Some(vec![
            locale("en", "English", "ltr"),
            locale("xx", r"<img src=x onerror=alert(1)>", r#"rtl" onclick="alert(1)"#),
        ]),
        vec![
            locale_path("en", Some("/docs/guide/index.html"), None),
            locale_path("xx", Some("/docs/xx/guide/index.html"), None),
        ],
    );

    let switcher = switcher_html(&html).expect("enabled switcher must render");
    assert!(
        switcher.contains("&lt;img src=x onerror=alert(1)&gt;")
            || switcher.contains("&#60;img src=x onerror=alert(1)&#62;"),
        "{switcher}"
    );
    assert!(!switcher.contains("<img src=x onerror=alert(1)>"), "{switcher}");
    assert!(!switcher.contains(r#"onclick="alert(1)""#), "{switcher}");
    assert!(switcher.contains(r#"dir="ltr""#) || switcher.contains(r#"dir="rtl""#), "{switcher}");
}

#[test]
fn javascript_root_rejected() {
    let html = render(
        true,
        Some("en"),
        Some(locales()),
        vec![
            locale_path("en", Some("/docs/guide/index.html"), Some("/docs/")),
            locale_path("ja", None, Some("javascript:alert(1)")),
        ],
    );

    let switcher = switcher_html(&html).expect("enabled switcher must render");
    assert!(!switcher.contains("javascript:"), "{switcher}");
    assert!(!html.contains("javascript:"), "{html}");
    assert!(!switcher.contains(r#"href="javascript"#), "{switcher}");
    assert!(switcher.contains("日本語"), "{switcher}");
}

#[test]
fn empty_available_locales_emits_nothing() {
    let html = render(true, Some("ja"), Some(vec![]), vec![]);
    assert!(switcher_html(&html).is_none(), "{html}");
    assert!(!html.contains(r#"<nav class="ox-locale-switcher""#), "{html}");
}

#[test]
fn enabled_without_available_locales_emits_nothing() {
    let html = render(true, Some("ja"), None, vec![]);
    assert!(switcher_html(&html).is_none(), "{html}");
}

#[test]
fn missing_sibling_defaults_to_base_locale_root() {
    let html = render(true, Some("en"), Some(locales()), vec![]);
    let switcher = switcher_html(&html).expect("default roots must still render");
    assert!(switcher.contains(r#"href="/docs/en/""#), "{switcher}");
    assert!(switcher.contains(r#"href="/docs/ja/""#), "{switcher}");
}

#[test]
fn hostile_code_escaped() {
    let html = render(
        true,
        Some(r#"en" onclick="alert(1)"#),
        Some(vec![locale(r#"en" onclick="alert(1)""#, "English", "ltr")]),
        vec![locale_path(r#"en" onclick="alert(1)""#, Some("/docs/en/"), None)],
    );

    let switcher = switcher_html(&html).expect("enabled switcher must render");
    assert!(!switcher.contains(r#"onclick="alert(1)""#), "{switcher}");
    assert!(switcher.contains("lang="), "{switcher}");
}

#[test]
fn data_and_vbscript_roots_rejected() {
    let html = render(
        true,
        Some("en"),
        Some(vec![locale("en", "English", "ltr"), locale("xx", "Other", "ltr")]),
        vec![
            locale_path("en", None, Some("data:text/html,hi")),
            locale_path("xx", None, Some("vbscript:msgbox(1)")),
        ],
    );

    assert!(!html.contains("data:text/html"), "{html}");
    assert!(!html.contains("vbscript:"), "{html}");
}

#[test]
fn generate_bare_html_is_unchanged() {
    let html = generate_bare_html("<h1>Hello</h1>", "Test Page");
    assert!(!html.contains(r#"<nav class="ox-locale-switcher""#), "{html}");
    assert_eq!(
        html,
        "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n  <meta charset=\"UTF-8\">\n  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n  <title>Test Page</title>\n</head>\n<body>\n<h1>Hello</h1>\n</body>\n</html>"
    );
}
