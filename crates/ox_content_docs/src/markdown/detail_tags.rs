//! `@description` and `@remarks`: the details of a doc comment. Each renders as
//! block Markdown in its own section, right after the comment's summary.

mod literals;

use std::borrow::Cow;
use std::sync::OnceLock;

use ox_content_allocator::Allocator;
use ox_content_parser::{Parser, ParserOptions};
use ox_content_renderer::{HtmlRenderer, HtmlRendererOptions};

use self::literals::{block_closer, literal_ranges, scan_block_closer};
use super::links::render_jsdoc_inline_link_at;
use super::regex_cache::{RegexCache, cached_regex};
use super::{MarkdownLinkContext, process_doc_text};
use crate::model::ApiDocTag;

/// The parser bounds block nesting but not inline nesting: a stack overflow aborts
/// the process, and its time doubles with each nested image. Text that could nest
/// deeper than this is not parsed.
const MAX_BRACKETS: usize = 64;
const MAX_IMAGES: usize = 16;
const MAX_EMPHASIS_DELIMITERS: usize = 1024;

struct DetailKind {
    title: &'static str,
    modifier: &'static str,
    tags: &'static [&'static str],
}

/// The detail sections, in the order they render. `@desc` is JSDoc's synonym for
/// `@description`.
const DETAIL_KINDS: [DetailKind; 2] = [
    DetailKind { title: "Description", modifier: "description", tags: &["description", "desc"] },
    DetailKind { title: "Remarks", modifier: "remarks", tags: &["remarks"] },
];

/// One `@description` or `@remarks` tag, ready to render.
pub(super) struct DetailSection {
    pub(super) title: &'static str,
    /// The section's CSS modifier.
    pub(super) modifier: &'static str,
    /// The tag's Markdown, with its links resolved.
    pub(super) markdown: String,
}

/// Whether `name` renders as a detail section instead of in the `Tags` list.
pub(super) fn is_detail_tag(name: &str) -> bool {
    DETAIL_KINDS.iter().any(|kind| kind.tags.contains(&name))
}

/// The non-empty detail tags of a doc comment, `@description` before `@remarks`.
pub(super) fn detail_sections(
    tags: &[ApiDocTag],
    context: Option<&MarkdownLinkContext<'_>>,
) -> Vec<DetailSection> {
    let mut sections = Vec::new();
    for kind in &DETAIL_KINDS {
        for tag in tags.iter().filter(|tag| kind.tags.contains(&tag.tag.as_str())) {
            let value = trim_blank_edges(&tag.value);
            if !value.is_empty() {
                sections.push(DetailSection {
                    title: kind.title,
                    modifier: kind.modifier,
                    markdown: resolve_detail_markdown(value, context).into_owned(),
                });
            }
        }
    }
    sections
}

/// Renders detail Markdown with the full parser, so its quotes, tables and
/// nested lists survive. Raw HTML is escaped, as in the rest of the HTML style.
pub(super) fn render_detail_markdown(markdown: &str) -> Option<String> {
    if nests_too_deep(markdown) {
        return None;
    }
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, markdown, parser_options()).parse().ok()?;
    let mut options = HtmlRendererOptions::new();
    options.sanitize = true;
    Some(without_heading_ids(&HtmlRenderer::with_options(options).render(&document)))
}

/// What closes the fence or HTML block `markdown` ends inside, so that it cannot
/// swallow the Markdown after it. The HTML style renders each section on its
/// own and needs none.
pub(super) fn open_block_closer(markdown: &str) -> Option<&str> {
    let may_end_inside_a_block =
        markdown.contains("```") || markdown.contains("~~~") || markdown.contains('<');
    if !may_end_inside_a_block {
        return None;
    }
    if nests_too_deep(markdown) {
        return scan_block_closer(markdown);
    }
    let allocator = Allocator::new();
    match Parser::with_options(&allocator, markdown, parser_options()).parse() {
        Ok(document) => block_closer(markdown, &document),
        Err(_) => scan_block_closer(markdown),
    }
}

/// Footnotes stay off: their ids would repeat across the sections of one page.
fn parser_options() -> ParserOptions {
    ParserOptions { footnotes: false, ..ParserOptions::gfm() }
}

/// Heading ids would repeat across sections and could take over API anchors.
fn without_heading_ids(html: &str) -> String {
    static HEADING_ID_RE: RegexCache = OnceLock::new();
    match cached_regex(&HEADING_ID_RE, r#"<h([1-6]) id="[^"]*""#) {
        Some(re) => re.replace_all(html, "<h$1").into_owned(),
        None => html.to_string(),
    }
}

/// Drops the blank lines around `text`, but keeps its first line's indentation,
/// which can be what makes that line an indented code block.
fn trim_blank_edges(text: &str) -> &str {
    let text = text.trim_end();
    let content = text.len() - text.trim_start().len();
    &text[text[..content].rfind('\n').map_or(0, |newline| newline + 1)..]
}

/// Whether `text` holds enough openers to reach those limits. The parser alone
/// decides which `]` closes what, so every `[` counts, even in code.
fn nests_too_deep(text: &str) -> bool {
    let (mut brackets, mut images, mut delimiters, mut previous) =
        (0_usize, 0_usize, 0_usize, b' ');
    for &byte in text.as_bytes() {
        match byte {
            b'[' => {
                brackets += 1;
                images += usize::from(previous == b'!');
            }
            b'*' => delimiters += 1,
            // An intraword `_` cannot open emphasis, so `snake_case` is free.
            b'_' if !previous.is_ascii_alphanumeric() => delimiters += 1,
            _ => {}
        }
        previous = byte;
    }
    brackets > MAX_BRACKETS || images > MAX_IMAGES || delimiters > MAX_EMPHASIS_DELIMITERS
}

/// Resolves doc links in the prose, and leaves code and Markdown link syntax as
/// written.
fn resolve_detail_markdown<'a>(
    text: &'a str,
    context: Option<&MarkdownLinkContext<'_>>,
) -> Cow<'a, str> {
    // Most text has nothing to resolve, and then nothing needs parsing.
    if matches!(process_doc_text(text, context), Cow::Borrowed(_)) || nests_too_deep(text) {
        return Cow::Borrowed(text);
    }
    let allocator = Allocator::new();
    // Without the parser the code cannot be told apart, so the text stays as written.
    let Ok(document) = Parser::with_options(&allocator, text, parser_options()).parse() else {
        return Cow::Borrowed(text);
    };
    Cow::Owned(resolve_prose(text, &literal_ranges(text, &document), context))
}

/// Resolves links in the prose between `literals`. A `{@link …}` is one unit,
/// even when its label holds code.
fn resolve_prose(
    text: &str,
    literals: &[(usize, usize)],
    context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let mut out = String::with_capacity(text.len());
    let mut cursor = 0;
    let mut literals = literals.iter().copied().peekable();
    loop {
        let (start, end) = literals.peek().copied().unwrap_or((text.len(), text.len()));
        if let Some((open, link, link_end)) = next_inline_link(text, cursor, start, context) {
            out.push_str(&process_doc_text(&text[cursor..open], context));
            out.push_str(&link);
            cursor = link_end;
            while literals.next_if(|&(literal, _)| literal < link_end).is_some() {}
            continue;
        }
        out.push_str(&process_doc_text(&text[cursor..start], context));
        if literals.next().is_none() {
            return out;
        }
        out.push_str(&text[start..end]);
        cursor = end;
    }
}

fn next_inline_link(
    text: &str,
    from: usize,
    to: usize,
    context: Option<&MarkdownLinkContext<'_>>,
) -> Option<(usize, String, usize)> {
    text[from..to].match_indices("{@").find_map(|(offset, _)| {
        let (link, end) = render_jsdoc_inline_link_at(text, from + offset, context)?;
        (!crosses_a_block(&text[from + offset..end])).then_some((from + offset, link, end))
    })
}

/// A tag left open runs to the next `}`, which can be in a later block.
fn crosses_a_block(tag: &str) -> bool {
    tag.contains("\n\n") || tag.contains("```") || tag.contains("~~~")
}
