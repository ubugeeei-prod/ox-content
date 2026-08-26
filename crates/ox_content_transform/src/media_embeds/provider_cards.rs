use super::html::{ComponentElement, attr};
use super::render::{escape_attr, escape_text};

pub(super) fn render_google_maps(element: &ComponentElement<'_>) -> Option<String> {
    let href = provider_url(element)?;
    let parsed = parse_https_url(href)?;
    if !host_in(
        &parsed.host,
        &["google.com", "www.google.com", "maps.google.com", "maps.app.goo.gl", "goo.gl"],
    ) {
        return None;
    }

    let title = first_attr(element, &["place", "title", "name"]).unwrap_or("Google Maps place");
    let address = attr(element, "address").or_else(|| body_text(element));
    let iframe = first_attr(element, &["embed", "embedUrl", "iframe", "iframeSrc"])
        .filter(|value| is_google_maps_embed(value));

    Some(render_card(Card {
        modifier: "google-maps",
        network: "Google Maps",
        href,
        title,
        body: address,
        source_label: "Open map",
        image: None,
        avatar: None,
        author: None,
        date: None,
        date_label: None,
        meta: vec![],
        iframe,
    }))
}

pub(super) fn render_qiita(element: &ComponentElement<'_>) -> Option<String> {
    let href = provider_url(element)?;
    let parsed = parse_https_url(href)?;
    if !host_in(&parsed.host, &["qiita.com"]) || !parsed.path.contains("/items/") {
        return None;
    }

    Some(render_card(article_card(element, "qiita", "Qiita", href, "Qiita article")))
}

pub(super) fn render_zenn(element: &ComponentElement<'_>) -> Option<String> {
    let href = provider_url(element)?;
    let parsed = parse_https_url(href)?;
    if !host_in(&parsed.host, &["zenn.dev"])
        || !["/articles/", "/books/", "/scraps/"].iter().any(|needle| parsed.path.contains(needle))
    {
        return None;
    }

    Some(render_card(article_card(element, "zenn", "Zenn", href, "Zenn article")))
}

pub(super) fn render_discord(element: &ComponentElement<'_>) -> Option<String> {
    let href = provider_url(element)?;
    let parsed = parse_https_url(href)?;
    if !host_in(&parsed.host, &["discord.com", "discordapp.com", "discord.gg"]) {
        return None;
    }

    let title = first_attr(element, &["title", "server", "name"]).unwrap_or_else(|| {
        if parsed.host == "discord.gg" || parsed.path.contains("/invite/") {
            "Discord invite"
        } else {
            "Discord message"
        }
    });
    let mut card = article_card(element, "discord", "Discord", href, title);
    card.meta.push(("Server", attr(element, "server")));
    card.meta.push(("Channel", attr(element, "channel")));
    Some(render_card(card))
}

pub(super) fn render_fediverse(element: &ComponentElement<'_>) -> Option<String> {
    render_social_card(element, "fediverse", "Fediverse", None)
}

pub(super) fn render_mastodon(element: &ComponentElement<'_>) -> Option<String> {
    render_social_card(element, "mastodon", "Mastodon", Some("Mastodon post"))
}

pub(super) fn render_misskey(element: &ComponentElement<'_>) -> Option<String> {
    render_social_card(element, "misskey", "Misskey", Some("Misskey note"))
}

pub(super) fn render_mixi2(element: &ComponentElement<'_>) -> Option<String> {
    render_social_card(element, "mixi2", "Mixi2", Some("Mixi2 post"))
}

pub(super) fn render_facebook(element: &ComponentElement<'_>) -> Option<String> {
    let href = provider_url(element)?;
    let parsed = parse_https_url(href)?;
    if !host_in(&parsed.host, &["facebook.com", "www.facebook.com", "m.facebook.com", "fb.watch"]) {
        return None;
    }

    Some(render_card(article_card(element, "facebook", "Facebook", href, "Facebook post")))
}

pub(super) fn render_threads(element: &ComponentElement<'_>) -> Option<String> {
    let href = provider_url(element)?;
    let parsed = parse_https_url(href)?;
    if !host_in(&parsed.host, &["threads.net", "www.threads.net"]) {
        return None;
    }

    Some(render_card(article_card(element, "threads", "Threads", href, "Threads post")))
}

pub(super) fn render_instagram(element: &ComponentElement<'_>) -> Option<String> {
    let href = provider_url(element)?;
    let parsed = parse_https_url(href)?;
    if !host_in(&parsed.host, &["instagram.com", "www.instagram.com"]) {
        return None;
    }

    Some(render_card(article_card(element, "instagram", "Instagram", href, "Instagram post")))
}

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

fn article_card<'a>(
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

fn render_social_card(
    element: &ComponentElement<'_>,
    modifier: &'static str,
    network: &'static str,
    fallback_title: Option<&str>,
) -> Option<String> {
    let href = provider_url(element)?;
    let parsed = parse_https_url(href)?;
    let title = fallback_title.unwrap_or("Fediverse post");
    let mut card = article_card(element, modifier, network, href, title);
    card.meta.push(("Instance", Some(parsed.host.as_str())));
    Some(render_card(card))
}

pub(super) fn render_card(card: Card<'_>) -> String {
    let mut html = String::new();
    html.push_str("<article class=\"ox-provider-card ox-provider-card--");
    html.push_str(card.modifier);
    html.push_str("\"><a class=\"ox-provider-card__main\" href=\"");
    escape_attr(card.href, &mut html);
    html.push_str("\" target=\"_blank\" rel=\"noopener noreferrer\"><header class=\"ox-provider-card__header\"><span class=\"ox-provider-card__network\">");
    escape_text(card.network, &mut html);
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

fn render_meta(html: &mut String, card: &Card<'_>) {
    html.push_str("<footer class=\"ox-provider-card__meta\">");
    for (label, value) in &card.meta {
        let Some(value) = value.filter(|value| !value.trim().is_empty()) else {
            continue;
        };
        html.push_str("<span class=\"ox-provider-card__metric\"><span>");
        escape_text(label, html);
        html.push_str("</span> ");
        escape_text(value, html);
        html.push_str("</span>");
    }
    html.push_str("<span class=\"ox-provider-card__source\">");
    escape_text(card.source_label, html);
    html.push_str("</span></footer>");
}

pub(super) fn provider_url<'a>(element: &'a ComponentElement<'_>) -> Option<&'a str> {
    attr(element, "url")
        .or_else(|| attr(element, "href"))
        .or_else(|| attr(element, "src"))
        .or_else(|| (!element.body.trim().is_empty()).then(|| element.body.trim()))
        .filter(|value| is_safe_https_url(value))
}

pub(super) fn body_text<'a>(element: &'a ComponentElement<'_>) -> Option<&'a str> {
    let body = element.body.trim();
    (!body.is_empty() && !body.starts_with("https://")).then_some(body)
}

pub(super) fn first_attr<'a>(element: &'a ComponentElement<'_>, names: &[&str]) -> Option<&'a str> {
    names.iter().find_map(|name| attr(element, name))
}

fn is_google_maps_embed(input: &str) -> bool {
    let Some(parsed) = parse_https_url(input) else {
        return false;
    };
    host_in(&parsed.host, &["www.google.com", "google.com", "maps.google.com"])
        && parsed.path.starts_with("/maps/embed")
}

pub(super) fn is_safe_https_url(input: &str) -> bool {
    parse_https_url(input).is_some()
}

pub(super) struct ParsedHttpsUrl<'a> {
    pub(super) host: String,
    pub(super) path: &'a str,
    pub(super) query: Option<&'a str>,
}

pub(super) fn parse_https_url(input: &str) -> Option<ParsedHttpsUrl<'_>> {
    let trimmed = input.trim();
    let rest = trimmed.strip_prefix("https://").or_else(|| trimmed.strip_prefix("HTTPS://"))?;
    let authority_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let authority = &rest[..authority_end];
    if authority.is_empty() || authority.contains('@') || authority.contains(':') {
        return None;
    }
    let host = authority.to_ascii_lowercase();
    if host.ends_with('.') {
        return None;
    }

    let after_authority = &rest[authority_end..];
    let path_end = after_authority.find(['?', '#']).unwrap_or(after_authority.len());
    let path = &after_authority[..path_end];
    if path.contains('\\') {
        return None;
    }
    let query = (after_authority.as_bytes().get(path_end) == Some(&b'?')).then(|| {
        let query = &after_authority[path_end + 1..];
        query.split_once('#').map_or(query, |(query, _fragment)| query)
    });
    Some(ParsedHttpsUrl {
        host,
        path: if path.is_empty() { "/" } else { path },
        query: query.filter(|value| !value.is_empty()),
    })
}

pub(super) fn host_in(host: &str, allowed: &[&str]) -> bool {
    allowed.contains(&host)
}
