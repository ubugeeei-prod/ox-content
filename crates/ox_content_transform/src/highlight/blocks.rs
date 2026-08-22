//! Highlighting every code block in a rendered document, without leaving Rust.
//!
//! The previous path walked the whole page through an HTML parser and
//! serializer to find `<pre><code>` elements, then parsed each highlighted
//! block back into a tree to splice it in. On the documentation corpus that
//! cost 139 ms for the page round trips and another 38 ms for the per-block
//! re-parses, against 14 ms for the highlighting itself — the plumbing was an
//! order of magnitude more expensive than the work.
//!
//! This finds the blocks with the same scanner the merge step already uses,
//! and splices the result straight back into the original HTML.

use super::scan::{collect_inline_code, collect_pre_blocks, find_tag_end};
use super::{
    extract_code_block_metadata, merge_highlighted_code_block, merge_highlighted_inline_code,
};

/// Outcome of highlighting a document's code blocks.
pub struct HighlightedDocument {
    /// The document with every handled block replaced.
    pub html: String,
    /// Languages of blocks left untouched, so the caller can decide whether
    /// another highlighter still needs to run over the result.
    pub skipped: Vec<String>,
}

/// What kind of element a scanned region holds.
///
/// The two are highlighted from the same source text but spliced back
/// differently: a block keeps its `<pre>` wrapper, an inline element must not
/// grow one.
enum Region {
    Block,
    Inline,
}

/// An element this pass has claimed, with the text to highlight it from.
struct Claim {
    start: usize,
    end: usize,
    region: Region,
    language: String,
    source: String,
}

/// Highlights every `class="language-…"` code element in `html`, or none.
///
/// `supports` answers whether a language has a grammar; `highlight` receives
/// an element's source text and language and returns a full replacement
/// `<pre>` element. Attributes, classes and per-line metadata on the original
/// are carried over by the same merges the rehype path used, so annotated
/// blocks and API signatures come out identical.
///
/// The document is all-or-nothing on purpose. A caller that gets a non-empty
/// `skipped` has to run its HTML-parser-based highlighter over the whole page
/// anyway, and that pass redoes every block — so highlighting the claimable
/// ones first would be work thrown away. Claims are therefore collected with a
/// scan that costs no highlighting, and the moment one element is out of reach
/// the original `html` is handed back untouched.
pub fn highlight_code_blocks(
    html: &str,
    supports: impl Fn(&str) -> bool,
    highlight: impl Fn(&str, &str) -> Option<String>,
) -> HighlightedDocument {
    let mut regions: Vec<(usize, usize, Region)> = collect_pre_blocks(html)
        .into_iter()
        .map(|(start, end)| (start, end, Region::Block))
        .chain(
            collect_inline_code(html).into_iter().map(|(start, end)| (start, end, Region::Inline)),
        )
        .collect();
    if regions.is_empty() {
        return HighlightedDocument { html: html.to_string(), skipped: Vec::new() };
    }
    // Both scanners walk the document in order and neither can overlap the
    // other, so ordering by start offset is enough to splice in one pass.
    regions.sort_by_key(|&(start, _, _)| start);

    let mut claims = Vec::with_capacity(regions.len());
    let mut skipped = Vec::new();

    for (start, end, region) in regions {
        let element = &html[start..end];
        let Some(language) = element_language(element, &region) else {
            // No language to highlight with. An inline `<code>` without one is
            // ordinary prose markup rather than a snippet we declined, so it
            // is not reported as skipped.
            continue;
        };

        if !supports(&language) {
            skipped.push(language);
            continue;
        }

        let Some(source) = code_text(element) else {
            skipped.push(language);
            continue;
        };

        claims.push(Claim { start, end, region, language, source });
    }

    if !skipped.is_empty() {
        return HighlightedDocument { html: html.to_string(), skipped };
    }

    let mut out = String::with_capacity(html.len() * 2);
    let mut cursor = 0;

    for claim in claims {
        let element = &html[claim.start..claim.end];
        let Some(highlighted) = highlight(&claim.source, &claim.language) else {
            skipped.push(claim.language);
            continue;
        };

        let merged = match claim.region {
            Region::Block => {
                Some(merge_highlighted_code_block(element, &highlighted, Some(&claim.language)))
            }
            Region::Inline => merge_highlighted_inline_code(element, &highlighted),
        };
        let Some(merged) = merged else {
            skipped.push(claim.language);
            continue;
        };

        out.push_str(&html[cursor..claim.start]);
        out.push_str(&merged);
        cursor = claim.end;
    }

    // `supports` promised every claim, so a failure here is the highlighter
    // contradicting itself rather than an expected fallback; hand back the
    // original so the caller's own pass produces the page.
    if !skipped.is_empty() {
        return HighlightedDocument { html: html.to_string(), skipped };
    }

    out.push_str(&html[cursor..]);
    HighlightedDocument { html: out, skipped }
}

/// The language to highlight the element as, if it should be highlighted.
///
/// A block without a `language-` class still gets highlighted, as plain text —
/// that is what a bare ``` fence produces, and the rehype pass rendered those
/// with the same wrapper markup as any other block. An inline `<code>` without
/// one is ordinary prose markup instead, and is left alone.
fn element_language(element: &str, region: &Region) -> Option<String> {
    match region {
        Region::Block => Some(
            extract_code_block_metadata(element).language.unwrap_or_else(|| "text".to_string()),
        ),
        Region::Inline => super::find_next_start_tag(element, 0)?
            .tag
            .class_names()
            .iter()
            .find_map(|class_name| class_name.strip_prefix("language-"))
            .map(ToString::to_string),
    }
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
fn code_text(element: &str) -> Option<String> {
    let code_open = element.find("<code")?;
    let content_start = element[code_open..].find('>')? + code_open + 1;
    let content_end = element.rfind("</code>")?;
    if content_end < content_start {
        return None;
    }

    let content = &element[content_start..content_end];
    if !content.contains('<') {
        return Some(decode_entities(content));
    }

    let mut text = String::with_capacity(content.len());
    let mut cursor = 0;
    while let Some(relative) = content[cursor..].find('<') {
        let open = cursor + relative;
        text.push_str(&content[cursor..open]);
        let Some(close) = find_tag_end(content, open) else {
            // A `<` with no closing `>` is text the renderer failed to escape,
            // not a tag; keep it rather than truncating the source.
            text.push_str(&content[open..]);
            return Some(decode_entities(&text));
        };
        if !is_text_wrapper(&content[open..close]) {
            return None;
        }
        cursor = close;
    }
    text.push_str(&content[cursor..]);
    Some(decode_entities(&text))
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
