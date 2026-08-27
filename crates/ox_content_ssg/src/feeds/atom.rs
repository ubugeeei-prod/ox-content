//! Atom 1.0 bodies.

use super::{
    FeedAttachment, FeedAuthor, FeedItem, FeedsOptions, dates, entry_id, escape_xml, item_date,
};

pub(super) fn generate_atom(options: &FeedsOptions, items: &[&FeedItem]) -> String {
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
        xml.push_str("  <entry");
        if let Some(language) = item.language.as_deref() {
            xml.push_str(" xml:lang=\"");
            escape_xml(language, &mut xml);
            xml.push('"');
        }
        xml.push_str(">\n    <title>");
        escape_xml(&item.title, &mut xml);
        xml.push_str("</title>\n    <link href=\"");
        escape_xml(&item.loc, &mut xml);
        xml.push_str("\"/>\n    <id>");
        escape_xml(entry_id(item), &mut xml);
        xml.push_str("</id>\n    <updated>");
        xml.push_str(&item_date(item).map_or_else(|| updated.clone(), dates::ParsedDate::rfc3339));
        xml.push_str("</updated>\n");

        // Atom carries both: `summary` is the short form, `content` the body,
        // so unlike RSS neither displaces the other.
        if let Some(description) = item.description.as_deref().filter(|value| !value.is_empty()) {
            xml.push_str("    <summary>");
            escape_xml(description, &mut xml);
            xml.push_str("</summary>\n");
        }
        if let Some(content) = item.content.as_deref().filter(|value| !value.is_empty()) {
            xml.push_str("    <content type=\"text\">");
            escape_xml(content, &mut xml);
            xml.push_str("</content>\n");
        }
        for author in &item.authors {
            push_author(author, &mut xml);
        }
        for attachment in &item.attachments {
            push_attachment(attachment, &mut xml);
        }
        xml.push_str("  </entry>\n");
    }

    xml.push_str("</feed>\n");
    xml
}

fn push_author(author: &FeedAuthor, xml: &mut String) {
    xml.push_str("    <author>\n      <name>");
    escape_xml(&author.name, xml);
    xml.push_str("</name>");
    if let Some(url) = author.url.as_deref() {
        xml.push_str("\n      <uri>");
        escape_xml(url, xml);
        xml.push_str("</uri>");
    }
    xml.push_str("\n    </author>\n");
}

fn push_attachment(attachment: &FeedAttachment, xml: &mut String) {
    xml.push_str("    <link");
    super::push_xml_attr(xml, "rel", Some("enclosure"));
    super::push_xml_attr(xml, "href", Some(attachment.url.as_str()));
    super::push_xml_attr(xml, "type", attachment.mime_type.as_deref());
    super::push_xml_attr_number(xml, "length", attachment.size_in_bytes);
    super::push_xml_attr(xml, "title", attachment.title.as_deref());
    xml.push_str("/>\n");
}
