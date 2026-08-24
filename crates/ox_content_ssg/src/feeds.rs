//! Opt-in RSS, Atom, and JSON Feed bodies.

mod dates;

use dates::parse_date;

const MISSING_SITE_URL: &str = "[ox-content] feeds is enabled but ssg.siteUrl is not set; RSS, Atom, and JSON feeds were not written";

/// Feed formats that can be emitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedFormat {
    /// RSS 2.0 (`feed.xml`).
    Rss,
    /// Atom 1.0 (`atom.xml`).
    Atom,
    /// JSON Feed 1.1 (`feed.json`).
    Json,
}

/// One collection entry considered for a feed.
#[derive(Debug, Clone)]
pub struct FeedItem {
    /// Item title.
    pub title: String,
    /// Optional plain-text summary.
    pub description: Option<String>,
    /// Absolute item URL.
    pub loc: String,
    /// Preferred sort / publish date (`date` frontmatter).
    pub date: Option<String>,
    /// Fallback sort / publish date (`lastUpdated` frontmatter).
    pub last_updated: Option<String>,
    /// When true, the item is omitted from every generated file.
    pub draft: bool,
    /// When true, the item is omitted from listing surfaces (feeds).
    pub unlisted: bool,
}

/// Switches and site metadata for feed generation.
#[derive(Debug, Clone)]
pub struct FeedsOptions {
    /// When false, no files are generated.
    pub enabled: bool,
    /// Absolute site origin, required when the feature is on.
    pub site_url: Option<String>,
    /// Site title written to each feed.
    pub site_name: String,
    /// Optional site summary written to each feed.
    pub site_description: Option<String>,
    /// Absolute site home page URL.
    pub home_page_url: String,
    /// Absolute URL of the generated RSS file.
    pub rss_url: String,
    /// Absolute URL of the generated Atom file.
    pub atom_url: String,
    /// Absolute URL of the generated JSON Feed file.
    pub json_url: String,
    /// Formats to emit when the feature is on.
    pub formats: Vec<FeedFormat>,
    /// Maximum number of published items.
    pub limit: usize,
}

/// Generated feed bodies, or a warning when generation is skipped.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FeedsOutput {
    /// RSS 2.0 body.
    pub rss_xml: Option<String>,
    /// Atom 1.0 body.
    pub atom_xml: Option<String>,
    /// JSON Feed 1.1 body.
    pub json_feed: Option<String>,
    /// Non-fatal skip reason. Never used as a panic.
    pub warning: Option<String>,
}

impl Default for FeedsOptions {
    fn default() -> Self {
        Self {
            enabled: false,
            site_url: None,
            site_name: String::new(),
            site_description: None,
            home_page_url: String::new(),
            rss_url: String::new(),
            atom_url: String::new(),
            json_url: String::new(),
            formats: vec![FeedFormat::Rss, FeedFormat::Atom, FeedFormat::Json],
            limit: 20,
        }
    }
}

/// Builds RSS / Atom / JSON Feed bodies without writing files.
pub fn generate_feeds(options: &FeedsOptions, items: &[FeedItem]) -> FeedsOutput {
    if !options.enabled {
        return FeedsOutput::default();
    }
    if !has_site_url(options.site_url.as_deref()) {
        return FeedsOutput {
            warning: Some(MISSING_SITE_URL.to_string()),
            ..FeedsOutput::default()
        };
    }

    let published = published_items(items, options.limit);
    FeedsOutput {
        rss_xml: wants(options, FeedFormat::Rss).then(|| generate_rss(options, &published)),
        atom_xml: wants(options, FeedFormat::Atom).then(|| generate_atom(options, &published)),
        json_feed: wants(options, FeedFormat::Json).then(|| generate_json(options, &published)),
        warning: None,
    }
}

fn has_site_url(site_url: Option<&str>) -> bool {
    site_url.is_some_and(|value| !value.trim().is_empty())
}

fn wants(options: &FeedsOptions, format: FeedFormat) -> bool {
    options.formats.contains(&format)
}

fn published_items(items: &[FeedItem], limit: usize) -> Vec<&FeedItem> {
    let mut published: Vec<&FeedItem> =
        items.iter().filter(|item| !item.draft && !item.unlisted && !item.loc.is_empty()).collect();
    published.sort_by(|left, right| {
        sort_unix(right).cmp(&sort_unix(left)).then_with(|| left.loc.cmp(&right.loc))
    });
    published.truncate(limit);
    published
}

fn sort_unix(item: &FeedItem) -> i64 {
    item_date(item).map_or(i64::MIN, dates::ParsedDate::unix)
}

fn item_date(item: &FeedItem) -> Option<dates::ParsedDate> {
    parse_date(item.date.as_deref()).or_else(|| parse_date(item.last_updated.as_deref()))
}

fn channel_description(options: &FeedsOptions) -> &str {
    options
        .site_description
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(options.site_name.as_str())
}

fn generate_rss(options: &FeedsOptions, items: &[&FeedItem]) -> String {
    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<rss version=\"2.0\">\n  <channel>\n    <title>",
    );
    escape_xml(&options.site_name, &mut xml);
    xml.push_str("</title>\n    <link>");
    escape_xml(&options.home_page_url, &mut xml);
    xml.push_str("</link>\n    <description>");
    escape_xml(channel_description(options), &mut xml);
    xml.push_str("</description>\n");
    for item in items {
        xml.push_str("    <item>\n      <title>");
        escape_xml(&item.title, &mut xml);
        xml.push_str("</title>\n      <link>");
        escape_xml(&item.loc, &mut xml);
        xml.push_str("</link>\n      <guid>");
        escape_xml(&item.loc, &mut xml);
        xml.push_str("</guid>\n");
        if let Some(description) = item.description.as_deref().filter(|value| !value.is_empty()) {
            xml.push_str("      <description>");
            escape_xml(description, &mut xml);
            xml.push_str("</description>\n");
        }
        if let Some(date) = item_date(item) {
            xml.push_str("      <pubDate>");
            xml.push_str(&date.rfc822());
            xml.push_str("</pubDate>\n");
        }
        xml.push_str("    </item>\n");
    }
    xml.push_str("  </channel>\n</rss>\n");
    xml
}

fn generate_atom(options: &FeedsOptions, items: &[&FeedItem]) -> String {
    let updated = items
        .first()
        .and_then(|item| item_date(item))
        .map_or_else(|| "1970-01-01T00:00:00Z".to_string(), dates::ParsedDate::rfc3339);
    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<feed xmlns=\"http://www.w3.org/2005/Atom\">\n  <title>",
    );
    escape_xml(&options.site_name, &mut xml);
    xml.push_str("</title>\n  <link href=\"");
    escape_xml(&options.atom_url, &mut xml);
    xml.push_str("\" rel=\"self\"/>\n  <link href=\"");
    escape_xml(&options.home_page_url, &mut xml);
    xml.push_str("\" rel=\"alternate\"/>\n  <id>");
    escape_xml(&options.home_page_url, &mut xml);
    xml.push_str("</id>\n  <updated>");
    xml.push_str(&updated);
    xml.push_str("</updated>\n");
    if let Some(description) =
        options.site_description.as_deref().filter(|value| !value.trim().is_empty())
    {
        xml.push_str("  <subtitle>");
        escape_xml(description, &mut xml);
        xml.push_str("</subtitle>\n");
    }
    for item in items {
        xml.push_str("  <entry>\n    <title>");
        escape_xml(&item.title, &mut xml);
        xml.push_str("</title>\n    <link href=\"");
        escape_xml(&item.loc, &mut xml);
        xml.push_str("\"/>\n    <id>");
        escape_xml(&item.loc, &mut xml);
        xml.push_str("</id>\n    <updated>");
        xml.push_str(&item_date(item).map_or_else(|| updated.clone(), dates::ParsedDate::rfc3339));
        xml.push_str("</updated>\n");
        if let Some(description) = item.description.as_deref().filter(|value| !value.is_empty()) {
            xml.push_str("    <summary>");
            escape_xml(description, &mut xml);
            xml.push_str("</summary>\n");
        }
        xml.push_str("  </entry>\n");
    }
    xml.push_str("</feed>\n");
    xml
}

fn generate_json(options: &FeedsOptions, items: &[&FeedItem]) -> String {
    let mut json =
        String::from("{\n  \"version\": \"https://jsonfeed.org/version/1.1\",\n  \"title\": ");
    push_json_string(&options.site_name, &mut json);
    json.push_str(",\n  \"home_page_url\": ");
    push_json_string(&options.home_page_url, &mut json);
    json.push_str(",\n  \"feed_url\": ");
    push_json_string(&options.json_url, &mut json);
    if let Some(description) =
        options.site_description.as_deref().filter(|value| !value.trim().is_empty())
    {
        json.push_str(",\n  \"description\": ");
        push_json_string(description, &mut json);
    }
    json.push_str(",\n  \"items\": [");
    for (index, item) in items.iter().enumerate() {
        if index > 0 {
            json.push(',');
        }
        json.push_str("\n    {\n      \"id\": ");
        push_json_string(&item.loc, &mut json);
        json.push_str(",\n      \"url\": ");
        push_json_string(&item.loc, &mut json);
        json.push_str(",\n      \"title\": ");
        push_json_string(&item.title, &mut json);
        if let Some(description) = item.description.as_deref().filter(|value| !value.is_empty()) {
            json.push_str(",\n      \"content_text\": ");
            push_json_string(description, &mut json);
        }
        if let Some(date) = item_date(item) {
            json.push_str(",\n      \"date_published\": ");
            push_json_string(&date.rfc3339(), &mut json);
        }
        json.push_str("\n    }");
    }
    json.push_str("\n  ]\n}\n");
    json
}

fn escape_xml(value: &str, output: &mut String) {
    for ch in value.chars() {
        match ch {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&#39;"),
            _ => output.push(ch),
        }
    }
}

fn push_hex4(output: &mut String, code: u32) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    output.push(HEX[((code >> 12) & 0xf) as usize] as char);
    output.push(HEX[((code >> 8) & 0xf) as usize] as char);
    output.push(HEX[((code >> 4) & 0xf) as usize] as char);
    output.push(HEX[(code & 0xf) as usize] as char);
}

fn push_json_string(value: &str, output: &mut String) {
    output.push('"');
    for ch in value.chars() {
        match ch {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            '<' => output.push_str("\\u003c"),
            '>' => output.push_str("\\u003e"),
            '&' => output.push_str("\\u0026"),
            ch if (ch as u32) < 0x20 => {
                output.push_str("\\u");
                push_hex4(output, ch as u32);
            }
            ch => output.push(ch),
        }
    }
    output.push('"');
}

#[cfg(test)]
mod tests;
