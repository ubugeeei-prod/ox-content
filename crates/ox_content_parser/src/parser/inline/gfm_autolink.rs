//! GFM autolink extension: bare `www.`, `http(s)://`, and email
//! addresses in plain text become links (GFM spec "Autolinks
//! (extension)").
//!
//! Runs as a post-pass over a parsed inline sequence so it cannot
//! interfere with emphasis pairing, and recurses into emphasis-like
//! containers while never descending into existing links.

use ox_content_allocator::Vec;
use ox_content_ast::{Link, Node, Span, Text};

mod scan;

use self::scan::find_candidate;
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
        let mut i = 0;
        while i < children.len() {
            match &mut children[i] {
                Node::Emphasis(node) => self.apply_gfm_autolinks(&mut node.children, scan),
                Node::Strong(node) => self.apply_gfm_autolinks(&mut node.children, scan),
                Node::Delete(node) => self.apply_gfm_autolinks(&mut node.children, scan),
                Node::Text(text) => {
                    let value = text.value;
                    if let Some(candidate) = find_candidate(value, scan) {
                        let span_start = text.span.start;
                        let link_value = &value[candidate.start..candidate.end];
                        let url: &'a str = if candidate.href_prefix.is_empty() {
                            link_value
                        } else {
                            let mut url = self.allocator.new_string_from(candidate.href_prefix);
                            url.push_str(link_value);
                            url.into_bump_str()
                        };

                        let link_span = Span::new(
                            span_start + candidate.start as u32,
                            span_start + candidate.end as u32,
                        );
                        let mut link_children = self.allocator.new_vec();
                        link_children.push(Node::Text(Text { value: link_value, span: link_span }));
                        let link_node = Node::Link(Link {
                            url,
                            title: None,
                            children: link_children,
                            span: link_span,
                        });

                        let after = &value[candidate.end..];
                        let after_node = (!after.is_empty()).then(|| {
                            Node::Text(Text {
                                value: after,
                                span: Span::new(span_start + candidate.end as u32, text.span.end),
                            })
                        });

                        if candidate.start == 0 {
                            children[i] = link_node;
                        } else {
                            text.value = &value[..candidate.start];
                            text.span = Span::new(span_start, span_start + candidate.start as u32);
                            i += 1;
                            children.insert(i, link_node);
                        }
                        if let Some(after_node) = after_node {
                            children.insert(i + 1, after_node);
                        }
                        // Continue scanning in the remainder text node.
                    }
                }
                _ => {}
            }
            i += 1;
        }
    }
}

impl<'a> Parser<'a> {
    fn coalesce_adjacent_text(&self, children: &mut Vec<'a, Node<'a>>) {
        profile_span_detail!("parser::coalesce_text");
        let mut i = 0;
        while i + 1 < children.len() {
            if !matches!(children[i], Node::Text(_)) || !matches!(children[i + 1], Node::Text(_)) {
                i += 1;
                continue;
            }
            let mut j = i + 1;
            while j < children.len() && matches!(children[j], Node::Text(_)) {
                j += 1;
            }
            let mut merged = self.allocator.new_string();
            let mut span = Span::new(0, 0);
            for (index, node) in children[i..j].iter().enumerate() {
                if let Node::Text(text) = node {
                    if index == 0 {
                        span = text.span;
                    }
                    span = Span::new(span.start, text.span.end);
                    merged.push_str(text.value);
                }
            }
            let merged_node = Node::Text(Text { value: merged.into_bump_str(), span });
            children.drain(i..j);
            children.insert(i, merged_node);
            i += 1;
        }
    }
}
