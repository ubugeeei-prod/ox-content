//! The outbound-link marker, and which anchors should carry one.

use super::tag_name;

const EXTERNAL_ICON: &str = "<span class=\"ox-external-icon\" aria-hidden=\"true\"></span>";

/// The marker for an anchor, or nothing when the anchor wraps a whole card.
pub(super) fn external_marker(inner: &str) -> &'static str {
    if wraps_block_content(inner) { "" } else { EXTERNAL_ICON }
}

/// Block-level tags an inline text link never contains.
///
/// Embed cards wrap their whole body in one anchor, so the marker would land
/// as a stray last child: a full-width row under a `display: grid` provider
/// card, and a lone glyph after the footer of a repository or link-preview
/// card. Anchors around an avatar plus a name stay inline and keep theirs.
const BLOCK_CONTENT_TAGS: [&str; 14] = [
    "address",
    "article",
    "blockquote",
    "div",
    "footer",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "header",
    "section",
    "table",
];

fn wraps_block_content(inner: &str) -> bool {
    let mut rest = inner;
    while let Some(offset) = rest.find('<') {
        let tag = &rest[offset..];
        let name = tag_name(tag);
        if !name.is_empty()
            && BLOCK_CONTENT_TAGS.iter().any(|block| name.eq_ignore_ascii_case(block))
        {
            return true;
        }
        rest = &tag[1..];
    }
    false
}
