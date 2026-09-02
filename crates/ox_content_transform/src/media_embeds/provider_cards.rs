use super::card_render::{Card, article_card, render_card, render_link_preview_card};
use super::html::{ComponentElement, attr};

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

    Some(render_link_preview_card(article_card(element, "qiita", "Qiita", href, "Qiita article")))
}

pub(super) fn render_zenn(element: &ComponentElement<'_>) -> Option<String> {
    let href = provider_url(element)?;
    let parsed = parse_https_url(href)?;
    if !host_in(&parsed.host, &["zenn.dev"])
        || !["/articles/", "/books/", "/scraps/"].iter().any(|needle| parsed.path.contains(needle))
    {
        return None;
    }

    Some(render_link_preview_card(article_card(element, "zenn", "Zenn", href, "Zenn article")))
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

/// Non-empty path segments, so `/a//b/` reads as `["a", "b"]`.
pub(super) fn path_segments(path: &str) -> Vec<&str> {
    path.split('/').filter(|segment| !segment.is_empty()).collect()
}

pub(super) fn host_in(host: &str, allowed: &[&str]) -> bool {
    allowed.contains(&host)
}
