//! Opt-in RSS 2.0, Atom, and JSON Feed 1.1 bodies.

mod dates;

use std::fmt::Write;

use dates::{format_rfc822, format_rfc3339, parse_date};

const MISSING_SITE_URL: &str = "[ox-content] feeds is enabled but ssg.siteUrl is not set; feed.xml, atom.xml, and feed.json were not written";

/// One feed entry built from a collection page.
#[derive(Debug, Clone)]
pub struct FeedItem {
    /// Item title.
    pub title: String,
    /// Absolute item URL.
    pub loc: String,
    /// Optional summary.
    pub description: Option<String>,
    /// Raw date field, usually frontmatter `date`.
    pub date: Option<String>,
}

/// Switches and site metadata for feed generation.
#[derive(Debug, Clone)]
pub struct FeedsOptions {
    /// When false, no files are generated.
    pub enabled: bool,
    /// Absolute site origin, required when the feature is on.
    pub site_url: Option<String>,
    /// Channel / feed title.
    pub site_name: String,
    /// Optional channel / feed summary.
    pub site_description: Option<String>,
    /// Absolute site home URL.
    pub home_page_url: String,
    /// Absolute URL of the generated RSS file.
    pub feed_rss_loc: String,
    /// Absolute URL of the generated Atom file.
    pub feed_atom_loc: String,
    /// Absolute URL of the generated JSON Feed file.
    pub feed_json_loc: String,
    /// Write RSS 2.0 when the feature is on.
    pub rss: bool,
    /// Write Atom when the feature is on.
    pub atom: bool,
    /// Write JSON Feed 1.1 when the feature is on.
    pub json: bool,
    /// Maximum number of items, newest first.
    pub limit: usize,
}

/// Generated feed bodies, or a warning when generation is skipped.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FeedsOutput {
    /// RSS 2.0 body.
    pub rss_xml: Option<String>,
    /// Atom body.
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
            feed_rss_loc: String::new(),
            feed_atom_loc: String::new(),
            feed_json_loc: String::new(),
            rss: true,
            atom: true,
            json: true,
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

    let selected = select_items(items, options.limit);
    let home = resolve_url(&options.home_page_url, options.site_url.as_deref(), "");
    let atom_loc = resolve_url(&options.feed_atom_loc, options.site_url.as_deref(), "atom.xml");
    let json_loc = resolve_url(&options.feed_json_loc, options.site_url.as_deref(), "feed.json");

    FeedsOutput {
        rss_xml: options.rss.then(|| generate_rss(options, &home, &selected)),
        atom_xml: options.atom.then(|| generate_atom(options, &home, &atom_loc, &selected)),
        json_feed: options.json.then(|| generate_json(options, &home, &json_loc, &selected)),
        warning: None,
    }
}

fn has_site_url(site_url: Option<&str>) -> bool {
    site_url.is_some_and(|value| !value.trim().is_empty())
}

fn resolve_url(explicit: &str, site_url: Option<&str>, suffix: &str) -> String {
    if !explicit.trim().is_empty() {
        return explicit.trim().to_string();
    }
    let origin = site_url.unwrap_or("").trim().trim_end_matches('/');
    if suffix.is_empty() { format!("{origin}/") } else { format!("{origin}/{suffix}") }
}

fn select_items(items: &[FeedItem], limit: usize) -> Vec<&FeedItem> {
    let mut selected: Vec<&FeedItem> = items.iter().filter(|item| !item.loc.is_empty()).collect();
    selected.sort_by(|left, right| {
        match (parse_date(left.date.as_deref()), parse_date(right.date.as_deref())) {
            (Some(left_date), Some(right_date)) => {
                right_date.cmp(&left_date).then_with(|| left.loc.cmp(&right.loc))
            }
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => left.loc.cmp(&right.loc),
        }
    });
    if selected.len() > limit {
        selected.truncate(limit);
    }
    selected
}

fn newest_updated(items: &[&FeedItem]) -> String {
    items
        .iter()
        .find_map(|item| parse_date(item.date.as_deref()))
        .map_or_else(|| "1970-01-01T00:00:00Z".to_string(), format_rfc3339)
}

fn generate_rss(options: &FeedsOptions, home: &str, items: &[&FeedItem]) -> String {
    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<rss version=\"2.0\">\n  <channel>\n    <title>",
    );
    escape_xml(&options.site_name, &mut xml);
    xml.push_str("</title>\n    <link>");
    escape_xml(home, &mut xml);
    xml.push_str("</link>\n    <description>");
    escape_xml(channel_description(options), &mut xml);
    xml.push_str("</description>\n");
    for item in items {
        xml.push_str("    <item>\n      <title>");
        escape_xml(&item.title, &mut xml);
        xml.push_str("</title>\n      <link>");
        escape_xml(&item.loc, &mut xml);
        xml.push_str("</link>\n      <guid isPermaLink=\"true\">");
        escape_xml(&item.loc, &mut xml);
        xml.push_str("</guid>\n");
        if let Some(description) = item.description.as_deref() {
            xml.push_str("      <description>");
            escape_xml(description, &mut xml);
            xml.push_str("</description>\n");
        }
        if let Some(date) = parse_date(item.date.as_deref()) {
            xml.push_str("      <pubDate>");
            xml.push_str(&format_rfc822(date));
            xml.push_str("</pubDate>\n");
        }
        xml.push_str("    </item>\n");
    }
    xml.push_str("  </channel>\n</rss>\n");
    xml
}

fn generate_atom(
    options: &FeedsOptions,
    home: &str,
    atom_loc: &str,
    items: &[&FeedItem],
) -> String {
    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<feed xmlns=\"http://www.w3.org/2005/Atom\">\n  <title>",
    );
    escape_xml(&options.site_name, &mut xml);
    xml.push_str("</title>\n  <link href=\"");
    escape_xml(home, &mut xml);
    xml.push_str("\" rel=\"alternate\"/>\n  <link href=\"");
    escape_xml(atom_loc, &mut xml);
    xml.push_str("\" rel=\"self\"/>\n  <id>");
    escape_xml(home, &mut xml);
    xml.push_str("</id>\n  <updated>");
    xml.push_str(&newest_updated(items));
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
        xml.push_str(
            &parse_date(item.date.as_deref())
                .map_or_else(|| "1970-01-01T00:00:00Z".to_string(), format_rfc3339),
        );
        xml.push_str("</updated>\n");
        if let Some(description) = item.description.as_deref() {
            xml.push_str("    <summary>");
            escape_xml(description, &mut xml);
            xml.push_str("</summary>\n");
        }
        xml.push_str("  </entry>\n");
    }
    xml.push_str("</feed>\n");
    xml
}

fn generate_json(
    options: &FeedsOptions,
    home: &str,
    json_loc: &str,
    items: &[&FeedItem],
) -> String {
    let mut json =
        String::from("{\n  \"version\": \"https://jsonfeed.org/version/1.1\",\n  \"title\": \"");
    escape_json(&options.site_name, &mut json);
    json.push_str("\",\n  \"home_page_url\": \"");
    escape_json(home, &mut json);
    json.push_str("\",\n  \"feed_url\": \"");
    escape_json(json_loc, &mut json);
    json.push('"');
    if let Some(description) =
        options.site_description.as_deref().filter(|value| !value.trim().is_empty())
    {
        json.push_str(",\n  \"description\": \"");
        escape_json(description, &mut json);
        json.push('"');
    }
    json.push_str(",\n  \"items\": [");
    for (index, item) in items.iter().enumerate() {
        if index > 0 {
            json.push(',');
        }
        json.push_str("\n    {\n      \"id\": \"");
        escape_json(&item.loc, &mut json);
        json.push_str("\",\n      \"url\": \"");
        escape_json(&item.loc, &mut json);
        json.push_str("\",\n      \"title\": \"");
        escape_json(&item.title, &mut json);
        json.push('"');
        if let Some(description) = item.description.as_deref() {
            json.push_str(",\n      \"content_text\": \"");
            escape_json(description, &mut json);
            json.push('"');
        }
        if let Some(date) = parse_date(item.date.as_deref()) {
            json.push_str(",\n      \"date_published\": \"");
            json.push_str(&format_rfc3339(date));
            json.push('"');
        }
        json.push_str("\n    }");
    }
    json.push_str("\n  ]\n}\n");
    json
}

fn channel_description(options: &FeedsOptions) -> &str {
    options
        .site_description
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(options.site_name.as_str())
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

fn escape_json(value: &str, output: &mut String) {
    for ch in value.chars() {
        match ch {
            '\\' => output.push_str("\\\\"),
            '"' => output.push_str("\\\""),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            '<' => output.push_str("\\u003c"),
            '>' => output.push_str("\\u003e"),
            '&' => output.push_str("\\u0026"),
            ch if u32::from(ch) < 0x20 => {
                let _ = write!(output, "\\u{:04x}", u32::from(ch));
            }
            _ => output.push(ch),
        }
    }
}

#[cfg(test)]
mod tests;
