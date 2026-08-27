//! Opt-in RSS, Atom, and JSON Feed bodies.

mod atom;
mod dates;
mod json;
mod rss;

use atom::generate_atom;
pub use dates::{ParsedDate, parse_date};
use json::generate_json;
use rss::generate_rss;

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

/// A person credited on a feed item.
#[derive(Debug, Clone, Default)]
pub struct FeedAuthor {
    /// Display name.
    pub name: String,
    /// Optional profile or home page.
    pub url: Option<String>,
}

/// A file carried alongside a feed item — audio, video, an image.
#[derive(Debug, Clone, Default)]
pub struct FeedAttachment {
    /// Absolute URL of the file.
    pub url: String,
    /// MIME type, when known.
    pub mime_type: Option<String>,
    /// Human-readable label.
    pub title: Option<String>,
    /// Size in bytes, when known.
    pub size_in_bytes: Option<i64>,
    /// Playing time in seconds, for audio and video.
    pub duration_in_seconds: Option<i64>,
}

/// One collection entry considered for a feed.
#[derive(Debug, Clone, Default)]
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
    /// Full item body. Preferred over `description` where a format carries one.
    pub content: Option<String>,
    /// Stable identity, when it differs from `loc`.
    pub id: Option<String>,
    /// People credited on the item.
    pub authors: Vec<FeedAuthor>,
    /// Representative image URL.
    pub image: Option<String>,
    /// Files carried alongside the item.
    pub attachments: Vec<FeedAttachment>,
    /// BCP 47 language tag for this item.
    pub language: Option<String>,
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
    /// BCP 47 language for the whole feed.
    pub language: Option<String>,
    /// Channel image URL — RSS `<image>`, Atom `<logo>`, JSON Feed `icon`.
    pub image: Option<String>,
    /// Small square icon — Atom `<icon>`, JSON Feed `favicon`.
    pub favicon: Option<String>,
    /// Rights statement — RSS `<copyright>`, Atom `<rights>`.
    pub copyright: Option<String>,
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
            language: None,
            image: None,
            favicon: None,
            copyright: None,
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

/// The identity a feed advertises for an item: its own `id` when it has one,
/// otherwise its URL.
fn entry_id(item: &FeedItem) -> &str {
    item.id.as_deref().filter(|value| !value.is_empty()).unwrap_or(&item.loc)
}

/// The text a single-body format shows. `content` is the fuller of the two, so
/// it wins where only one can be carried.
fn item_description(item: &FeedItem) -> Option<&str> {
    item.content
        .as_deref()
        .filter(|value| !value.is_empty())
        .or_else(|| item.description.as_deref().filter(|value| !value.is_empty()))
}

/// Writes ` name="value"`, or nothing when the value is absent — an attribute
/// with no value would claim something the item never said.
fn push_xml_attr(output: &mut String, name: &str, value: Option<&str>) {
    let Some(value) = value else { return };
    output.push(' ');
    output.push_str(name);
    output.push_str("=\"");
    escape_xml(value, output);
    output.push('"');
}

fn push_xml_attr_number(output: &mut String, name: &str, value: Option<i64>) {
    if let Some(value) = value {
        push_xml_attr(output, name, Some(&value.to_string()));
    }
}

/// A channel field worth emitting: present, and not just whitespace.
fn channel_field(value: Option<&String>) -> Option<&str> {
    value.map(String::as_str).filter(|value| !value.trim().is_empty())
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
