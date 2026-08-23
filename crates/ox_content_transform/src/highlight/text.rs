//! Recovering a code element's source text from the markup around it.
//!
//! The rehype pass read a code element through the DOM's text nodes. This is
//! the same reading done over the raw HTML, which is what lets a page be
//! highlighted without an HTML parser in the loop.

use super::scan::find_tag_end;
use super::tag::ParsedStartTag;

/// A type cross-reference recovered from an `<a>` inside a code element.
///
/// Offsets are into the decoded source the highlighter sees, so they line up
/// with the text nodes of the highlighted markup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CodeLink {
    pub start: usize,
    pub end: usize,
    pub href: String,
}

/// Source text of a `<code>` element, plus any cross-reference ranges in it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RecoveredCode {
    pub text: String,
    pub links: Vec<CodeLink>,
}

/// The source text of a `<code>` element.
///
/// The element is not always plain: a block carrying code annotations arrives
/// wrapped in `<span class="line">`, and a member type arrives with an `<a>`
/// around the type name it cross-references. The rehype pass read both through
/// the DOM's text nodes — dropping the wrappers, and for a block re-applying
/// the line metadata afterwards — so those are stripped here too rather than
/// treated as a reason to decline the element. Declining sends the whole page
/// back through the HTML round trip this exists to avoid.
///
/// Anything heavier than a `<span>` means the block is not what it claims to
/// be. A JSDoc `@example` whose body was itself run through the Markdown
/// renderer arrives with `<p>` and `<a>` nested inside `<code>`, which is not
/// well-formed and which a text scan and a real HTML parser recover
/// differently. Those are declined so the DOM path stays the authority on
/// malformed input.
pub(super) fn code_text(element: &str) -> Option<String> {
    recover_code(element).map(|recovered| recovered.text)
}

/// [`code_text`] plus the `<a href>` ranges that wrapping dropped.
pub(super) fn recover_code(element: &str) -> Option<RecoveredCode> {
    let code_open = element.find("<code")?;
    let content_start = element[code_open..].find('>')? + code_open + 1;
    let content_end = element.rfind("</code>")?;
    if content_end < content_start {
        return None;
    }

    let content = &element[content_start..content_end];
    if !content.contains('<') {
        return Some(RecoveredCode { text: decode_entities(content), links: Vec::new() });
    }

    let mut text = String::with_capacity(content.len());
    let mut links = Vec::new();
    let mut open_link: Option<(usize, String)> = None;
    let mut cursor = 0;
    while let Some(relative) = content[cursor..].find('<') {
        let open = cursor + relative;
        text.push_str(&decode_entities(&content[cursor..open]));
        let Some(close) = find_tag_end(content, open) else {
            // A `<` with no closing `>` is text the renderer failed to escape,
            // not a tag; keep it rather than truncating the source.
            text.push_str(&decode_entities(&content[open..]));
            return Some(RecoveredCode { text, links });
        };
        let tag = &content[open..close];
        if !is_text_wrapper(tag) {
            return None;
        }
        if let Some(href) = anchor_href(tag) {
            open_link = Some((text.len(), href));
        } else if is_closing_anchor(tag)
            && let Some((start, href)) = open_link.take()
            && start < text.len()
        {
            links.push(CodeLink { start, end: text.len(), href });
        }
        cursor = close;
    }
    text.push_str(&decode_entities(&content[cursor..]));
    Some(RecoveredCode { text, links })
}

fn anchor_href(tag: &str) -> Option<String> {
    let parsed = ParsedStartTag::parse(tag)?;
    if !parsed.name.eq_ignore_ascii_case("a") {
        return None;
    }
    parsed.attribute_value("href").filter(|href| !href.is_empty()).map(str::to_string)
}

fn is_closing_anchor(tag: &str) -> bool {
    tag.strip_prefix("</")
        .and_then(|rest| rest.strip_suffix('>'))
        .is_some_and(|rest| rest.trim().eq_ignore_ascii_case("a"))
}

/// Elements that may wrap part of a code element's text.
///
/// `span` comes from code annotations; `a` is the type cross-reference the
/// docs generator puts inside a member type. Both wrap text and nothing else,
/// so dropping them recovers exactly the source the DOM walk read.
const TEXT_WRAPPERS: [&str; 2] = ["span", "a"];

/// Whether `tag` opens or closes one of [`TEXT_WRAPPERS`].
fn is_text_wrapper(tag: &str) -> bool {
    let name = tag.trim_start_matches('<').trim_start_matches('/');
    let end = name.find(|c: char| !c.is_ascii_alphanumeric()).unwrap_or(name.len());
    TEXT_WRAPPERS.iter().any(|wrapper| name[..end].eq_ignore_ascii_case(wrapper))
}

/// Next HTML character in `text`: raw byte length, then decoded UTF-8 length.
///
/// Highlighted markup re-escapes the same five characters the renderer does,
/// so walking it with this keeps a cursor lined up with [`RecoveredCode::text`].
pub(super) fn next_decoded(text: &str) -> (usize, usize) {
    if text.is_empty() {
        return (0, 0);
    }
    if let Some(rest) = text.strip_prefix('&') {
        let bound = rest.len().min(11);
        if let Some(semi) = rest.as_bytes()[..bound].iter().position(|&b| b == b';') {
            let decoded = match &rest[..semi] {
                "amp" => Some('&'),
                "lt" => Some('<'),
                "gt" => Some('>'),
                "quot" => Some('"'),
                "apos" => Some('\''),
                entity => numeric_entity(entity),
            };
            if let Some(ch) = decoded {
                return (semi + 2, ch.len_utf8());
            }
        }
    }
    let len = text.chars().next().map_or(0, char::len_utf8);
    (len, len)
}

/// Reverses the renderer's escaping, plus the numeric forms other producers
/// emit for the same five bytes.
fn decode_entities(text: &str) -> String {
    if !text.contains('&') {
        return text.to_string();
    }

    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find('&') {
        out.push_str(&rest[..at]);
        let tail = &rest[at..];
        // Scan bytes, not a string slice: capping the search by byte length
        // and then slicing would split a multi-byte character and panic. `;`
        // is ASCII, so a byte search finds exactly the same positions.
        let Some(semi) = tail.as_bytes()[..tail.len().min(12)].iter().position(|&b| b == b';')
        else {
            out.push('&');
            rest = &tail[1..];
            continue;
        };
        let entity = &tail[1..semi];
        let decoded = match entity {
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" => Some('\''),
            _ => numeric_entity(entity),
        };
        if let Some(ch) = decoded {
            out.push(ch);
            rest = &tail[semi + 1..];
        } else {
            out.push('&');
            rest = &tail[1..];
        }
    }
    out.push_str(rest);
    out
}

fn numeric_entity(entity: &str) -> Option<char> {
    let digits = entity.strip_prefix('#')?;
    let code = match digits.strip_prefix(['x', 'X']) {
        Some(hex) => u32::from_str_radix(hex, 16).ok()?,
        None => digits.parse().ok()?,
    };
    char::from_u32(code)
}
