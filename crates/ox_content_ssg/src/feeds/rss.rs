//! RSS 2.0 bodies.

use super::{
    FeedAttachment, FeedItem, FeedsOptions, channel_description, channel_field, entry_id,
    escape_xml, item_date, item_description,
};

/// The Dublin Core namespace, declared only when an item uses it, so a feed
/// with no authors or languages keeps the markup it had before.
const DC_NS: &str = " xmlns:dc=\"http://purl.org/dc/elements/1.1/\"";

pub(super) fn generate_rss(options: &FeedsOptions, items: &[&FeedItem]) -> String {
    let needs_dc = items.iter().any(|item| !item.authors.is_empty() || item.language.is_some());

    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<rss version=\"2.0\"");
    if needs_dc {
        xml.push_str(DC_NS);
    }
    xml.push_str(">\n  <channel>\n    <title>");
    escape_xml(&options.site_name, &mut xml);
    xml.push_str("</title>\n    <link>");
    escape_xml(&options.home_page_url, &mut xml);
    xml.push_str("</link>\n    <description>");
    escape_xml(channel_description(options), &mut xml);
    xml.push_str("</description>\n");
    push_channel_meta(options, &mut xml);

    for item in items {
        xml.push_str("    <item>\n      <title>");
        escape_xml(&item.title, &mut xml);
        xml.push_str("</title>\n      <link>");
        escape_xml(&item.loc, &mut xml);
        xml.push_str("</link>\n");
        push_guid(item, &mut xml);
        if let Some(description) = item_description(item) {
            xml.push_str("      <description>");
            escape_xml(description, &mut xml);
            xml.push_str("</description>\n");
        }
        if let Some(date) = item_date(item) {
            xml.push_str("      <pubDate>");
            xml.push_str(&date.rfc822());
            xml.push_str("</pubDate>\n");
        }
        push_item_meta(item, &mut xml);
        xml.push_str("    </item>\n");
    }

    xml.push_str("  </channel>\n</rss>\n");
    xml
}

/// A guid equal to the link is a permalink; anything else has to say it is not.
fn push_guid(item: &FeedItem, xml: &mut String) {
    let id = entry_id(item);
    xml.push_str("      <guid");
    if id != item.loc {
        xml.push_str(" isPermaLink=\"false\"");
    }
    xml.push('>');
    escape_xml(id, xml);
    xml.push_str("</guid>\n");
}

fn push_item_meta(item: &FeedItem, xml: &mut String) {
    for author in &item.authors {
        xml.push_str("      <dc:creator>");
        escape_xml(&author.name, xml);
        xml.push_str("</dc:creator>\n");
    }
    if let Some(language) = item.language.as_deref() {
        xml.push_str("      <dc:language>");
        escape_xml(language, xml);
        xml.push_str("</dc:language>\n");
    }
    for attachment in &item.attachments {
        push_enclosure(attachment, xml);
    }
}

fn push_enclosure(attachment: &FeedAttachment, xml: &mut String) {
    xml.push_str("      <enclosure");
    super::push_xml_attr(xml, "url", Some(attachment.url.as_str()));
    super::push_xml_attr(xml, "type", attachment.mime_type.as_deref());
    super::push_xml_attr_number(xml, "length", attachment.size_in_bytes);
    xml.push_str("/>\n");
}

/// RSS channel metadata, in the order the standard lists it.
fn push_channel_meta(options: &FeedsOptions, xml: &mut String) {
    if let Some(language) = channel_field(options.language.as_ref()) {
        xml.push_str("    <language>");
        escape_xml(language, xml);
        xml.push_str("</language>\n");
    }
    if let Some(copyright) = channel_field(options.copyright.as_ref()) {
        xml.push_str("    <copyright>");
        escape_xml(copyright, xml);
        xml.push_str("</copyright>\n");
    }
    if let Some(image) = channel_field(options.image.as_ref()) {
        xml.push_str("    <image>\n      <url>");
        escape_xml(image, xml);
        xml.push_str("</url>\n      <title>");
        escape_xml(&options.site_name, xml);
        xml.push_str("</title>\n      <link>");
        escape_xml(&options.home_page_url, xml);
        xml.push_str("</link>\n    </image>\n");
    }
}
