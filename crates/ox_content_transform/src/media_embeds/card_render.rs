//! The provider-card model and the two shapes it renders as.
//!
//! Every provider builds the same [`Card`] and then picks a shape: the framed
//! card, or the link preview the article platforms share with OGP. Keeping
//! both here means a provider module is only the rules for recognising its own
//! URLs.

use super::card_icons::{metric_icon, network_logo};
use super::html::{ComponentElement, attr};
use super::provider_cards::{body_text, first_attr, is_safe_https_url, parse_https_url};
use super::render::{escape_attr, escape_text};

pub(super) struct Card<'a> {
    pub(super) modifier: &'static str,
    pub(super) network: &'static str,
    pub(super) href: &'a str,
    pub(super) title: &'a str,
    pub(super) body: Option<&'a str>,
    pub(super) source_label: &'static str,
    pub(super) image: Option<&'a str>,
    pub(super) avatar: Option<&'a str>,
    pub(super) author: Option<&'a str>,
    pub(super) date: Option<&'a str>,
    pub(super) date_label: Option<&'a str>,
    pub(super) meta: Vec<(&'static str, Option<&'a str>)>,
    pub(super) iframe: Option<&'a str>,
}

pub(super) fn article_card<'a>(
    element: &'a ComponentElement<'_>,
    modifier: &'static str,
    network: &'static str,
    href: &'a str,
    fallback_title: &'a str,
) -> Card<'a> {
    Card {
        modifier,
        network,
        href,
        title: first_attr(element, &["title", "name"]).unwrap_or(fallback_title),
        body: body_text(element)
            .or_else(|| attr(element, "description"))
            .or_else(|| attr(element, "excerpt")),
        source_label: "Open source",
        image: first_attr(element, &["image", "thumbnail", "preview"])
            .filter(|value| is_safe_https_url(value)),
        avatar: first_attr(element, &["avatar", "avatarUrl", "authorAvatar"])
            .filter(|value| is_safe_https_url(value)),
        author: first_attr(element, &["author", "authorName", "displayName", "handle"]),
        date: first_attr(element, &["datetime", "dateTime", "createdAt", "date", "timestamp"]),
        date_label: first_attr(element, &["dateLabel", "time", "publishedAtLabel"]),
        meta: vec![
            ("Tags", attr(element, "tags")),
            ("Likes", first_attr(element, &["likes", "likeCount", "reactions"])),
            ("Comments", first_attr(element, &["comments", "commentCount", "replies"])),
            ("Reposts", first_attr(element, &["reposts", "boosts", "shares"])),
        ],
        iframe: None,
    }
}

pub(super) fn render_card(card: Card<'_>) -> String {
    let mut html = String::new();
    html.push_str("<article class=\"ox-provider-card ox-provider-card--");
    html.push_str(card.modifier);
    html.push_str("\"><a class=\"ox-provider-card__main\" href=\"");
    escape_attr(card.href, &mut html);
    html.push_str("\" target=\"_blank\" rel=\"noopener noreferrer\"><header class=\"ox-provider-card__header\"><span class=\"ox-provider-card__network\">");
    // A registry's own mark stands in for its name; the name stays for screen
    // readers, which have nothing to go on once the text is gone.
    if let Some(logo) = network_logo(card.modifier) {
        html.push_str(logo);
        html.push_str("<span class=\"ox-provider-card__sr\">");
        escape_text(card.network, &mut html);
        html.push_str("</span>");
    } else {
        escape_text(card.network, &mut html);
    }
    html.push_str("</span>");
    if let Some(author) = card.author {
        html.push_str("<span class=\"ox-provider-card__author\">");
        escape_text(author, &mut html);
        html.push_str("</span>");
    }
    if let Some(label) = card.date_label.or(card.date) {
        html.push_str("<time class=\"ox-provider-card__time\"");
        if let Some(date) = card.date {
            html.push_str(" datetime=\"");
            escape_attr(date, &mut html);
            html.push('"');
        }
        html.push('>');
        escape_text(label, &mut html);
        html.push_str("</time>");
    }
    html.push_str("</header><div class=\"ox-provider-card__body-wrap\">");
    if let Some(avatar) = card.avatar {
        html.push_str("<img class=\"ox-provider-card__avatar\" src=\"");
        escape_attr(avatar, &mut html);
        html.push_str("\" alt=\"\" loading=\"lazy\" decoding=\"async\">");
    }
    html.push_str("<div class=\"ox-provider-card__copy\"><h3 class=\"ox-provider-card__title\">");
    escape_text(card.title, &mut html);
    html.push_str("</h3>");
    if let Some(body) = card.body {
        html.push_str("<p class=\"ox-provider-card__excerpt\">");
        escape_text(body, &mut html);
        html.push_str("</p>");
    }
    html.push_str("</div></div>");
    if let Some(image) = card.image {
        html.push_str("<img class=\"ox-provider-card__image\" src=\"");
        escape_attr(image, &mut html);
        html.push_str("\" alt=\"\" loading=\"lazy\" decoding=\"async\">");
    }
    render_meta(&mut html, &card);
    html.push_str("</a>");
    if let Some(iframe) = card.iframe {
        html.push_str("<iframe class=\"ox-provider-card__map\" src=\"");
        escape_attr(iframe, &mut html);
        html.push_str("\" title=\"");
        escape_attr(card.title, &mut html);
        html.push_str("\" width=\"100%\" height=\"320\" loading=\"lazy\" referrerpolicy=\"strict-origin-when-cross-origin\" sandbox=\"allow-scripts allow-same-origin allow-popups\"></iframe>");
    }
    html.push_str("</article>");
    html
}

/// Render an article embed with the OGP link-card chrome.
///
/// Qiita, Zenn and note carry exactly what a link preview carries — a title, a
/// lead, an author and a source — and the provider frame stacked a network
/// band above all of it, so the platform read louder than the article. These
/// reuse `.ox-ogp-card` so a page has one link-preview shape; the SSG keys
/// `ogp.css` off the `ox-ogp-card` marker, so a page built only from these
/// still ships their styles. The `--<provider>` modifier rides along for sites
/// that target one platform.
pub(super) fn render_link_preview_card(card: Card<'_>) -> String {
    let mut html = String::new();
    html.push_str("<a class=\"ox-ogp-card ox-ogp-card--");
    html.push_str(card.modifier);
    html.push_str("\" href=\"");
    escape_attr(card.href, &mut html);
    html.push_str(
        "\" target=\"_blank\" rel=\"noopener noreferrer\"><div class=\"ox-ogp-content\"><div class=\"ox-ogp-title\">",
    );
    escape_text(card.title, &mut html);
    html.push_str("</div>");
    if let Some(body) = card.body {
        html.push_str("<div class=\"ox-ogp-description\">");
        escape_text(body, &mut html);
        html.push_str("</div>");
    }
    html.push_str("<div class=\"ox-ogp-meta\">");
    if let Some(avatar) = card.avatar {
        html.push_str("<img class=\"ox-ogp-favicon\" src=\"");
        escape_attr(avatar, &mut html);
        html.push_str("\" alt=\"\" loading=\"lazy\" decoding=\"async\">");
    }
    if let Some(parsed) = parse_https_url(card.href) {
        html.push_str("<span class=\"ox-ogp-domain\">");
        escape_text(&parsed.host, &mut html);
        html.push_str("</span>");
    }
    if let Some(author) = card.author {
        html.push_str("<span>");
        escape_text(author, &mut html);
        html.push_str("</span>");
    }
    if let Some(label) = card.date_label.or(card.date) {
        html.push_str("<time");
        if let Some(date) = card.date {
            html.push_str(" datetime=\"");
            escape_attr(date, &mut html);
            html.push('"');
        }
        html.push('>');
        escape_text(label, &mut html);
        html.push_str("</time>");
    }
    for (label, value) in &card.meta {
        let Some(value) = value.filter(|value| !value.trim().is_empty()) else {
            continue;
        };
        html.push_str("<span>");
        escape_text(label, &mut html);
        html.push(' ');
        escape_text(value, &mut html);
        html.push_str("</span>");
    }
    html.push_str("</div></div>");
    if let Some(image) = card.image {
        html.push_str("<img class=\"ox-ogp-image\" src=\"");
        escape_attr(image, &mut html);
        html.push_str("\" alt=\"\" loading=\"lazy\" decoding=\"async\">");
    }
    html.push_str("</a>");
    html
}

fn render_meta(html: &mut String, card: &Card<'_>) {
    html.push_str("<footer class=\"ox-provider-card__meta\">");
    for (label, value) in &card.meta {
        let Some(value) = value.filter(|value| !value.trim().is_empty()) else {
            continue;
        };
        // Both shapes carry the label; the mark decides whether it is read or
        // only heard.
        if let Some(icon) = metric_icon(label) {
            html.push_str(
                "<span class=\"ox-provider-card__metric ox-provider-card__metric--icon\">",
            );
            html.push_str(icon);
            html.push_str("<span class=\"ox-provider-card__sr\">");
        } else {
            html.push_str("<span class=\"ox-provider-card__metric\"><span>");
        }
        escape_text(label, html);
        html.push_str("</span>");
        html.push(' ');
        escape_text(value, html);
        html.push_str("</span>");
    }
    html.push_str("<span class=\"ox-provider-card__source\">");
    escape_text(card.source_label, html);
    html.push_str("</span></footer>");
}
