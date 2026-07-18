use ox_content_allocator::Vec;
use ox_content_ast::{List, ListItem, Node, Span};

use super::list_item::ParsedListItem;
use super::Parser;
use crate::error::ParseResult;
#[allow(unused_imports)]
use crate::profile_span;

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

        let mut children: Vec<'a, ListItem<'a>> = self.allocator.new_vec();
        let mut list_spread = false;

        loop {
            if self.is_at_end() {
                break;
            }

            let line_start = self.position;
            self.skip_whitespace();
            if self.peek() == Some('\n') || self.is_at_end() {
                self.position = line_start; // Backtrack to handle end of block
                break;
            }
            self.position = line_start;

            // A sibling marker may sit anywhere between the baseline and
            // the previous content column; less indent ends the list.
            if self.calc_indentation(line_start) < baseline_indent {
                break;
            }
            let line = self.line_at(line_start);
            // A thematic break takes precedence over a sibling marker
            // (`* * *` between items splits the list around an <hr>).
            if Self::try_parse_thematic_break_line(line) {
                break;
            }
            let Some(item) = self.parse_list_item_line_from_line(line_start, line) else {
                break;
            };
            if item.ordered != ordered || item.marker != marker {
                // A different marker starts a new list at the block level.
                break;
            }

            // Consume the marker line.
            self.position += line.len();
            let consumed_newline = self.peek() == Some('\n');
            if consumed_newline {
                self.advance();
            }

            let (item_spread, item_end, item_source) =
                self.consume_item_continuation(&item, baseline_indent, consumed_newline);
            list_spread |= item_spread;

            let item_children = if item_source.is_none()
                && Self::can_inline_parse_list_item(item.content)
            {
                self.parse_inline_list_item_children(item.content, item.content_offset, item_end)?
            } else {
                let item_source = item_source
                    .unwrap_or_else(|| self.init_list_item_source(item.content, consumed_newline))
                    .into_bump_str();
                let sub_parser = self.sub_parser(item_source);
                let sub_doc = sub_parser.parse()?;
                let mut item_children = sub_doc.children;
                for child in &mut item_children {
                    Self::offset_node_spans(child, item.content_offset as u32);
                }
                item_children
            };

            let list_item = ListItem {
                checked: item.checked,
                spread: item_spread,
                children: item_children,
                span: Span::new(line_start as u32, item_end as u32),
            };
            children.push(list_item);
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
    /// dedented source when block re-parsing is needed.
    fn consume_item_continuation(
        &mut self,
        item: &ParsedListItem<'a>,
        baseline_indent: usize,
        consumed_newline: bool,
    ) -> (bool, usize, Option<ox_content_allocator::String<'a>>) {
        let content_indent = item.content_indent;
        let item_is_empty = item.content.trim().is_empty();
        let mut item_source = None;
        let mut item_end = self.position;
        let mut item_spread = false;

        loop {
            if self.is_at_end() {
                break;
            }

            let continuation_start = self.position;
            let continuation_line = self.line_at(continuation_start);
            let continuation_next = self.next_line_start(continuation_start);

            if continuation_line.trim().is_empty() {
                // An item with no content yet cannot continue past a
                // blank line at all.
                if item_is_empty && item_source.is_none() {
                    break;
                }

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
                if next_indent >= content_indent {
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
                    item_spread = true;
                    item_end = self.position;
                    continue;
                }

                if next_indent >= baseline_indent
                    && self.parse_list_item_line(lookahead).as_ref().is_some_and(|next| {
                        next.ordered == item.ordered && next.marker == item.marker
                    })
                {
                    // Blank line between siblings: the list is loose.
                    self.position = lookahead;
                    item_spread = true;
                    break;
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
                continue;
            }

            if current_indent < baseline_indent {
                break;
            }

            // Any list marker here ends this item (sibling or new list).
            if self.parse_list_item_line_from_line(continuation_start, continuation_line).is_some()
            {
                break;
            }

            // A block start interrupts the item; anything else lazily
            // continues the item's trailing paragraph.
            if item_is_empty || self.line_starts_block() {
                break;
            }
            if item_source.is_none() {
                item_source = Some(self.init_list_item_source(item.content, consumed_newline));
            }
            let source = item_source.as_mut().expect("item source initialized");
            Self::push_line_without_indent(source, continuation_line, content_indent);
            source.push('\n');
            self.position = continuation_next;
            item_end = self.position;
        }

        (item_spread, item_end, item_source)
    }
}
