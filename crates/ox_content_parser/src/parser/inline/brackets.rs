//! Link text parsed in place.
//!
//! CommonMark forbids a link inside a link, so whether `[text](url)` is a
//! link depends on what `text` parses to. That used to be answered by
//! parsing the text on the side and, whenever the answer was "it holds a
//! link", throwing the nodes away and parsing the same bytes again as
//! ordinary content. Each nesting level repeated the level below it:
//! `[[[[a](u)](u)](u)](u)` was parsed once per bracket, O(n²) nodes for n
//! brackets, on a call stack as deep as the nesting.
//!
//! A bracket that could become a link is instead left open while the main
//! loop carries on through its text. Reaching its `]` with no link inside
//! turns the nodes pushed since into the link's children. A link turning up
//! first settles the bracket the other way: its text is recorded as holding
//! a link, and the walk goes back to its `[` to read on with the bracket
//! literal. Nothing recurses, and the going back is paid for out of a
//! budget proportional to the content, so the whole walk stays linear.

use ox_content_allocator::Vec;
use ox_content_ast::{Link, Node, Span};
use rustc_hash::FxHashSet;
use smallvec::SmallVec;

use super::emphasis::Delimiter;
use super::wiki_link::WikiScan;
use crate::error::ParseResult;
use crate::parser::Parser;

/// The state of one `parse_inline` call.
pub(in crate::parser) struct InlineRun<'a> {
    pub(in crate::parser) children: Vec<'a, Node<'a>>,
    pub(in crate::parser) delimiters: Vec<'a, Delimiter>,
    /// Brackets whose text is being parsed, outermost first. Every one of
    /// them still has no link inside.
    pub(in crate::parser) open: SmallVec<[OpenBracket<'a>; 1]>,
    /// Height of the tallest child pushed since the innermost open bracket
    /// (or the start of the content). See `nesting.rs`.
    pub(in crate::parser) height: u32,
    /// Bytes the walk may still read a second time. See
    /// [`Parser::settle_open_bracket`].
    reread_budget: usize,
    pub(in crate::parser) wiki: WikiScan,
}

/// A bracket that becomes a link if its text holds none.
pub(in crate::parser) struct OpenBracket<'a> {
    /// Index of the `[` in the content.
    pub(in crate::parser) open: usize,
    /// The text is parsed as the slice `text_start..text_end`, exactly as
    /// the separate parse saw it: nothing inside can look past either end.
    pub(in crate::parser) text_start: usize,
    pub(in crate::parser) text_end: usize,
    /// Index just past the whole link.
    pub(in crate::parser) end: usize,
    pub(in crate::parser) url: &'a str,
    pub(in crate::parser) title: Option<&'a str>,
    /// `children.len()` and `delimiters.len()` when the bracket opened.
    pub(in crate::parser) child_index: usize,
    pub(in crate::parser) delimiter_index: usize,
    /// `InlineRun::height` when the bracket opened.
    pub(in crate::parser) outer_height: u32,
    pub(in crate::parser) kind: BracketKind,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(in crate::parser) enum BracketKind {
    /// `[text](url)` or a reference form. A literal `[` text node sits at
    /// `child_index` until the bracket closes.
    Link,
    /// `[[target|label]]`, with its `]]` at `close`. Nothing is pushed for
    /// the part before the label.
    Wiki { close: usize },
}

/// Bracketed text already found to parse to a link, as the addresses of its
/// two ends.
///
/// Every slice the inline parser sees lives in the source or the arena,
/// both of which outlive the parser, so the pair names one byte range for
/// as long as the set exists. It outlives the `parse_inline` call that
/// filled it on purpose: content parsed by a nested call — image alt text,
/// a strikethrough — is parsed again whenever the walk around it goes back,
/// and without the record each level would repeat the discovery, and the
/// going back, of every level inside it.
pub(in crate::parser) type LinkText = FxHashSet<(usize, usize)>;

/// How many times its own length a `parse_inline` call may read again
/// before open brackets are settled without going back one at a time.
const REREAD_FACTOR: usize = 8;

impl<'a> InlineRun<'a> {
    pub(in crate::parser) fn new(parser: &Parser<'a>, content_len: usize) -> Self {
        Self {
            children: parser
                .allocator
                .new_vec_with_capacity(Parser::inline_children_capacity(content_len)),
            delimiters: parser.allocator.new_vec(),
            open: SmallVec::new(),
            height: 0,
            reread_budget: content_len.saturating_mul(REREAD_FACTOR),
            wiki: WikiScan::new(),
        }
    }

    /// The slice inline constructs may currently read: the text of the
    /// innermost open bracket, or the whole content.
    pub(in crate::parser) fn window(&self, content_len: usize) -> (usize, usize) {
        self.open.last().map_or((0, content_len), |bracket| (bracket.text_start, bracket.text_end))
    }
}

impl<'a> Parser<'a> {
    /// Turns the innermost open bracket into a link: `pos` has reached the
    /// end of its text without meeting one.
    pub(super) fn close_bracket(
        &self,
        run: &mut InlineRun<'a>,
        offset: usize,
        pos: &mut usize,
    ) -> ParseResult<()> {
        let Some(bracket) = run.open.pop() else {
            return Ok(());
        };
        let first = match bracket.kind {
            BracketKind::Link => bracket.child_index + 1,
            BracketKind::Wiki { .. } => bracket.child_index,
        };

        let mut children = self.allocator.new_vec_with_capacity(run.children.len() - first);
        children.extend(run.children.drain(first..));
        run.children.truncate(bracket.child_index);

        let mut emphasis = 0;
        if run.delimiters.len() > bracket.delimiter_index {
            let delimiters = &mut run.delimiters[bracket.delimiter_index..];
            for delimiter in delimiters.iter_mut() {
                delimiter.rebase(first);
            }
            emphasis = self.process_emphasis(&mut children, delimiters)?;
            run.delimiters.truncate(bracket.delimiter_index);
        }

        run.height = bracket.outer_height.max(run.height.saturating_add(emphasis + 1));
        run.children.push(Node::Link(self.allocator.boxed(Link {
            url: bracket.url,
            title: bracket.title,
            children,
            span: Span::new((offset + bracket.open) as u32, (offset + bracket.end) as u32),
        })));
        *pos = bracket.end;
        Ok(())
    }

    /// A link starting at `link_start` was just pushed inside the innermost
    /// open bracket, so that bracket is not one.
    ///
    /// Its text is recorded and the walk goes back to its `[`: what was
    /// pushed since was read as link text, cut off from everything outside
    /// the bracket, and has to be read again as part of the content around
    /// it. The brackets further out stay open — read that way, the link may
    /// end up inside something else and never reach them.
    ///
    /// It usually does reach them, one going-back per bracket, each over
    /// the same stretch of text: quadratic for a deep nest. So the bytes
    /// read again are counted, and once that passes [`REREAD_FACTOR`] times
    /// the content every open bracket is settled together and the walk goes
    /// back once, to the outermost.
    pub(super) fn settle_open_bracket(
        &self,
        run: &mut InlineRun<'a>,
        content: &'a str,
        link_start: usize,
        pos: &mut usize,
    ) {
        let Some(innermost) = run.open.last() else {
            return;
        };
        let reread = pos.saturating_sub(innermost.open);
        let exhausted = run.reread_budget < reread;
        let keep = if exhausted {
            run.reread_budget = 0;
            0
        } else {
            run.reread_budget -= reread;
            run.open.len() - 1
        };

        let base = content.as_ptr().addr();
        let mut link_text = self.link_text.borrow_mut();
        for bracket in &run.open[keep..] {
            link_text.insert((base + bracket.text_start, base + bracket.text_end));
            // Past the budget, the link also counts against every later
            // candidate in front of the same `]]` (see `wiki_link.rs`).
            // That is an approximation — a candidate starting further in
            // may read its label differently — so it is kept to inputs
            // that would otherwise go quadratic.
            if let BracketKind::Wiki { close } = bracket.kind
                && exhausted
            {
                run.wiki.reject(close, link_start);
            }
        }

        let bracket = &run.open[keep];
        run.children.truncate(bracket.child_index);
        run.delimiters.truncate(bracket.delimiter_index);
        run.height = bracket.outer_height;
        *pos = bracket.open;
        run.open.truncate(keep);
    }

    /// Whether `content[start..end]` is already known to parse to a link.
    pub(in crate::parser) fn holds_link(&self, content: &str, start: usize, end: usize) -> bool {
        let link_text = self.link_text.borrow();
        if link_text.is_empty() {
            return false;
        }
        let base = content.as_ptr().addr();
        link_text.contains(&(base + start, base + end))
    }
}

/// Does any node in the tree contain a link?
pub(in crate::parser) fn contains_link(nodes: &[Node<'_>]) -> bool {
    nodes.iter().any(|node| match node {
        Node::Link(_) => true,
        Node::Emphasis(n) => contains_link(&n.children),
        Node::Strong(n) => contains_link(&n.children),
        Node::Delete(n) => contains_link(&n.children),
        Node::Superscript(n) => contains_link(&n.children),
        Node::Subscript(n) => contains_link(&n.children),
        _ => false,
    })
}
