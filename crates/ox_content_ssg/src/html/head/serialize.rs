//! Serialize resolved head tags to HTML.

use super::input::{HeadLink, HeadMeta};
use super::resolve::ResolvedTag;
use crate::html::utils::escape_html;

pub fn serialize_head(tags: &[ResolvedTag]) -> String {
    let mut out = String::new();
    for tag in tags {
        match tag {
            ResolvedTag::Title(title) => {
                out.push_str("  <title>");
                out.push_str(&escape_html(title));
                out.push_str("</title>\n");
            }
            ResolvedTag::Meta(meta) => push_meta(&mut out, meta),
            ResolvedTag::Link(link) => push_link(&mut out, link),
            ResolvedTag::JsonLd { json, .. } => {
                out.push_str("  <script type=\"application/ld+json\">");
                out.push_str(json);
                out.push_str("</script>\n");
            }
        }
    }
    if out.ends_with('\n') {
        out.pop();
    }
    out
}

fn push_meta(out: &mut String, meta: &HeadMeta) {
    out.push_str("  <meta");
    push_attr(out, "name", meta.name.as_deref());
    push_attr(out, "property", meta.property.as_deref());
    push_attr(out, "http-equiv", meta.http_equiv.as_deref());
    push_attr(out, "content", Some(meta.content.as_str()));
    out.push_str(">\n");
}

fn push_link(out: &mut String, link: &HeadLink) {
    out.push_str("  <link");
    push_attr(out, "rel", Some(link.rel.as_str()));
    push_attr(out, "hreflang", link.hreflang.as_deref());
    push_attr(out, "href", Some(link.href.as_str()));
    push_attr(out, "type", link.r#type.as_deref());
    push_attr(out, "sizes", link.sizes.as_deref());
    out.push_str(">\n");
}

fn push_attr(out: &mut String, name: &str, value: Option<&str>) {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return;
    };
    out.push(' ');
    out.push_str(name);
    out.push_str("=\"");
    out.push_str(&escape_html(value));
    out.push('"');
}
