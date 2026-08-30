//! Emphasis and strong emphasis via the CommonMark delimiter stack.
//!
//! During inline parsing every `*`/`_` run is pushed as a plain text node
//! plus a [`Delimiter`] record carrying its flanking classification. Once
//! the inline sequence is complete, [`Parser::process_emphasis`] pairs
//! closers with openers (nearest matching opener, rule of three), wraps
//! the nodes between into `Emphasis`/`Strong`, and trims the delimiter
//! text nodes in place. Unpaired runs simply stay literal text.

use ox_content_allocator::Vec;
use ox_content_ast::{Node, Span, Text};

use crate::parser::Parser;
#[allow(unused_imports)]
use crate::{profile_span, profile_span_detail};

pub(in crate::parser) struct Delimiter {
    /// Index of the run's text node in the children vec.
    node_index: usize,
    marker: u8,
    /// Original run length (rule-of-three checks use this).
    orig_len: usize,
    /// Unconsumed delimiter characters remaining in the text node.
    remaining: usize,
    can_open: bool,
    can_close: bool,
}

impl<'a> Parser<'a> {
    /// Records a `*`/`_` run: pushes its text node and the delimiter
    /// entry describing how it may participate in emphasis.
    pub(in crate::parser) fn push_delimiter_run(
        &self,
        content: &'a str,
        offset: usize,
        children: &mut Vec<'a, Node<'a>>,
        delimiters: &mut Vec<'a, Delimiter>,
        pos: &mut usize,
    ) {
        profile_span_detail!("parser::inline_delimiter_run");
        let bytes = content.as_bytes();
        let marker = bytes[*pos];
        let run_len = Self::marker_run_len(bytes, *pos, marker);

        let prev_char = content[..*pos].chars().next_back();
        let next_char = content[*pos + run_len..].chars().next();
        let (can_open, can_close) =
            classify_flanking(marker, prev_char, next_char, self.options.cjk_emphasis);

        Self::push_text(
            children,
            &content[*pos..*pos + run_len],
            offset + *pos,
            offset + *pos + run_len,
        );
        delimiters.push(Delimiter {
            node_index: children.len() - 1,
            marker,
            orig_len: run_len,
            remaining: run_len,
            can_open,
            can_close,
        });
        *pos += run_len;
    }

    /// Pairs delimiters and restructures `children` into the emphasis
    /// tree, spec algorithm "process emphasis".
    ///
    /// Two things keep it linear on delimiter-dense sequences. Paired
    /// nodes are lifted out of `children` in place, leaving empty text
    /// slots behind, so `node_index` never moves and no pairing has to
    /// walk the vector fixing indices up. And a failed opener search
    /// records how far down it looked (`openers_bottom`), so the next
    /// closer of the same class does not repeat it.
    pub(in crate::parser) fn process_emphasis(
        &self,
        children: &mut Vec<'a, Node<'a>>,
        delimiters: &mut Vec<'a, Delimiter>,
    ) {
        profile_span!("parser::process_emphasis");
        let mut openers_bottom = OpenersBottom::default();
        let mut closer_idx = 0;
        while closer_idx < delimiters.len() {
            if !delimiters[closer_idx].can_close || delimiters[closer_idx].remaining == 0 {
                closer_idx += 1;
                continue;
            }

            let bottom = openers_bottom.get(&delimiters[closer_idx]);
            let Some(opener_idx) = find_opener(delimiters, closer_idx, bottom) else {
                // Nothing below this closer can pair with a later closer of
                // the same class either, so remember where to stop next time.
                openers_bottom.set(&delimiters[closer_idx]);
                if !delimiters[closer_idx].can_open {
                    // It cannot open and has just failed to close: retire it
                    // rather than shifting the rest of the vector down.
                    delimiters[closer_idx].remaining = 0;
                }
                closer_idx += 1;
                continue;
            };

            let use_delims: u32 =
                if delimiters[opener_idx].remaining >= 2 && delimiters[closer_idx].remaining >= 2 {
                    2
                } else {
                    1
                };
            let opener_node = delimiters[opener_idx].node_index;
            let closer_node = delimiters[closer_idx].node_index;

            // Lift the nodes between the delimiters into the new emphasis
            // node and leave empty text behind, so every index stays put.
            // Text nodes emptied by earlier (inner) pairings are dropped on
            // the way — they render as nothing. The range is never empty:
            // two runs of the same marker cannot be adjacent children.
            let mut inner = self.allocator.new_vec();
            for slot in &mut children[opener_node + 1..closer_node] {
                let node = core::mem::replace(slot, empty_text());
                if !matches!(&node, Node::Text(text) if text.value.is_empty()) {
                    inner.push(node);
                }
            }
            let span = inner_span(&inner, use_delims);
            let node = if use_delims == 2 {
                Node::Strong(self.allocator.boxed(ox_content_ast::Strong { children: inner, span }))
            } else {
                Node::Emphasis(
                    self.allocator.boxed(ox_content_ast::Emphasis { children: inner, span }),
                )
            };
            children[opener_node + 1] = node;

            // Trim the delimiter text nodes in place (they may end up
            // empty, which renders as nothing).
            trim_text_tail(&mut children[opener_node], use_delims);
            trim_text_head(&mut children[closer_node], use_delims);

            // Delimiters strictly inside the pair are now unreachable, and
            // the pair itself has spent `use_delims` characters. Retiring an
            // entry means zeroing `remaining`: both the loop above and
            // `find_opener` already skip those, and erasing it from the
            // vector instead would shift every later entry down — which is
            // what made a paragraph full of emphasis quadratic.
            for delimiter in &mut delimiters[opener_idx + 1..closer_idx] {
                delimiter.remaining = 0;
            }
            delimiters[opener_idx].remaining -= use_delims as usize;
            delimiters[closer_idx].remaining -= use_delims as usize;
            // Stay on the closer: with characters left it retries against an
            // earlier opener, and otherwise the top of the loop steps over it.
        }

        // Emptied delimiter text nodes at this level render as nothing;
        // drop them so consumers see a clean tree.
        children.retain(|node| !matches!(node, Node::Text(text) if text.value.is_empty()));
    }
}

/// Placeholder left where a node has been lifted into an emphasis node.
/// Empty text renders as nothing and is dropped once pairing is done.
fn empty_text<'a>() -> Node<'a> {
    Node::Text(Text { value: "", span: Span::new(0, 0) })
}

/// Lowest opener still worth examining, per closer class.
///
/// The spec keys this on the delimiter character, the closer's original
/// run length modulo three, and whether the closer can also open — the
/// three things the rule of three consults. The stored value is a
/// `node_index`, which no longer moves, so it stays a valid bound for the
/// whole sequence.
#[derive(Default)]
struct OpenersBottom {
    star: [[Option<usize>; 2]; 3],
    underscore: [[Option<usize>; 2]; 3],
}

impl OpenersBottom {
    fn slot(&mut self, closer: &Delimiter) -> &mut Option<usize> {
        let table = if closer.marker == b'_' { &mut self.underscore } else { &mut self.star };
        &mut table[closer.orig_len % 3][usize::from(closer.can_open)]
    }

    fn get(&mut self, closer: &Delimiter) -> Option<usize> {
        *self.slot(closer)
    }

    fn set(&mut self, closer: &Delimiter) {
        *self.slot(closer) = Some(closer.node_index);
    }
}

/// Nearest opener that may pair with `delimiters[closer_idx]`, stopping at
/// `bottom` — a `node_index` a previous failed search for this closer class
/// already proved nothing below could match.
fn find_opener(
    delimiters: &[Delimiter],
    closer_idx: usize,
    bottom: Option<usize>,
) -> Option<usize> {
    let closer = &delimiters[closer_idx];
    for opener_idx in (0..closer_idx).rev() {
        let opener = &delimiters[opener_idx];
        if bottom.is_some_and(|bottom| opener.node_index < bottom) {
            return None;
        }
        if opener.marker != closer.marker || !opener.can_open || opener.remaining == 0 {
            continue;
        }
        // Rule of three: when one side can both open and close, sums
        // divisible by three only pair if both lengths are.
        let sum_of_three = (opener.can_close || closer.can_open)
            && (opener.orig_len + closer.orig_len).is_multiple_of(3)
            && !(opener.orig_len.is_multiple_of(3) && closer.orig_len.is_multiple_of(3));
        if sum_of_three {
            continue;
        }
        return Some(opener_idx);
    }
    None
}

fn inner_span(inner: &Vec<'_, Node<'_>>, use_delims: u32) -> Span {
    let start = inner.first().map_or(0, |node| node_span(node).start);
    let end = inner.last().map_or(start, |node| node_span(node).end);
    Span::new(start.saturating_sub(use_delims), end + use_delims)
}

fn node_span(node: &Node<'_>) -> Span {
    match node {
        Node::Text(n) => n.span,
        Node::Emphasis(n) => n.span,
        Node::Strong(n) => n.span,
        Node::InlineCode(n) => n.span,
        Node::Link(n) => n.span,
        Node::Image(n) => n.span,
        Node::Delete(n) => n.span,
        Node::Break(n) => n.span,
        Node::Html(n) => n.span,
        Node::FootnoteReference(n) => n.span,
        _ => Span::new(0, 0),
    }
}

fn trim_text_tail(node: &mut Node<'_>, count: u32) {
    if let Node::Text(text) = node {
        let new_len = text.value.len().saturating_sub(count as usize);
        text.value = &text.value[..new_len];
        text.span = Span::new(text.span.start, text.span.end - count);
    }
}

fn trim_text_head(node: &mut Node<'_>, count: u32) {
    if let Node::Text(text) = node {
        text.value = &text.value[(count as usize).min(text.value.len())..];
        text.span = Span::new(text.span.start + count, text.span.end);
    }
}

/// Flanking classification (CommonMark "Emphasis and strong emphasis").
/// Sequence boundaries count as whitespace.
///
/// `cjk_emphasis` reclassifies East Asian punctuation as an ordinary character
/// here and nowhere else; see [`ParserOptions::cjk_emphasis`].
///
/// [`ParserOptions::cjk_emphasis`]: crate::ParserOptions::cjk_emphasis
fn classify_flanking(
    marker: u8,
    prev: Option<char>,
    next: Option<char>,
    cjk_emphasis: bool,
) -> (bool, bool) {
    let is_punct =
        |ch: char| is_punctuation_like(ch) && !(cjk_emphasis && is_east_asian_punctuation(ch));

    let prev_ws = prev.is_none_or(char::is_whitespace);
    let next_ws = next.is_none_or(char::is_whitespace);
    let prev_punct = prev.is_some_and(is_punct);
    let next_punct = next.is_some_and(is_punct);

    let left_flanking = !next_ws && (!next_punct || prev_ws || prev_punct);
    let right_flanking = !prev_ws && (!prev_punct || next_ws || next_punct);

    if marker == b'*' {
        (left_flanking, right_flanking)
    } else {
        (
            left_flanking && (!right_flanking || prev_punct),
            right_flanking && (!left_flanking || next_punct),
        )
    }
}

/// Approximates the spec's Unicode punctuation class (general categories
/// P and S): anything printable that is neither alphanumeric nor
/// whitespace.
fn is_punctuation_like(ch: char) -> bool {
    ch.is_ascii_punctuation()
        || (!ch.is_ascii() && !ch.is_alphanumeric() && !ch.is_whitespace() && !ch.is_control())
}

/// Punctuation that East Asian scripts set directly against the text, with no
/// separating space — the reason CommonMark's flanking rules reject emphasis
/// that Latin text would accept.
///
/// The ranges are the fullwidth and CJK-specific blocks only. Halfwidth ASCII
/// punctuation is deliberately excluded even in CJK text: it is written the
/// same way in every script, so reclassifying it would change how ordinary
/// Latin documents parse.
///
/// U+3000 IDEOGRAPHIC SPACE falls inside the first range but is whitespace, and
/// the flanking rules test whitespace before punctuation, so it is unaffected.
fn is_east_asian_punctuation(ch: char) -> bool {
    matches!(ch,
        // CJK Symbols and Punctuation: 、。〈〉《》「」『』【】〜 and friends.
        '\u{3000}'..='\u{303F}'
        // Vertical forms and CJK Compatibility Forms: vertical/rotated variants.
        | '\u{FE10}'..='\u{FE19}'
        | '\u{FE30}'..='\u{FE4F}'
        // Small Form Variants: small comma, small full stop, small brackets.
        | '\u{FE50}'..='\u{FE6F}'
        // Fullwidth ASCII punctuation, split around the fullwidth digits and
        // letters, which stay alphanumeric.
        | '\u{FF01}'..='\u{FF0F}'
        | '\u{FF1A}'..='\u{FF20}'
        | '\u{FF3B}'..='\u{FF40}'
        | '\u{FF5B}'..='\u{FF65}'
    )
}
