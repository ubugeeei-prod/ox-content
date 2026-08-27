use napi_derive::napi;

/// A person credited on a feed item.
#[napi(object)]
pub struct JsFeedAuthor {
    pub name: String,
    pub url: Option<String>,
}

/// A file carried alongside a feed item.
#[napi(object)]
pub struct JsFeedAttachment {
    pub url: String,
    pub mime_type: Option<String>,
    pub title: Option<String>,
    pub size_in_bytes: Option<i64>,
    pub duration_in_seconds: Option<i64>,
}

/// One collection entry considered for a feed.
#[napi(object)]
pub struct JsFeedItem {
    pub title: String,
    pub description: Option<String>,
    pub loc: String,
    pub date: Option<String>,
    pub last_updated: Option<String>,
    pub draft: Option<bool>,
    pub unlisted: Option<bool>,
    pub content: Option<String>,
    pub id: Option<String>,
    pub authors: Option<Vec<JsFeedAuthor>>,
    pub image: Option<String>,
    pub attachments: Option<Vec<JsFeedAttachment>>,
    pub language: Option<String>,
}

/// Switches and site metadata for feed generation.
#[napi(object)]
pub struct JsFeedsOptions {
    pub enabled: bool,
    pub site_url: Option<String>,
    pub site_name: String,
    pub site_description: Option<String>,
    pub home_page_url: String,
    pub rss_url: String,
    pub atom_url: String,
    pub json_url: String,
    /// Any of `"rss"`, `"atom"`, `"json"`. Unknown names are ignored.
    pub formats: Vec<String>,
    pub limit: u32,
    pub language: Option<String>,
    pub image: Option<String>,
    pub favicon: Option<String>,
    pub copyright: Option<String>,
}

/// Generated feed bodies, or a warning when generation is skipped.
#[napi(object)]
pub struct JsFeedsOutput {
    pub rss_xml: Option<String>,
    pub atom_xml: Option<String>,
    pub json_feed: Option<String>,
    pub warning: Option<String>,
}

fn convert_author(author: JsFeedAuthor) -> ox_content_ssg::FeedAuthor {
    ox_content_ssg::FeedAuthor { name: author.name, url: author.url }
}

fn convert_attachment(attachment: JsFeedAttachment) -> ox_content_ssg::FeedAttachment {
    ox_content_ssg::FeedAttachment {
        url: attachment.url,
        mime_type: attachment.mime_type,
        title: attachment.title,
        size_in_bytes: attachment.size_in_bytes,
        duration_in_seconds: attachment.duration_in_seconds,
    }
}

fn convert_item(item: JsFeedItem) -> ox_content_ssg::FeedItem {
    ox_content_ssg::FeedItem {
        title: item.title,
        description: item.description,
        loc: item.loc,
        date: item.date,
        last_updated: item.last_updated,
        draft: item.draft.unwrap_or(false),
        unlisted: item.unlisted.unwrap_or(false),
        content: item.content,
        id: item.id,
        authors: item.authors.unwrap_or_default().into_iter().map(convert_author).collect(),
        image: item.image,
        attachments: item
            .attachments
            .unwrap_or_default()
            .into_iter()
            .map(convert_attachment)
            .collect(),
        language: item.language,
    }
}

fn convert_format(name: &str) -> Option<ox_content_ssg::FeedFormat> {
    match name {
        "rss" => Some(ox_content_ssg::FeedFormat::Rss),
        "atom" => Some(ox_content_ssg::FeedFormat::Atom),
        "json" => Some(ox_content_ssg::FeedFormat::Json),
        _ => None,
    }
}

/// Builds RSS / Atom / JSON Feed bodies without writing files.
#[napi(js_name = "generateFeedBodies")]
pub fn generate_feed_bodies(options: JsFeedsOptions, items: Vec<JsFeedItem>) -> JsFeedsOutput {
    let resolved = ox_content_ssg::FeedsOptions {
        enabled: options.enabled,
        site_url: options.site_url,
        site_name: options.site_name,
        site_description: options.site_description,
        home_page_url: options.home_page_url,
        rss_url: options.rss_url,
        atom_url: options.atom_url,
        json_url: options.json_url,
        formats: options.formats.iter().filter_map(|name| convert_format(name)).collect(),
        limit: options.limit as usize,
        language: options.language,
        image: options.image,
        favicon: options.favicon,
        copyright: options.copyright,
    };
    let items: Vec<_> = items.into_iter().map(convert_item).collect();
    let output = ox_content_ssg::generate_feeds(&resolved, &items);

    JsFeedsOutput {
        rss_xml: output.rss_xml,
        atom_xml: output.atom_xml,
        json_feed: output.json_feed,
        warning: output.warning,
    }
}
