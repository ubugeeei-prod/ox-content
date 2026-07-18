//! Emphasis and strong emphasis via the CommonMark delimiter stack.
//!
//! During inline parsing every `*`/`_` run is pushed as a plain text node
//! plus a [`Delimiter`] record carrying its flanking classification. Once
//! the inline sequence is complete, [`Parser::process_emphasis`] pairs
//! closers with openers (nearest matching opener, rule of three), wraps
//! the nodes between into `Emphasis`/`Strong`, and trims the delimiter
//! text nodes in place. Unpaired runs simply stay literal text.

use ox_content_allocator::Vec;
use ox_content_ast::{Node, Span};

use crate::parser::Parser;

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
        content: &'a str,
        offset: usize,
        children: &mut Vec<'a, Node<'a>>,
        delimiters: &mut Vec<'a, Delimiter>,
        pos: &mut usize,
    ) {
        let bytes = content.as_bytes();
        let marker = bytes[*pos];
        let run_len = Self::marker_run_len(bytes, *pos, marker);

        let prev_char = content[..*pos].chars().next_back();
        let next_char = content[*pos + run_len..].chars().next();
        let (can_open, can_close) = classify_flanking(marker, prev_char, next_char);

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
    /// tree, spec algorithm "process emphasis" (without the
    /// openers_bottom optimization; delimiter counts per sequence are
    /// small in practice).
    pub(in crate::parser) fn process_emphasis(
        &self,
        children: &mut Vec<'a, Node<'a>>,
        delimiters: &mut Vec<'a, Delimiter>,
    ) {
        let mut closer_idx = 0;
        while closer_idx < delimiters.len() {
            if !delimiters[closer_idx].can_close || delimiters[closer_idx].remaining == 0 {
                closer_idx += 1;
                continue;
            }

            let Some(opener_idx) = find_opener(delimiters, closer_idx) else {
                if !delimiters[closer_idx].can_open {
                    delimiters.remove(closer_idx);
                } else {
                    closer_idx += 1;
                }
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

            // Move the nodes between the delimiters into the new
            // emphasis node, inserted right after the opener's text node.
            // Delimiter text nodes emptied by earlier (inner) pairings are
            // dropped on the way — they render as nothing.
            let mut inner = self.allocator.new_vec();
            inner.extend(
                children
                    .drain(opener_node + 1..closer_node)
                    .filter(|node| !matches!(node, Node::Text(text) if text.value.is_empty())),
            );
            let span = inner_span(&inner, use_delims);
            let node = if use_delims == 2 {
                Node::Strong(ox_content_ast::Strong { children: inner, span })
            } else {
                Node::Emphasis(ox_content_ast::Emphasis { children: inner, span })
            };
            children.insert(opener_node + 1, node);

            // Trim the delimiter text nodes in place (they may end up
            // empty, which renders as nothing).
            trim_text_tail(&mut children[opener_node], use_delims);
            let closer_node_now = opener_node + 2;
            trim_text_head(&mut children[closer_node_now], use_delims);

            // Drop delimiters between the pair and fix indices right of it.
            let removed_nodes = closer_node - opener_node - 2;
            delimiters.retain(|delimiter| {
                delimiter.node_index <= opener_node || delimiter.node_index >= closer_node
            });
            for delimiter in delimiters.iter_mut() {
                if delimiter.node_index >= closer_node {
                    delimiter.node_index -= removed_nodes;
                }
            }

            let opener_pos = delimiters
                .iter()
                .position(|d| d.node_index == opener_node)
                .expect("opener survives retain");
            let closer_pos = delimiters
                .iter()
                .position(|d| d.node_index == closer_node_now)
                .expect("closer survives retain");
            delimiters[opener_pos].remaining -= use_delims as usize;
            delimiters[closer_pos].remaining -= use_delims as usize;

            // Remove exhausted entries, higher index first so the lower
            // one stays valid. The next iteration resumes at the closer
            // (which may retry with a new opener if it has characters
            // left) or at its successor.
            let mut next_closer = closer_pos;
            if delimiters[closer_pos].remaining == 0 {
                delimiters.remove(closer_pos);
            }
            if delimiters[opener_pos].remaining == 0 {
                delimiters.remove(opener_pos);
                next_closer -= 1;
            }
            closer_idx = next_closer;
        }

        // Emptied delimiter text nodes at this level render as nothing;
        // drop them so consumers see a clean tree.
        children.retain(|node| !matches!(node, Node::Text(text) if text.value.is_empty()));
    }
}

fn find_opener(delimiters: &[Delimiter], closer_idx: usize) -> Option<usize> {
    let closer = &delimiters[closer_idx];
    for opener_idx in (0..closer_idx).rev() {
        let opener = &delimiters[opener_idx];
        if opener.marker != closer.marker || !opener.can_open || opener.remaining == 0 {
            continue;
        }
        // Rule of three: when one side can both open and close, sums
        // divisible by three only pair if both lengths are.
        let sum_of_three = (opener.can_close || closer.can_open)
            && (opener.orig_len + closer.orig_len) % 3 == 0
            && !(opener.orig_len % 3 == 0 && closer.orig_len % 3 == 0);
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
fn classify_flanking(marker: u8, prev: Option<char>, next: Option<char>) -> (bool, bool) {
    let prev_ws = prev.is_none_or(char::is_whitespace);
    let next_ws = next.is_none_or(char::is_whitespace);
    let prev_punct = prev.is_some_and(is_punctuation_like);
    let next_punct = next.is_some_and(is_punctuation_like);

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
