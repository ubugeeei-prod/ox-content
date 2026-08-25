//! Opt-in generated section index listings.

use serde::{Deserialize, Serialize};

use super::utils::escape_html;

/// One child link on a generated section index.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SectionIndexItem {
    /// Visible title. Escaped in HTML.
    pub title: String,
    /// Destination. `javascript:` and other schemes are rejected.
    pub href: String,
    /// Optional card description. Escaped in HTML.
    pub description: Option<String>,
}

/// Listing style for a generated section index.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SectionIndexStyle {
    /// Simple title list.
    List,
    /// Card grid. Default when the feature is on.
    #[default]
    Cards,
}

impl SectionIndexStyle {
    /// Parses `"list"` / `"cards"`. Unknown values become cards.
    pub fn parse(value: &str) -> Self {
        if value.eq_ignore_ascii_case("list") { Self::List } else { Self::Cards }
    }
}

pub(super) const SECTION_INDEX_CSS: &str = include_str!("section_index.css");

/// Renders a section index listing. Titles are escaped; hostile hrefs are dropped.
pub fn render_section_index(
    title: &str,
    items: &[SectionIndexItem],
    style: SectionIndexStyle,
) -> String {
    let safe: Vec<&SectionIndexItem> =
        items.iter().filter(|item| is_safe_section_href(&item.href)).collect();
    let (modifier, list_class) = match style {
        SectionIndexStyle::List => ("list", "ox-section-index__list"),
        SectionIndexStyle::Cards => ("cards", "ox-section-index__cards"),
    };

    let mut out = String::from(r#"<nav class="ox-section-index ox-section-index--"#);
    out.push_str(modifier);
    out.push_str(r#"" aria-label="Section pages"><h1>"#);
    out.push_str(&escape_html(title));
    out.push_str("</h1><ul class=\"");
    out.push_str(list_class);
    out.push_str("\">");
    for item in safe {
        append_item(&mut out, item, style);
    }
    out.push_str("</ul></nav>");
    out
}

/// Site-relative `/` path or a schemeless relative path. Schemes are rejected.
pub fn is_safe_section_href(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || trimmed.bytes().any(|byte| matches!(byte, b'\n' | b'\r' | b'\0' | b'\t'))
    {
        return false;
    }
    if trimmed.starts_with("//") {
        return false;
    }
    if trimmed.starts_with('/') {
        return true;
    }
    !has_url_scheme(trimmed)
}

fn has_url_scheme(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.is_empty() || !bytes[0].is_ascii_alphabetic() {
        return false;
    }
    let mut index = 1;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte == b':' {
            return true;
        }
        if !(byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'.' | b'-')) {
            return false;
        }
        index += 1;
    }
    false
}

fn append_item(out: &mut String, item: &SectionIndexItem, style: SectionIndexStyle) {
    let href = escape_html(item.href.trim());
    let title = escape_html(&item.title);
    match style {
        SectionIndexStyle::List => {
            out.push_str("<li><a href=\"");
            out.push_str(&href);
            out.push_str("\">");
            out.push_str(&title);
            out.push_str("</a></li>");
        }
        SectionIndexStyle::Cards => {
            out.push_str(r#"<li class="ox-section-index__card"><a href=""#);
            out.push_str(&href);
            out.push_str(r#""><span class="ox-section-index__title">"#);
            out.push_str(&title);
            out.push_str("</span>");
            if let Some(description) =
                item.description.as_deref().map(str::trim).filter(|text| !text.is_empty())
            {
                out.push_str(r#"<span class="ox-section-index__desc">"#);
                out.push_str(&escape_html(description));
                out.push_str("</span>");
            }
            out.push_str("</a></li>");
        }
    }
}

#[cfg(test)]
mod tests;
