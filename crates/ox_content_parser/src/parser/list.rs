use ox_content_allocator::Vec;
use ox_content_ast::{List, ListItem, Node, Span};

use super::list_item::ParsedListItem;
use super::Parser;
use crate::error::ParseResult;
#[allow(unused_imports)]
use crate::{profile_span, profile_span_detail};

mod item_source;

impl<'a> Parser<'a> {
    pub(super) fn parse_list(&mut self, start: usize) -> ParseResult<Option<Node<'a>>> {
        profile_span!("parser::parse_list");
        let baseline_indent = self.calc_indentation(start);

        // Determine list type from the first line (already verified by try_parse_list)
        let first_line_start = self.position;
        let Some(first_item) = self.parse_list_item_line(first_line_start) else {
            return Ok(None);
        };
        let ordered = first_item.ordered;
        let marker = first_item.marker;
        let list_start = first_item.start;
        let mut item = first_item;

        let mut children: Vec<'a, ListItem<'a>> = self.allocator.new_vec();
        let mut list_spread = false;

        loop {
            let line_start = self.position;
            let line = self.line_at(line_start);

            // Consume the marker line.
            self.position += line.len();
            let consumed_newline = self.peek() == Some('\n');
            if consumed_newline {
                self.advance();
            }

            let mut lazy_lines = rustc_hash::FxHashSet::default();
            let (gap_spread, item_end, item_source, next_item) = if self.is_at_end() {
                (false, self.position, None, None)
            } else {
                self.consume_item_continuation(
                    &item,
                    baseline_indent,
                    consumed_newline,
                    &mut lazy_lines,
                )
            };

            let mut content_spread = false;
            let item_children = if item_source.is_none()
                && Self::can_inline_parse_list_item(item.content)
            {
                self.parse_inline_list_item_children(item.content, item.content_offset, item_end)?
            } else {
                let item_source = item_source
                    .unwrap_or_else(|| self.init_list_item_source(item.content, consumed_newline))
                    .into_bump_str();
                let sub_parser = self.sub_parser_with_lazy_lines(item_source, lazy_lines);
                let sub_doc = sub_parser.parse()?;
                // The item directly contains blank-separated blocks iff a
                // gap between consecutive top-level children spans a line
                // break (spans are still in item-source coordinates).
                content_spread = item_content_has_blank_gap(item_source, &sub_doc.children);
                let mut item_children = sub_doc.children;
                for child in &mut item_children {
                    Self::offset_node_spans(child, item.content_offset as u32);
                }
                item_children
            };
            list_spread |= gap_spread || content_spread;

            let list_item = ListItem {
                checked: item.checked,
                spread: content_spread,
                children: item_children,
                span: Span::new(line_start as u32, item_end as u32),
            };
            // The first push keeps one-item lists at bumpalo's exact minimum.
            // Once a second sibling is known to exist, skip the otherwise
            // inevitable two-slot allocation and its copy; four slots cover
            // the common short list without penalizing the single-item case.
            if children.len() == 1 {
                children.reserve(3);
            }
            children.push(list_item);

            let Some(next_item) = next_item else {
                break;
            };
            if next_item.ordered != ordered || next_item.marker != marker {
                // A different marker starts a new list at the block level.
                break;
            }
            item = next_item;
        }

        let span = Span::new(start as u32, self.position as u32);
        Ok(Some(Node::List(List {
            ordered,
            start: list_start,
            spread: list_spread,
            children,
            span,
        })))
    }

    /// Consumes one item's continuation lines: indented content
    /// (paragraphs, nested blocks — the item sub-parser sorts them out),
    /// interior blank lines, and lazy paragraph continuation. Returns
    /// whether the item/list turned loose, the item end position, and the
    /// dedented source when block re-parsing is needed. A parsed sibling is
    /// carried back to the caller so its marker is not scanned twice.
    fn consume_item_continuation(
        &mut self,
        item: &ParsedListItem<'a>,
        baseline_indent: usize,
        consumed_newline: bool,
        lazy_lines: &mut rustc_hash::FxHashSet<u32>,
    ) -> (bool, usize, Option<ox_content_allocator::String<'a>>, Option<ParsedListItem<'a>>) {
        profile_span_detail!("parser::list_item_continuation");
        let content_indent = item.content_indent;
        let item_is_empty = item.content.trim().is_empty();
        let mut item_source = None;
        let mut item_end = self.position;
        let mut gap_spread = false;
        let mut next_item = None;
        // Lazy paragraph continuation is only valid while the item's last
        // consumed line kept a paragraph open (not right after blanks).
        let mut after_blank = false;

        loop {
            if self.is_at_end() {
                break;
            }

            let continuation_start = self.position;
            let continuation_line = self.line_at(continuation_start);
            let continuation_next = self.next_line_start(continuation_start);

            if continuation_line.trim().is_empty() {
                let mut lookahead = continuation_next;
                let mut blank_count = 1;
                while lookahead < self.source.len() {
                    let line = self.line_at(lookahead);
                    if !line.trim().is_empty() {
                        break;
                    }
                    blank_count += 1;
                    lookahead = self.next_line_start(lookahead);
                }

                if lookahead >= self.source.len() {
                    break;
                }

                let next_indent = self.calc_indentation(lookahead);
                // An item with no content yet cannot continue past a
                // blank line, but its list may (`* a\n*\n\n* c`).
                if next_indent >= content_indent && !(item_is_empty && item_source.is_none()) {
                    // Interior blank line(s): the item continues below.
                    if item_source.is_none() {
                        item_source =
                            Some(self.init_list_item_source(item.content, consumed_newline));
                    }
                    let item_source = item_source.as_mut().expect("item source initialized");
                    for _ in 0..blank_count {
                        item_source.push('\n');
                    }
                    self.position = lookahead;
                    item_end = self.position;
                    after_blank = true;
                    continue;
                }

                if next_indent >= baseline_indent && next_indent <= baseline_indent + 3 {
                    if let Some(sibling) = self
                        .parse_list_item_line(lookahead)
                        .filter(|next| next.ordered == item.ordered && next.marker == item.marker)
                    {
                        // Blank line between siblings: the list is loose.
                        self.position = lookahead;
                        gap_spread = true;
                        next_item = Some(sibling);
                        break;
                    }
                }

                break;
            }

            let current_indent = self.calc_indentation(continuation_start);
            if current_indent >= content_indent {
                // Indented continuation content.
                if item_source.is_none() {
                    item_source = Some(self.init_list_item_source(item.content, consumed_newline));
                }
                let item_source = item_source.as_mut().expect("item source initialized");
                Self::push_line_without_indent(item_source, continuation_line, content_indent);
                item_source.push('\n');
                self.position = continuation_next;
                item_end = self.position;
                after_blank = false;
                continue;
            }

            // A list marker (indented at most three columns past the
            // baseline — deeper "markers" are just text) ends this item.
            if current_indent >= baseline_indent
                && current_indent <= baseline_indent + 3
                && !Self::try_parse_thematic_break_line(continuation_line)
            {
                if let Some(sibling) =
                    self.parse_list_item_line_from_line(continuation_start, continuation_line)
                {
                    next_item = Some(sibling);
                    break;
                }
            }

            // A block start interrupts the item; anything else lazily
            // continues the item's trailing paragraph regardless of its
            // indentation (CommonMark laziness).
            if item_is_empty || after_blank || self.line_starts_block() {
                break;
            }
            if item_source.is_none() {
                item_source = Some(self.init_list_item_source(item.content, consumed_newline));
            }
            let source = item_source.as_mut().expect("item source initialized");
            // Keep the lazy line's own indentation: the sub-parse then
            // treats it as paragraph continuation even when it looks like
            // an (over-indented) marker, e.g. `- e` five columns deep.
            // Recording the offset stops setext reinterpretation.
            lazy_lines.insert(source.len() as u32);
            source.push_str(continuation_line);
            source.push('\n');
            self.position = continuation_next;
            item_end = self.position;
        }

        (gap_spread, item_end, item_source, next_item)
    }
}

/// Whether a gap between consecutive top-level children of an item's
/// sub-parsed source spans a line break — the spec's "directly contain
/// two block-level elements with a blank line between them".
fn item_content_has_blank_gap(source: &str, children: &[Node<'_>]) -> bool {
    children.windows(2).any(|pair| {
        let gap_start = block_span(&pair[0]).end as usize;
        let gap_end = block_span(&pair[1]).start as usize;
        source.get(gap_start..gap_end).is_some_and(|gap| gap.contains('\n'))
    })
}

fn block_span(node: &Node<'_>) -> Span {
    match node {
        Node::Paragraph(n) => n.span,
        Node::Heading(n) => n.span,
        Node::ThematicBreak(n) => n.span,
        Node::BlockQuote(n) => n.span,
        Node::List(n) => n.span,
        Node::CodeBlock(n) => n.span,
        Node::Html(n) => n.span,
        Node::Table(n) => n.span,
        Node::Definition(n) => n.span,
        Node::FootnoteDefinition(n) => n.span,
        _ => Span::new(0, 0),
    }
}
