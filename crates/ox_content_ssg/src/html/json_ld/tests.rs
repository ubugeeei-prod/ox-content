use super::super::{
    A11y, HeadValidation, JsonLd, JsonLdPublisher, NavGroup, NavItem, PageChromeFlags, PageData,
    ReaderChrome, SsgConfig, generate_html,
};

fn nav_item(title: &str, path: &str, href: &str) -> NavItem {
    NavItem {
        title: title.to_string(),
        path: path.to_string(),
        href: href.to_string(),
        children: vec![],
        collapsed: None,
        sticky_collapsed: None,
    }
}

fn nav_item_with_children(title: &str, path: &str, href: &str, children: Vec<NavItem>) -> NavItem {
    NavItem {
        title: title.to_string(),
        path: path.to_string(),
        href: href.to_string(),
        children,
        collapsed: None,
        sticky_collapsed: None,
    }
}

fn nested_nav() -> Vec<NavGroup> {
    vec![NavGroup {
        title: "Guide".to_string(),
        items: vec![nav_item_with_children(
            "Features",
            "features",
            "/docs/features/index.html",
            vec![nav_item("Getting Started", "guide", "/docs/guide/index.html")],
        )],
        collapsed: None,
        sticky_collapsed: None,
    }]
}

fn page(title: &str) -> PageData {
    PageData {
        title: title.to_string(),
        description: Some("Learn the basics".to_string()),
        content: "<p>Body</p>".to_string(),
        toc: vec![],
        last_updated: None,
        contributors: vec![],
        path: "guide".to_string(),
        entry_page: None,
        prev: None,
        next: None,
        breadcrumbs: None,
        chrome: PageChromeFlags::default(),
        robots: None,
        canonical: None,
    }
}

fn config(json_ld: JsonLd, breadcrumbs: bool) -> SsgConfig {
    SsgConfig {
        site_name: "Docs".to_string(),
        base: "/docs/".to_string(),
        breadcrumb_root_href: None,
        og_image: None,
        theme: None,
        locale: None,
        available_locales: None,
        pagination: false,
        breadcrumbs,
        reader_chrome: ReaderChrome::default(),
        locale_switcher: false,
        locale_paths: vec![],
        a11y: A11y::default(),
        page_chrome: false,
        json_ld,
        site_url: None,
        head_validation: HeadValidation::Off,
    }
}

fn json_ld_script(html: &str) -> Option<&str> {
    let start_tag = r#"<script type="application/ld+json">"#;
    let start = html.find(start_tag)?;
    let rest = &html[start + start_tag.len()..];
    let end = rest.find("</script>")?;
    Some(&rest[..end])
}

fn parse_json_ld(html: &str) -> serde_json::Value {
    let raw = json_ld_script(html).expect("expected JSON-LD script");
    serde_json::from_str(raw)
        .unwrap_or_else(|err| panic!("JSON-LD must be valid JSON: {err}: {raw}"))
}

fn graph_types(value: &serde_json::Value) -> Vec<&str> {
    value["@graph"]
        .as_array()
        .expect("@graph")
        .iter()
        .filter_map(|node| node["@type"].as_str())
        .collect()
}

#[test]
fn disabled_by_default() {
    let html =
        generate_html(&page("Getting Started"), &nested_nav(), &config(JsonLd::default(), false));

    assert!(json_ld_script(&html).is_none(), "jsonLd must be off by default: {html}");
    assert!(!html.contains("application/ld+json"), "{html}");
    assert!(!html.contains("TechArticle"), "{html}");
    assert!(!html.contains("WebSite"), "{html}");
    assert!(!html.contains("BreadcrumbList"), "{html}");
}

#[test]
fn happy_path_tech_article_and_website() {
    let mut json_ld = JsonLd::enabled();
    json_ld.site_url = Some("https://docs.example".to_string());
    let html = generate_html(&page("Getting Started"), &nested_nav(), &config(json_ld, false));

    let raw = json_ld_script(&html).expect("enabled jsonLd should emit a script");
    assert!(
        html.contains(r#"<meta name="twitter:title" content="Getting Started - Docs">"#),
        "JSON-LD must follow existing OG tags: {html}"
    );
    let twitter = html.find(r#"name="twitter:title""#).expect("twitter title");
    let script = html.find(r"application/ld+json").expect("json-ld");
    let styles = html.find("<!-- ox-content:styles:start -->").expect("styles");
    assert!(twitter < script && script < styles, "JSON-LD placement is wrong: {html}");

    let value = parse_json_ld(&html);
    assert_eq!(value["@context"], "https://schema.org");
    let types = graph_types(&value);
    assert_eq!(types, ["WebSite", "TechArticle"], "{raw}");

    let website = &value["@graph"][0];
    assert_eq!(website["name"], "Docs");
    assert_eq!(website["url"], "https://docs.example");
    assert_eq!(website["@id"], "https://docs.example#website");

    let article = &value["@graph"][1];
    assert_eq!(article["headline"], "Getting Started");
    assert_eq!(article["description"], "Learn the basics");
    assert_eq!(article["url"], "https://docs.example/docs/guide/");
    assert_eq!(article["@id"], "https://docs.example/docs/guide/#article");
    assert_eq!(article["isPartOf"]["@id"], "https://docs.example#website");
    assert!(article.get("publisher").is_none(), "happy path must not invent a publisher: {raw}");
    assert!(!types.contains(&"BreadcrumbList"), "breadcrumbs are off: {raw}");
}

#[test]
fn breadcrumb_list_only_when_breadcrumbs_and_json_ld_allow_it() {
    let mut with_trail = JsonLd::enabled();
    with_trail.site_url = Some("https://docs.example".to_string());
    let html =
        generate_html(&page("Getting Started"), &nested_nav(), &config(with_trail.clone(), true));
    let value = parse_json_ld(&html);
    assert!(
        graph_types(&value).contains(&"BreadcrumbList"),
        "BreadcrumbList should emit when both trails exist: {}",
        json_ld_script(&html).unwrap_or("")
    );
    let crumbs = value["@graph"]
        .as_array()
        .unwrap()
        .iter()
        .find(|node| node["@type"] == "BreadcrumbList")
        .expect("BreadcrumbList node");
    let items = crumbs["itemListElement"].as_array().expect("itemListElement");
    assert!(items.len() >= 2, "{crumbs}");
    assert_eq!(items[0]["@type"], "ListItem");
    assert_eq!(items[0]["position"], 1);
    assert_eq!(items[0]["name"], "Docs");
    assert_eq!(items[0]["item"], "https://docs.example/docs/index.html");

    let mut json_ld_hides_crumbs = JsonLd::enabled();
    json_ld_hides_crumbs.breadcrumbs = false;
    json_ld_hides_crumbs.site_url = Some("https://docs.example".to_string());
    let html =
        generate_html(&page("Getting Started"), &nested_nav(), &config(json_ld_hides_crumbs, true));
    assert!(html.contains(r#"class="ox-breadcrumbs""#), "visible trail must still render: {html}");
    assert!(
        !graph_types(&parse_json_ld(&html)).contains(&"BreadcrumbList"),
        "jsonLd.breadcrumbs: false must omit BreadcrumbList: {}",
        json_ld_script(&html).unwrap_or("")
    );

    let html = generate_html(&page("Getting Started"), &nested_nav(), &config(with_trail, false));
    assert!(
        !graph_types(&parse_json_ld(&html)).contains(&"BreadcrumbList"),
        "no BreadcrumbList without a visible trail: {}",
        json_ld_script(&html).unwrap_or("")
    );
}

#[test]
fn hostile_title_escaped() {
    let mut json_ld = JsonLd::enabled();
    json_ld.site_url = Some("https://docs.example".to_string());
    let html =
        generate_html(&page("<script>alert(1)</script>"), &nested_nav(), &config(json_ld, false));
    let raw = json_ld_script(&html).expect("hostile title should still emit JSON-LD");

    assert!(!raw.contains("<script>"), "raw <script> must not appear inside JSON-LD: {raw}");
    assert!(!raw.contains("</script>"), "raw </script> would break out of the script tag: {raw}");
    assert!(
        raw.contains(r"\u003cscript\u003ealert(1)\u003c/script\u003e"),
        "title must be JSON-encoded and HTML-script-safe: {raw}"
    );
    assert!(!html.contains("<script>alert(1)</script>"), "{html}");
    assert_eq!(html.matches(r#"<script type="application/ld+json">"#).count(), 1);

    let value = parse_json_ld(&html);
    assert_eq!(value["@graph"][1]["headline"], "<script>alert(1)</script>");
}

#[test]
fn no_publisher_invented() {
    let html =
        generate_html(&page("Getting Started"), &nested_nav(), &config(JsonLd::enabled(), false));
    let raw = json_ld_script(&html).expect("enabled jsonLd should emit a script");
    let value = parse_json_ld(&html);

    assert!(raw.contains("TechArticle"), "{raw}");
    assert!(raw.contains("WebSite"), "{raw}");
    assert!(!raw.contains("publisher"), "{raw}");
    assert!(!raw.contains("logo"), "{raw}");
    assert!(!raw.contains("Organization"), "{raw}");
    assert!(
        value["@graph"][0].get("url").is_none(),
        "missing siteUrl must not invent WebSite url: {raw}"
    );
    assert!(
        value["@graph"][1].get("url").is_none(),
        "missing siteUrl must not invent TechArticle url: {raw}"
    );
    assert!(value["@graph"][0].get("@id").is_none(), "{raw}");
    assert!(value["@graph"][1].get("@id").is_none(), "{raw}");
    assert!(value["@graph"][1].get("isPartOf").is_none(), "{raw}");
}

#[test]
fn themed_canonical_only_when_site_url_is_set() {
    let html =
        generate_html(&page("Getting Started"), &nested_nav(), &config(JsonLd::default(), false));
    assert!(!html.contains("rel=\"canonical\""), "{html}");

    let mut with_url = config(JsonLd::default(), false);
    with_url.site_url = Some("https://docs.example".into());
    let html = generate_html(&page("Getting Started"), &nested_nav(), &with_url);
    assert!(
        html.contains("<link rel=\"canonical\" href=\"https://docs.example/docs/guide/\">"),
        "{html}"
    );
    assert!(
        html.contains("<meta property=\"og:url\" content=\"https://docs.example/docs/guide/\">"),
        "{html}"
    );
}

#[test]
fn page_type_and_extra_graph_nodes() {
    let mut json_ld = JsonLd::enabled();
    json_ld.page_type = Some("BlogPosting".into());
    json_ld.graph = vec![serde_json::json!({"@type": "Person", "name": "Ada"})];
    let html = generate_html(&page("Getting Started"), &nested_nav(), &config(json_ld, false));
    let value = parse_json_ld(&html);
    let types = graph_types(&value);
    assert_eq!(types, ["WebSite", "BlogPosting", "Person"], "{html}");
}

#[test]
fn configured_publisher_is_emitted_and_javascript_urls_are_rejected() {
    let mut json_ld = JsonLd::enabled();
    json_ld.publisher = Some(JsonLdPublisher {
        name: Some("Acme Docs".to_string()),
        url: Some("https://acme.example".to_string()),
    });
    let html = generate_html(&page("Getting Started"), &nested_nav(), &config(json_ld, false));
    let article = &parse_json_ld(&html)["@graph"][1];
    assert_eq!(article["publisher"]["@type"], "Organization");
    assert_eq!(article["publisher"]["name"], "Acme Docs");
    assert_eq!(article["publisher"]["url"], "https://acme.example");
    assert!(article["publisher"].get("logo").is_none());

    let mut hostile = JsonLd::enabled();
    hostile.publisher = Some(JsonLdPublisher {
        name: Some("Evil".to_string()),
        url: Some("javascript:alert(1)".to_string()),
    });
    let html = generate_html(&page("Getting Started"), &nested_nav(), &config(hostile, false));
    let raw = json_ld_script(&html).expect("name-only publisher should still emit");
    assert!(!raw.contains("javascript:"), "{raw}");
    let publisher = &parse_json_ld(&html)["@graph"][1]["publisher"];
    assert_eq!(publisher["name"], "Evil");
    assert!(publisher.get("url").is_none(), "{publisher}");
}
