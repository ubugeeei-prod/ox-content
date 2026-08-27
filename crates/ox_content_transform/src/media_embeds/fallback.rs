//! What an enabled provider renders when it cannot resolve its input.
//!
//! A renderer returns `None` when the tag names something it will not embed —
//! a host it does not serve, a path shape it does not recognise, a scheme it
//! refuses. The tag was previously left in the document verbatim, which ships
//! `<zenn url="…">` to the browser. That parses as an unknown element, so the
//! card is not merely unstyled: a self-closing tag renders nothing at all and
//! the link the author wrote disappears from the page.
//!
//! A link is always renderable from a safe URL, needs no network, and is the
//! same in an offline build as an online one.
//!
//! It carries no provider identity — no `--speakerdeck` modifier, no data
//! attribute. A renderer says only "not mine", never why, so the tag that
//! reaches here may be a real provider URL of an unrecognised shape or a
//! look-alike host like `speakerdeck.com.evil.com`. Naming the provider on the
//! second would let a spoofed host borrow that provider's styling, and there
//! is no way to tell the two apart from `None`.

use super::html::{ComponentElement, attr};
use super::provider_cards::{is_safe_https_url, provider_url};
use super::render::{escape_attr, escape_text};

/// The URL a refused tag can still be linked to, if any.
///
/// Shared with the diagnostic so a report names the same URL the reader ends
/// up with, rather than one the transform decided against.
pub(super) fn fallback_url<'a>(element: &'a ComponentElement<'_>) -> Option<&'a str> {
    provider_url(element).or_else(|| {
        attr(element, "url")
            .or_else(|| attr(element, "href"))
            .filter(|value| is_safe_https_url(value))
    })
}

/// A plain link standing in for an embed that could not be resolved.
///
/// Returns `None` when the tag carries no URL safe enough to link to, in which
/// case the caller keeps the original markup — there is nothing better to say.
pub(super) fn render_fallback(element: &ComponentElement<'_>) -> Option<String> {
    let href = fallback_url(element)?;

    let label = link_label(element, href);

    let mut html = String::new();
    html.push_str("<a class=\"ox-embed-fallback\" href=\"");
    escape_attr(href, &mut html);
    html.push_str("\" target=\"_blank\" rel=\"noopener noreferrer\">");
    escape_text(label, &mut html);
    html.push_str("</a>");
    Some(html)
}

/// Prefer what the author wrote over the raw URL, so the fallback still reads
/// as the sentence it was written into.
fn link_label<'a>(element: &'a ComponentElement<'_>, href: &'a str) -> &'a str {
    let body = element.body.trim();
    if !body.is_empty() && body != href {
        return body;
    }
    attr(element, "title").unwrap_or(href)
}
