//! GFM autolink extension: bare `www.`, `http(s)://`, and email
//! addresses in plain text become links (GFM spec "Autolinks
//! (extension)").
//!
//! Runs as a post-pass over a parsed inline sequence so it cannot
//! interfere with emphasis pairing, and recurses into emphasis-like
//! containers while never descending into existing links.

use ox_content_allocator::Vec;
use ox_content_ast::{Link, Node, Span, Text};

mod candidate;
mod domain;
mod scan;
mod trim;

use self::candidate::CandidateScan;
pub(super) use self::scan::may_contain_autolink;

use crate::parser::Parser;
#[allow(unused_imports)]
use crate::{profile_span, profile_span_detail};

pub(super) struct Candidate {
    pub(super) start: usize,
    pub(super) end: usize,
    pub(super) href_prefix: &'static str,
}

/// What the block-level pre-flight proved about a block's raw content, so
/// the per-node scan does not re-derive it.
///
/// `www.` is the only needle with no `:` and no `@` in it, which makes it
/// the only reason a node has to pay for a substring search when the cheap
/// `memchr2` comes back empty. Whether a block can hold one is a property
/// of its raw source, so [`may_contain_autolink`] answers it once for the
/// whole block instead of once per text node.
#[derive(Clone, Copy)]
pub(in crate::parser::inline) struct AutolinkScan {
    pub(super) may_have_www: bool,
}

impl<'a> Parser<'a> {
    pub(in crate::parser::inline) fn apply_gfm_autolinks(
        &self,
        children: &mut Vec<'a, Node<'a>>,
        scan: AutolinkScan,
    ) {
        profile_span!("parser::apply_gfm_autolinks");
        // Entity, escape, and unpaired-delimiter handling fragment plain
        // prose into adjacent text nodes; autolinks must see the joined
        // run (`...?q=x&hl=en` is one URL despite the `&` split).
        self.coalesce_adjacent_text(children);
        let mut index = 0;
        while index < children.len() {
            if let Node::Text(text) = &children[index] {
                let mut candidates = CandidateScan::new(text.value, scan);
                if let Some(first) = candidates.next() {
                    // Nothing in front of the first link changes, so the
                    // common block without one never leaves this loop.
                    self.relink_from(children, index, (candidates, first), scan);
                    return;
                }
            } else {
                self.apply_gfm_autolinks_inside(&mut children[index], scan);
            }
            index += 1;
        }
    }

    fn apply_gfm_autolinks_inside(&self, node: &mut Node<'a>, scan: AutolinkScan) {
        match node {
            Node::Emphasis(node) => self.apply_gfm_autolinks(&mut node.children, scan),
            Node::Strong(node) => self.apply_gfm_autolinks(&mut node.children, scan),
            Node::Delete(node) => self.apply_gfm_autolinks(&mut node.children, scan),
            Node::Superscript(node) => self.apply_gfm_autolinks(&mut node.children, scan),
            Node::Subscript(node) => self.apply_gfm_autolinks(&mut node.children, scan),
            _ => {}
        }
    }

    /// Rewrites `children[from..]`, whose first node is the text node that
    /// `started` is already scanning.
    ///
    /// A link splits its text node in up to three, and each piece used to be
    /// `insert`ed where it belongs, shifting every node after it. A
    /// paragraph with many links *and* many nodes behind them — a link, then
    /// an emphasis or a code span, repeated — therefore cost O(n²): 125 KB
    /// of `www.a.b *x* ` took 0.19 s and grew x19 for every x4 of input. The
    /// tail is moved out once instead and written back behind the nodes
    /// already in place, so each node is touched once.
    fn relink_from(
        &self,
        children: &mut Vec<'a, Node<'a>>,
        from: usize,
        started: (CandidateScan<'a>, Candidate),
        scan: AutolinkScan,
    ) {
        let mut started = Some(started);
        for mut node in children.split_off(from) {
            let Node::Text(text) = node else {
                self.apply_gfm_autolinks_inside(&mut node, scan);
                children.push(node);
                continue;
            };
            let (mut candidates, mut next) = started.take().map_or_else(
                || {
                    let mut candidates = CandidateScan::new(text.value, scan);
                    let first = candidates.next();
                    (candidates, first)
                },
                |(candidates, first)| (candidates, Some(first)),
            );

            // `done` bytes of the node are already written out. Each piece
            // keeps the span it had when the pieces were split off one link
            // at a time: offsets into the value from the node's start, and
            // the node's own end for whatever follows the last link.
            let mut done = 0;
            while let Some(candidate) = next {
                if candidate.start > done {
                    children.push(Node::Text(Text {
                        value: &text.value[done..candidate.start],
                        span: Span::new(
                            text.span.start + done as u32,
                            text.span.start + candidate.start as u32,
                        ),
                    }));
                }
                children.push(self.autolink_node(&text, &candidate));
                done = candidate.end;
                next = candidates.next();
            }
            // A node with no link in it comes back whole, empty or not.
            if done < text.value.len() || done == 0 {
                children.push(Node::Text(Text {
                    value: &text.value[done..],
                    span: Span::new(text.span.start + done as u32, text.span.end),
                }));
            }
        }
    }

    fn autolink_node(&self, text: &Text<'a>, candidate: &Candidate) -> Node<'a> {
        let link_value = &text.value[candidate.start..candidate.end];
        let url: &'a str = if candidate.href_prefix.is_empty() {
            link_value
        } else {
            let mut url = self.allocator.new_string_from(candidate.href_prefix);
            url.push_str(link_value);
            url.into_bump_str()
        };

        let link_span = Span::new(
            text.span.start + candidate.start as u32,
            text.span.start + candidate.end as u32,
        );
        let mut link_children = self.allocator.new_vec();
        link_children.push(Node::Text(Text { value: link_value, span: link_span }));
        Node::Link(self.allocator.boxed(Link {
            url,
            title: None,
            children: link_children,
            span: link_span,
        }))
    }
}

impl<'a> Parser<'a> {
    /// Merges runs of adjacent `Text` nodes into one.
    ///
    /// Each run used to be `drain`ed and the merged node `insert`ed, and both
    /// shift every node after the run. A paragraph that alternates text with
    /// something else is all runs — `@[a](b) ` repeated is exactly that — so
    /// the shifting cost O(n²): 256 KiB took 1.6 s and grew x19 for every x4
    /// of input. A write cursor touches each node once.
    fn coalesce_adjacent_text(&self, children: &mut Vec<'a, Node<'a>>) {
        profile_span_detail!("parser::coalesce_text");
        let mut write = 0usize;
        let mut read = 0usize;

        while read < children.len() {
            let run_start = read;
            while read < children.len() && matches!(children[read], Node::Text(_)) {
                read += 1;
            }

            match read - run_start {
                // Not text at all: keep it, at the cursor.
                0 => {
                    children.swap(write, read);
                    read += 1;
                }
                // A lone text node has nothing to merge with.
                1 => children.swap(write, run_start),
                _ => {
                    let mut merged = self.allocator.new_string();
                    let mut span = Span::new(0, 0);
                    for (index, node) in children[run_start..read].iter().enumerate() {
                        if let Node::Text(text) = node {
                            if index == 0 {
                                span = text.span;
                            }
                            span = Span::new(span.start, text.span.end);
                            merged.push_str(text.value);
                        }
                    }
                    children[write] = Node::Text(Text { value: merged.into_bump_str(), span });
                }
            }
            write += 1;
        }

        children.truncate(write);
    }
}
