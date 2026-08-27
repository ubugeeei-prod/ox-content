//! JSON Feed 1.1 bodies.

use super::{
    FeedAttachment, FeedAuthor, FeedItem, FeedsOptions, channel_field, entry_id, item_date,
    item_description, push_json_string,
};

pub(super) fn generate_json(options: &FeedsOptions, items: &[&FeedItem]) -> String {
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
    for (key, value) in [
        ("language", channel_field(options.language.as_ref())),
        ("icon", channel_field(options.image.as_ref())),
        ("favicon", channel_field(options.favicon.as_ref())),
    ] {
        let Some(value) = value else { continue };
        json.push_str(",\n  \"");
        json.push_str(key);
        json.push_str("\": ");
        push_json_string(value, &mut json);
    }
    json.push_str(",\n  \"items\": [");

    for (index, item) in items.iter().enumerate() {
        if index > 0 {
            json.push(',');
        }
        json.push_str("\n    {\n      \"id\": ");
        push_json_string(entry_id(item), &mut json);
        json.push_str(",\n      \"url\": ");
        push_json_string(&item.loc, &mut json);
        json.push_str(",\n      \"title\": ");
        push_json_string(&item.title, &mut json);

        let description = item.description.as_deref().filter(|value| !value.is_empty());
        let content = item.content.as_deref().filter(|value| !value.is_empty());
        // `summary` only earns its place when it is saying something the
        // content does not.
        if let (Some(description), Some(_)) = (description, content) {
            json.push_str(",\n      \"summary\": ");
            push_json_string(description, &mut json);
        }
        if let Some(text) = item_description(item) {
            json.push_str(",\n      \"content_text\": ");
            push_json_string(text, &mut json);
        }
        if let Some(date) = item_date(item) {
            json.push_str(",\n      \"date_published\": ");
            push_json_string(&date.rfc3339(), &mut json);
        }
        push_item_meta(item, &mut json);
        json.push_str("\n    }");
    }

    json.push_str("\n  ]\n}\n");
    json
}

fn push_item_meta(item: &FeedItem, json: &mut String) {
    if !item.authors.is_empty() {
        json.push_str(",\n      \"authors\": [");
        for (index, author) in item.authors.iter().enumerate() {
            if index > 0 {
                json.push_str(", ");
            }
            push_author(author, json);
        }
        json.push(']');
    }
    if let Some(image) = item.image.as_deref() {
        json.push_str(",\n      \"image\": ");
        push_json_string(image, json);
    }
    if let Some(language) = item.language.as_deref() {
        json.push_str(",\n      \"language\": ");
        push_json_string(language, json);
    }
    if !item.attachments.is_empty() {
        json.push_str(",\n      \"attachments\": [");
        for (index, attachment) in item.attachments.iter().enumerate() {
            if index > 0 {
                json.push_str(", ");
            }
            push_attachment(attachment, json);
        }
        json.push(']');
    }
}

fn push_author(author: &FeedAuthor, json: &mut String) {
    json.push_str("{\n        \"name\": ");
    push_json_string(&author.name, json);
    if let Some(url) = author.url.as_deref() {
        json.push_str(", \"url\": ");
        push_json_string(url, json);
    }
    json.push_str("\n      }");
}

fn push_attachment(attachment: &FeedAttachment, json: &mut String) {
    let mut fields: Vec<String> = Vec::with_capacity(5);
    let field = |name: &str, value: &str| format!("\"{name}\": {value}");

    let mut url = String::new();
    push_json_string(&attachment.url, &mut url);
    fields.push(field("url", &url));

    if let Some(mime) = attachment.mime_type.as_deref() {
        let mut value = String::new();
        push_json_string(mime, &mut value);
        fields.push(field("mime_type", &value));
    }
    if let Some(title) = attachment.title.as_deref() {
        let mut value = String::new();
        push_json_string(title, &mut value);
        fields.push(field("title", &value));
    }
    if let Some(size) = attachment.size_in_bytes {
        fields.push(field("size_in_bytes", &size.to_string()));
    }
    if let Some(duration) = attachment.duration_in_seconds {
        fields.push(field("duration_in_seconds", &duration.to_string()));
    }

    json.push_str("{\n        ");
    json.push_str(&fields.join(",\n        "));
    json.push_str("\n      }");
}
