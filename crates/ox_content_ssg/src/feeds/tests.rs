use super::{FeedItem, FeedsOptions, generate_feeds};

fn item(title: &str, loc: &str, description: Option<&str>, date: Option<&str>) -> FeedItem {
    FeedItem {
        title: title.to_string(),
        loc: loc.to_string(),
        description: description.map(str::to_string),
        date: date.map(str::to_string),
    }
}

fn enabled(site_url: Option<&str>) -> FeedsOptions {
    FeedsOptions {
        enabled: true,
        site_url: site_url.map(str::to_string),
        site_name: "Docs".to_string(),
        site_description: Some("Example docs".to_string()),
        home_page_url: "https://example.com/".to_string(),
        feed_rss_loc: "https://example.com/feed.xml".to_string(),
        feed_atom_loc: "https://example.com/atom.xml".to_string(),
        feed_json_loc: "https://example.com/feed.json".to_string(),
        rss: true,
        atom: true,
        json: true,
        limit: 20,
    }
}

fn sample_items() -> Vec<FeedItem> {
    vec![
        item("Old Post", "https://example.com/old/", Some("Earlier"), Some("2024-01-15")),
        item("New Post", "https://example.com/new/", Some("Latest"), Some("2024-03-01")),
    ]
}

#[test]
fn disabled_by_default() {
    let output = generate_feeds(&FeedsOptions::default(), &sample_items());

    assert_eq!(output, super::FeedsOutput::default());
}

#[test]
fn missing_site_url_warns_and_writes_nothing() {
    let output = generate_feeds(&enabled(None), &sample_items());

    assert_eq!(output.rss_xml, None);
    assert_eq!(output.atom_xml, None);
    assert_eq!(output.json_feed, None);
    assert_eq!(
        output.warning.as_deref(),
        Some(
            "[ox-content] feeds is enabled but ssg.siteUrl is not set; feed.xml, atom.xml, and feed.json were not written"
        )
    );
}

#[test]
fn blank_site_url_is_treated_as_missing() {
    let output = generate_feeds(&enabled(Some("   ")), &sample_items());

    assert_eq!(output.rss_xml, None);
    assert!(output.warning.is_some(), "blank siteUrl must warn: {output:?}");
}

#[test]
fn happy_path_rss_atom_json() {
    let output = generate_feeds(&enabled(Some("https://example.com")), &sample_items());
    let rss = output.rss_xml.expect("rss");
    let atom = output.atom_xml.expect("atom");
    let json = output.json_feed.expect("json");

    assert!(rss.contains("<rss version=\"2.0\">"), "{rss}");
    assert!(rss.contains("<title>Docs</title>"), "{rss}");
    assert!(rss.contains("<link>https://example.com/</link>"), "{rss}");
    assert!(rss.contains("<title>New Post</title>"), "{rss}");
    assert!(rss.contains("<link>https://example.com/new/</link>"), "{rss}");
    assert!(rss.contains("<description>Latest</description>"), "{rss}");
    assert!(rss.contains("<pubDate>Fri, 01 Mar 2024 00:00:00 +0000</pubDate>"), "{rss}");
    assert!(rss.contains("<pubDate>Mon, 15 Jan 2024 00:00:00 +0000</pubDate>"), "{rss}");
    assert!(rss.find("New Post").expect("new") < rss.find("Old Post").expect("old"), "{rss}");

    assert!(atom.contains("<feed xmlns=\"http://www.w3.org/2005/Atom\">"), "{atom}");
    assert!(atom.contains("<title>Docs</title>"), "{atom}");
    assert!(atom.contains("rel=\"self\""), "{atom}");
    assert!(atom.contains("https://example.com/atom.xml"), "{atom}");
    assert!(atom.contains("<updated>2024-03-01T00:00:00Z</updated>"), "{atom}");
    assert!(atom.contains("<title>New Post</title>"), "{atom}");
    assert!(atom.contains("<summary>Latest</summary>"), "{atom}");

    assert!(json.contains("\"version\": \"https://jsonfeed.org/version/1.1\""), "{json}");
    assert!(json.contains("\"title\": \"Docs\""), "{json}");
    assert!(json.contains("\"home_page_url\": \"https://example.com/\""), "{json}");
    assert!(json.contains("\"feed_url\": \"https://example.com/feed.json\""), "{json}");
    assert!(json.contains("\"title\": \"New Post\""), "{json}");
    assert!(json.contains("\"date_published\": \"2024-03-01T00:00:00Z\""), "{json}");
    assert_eq!(output.warning, None);
}

#[test]
fn limit_truncates() {
    let items = vec![
        item("A", "https://example.com/a/", None, Some("2024-01-01")),
        item("B", "https://example.com/b/", None, Some("2024-03-01")),
        item("C", "https://example.com/c/", None, Some("2024-02-01")),
    ];
    let options = FeedsOptions { limit: 1, ..enabled(Some("https://example.com")) };
    let output = generate_feeds(&options, &items);
    let rss = output.rss_xml.expect("rss");

    assert!(rss.contains("<title>B</title>"), "{rss}");
    assert!(!rss.contains("<title>A</title>"), "{rss}");
    assert!(!rss.contains("<title>C</title>"), "{rss}");
    assert_eq!(rss.matches("<item>").count(), 1);
    assert_eq!(output.json_feed.expect("json").matches("\"id\":").count(), 1);
}

#[test]
fn sort_by_date_descending() {
    let items = vec![
        item("Zeta", "https://example.com/zeta/", None, None),
        item("Old", "https://example.com/old/", None, Some("2024-01-15")),
        item("Alpha", "https://example.com/alpha/", None, Some("not-a-date")),
        item("New", "https://example.com/new/", None, Some("2024-03-01T12:00:00Z")),
        item("Mid", "https://example.com/mid/", None, Some("2024-02-01")),
    ];
    let output = generate_feeds(&enabled(Some("https://example.com")), &items);
    let rss = output.rss_xml.expect("rss");
    let new = rss.find("<title>New</title>").expect("new");
    let mid = rss.find("<title>Mid</title>").expect("mid");
    let old = rss.find("<title>Old</title>").expect("old");
    let alpha = rss.find("<title>Alpha</title>").expect("alpha");
    let zeta = rss.find("<title>Zeta</title>").expect("zeta");

    assert!(new < mid && mid < old && old < alpha && alpha < zeta, "{rss}");
}

#[test]
fn hostile_title_and_description_escaped() {
    let items = vec![item(
        "</title></item><script>alert(1)</script>",
        "https://example.com/x?a=1&b=2<>\"'",
        Some("\">\n<img src=x onerror=alert(1)>&amp;"),
        Some("2024-03-01"),
    )];
    let output = generate_feeds(&enabled(Some("https://example.com")), &items);
    let rss = output.rss_xml.expect("rss");
    let atom = output.atom_xml.expect("atom");
    let json = output.json_feed.expect("json");

    for body in [&rss, &atom] {
        assert!(!body.contains("<script>"), "{body}");
        assert!(!body.contains("</title></item><script>"), "{body}");
        assert!(body.contains("&lt;"), "{body}");
        assert!(body.contains("&amp;"), "{body}");
        assert!(body.contains("&quot;"), "{body}");
    }
    assert!(!json.contains("<script>"), "{json}");
    assert!(!json.contains("<img"), "{json}");
    assert!(json.contains("\\u003c"), "{json}");
}

#[test]
fn object_overrides_can_disable_formats() {
    let options = FeedsOptions {
        rss: false,
        atom: false,
        json: true,
        ..enabled(Some("https://example.com"))
    };
    let output = generate_feeds(&options, &sample_items());

    assert_eq!(output.rss_xml, None);
    assert_eq!(output.atom_xml, None);
    assert!(output.json_feed.is_some(), "{output:?}");
    assert_eq!(output.warning, None);
}

#[test]
fn empty_loc_items_are_omitted() {
    let items = vec![
        item("Missing", "", None, Some("2024-03-01")),
        item("Kept", "https://example.com/kept/", None, Some("2024-01-01")),
    ];
    let output = generate_feeds(&enabled(Some("https://example.com")), &items);
    let rss = output.rss_xml.expect("rss");

    assert!(rss.contains("Kept"), "{rss}");
    assert!(!rss.contains("Missing"), "{rss}");
}
