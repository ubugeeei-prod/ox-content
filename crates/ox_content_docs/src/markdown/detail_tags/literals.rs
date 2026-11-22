//! What detail Markdown keeps as written: code, raw HTML and Markdown link
//! syntax. Also finds the delimiter of a block the text leaves open.

use ox_content_ast::{
    CodeBlock, Definition, Document, Html, Image, InlineCode, Link, ListItem, Node, Span, Visit,
    walk_list_item,
};

/// Elements whose HTML block CommonMark ends at one of these end tags, not at a
/// blank line.
const RAW_TEXT_ELEMENTS: [(&str, &str); 4] = [
    ("pre", "</pre>"),
    ("script", "</script>"),
    ("style", "</style>"),
    ("textarea", "</textarea>"),
];

/// The other HTML blocks that only end at a delimiter. `<!` is any other
/// declaration, so it comes last.
const DELIMITED_HTML_BLOCKS: [(&str, &str); 4] =
    [("<!--", "-->"), ("<![cdata[", "]]>"), ("<?", "?>"), ("<!", ">")];

/// The source ranges that keep their text, sorted and not overlapping. A span
/// that does not match its node is skipped: the parser's offsets drift after a
/// tab in a list or a quote. A span can start with its line's indentation.
pub(super) fn literal_ranges(text: &str, document: &Document<'_>) -> Vec<(usize, usize)> {
    let mut literals = LiteralRanges { text, ranges: Vec::new() };
    literals.visit_document(document);
    literals.ranges.sort_unstable();
    let mut merged: Vec<(usize, usize)> = Vec::with_capacity(literals.ranges.len());
    for (start, end) in literals.ranges {
        match merged.last_mut() {
            Some((_, last_end)) if start < *last_end => *last_end = end.max(*last_end),
            _ => merged.push((start, end)),
        }
    }
    merged
}

/// The delimiter that closes the fence or HTML block `text` ends inside, if any.
/// Only a top-level block can run past the end of the text: a quote or a list
/// item ends with the line that leaves it.
pub(super) fn block_closer<'t>(text: &'t str, document: &Document<'_>) -> Option<&'t str> {
    match document.children.last()? {
        Node::CodeBlock(code_block) => open_fence(source(text, code_block.span)?),
        Node::Html(html) => open_html_block_closer(html.value),
        _ => None,
    }
}

/// Track top-level fences and delimited HTML without parsing inline Markdown.
pub(super) fn scan_block_closer(text: &str) -> Option<&str> {
    enum OpenBlock<'a> {
        Fence(&'a str),
        Html(&'static str),
    }
    let mut open = None;
    for line in text.lines() {
        match open {
            Some(OpenBlock::Fence(marker)) if closes_fence(line, marker) => open = None,
            Some(OpenBlock::Html(end)) if html_block_ended(&line.to_ascii_lowercase(), end) => {
                open = None;
            }
            Some(_) => {}
            None => {
                let trimmed = line.trim_start_matches(' ');
                if line.len() - trimmed.len() <= 3 {
                    open = fence_marker(trimmed)
                        .map(OpenBlock::Fence)
                        .or_else(|| open_html_block_closer(trimmed).map(OpenBlock::Html));
                }
            }
        }
    }
    open.map(|block| match block {
        OpenBlock::Fence(marker) => marker,
        OpenBlock::Html(end) => end,
    })
}

fn fence_marker(line: &str) -> Option<&str> {
    let marker = *line.as_bytes().first()?;
    if !matches!(marker, b'`' | b'~') {
        return None;
    }
    let length = line.bytes().take_while(|byte| *byte == marker).count();
    // A backtick in the info string makes the line an inline code span.
    (length >= 3 && (marker == b'~' || !line[length..].contains('`'))).then(|| &line[..length])
}

fn closes_fence(line: &str, marker: &str) -> bool {
    let closing = line.trim_start_matches(' ');
    let run = closing.trim_end();
    line.len() - closing.len() <= 3
        && run.len() >= marker.len()
        && run.bytes().all(|byte| marker.as_bytes().first() == Some(&byte))
}

fn open_fence(block: &str) -> Option<&str> {
    let line = block.trim_start_matches(' ');
    if block.len() - line.len() > 3 {
        return None;
    }
    let marker = fence_marker(line.lines().next()?)?;
    let closed = block.lines().skip(1).last().is_some_and(|last| closes_fence(last, marker));
    (!closed).then_some(marker)
}

fn open_html_block_closer(html: &str) -> Option<&'static str> {
    let html = html.trim_start();
    if !html.starts_with('<') {
        return None;
    }
    let html = html.to_ascii_lowercase();
    let (_, closer) = RAW_TEXT_ELEMENTS
        .iter()
        .find(|(name, _)| opens(&html, name))
        .or_else(|| DELIMITED_HTML_BLOCKS.iter().find(|(opener, _)| html.starts_with(opener)))?;
    (!html_block_ended(&html, closer)).then_some(*closer)
}

fn html_block_ended(html: &str, closer: &str) -> bool {
    if closer.starts_with("</") {
        RAW_TEXT_ELEMENTS.iter().any(|(_, end_tag)| html.contains(end_tag))
    } else {
        html.contains(closer)
    }
}

/// Whether `html` starts with the tag `name`, and not a longer name like `<preview>`.
fn opens(html: &str, name: &str) -> bool {
    html.strip_prefix('<')
        .and_then(|rest| rest.strip_prefix(name))
        .is_some_and(|rest| rest.is_empty() || rest.starts_with([' ', '\t', '\r', '\n', '>']))
}

fn source(text: &str, span: Span) -> Option<&str> {
    text.get(span.start as usize..span.end as usize)
}

struct LiteralRanges<'t> {
    text: &'t str,
    ranges: Vec<(usize, usize)>,
}

impl LiteralRanges<'_> {
    /// Keeps `span` if its text is what the node says it is.
    fn keep(&mut self, span: Span, is_expected: impl FnOnce(&str) -> bool) {
        if source(self.text, span).is_some_and(is_expected) {
            self.ranges.push((span.start as usize, span.end as usize));
        }
    }
}

impl<'a> Visit<'a> for LiteralRanges<'_> {
    fn visit_code_block(&mut self, code_block: &CodeBlock<'a>) {
        let first_line = code_block.value.lines().next().unwrap_or_default().trim();
        self.keep(code_block.span, |text| text.contains(first_line));
    }

    fn visit_inline_code(&mut self, inline_code: &InlineCode<'a>) {
        self.keep(inline_code.span, |text| {
            text.trim_start().starts_with('`') && text.ends_with('`')
        });
    }

    fn visit_html(&mut self, html: &Html<'a>) {
        self.keep(html.span, |text| text.trim_start().starts_with('<'));
    }

    fn visit_definition(&mut self, definition: &Definition<'a>) {
        self.keep(definition.span, |text| text.trim_start().starts_with('['));
    }

    fn visit_image(&mut self, image: &Image<'a>) {
        self.keep(image.span, |text| text.trim_start().starts_with("!["));
    }

    fn visit_link(&mut self, link: &Link<'a>) {
        // A bare autolink has no delimiters, so it is compared with its URL.
        self.keep(link.span, |text| {
            let text = text.trim_start();
            text.starts_with(['[', '<']) || link.url.ends_with(text)
        });
    }

    fn visit_list_item(&mut self, list_item: &ListItem<'a>) {
        if list_item.checked.is_some() {
            let item = list_item.span.start as usize;
            if let Some(offset) = self.text.get(item..).and_then(|text| text.find('[')) {
                let marker = item + offset;
                if matches!(self.text.get(marker..marker + 3), Some("[x]" | "[X]" | "[ ]")) {
                    self.ranges.push((marker, marker + 3));
                }
            }
        }
        walk_list_item(self, list_item);
    }
}
