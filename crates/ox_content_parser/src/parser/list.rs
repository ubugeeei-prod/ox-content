use ox_content_allocator::Vec;
use ox_content_ast::{List, ListItem, Node, Paragraph, Span};

use super::Parser;
use crate::error::ParseResult;
#[allow(unused_imports)]
use crate::profile_span;

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

            // Check indentation
            let current_indent = self.calc_indentation(line_start);

            // If less indented, list ends
            if current_indent < baseline_indent {
                self.position = line_start;
                break;
            }

            // If indented more, check if it's a nested list
            if current_indent > baseline_indent {
                // Peek to see if it's a list marker
                self.position = line_start; // Reset position to check marker properly
                if self.try_parse_list() {
                    // Parse nested list
                    if let Some(Node::List(nested_list)) = self.parse_list(line_start)? {
                        // Add to the LAST item's children
                        if let Some(last_item) = children.last_mut() {
                            last_item.span = last_item.span.merge(nested_list.span);
                            last_item.children.push(Node::List(nested_list));
                        }
                    }
                } else {
                    // Continuation content?
                    // For now, we only support simple lists.
                    // Just skip line to avoid infinite loop
                    while let Some(ch) = self.peek() {
                        self.advance();
                        if ch == '\n' {
                            break;
                        }
                    }
                }
                continue;
            }

            // Same indentation (or close enough? Standard is complex, we use strict >= baseline)
            self.position = line_start; // Reset

            // Check if it's a list item
            let remaining = self.remaining();
            let line = remaining.lines().next().unwrap_or("");
            let Some(item) = self.parse_list_item_line_from_line(line_start, line) else {
                break;
            };
            if item.ordered != ordered {
                // Not a list item, break
                break;
            }

            // Consume line
            self.position += line.len();
            let consumed_newline = self.peek() == Some('\n');
            if consumed_newline {
                self.advance();
            }

            let content_indent = item.content_offset.saturating_sub(line_start);
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
                    let next_item = self.parse_list_item_line(lookahead);
                    if next_indent == baseline_indent
                        && next_item.as_ref().is_some_and(|next| next.ordered == ordered)
                    {
                        self.position = lookahead;
                        item_spread = true;
                        list_spread = true;
                        break;
                    }

                    if next_indent >= content_indent {
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
                        list_spread = true;
                        item_end = self.position;
                        continue;
                    }

                    break;
                }

                let current_indent = self.calc_indentation(continuation_start);
                if current_indent < baseline_indent {
                    break;
                }

                if current_indent == baseline_indent {
                    // Reuse the line already scanned at the top of the loop
                    // (`continuation_line`) instead of re-finding the newline;
                    // only `next.ordered` is read, which a trailing `\r` cannot
                    // affect.
                    if self
                        .parse_list_item_line_from_line(continuation_start, continuation_line)
                        .is_some_and(|next| next.ordered == ordered)
                    {
                        break;
                    }

                    break;
                }

                if item_source.is_none() {
                    item_source = Some(self.init_list_item_source(item.content, consumed_newline));
                }
                let item_source = item_source.as_mut().expect("item source initialized");
                let stripped = Self::strip_indent_columns(continuation_line, content_indent);
                item_source.push_str(stripped);
                item_source.push('\n');
                self.position = continuation_next;
                item_end = self.position;
            }

            let item_children = if item_source.is_none()
                && Self::can_inline_parse_list_item(item.content)
            {
                self.parse_inline_list_item_children(item.content, item.content_offset, item_end)?
            } else {
                let item_source = item_source
                    .unwrap_or_else(|| self.init_list_item_source(item.content, consumed_newline))
                    .into_bump_str();
                let sub_parser =
                    Parser::with_options(self.allocator, item_source, self.options.clone());
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

    fn init_list_item_source(
        &self,
        content: &'a str,
        consumed_newline: bool,
    ) -> ox_content_allocator::String<'a> {
        // Bump-allocate the per-item buffer so we don't go System → arena.
        // This is now only paid for list items that actually need block
        // parsing: multi-line items, nested blocks, or block-looking
        // single-line contents such as `# heading`.
        let mut source = ox_content_allocator::String::with_capacity_in(
            content.len() + usize::from(consumed_newline),
            self.allocator.bump(),
        );
        source.push_str(content);
        if consumed_newline {
            source.push('\n');
        }
        source
    }

    fn can_inline_parse_list_item(content: &str) -> bool {
        let Some(&first) = content.trim_start().as_bytes().first() else {
            return true;
        };

        // These are the same leading-byte families that `parse_block` may
        // treat as block syntax in a freshly spawned sub-parser. Keep those
        // on the old path so `- # heading`, nested lists, fenced code, raw
        // HTML blocks, etc. preserve their current AST.
        !matches!(first, b'#' | b'-' | b'*' | b'_' | b'>' | b'`' | b'~' | b'<' | b'+' | b'0'..=b'9')
    }

    fn parse_inline_list_item_children(
        &self,
        content: &'a str,
        content_offset: usize,
        item_end: usize,
    ) -> ParseResult<Vec<'a, Node<'a>>> {
        let mut children = self.allocator.new_vec();
        let inline = content.trim();
        if inline.is_empty() {
            return Ok(children);
        }

        let paragraph_children = self.parse_inline(inline, content_offset)?;
        children.push(Node::Paragraph(Paragraph {
            children: paragraph_children,
            span: Span::new(content_offset as u32, item_end as u32),
        }));
        Ok(children)
    }
}
