//! Opt-in PHP Markdown Extra / mdBook-style definition list nodes.

use ox_content_allocator::{String as ArenaString, Vec as ArenaVec};
use ox_content_ast::{DefinitionList, DefinitionListDefinition, DefinitionListTerm, Node, Span};
use rustc_hash::FxHashSet;

use super::Parser;
use super::spans::SourceMap;
use crate::error::ParseResult;
#[allow(unused_imports)]
use crate::profile_span_detail;

mod lines;

use self::lines::{DefinitionBodyLine, has_unclosed_inline_code, trim_line_span};

impl<'a> Parser<'a> {
    pub(super) fn parse_definition_list(&mut self, start: usize) -> ParseResult<Option<Node<'a>>> {
        profile_span_detail!("parser::definition_list");
        debug_assert!(self.options.definition_lists);

        let Some((children, end)) = self.collect_definition_list(start)? else {
            return Ok(None);
        };

        self.position = end;
        Ok(Some(Node::DefinitionList(
            self.allocator
                .boxed(DefinitionList { span: Span::new(start as u32, end as u32), children }),
        )))
    }

    fn collect_definition_list(
        &self,
        start: usize,
    ) -> ParseResult<Option<(ArenaVec<'a, Node<'a>>, usize)>> {
        let mut children = self.allocator.new_vec();
        let mut cursor = start;
        let mut parsed_any = false;

        while let Some((item_nodes, next)) = self.collect_definition_item(cursor)? {
            for node in item_nodes {
                children.push(node);
            }
            parsed_any = true;
            cursor = next;

            cursor = self.skip_blank_lines_from(cursor);
            if cursor < self.source.len() && self.can_start_definition_item_at(cursor) {
                continue;
            }
            break;
        }

        Ok(parsed_any.then_some((children, cursor)))
    }

    fn collect_definition_item(
        &self,
        start: usize,
    ) -> ParseResult<Option<(ArenaVec<'a, Node<'a>>, usize)>> {
        let Some((term_lines, mut cursor)) = self.collect_definition_terms(start) else {
            return Ok(None);
        };

        let mut nodes = self.allocator.new_vec_with_capacity(term_lines.len() + 1);
        for (line_start, line_end) in term_lines {
            let (term_start, term_end) = trim_line_span(self.source, line_start, line_end);
            let value = &self.source[term_start..term_end];
            let term = DefinitionListTerm {
                children: self.parse_inline_block(value, term_start)?,
                span: Span::new(line_start as u32, line_end as u32),
            };
            nodes.push(Node::DefinitionListTerm(self.allocator.boxed(term)));
        }

        let mut parsed_definition = false;
        while let Some((definition, next)) = self.collect_definition_body(cursor)? {
            nodes.push(definition);
            parsed_definition = true;
            cursor = next;
            if !self.starts_definition_body_at(cursor) {
                break;
            }
        }

        if !parsed_definition {
            return Ok(None);
        }

        Ok(Some((nodes, cursor)))
    }

    fn collect_definition_terms(
        &self,
        start: usize,
    ) -> Option<(ArenaVec<'a, (usize, usize)>, usize)> {
        if start >= self.source.len() || self.is_blank_line_at(start) {
            return None;
        }

        let mut terms = self.allocator.new_vec();
        let mut cursor = start;

        while cursor < self.source.len() {
            if self.is_blank_line_at(cursor) || self.definition_body_at(cursor).is_some() {
                break;
            }
            if !self.is_definition_term_line(cursor) {
                return None;
            }
            let line_end = self.line_end_at(cursor);
            terms.push((cursor, line_end));
            cursor = self.next_line_start(cursor);
        }

        if terms.is_empty() {
            return None;
        }

        cursor = self.skip_blank_lines_from(cursor);
        self.starts_definition_body_at(cursor).then_some((terms, cursor))
    }

    fn collect_definition_body(&self, start: usize) -> ParseResult<Option<(Node<'a>, usize)>> {
        let Some(first_line) = self.definition_body_at(start) else {
            return Ok(None);
        };

        let mut body_source = ArenaString::with_capacity_in(
            first_line.body_end.saturating_sub(first_line.body_start) + 1,
            self.allocator.bump(),
        );
        let mut source_map = SourceMap::default();
        let mut lazy_lines = FxHashSet::default();

        self.push_definition_body_line(
            &mut body_source,
            &mut source_map,
            first_line.body_start,
            first_line.body_end,
            first_line.body_start,
            self.next_line_start(start),
        );

        let mut cursor = self.next_line_start(start);
        let mut end = cursor;
        let mut after_blank = false;

        while cursor < self.source.len() {
            let line = self.line_at(cursor);
            let next = self.next_line_start(cursor);

            if line.trim().is_empty() {
                let lookahead = self.skip_blank_lines_from(cursor);
                if lookahead >= self.source.len()
                    || self.starts_definition_body_at(lookahead)
                    || self.can_start_definition_item_at(lookahead)
                {
                    break;
                }
                let Some(trimmed_start) = self.first_non_whitespace_in_line(lookahead) else {
                    break;
                };
                if self.line_indent_width(lookahead, trimmed_start) < 4 {
                    break;
                }

                while cursor < lookahead {
                    let blank_next = self.next_line_start(cursor);
                    let generated_start = body_source.len();
                    body_source.push('\n');
                    source_map.push_line(
                        generated_start,
                        1,
                        cursor,
                        blank_next.saturating_sub(cursor),
                    );
                    cursor = blank_next;
                    end = cursor;
                }
                after_blank = true;
                continue;
            }

            if self.starts_definition_body_at(cursor) || self.can_start_definition_item_at(cursor) {
                break;
            }

            let Some(trimmed_start) = self.first_non_whitespace_in_line(cursor) else {
                break;
            };
            let indent = self.line_indent_width(cursor, trimmed_start);
            if indent >= 4 {
                let generated_start = body_source.len();
                let source_offset_in_line =
                    Self::push_line_without_indent(&mut body_source, line, 4);
                body_source.push('\n');
                let source_start = cursor + source_offset_in_line;
                source_map.push_line_with_block_start(
                    generated_start,
                    body_source.len() - generated_start,
                    cursor,
                    source_start,
                    next.saturating_sub(source_start),
                );
                cursor = next;
                end = cursor;
                after_blank = false;
                continue;
            }

            if after_blank || self.probe_line_without_table(cursor, trimmed_start).starts_block {
                break;
            }

            let generated_start = body_source.len();
            lazy_lines.insert(body_source.len() as u32);
            body_source.push_str(line);
            body_source.push('\n');
            source_map.push_line(
                generated_start,
                body_source.len() - generated_start,
                cursor,
                next.saturating_sub(cursor),
            );
            cursor = next;
            end = cursor;
        }

        let body_source = body_source.into_bump_str();
        let sub_doc = self.sub_parser_with_lazy_lines(body_source, lazy_lines).parse()?;
        let mut children = sub_doc.children;
        for child in &mut children {
            source_map.remap_node_spans(child);
        }

        Ok(Some((
            Node::DefinitionListDefinition(self.allocator.boxed(DefinitionListDefinition {
                children,
                span: Span::new(start as u32, end as u32),
            })),
            end,
        )))
    }

    fn push_definition_body_line(
        &self,
        body_source: &mut ArenaString<'a>,
        source_map: &mut SourceMap,
        source_start: usize,
        source_end: usize,
        map_start: usize,
        map_end: usize,
    ) {
        let generated_start = body_source.len();
        body_source.push_str(&self.source[source_start..source_end]);
        body_source.push('\n');
        source_map.push_line(
            generated_start,
            body_source.len() - generated_start,
            map_start,
            map_end.saturating_sub(map_start),
        );
    }

    pub(super) fn starts_definition_body_at(&self, line_start: usize) -> bool {
        self.options.definition_lists && self.definition_body_at(line_start).is_some()
    }

    fn can_start_definition_item_at(&self, start: usize) -> bool {
        self.options.definition_lists && self.collect_definition_terms(start).is_some()
    }

    fn is_definition_term_line(&self, line_start: usize) -> bool {
        let Some(trimmed_start) = self.first_non_whitespace_in_line(line_start) else {
            return false;
        };
        if self.line_indent_width(line_start, trimmed_start) >= 4 {
            return false;
        }
        let line = self.line_at(line_start);
        let trimmed = &line[trimmed_start - line_start..];
        !trimmed.starts_with(':')
            && !trimmed.starts_with('>')
            && !trimmed.starts_with(":::")
            && !trimmed.starts_with('<')
            && !Self::try_parse_thematic_break_line(line)
            && !self.try_parse_heading_start(line_start, trimmed_start)
            && !Self::try_parse_fenced_code_at(line, trimmed)
            && self.parse_list_item_line_from_trimmed(line_start, line, trimmed).is_none()
            && !has_unclosed_inline_code(line)
    }

    fn definition_body_at(&self, line_start: usize) -> Option<DefinitionBodyLine> {
        if line_start >= self.source.len() {
            return None;
        }
        let line_end = self.line_end_at(line_start);
        let bytes = self.source.as_bytes();
        let mut cursor = line_start;
        let mut spaces = 0usize;
        while cursor < line_end && spaces < 3 && bytes[cursor] == b' ' {
            cursor += 1;
            spaces += 1;
        }
        if bytes.get(cursor) != Some(&b':') {
            return None;
        }
        let after_colon = cursor + 1;
        if after_colon < line_end && !matches!(bytes[after_colon], b' ' | b'\t') {
            return None;
        }
        let mut body_start = after_colon;
        while body_start < line_end && matches!(bytes[body_start], b' ' | b'\t') {
            body_start += 1;
        }
        let (_, body_end) = trim_line_span(self.source, body_start, line_end);
        Some(DefinitionBodyLine { body_start, body_end })
    }
}
