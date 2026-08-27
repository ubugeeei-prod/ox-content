//! Providers whose subject is a document rather than a post or a video:
//! note.com articles, Figma files, and Google Slides decks.

use super::html::{ComponentElement, attr};
use super::provider_cards::{
    Card, article_card, body_text, first_attr, host_in, is_safe_https_url, parse_https_url,
    path_segments, provider_url, render_card,
};

/// note.com long-form posts: `/{user}/n/{id}`, and the magazine form
/// `/{user}/m/{id}`. A bare profile is not an article, so it falls through.
pub(super) fn render_note(element: &ComponentElement<'_>) -> Option<String> {
    let href = provider_url(element)?;
    let parsed = parse_https_url(href)?;
    if !host_in(&parsed.host, &["note.com"]) {
        return None;
    }
    let segments = path_segments(parsed.path);
    if !matches!(segments.as_slice(), [_user, "n" | "m", _id, ..]) {
        return None;
    }

    Some(render_card(article_card(element, "note", "note", href, "note article")))
}

/// Figma files, designs, boards, and prototypes.
///
/// The share URL carries the file key in the second segment; the rest is a
/// human-readable slug that Figma ignores.
pub(super) fn render_figma(element: &ComponentElement<'_>) -> Option<String> {
    let href = provider_url(element)?;
    let parsed = parse_https_url(href)?;
    if !host_in(&parsed.host, &["figma.com", "www.figma.com"]) {
        return None;
    }
    let segments = path_segments(parsed.path);
    let (kind, key) = match segments.as_slice() {
        [kind @ ("file" | "design" | "board" | "proto" | "slides"), key, ..] => (*kind, *key),
        _ => return None,
    };
    safe_file_key(key)?;

    let fallback = match kind {
        "proto" => "Figma prototype",
        "board" => "FigJam board",
        "slides" => "Figma slides",
        _ => "Figma file",
    };
    Some(render_card(Card {
        modifier: "figma",
        network: "Figma",
        href,
        title: first_attr(element, &["title", "name"]).unwrap_or(fallback),
        body: body_text(element).or_else(|| attr(element, "description")),
        source_label: "Open in Figma",
        image: first_attr(element, &["image", "thumbnail", "preview"])
            .filter(|value| is_safe_https_url(value)),
        avatar: None,
        author: first_attr(element, &["author", "authorName", "team"]),
        date: first_attr(element, &["dateTime", "updatedAt", "date"]),
        date_label: first_attr(element, &["dateLabel", "updatedLabel"]),
        meta: vec![("Project", attr(element, "project"))],
        iframe: first_attr(element, &["embed", "embedUrl", "iframe"])
            .filter(|value| is_figma_embed(value)),
    }))
}

/// Google Slides decks: `docs.google.com/presentation/d/{id}/…`.
///
/// `/d/e/{token}/` is the published-to-web form, whose token is not the file
/// id; both are accepted, and neither is dereferenced at build time.
pub(super) fn render_google_slides(element: &ComponentElement<'_>) -> Option<String> {
    let href = provider_url(element)?;
    let parsed = parse_https_url(href)?;
    if !host_in(&parsed.host, &["docs.google.com"]) {
        return None;
    }
    let segments = path_segments(parsed.path);
    let key = match segments.as_slice() {
        ["presentation", "d", "e", token, ..] => *token,
        ["presentation", "d", id, ..] => *id,
        _ => return None,
    };
    safe_file_key(key)?;

    Some(render_card(Card {
        modifier: "googleslides",
        network: "Google Slides",
        href,
        title: first_attr(element, &["title", "name"]).unwrap_or("Google Slides deck"),
        body: body_text(element).or_else(|| attr(element, "description")),
        source_label: "Open deck",
        image: first_attr(element, &["image", "thumbnail", "preview"])
            .filter(|value| is_safe_https_url(value)),
        avatar: None,
        author: first_attr(element, &["author", "authorName", "presenter"]),
        date: first_attr(element, &["dateTime", "updatedAt", "date"]),
        date_label: first_attr(element, &["dateLabel", "updatedLabel"]),
        meta: vec![("Slides", first_attr(element, &["slides", "slideCount"]))],
        iframe: first_attr(element, &["embed", "embedUrl", "iframe"])
            .filter(|value| is_google_slides_embed(value)),
    }))
}

/// A Figma or Drive file key: URL-safe, and long enough not to be a slug.
fn safe_file_key(value: &str) -> Option<&str> {
    let ok = !value.is_empty()
        && value.len() <= 128
        && value.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_');
    ok.then_some(value)
}

/// Only Figma's own embed host, so a look-alike cannot supply the frame.
fn is_figma_embed(input: &str) -> bool {
    parse_https_url(input)
        .is_some_and(|parsed| host_in(&parsed.host, &["figma.com", "www.figma.com"]))
}

fn is_google_slides_embed(input: &str) -> bool {
    parse_https_url(input).is_some_and(|parsed| {
        host_in(&parsed.host, &["docs.google.com"]) && parsed.path.starts_with("/presentation/")
    })
}
