use super::{FeedFormat, FeedItem, FeedsOptions, generate_feeds};

fn item(
    title: &str,
    loc: &str,
    description: Option<&str>,
    date: Option<&str>,
    last_updated: Option<&str>,
) -> FeedItem {
    FeedItem {
        title: title.to_string(),
        description: description.map(str::to_string),
        loc: loc.to_string(),
        date: date.map(str::to_string),
        last_updated: last_updated.map(str::to_string),
        draft: false,
        unlisted: false,
        ..FeedItem::default()
    }
}

fn draft_item(title: &str, loc: &str, date: &str) -> FeedItem {
    FeedItem { draft: true, ..item(title, loc, None, Some(date), None) }
}

fn unlisted_item(title: &str, loc: &str, date: &str) -> FeedItem {
    FeedItem { unlisted: true, ..item(title, loc, None, Some(date), None) }
}

fn enabled(site_url: Option<&str>) -> FeedsOptions {
    FeedsOptions {
        enabled: true,
        site_url: site_url.map(str::to_string),
        site_name: "Docs".to_string(),
        site_description: Some("Example docs".to_string()),
        home_page_url: "https://example.com/".to_string(),
        rss_url: "https://example.com/feed.xml".to_string(),
        atom_url: "https://example.com/atom.xml".to_string(),
        json_url: "https://example.com/feed.json".to_string(),
        formats: vec![FeedFormat::Rss, FeedFormat::Atom, FeedFormat::Json],
        limit: 20,
        ..FeedsOptions::default()
    }
}

fn sample_items() -> Vec<FeedItem> {
    vec![
        item(
            "Older",
            "https://example.com/blog/older/",
            Some("First post"),
            Some("2024-01-01"),
            None,
        ),
        item(
            "Newer",
            "https://example.com/blog/newer/",
            Some("Second post"),
            Some("2024-02-01"),
            None,
        ),
    ]
}

#[test]
fn disabled_by_default_writes_nothing() {
    let output = generate_feeds(&FeedsOptions::default(), &sample_items());

    assert_eq!(output, super::FeedsOutput::default());
}

#[test]
fn enabled_without_site_url_warns_and_writes_nothing() {
    let output = generate_feeds(&enabled(None), &sample_items());

    assert_eq!(output.rss_xml, None);
    assert_eq!(output.atom_xml, None);
    assert_eq!(output.json_feed, None);
    assert_eq!(
        output.warning.as_deref(),
        Some(
            "[ox-content] feeds is enabled but ssg.siteUrl is not set; RSS, Atom, and JSON feeds were not written"
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
fn happy_path_writes_sorted_rss_atom_and_json() {
    let output = generate_feeds(&enabled(Some("https://example.com")), &sample_items());

    assert_eq!(
        output.rss_xml.as_deref(),
        Some(
            "\
<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<rss version=\"2.0\">
  <channel>
    <title>Docs</title>
    <link>https://example.com/</link>
    <description>Example docs</description>
    <item>
      <title>Newer</title>
      <link>https://example.com/blog/newer/</link>
      <guid>https://example.com/blog/newer/</guid>
      <description>Second post</description>
      <pubDate>Thu, 01 Feb 2024 00:00:00 +0000</pubDate>
    </item>
    <item>
      <title>Older</title>
      <link>https://example.com/blog/older/</link>
      <guid>https://example.com/blog/older/</guid>
      <description>First post</description>
      <pubDate>Mon, 01 Jan 2024 00:00:00 +0000</pubDate>
    </item>
  </channel>
</rss>
"
        )
    );
    assert_eq!(
        output.atom_xml.as_deref(),
        Some(
            "\
<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<feed xmlns=\"http://www.w3.org/2005/Atom\">
  <title>Docs</title>
  <link href=\"https://example.com/atom.xml\" rel=\"self\"/>
  <link href=\"https://example.com/\" rel=\"alternate\"/>
  <id>https://example.com/</id>
  <updated>2024-02-01T00:00:00Z</updated>
  <subtitle>Example docs</subtitle>
  <entry>
    <title>Newer</title>
    <link href=\"https://example.com/blog/newer/\"/>
    <id>https://example.com/blog/newer/</id>
    <updated>2024-02-01T00:00:00Z</updated>
    <summary>Second post</summary>
  </entry>
  <entry>
    <title>Older</title>
    <link href=\"https://example.com/blog/older/\"/>
    <id>https://example.com/blog/older/</id>
    <updated>2024-01-01T00:00:00Z</updated>
    <summary>First post</summary>
  </entry>
</feed>
"
        )
    );
    assert_eq!(
        output.json_feed.as_deref(),
        Some(
            "\
{
  \"version\": \"https://jsonfeed.org/version/1.1\",
  \"title\": \"Docs\",
  \"home_page_url\": \"https://example.com/\",
  \"feed_url\": \"https://example.com/feed.json\",
  \"description\": \"Example docs\",
  \"items\": [
    {
      \"id\": \"https://example.com/blog/newer/\",
      \"url\": \"https://example.com/blog/newer/\",
      \"title\": \"Newer\",
      \"content_text\": \"Second post\",
      \"date_published\": \"2024-02-01T00:00:00Z\"
    },
    {
      \"id\": \"https://example.com/blog/older/\",
      \"url\": \"https://example.com/blog/older/\",
      \"title\": \"Older\",
      \"content_text\": \"First post\",
      \"date_published\": \"2024-01-01T00:00:00Z\"
    }
  ]
}
"
        )
    );
    assert_eq!(output.warning, None);
}

#[test]
fn object_overrides_can_limit_formats() {
    let options = FeedsOptions {
        formats: vec![FeedFormat::Rss],
        limit: 1,
        ..enabled(Some("https://example.com"))
    };
    let output = generate_feeds(&options, &sample_items());

    assert!(output.rss_xml.is_some(), "rss stays on: {output:?}");
    assert!(output.rss_xml.as_deref().is_some_and(|rss| rss.contains("Newer")), "{output:?}");
    assert!(!output.rss_xml.as_deref().is_some_and(|rss| rss.contains("Older")), "{output:?}");
    assert_eq!(output.atom_xml, None);
    assert_eq!(output.json_feed, None);
    assert_eq!(output.warning, None);
}

#[test]
fn draft_items_are_omitted() {
    let items = vec![
        draft_item("Secret", "https://example.com/secret/", "2024-03-01"),
        item("Public", "https://example.com/public/", None, Some("2024-01-01"), None),
    ];
    let output = generate_feeds(&enabled(Some("https://example.com")), &items);
    let rss = output.rss_xml.expect("published items should still emit rss");
    let atom = output.atom_xml.expect("published items should still emit atom");
    let json = output.json_feed.expect("published items should still emit json");

    assert!(rss.contains("https://example.com/public/"), "{rss}");
    assert!(!rss.contains("secret"), "{rss}");
    assert!(atom.contains("Public"), "{atom}");
    assert!(!atom.contains("Secret"), "{atom}");
    assert!(json.contains("Public"), "{json}");
    assert!(!json.contains("Secret"), "{json}");
}

#[test]
fn unlisted_items_are_omitted() {
    let items = vec![
        unlisted_item("Hidden", "https://example.com/hidden/", "2024-03-01"),
        item("Public", "https://example.com/public/", None, Some("2024-01-01"), None),
    ];
    let output = generate_feeds(&enabled(Some("https://example.com")), &items);
    let rss = output.rss_xml.expect("published items should still emit rss");

    assert!(rss.contains("https://example.com/public/"), "{rss}");
    assert!(!rss.contains("hidden"), "{rss}");
}

#[test]
fn hostile_title_and_description_cannot_break_out() {
    let items = vec![item(
        "</title></channel><script>alert(1)</script>",
        "https://example.com/x?a=1&b=2<>\"'",
        Some("\">\n<img src=x onerror=alert(1)>"),
        Some("2024-01-01"),
        None,
    )];
    let output = generate_feeds(&enabled(Some("https://example.com")), &items);
    let rss = output.rss_xml.expect("hostile input must still emit rss");
    let atom = output.atom_xml.expect("hostile input must still emit atom");
    let json = output.json_feed.expect("hostile input must still emit json");

    assert!(!rss.contains("<script>"), "{rss}");
    assert!(!rss.contains("</title></channel><script>"), "{rss}");
    assert!(rss.contains("&amp;"), "{rss}");
    assert!(rss.contains("&lt;"), "{rss}");
    assert!(rss.contains("&gt;"), "{rss}");
    assert!(!atom.contains("<script>"), "{atom}");
    assert!(!atom.contains("<img"), "{atom}");
    assert!(!json.contains("</title>"), "{json}");
    assert!(!json.contains("<script>"), "{json}");
    assert!(json.contains("\\n"), "{json}");
}

#[test]
fn sorts_by_date_then_last_updated_and_limits() {
    let items = vec![
        item("By lastUpdated", "https://example.com/c/", None, None, Some("2024-03-01")),
        item("Oldest date", "https://example.com/a/", None, Some("2024-01-01"), Some("2024-04-01")),
        item("Newest date", "https://example.com/b/", None, Some("2024-02-01"), None),
        item("Tied date z", "https://example.com/z/", None, Some("2024-02-01"), None),
        item("No date", "https://example.com/m/", None, None, None),
    ];
    let options = FeedsOptions { limit: 3, ..enabled(Some("https://example.com")) };
    let output = generate_feeds(&options, &items);
    let rss = output.rss_xml.expect("sorted items should emit rss");

    let last_updated = rss.find("https://example.com/c/").expect("lastUpdated");
    let newest = rss.find("https://example.com/b/").expect("newest");
    let tied = rss.find("https://example.com/z/").expect("tied");
    assert!(last_updated < newest && newest < tied, "{rss}");
    assert!(!rss.contains("https://example.com/a/"), "{rss}");
    assert!(!rss.contains("https://example.com/m/"), "{rss}");
}
