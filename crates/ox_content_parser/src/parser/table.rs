use memchr::memchr;
use ox_content_allocator::Vec;
use ox_content_ast::{AlignKind, Node, Span, Table, TableCell, TableRow};

use super::Parser;
use super::line_scan::{line_end, next_line_start};
use super::table_cell_source::{
    is_escaped_table_pipe, remap_table_cell_inline_spans, unescape_table_pipes,
};
use crate::error::ParseResult;
#[allow(unused_imports)]
use crate::{profile_span, profile_span_detail};

impl<'a> Parser<'a> {
    /// Returns true when the next two lines look like a GFM table header.
    ///
    /// Table detection sits behind a cheap `|` guard in block dispatch, but it
    /// still runs on many prose lines containing pipes. Peek the first two
    /// lines directly with `memchr` and inspect slices in place instead of
    /// collecting `lines().take(2)` into a temporary `Vec`.
    pub(super) fn try_parse_table(&self) -> bool {
        profile_span_detail!("parser::table_probe");
        let bytes = self.source.as_bytes();
        let p0 = self.position;
        let nl0 = line_end(bytes, p0);
        if nl0 == bytes.len() {
            return false;
        }
        let p1 = next_line_start(bytes, p0);
        if p1 >= bytes.len() {
            return false;
        }
        let nl1 = line_end(bytes, p1);

        let first_line = self.source[p0..nl0].trim();
        if memchr(b'|', first_line.as_bytes()).is_none() {
            return false;
        }

        // Second line must be the delimiter row (contains | and -)
        let second_line = self.source[p1..nl1].trim();
        if memchr(b'|', second_line.as_bytes()).is_none()
            || memchr(b'-', second_line.as_bytes()).is_none()
        {
            return false;
        }

        let header_cells = Self::table_row_cells(first_line).count();
        let mut delimiter_cells = 0;
        for cell in Self::table_row_cells(second_line) {
            if delimiter_alignment(cell).is_none() {
                return false;
            }
            delimiter_cells += 1;
        }

        header_cells > 0 && header_cells == delimiter_cells
    }

    pub(super) fn parse_table(&mut self, start: usize) -> ParseResult<Option<Node<'a>>> {
        profile_span!("parser::parse_table");
        let mut align: Vec<'a, AlignKind> = self.allocator.new_vec();

        // Parse header row
        let header_start = self.position;
        let header_line = self.consume_line();

        // Parse delimiter row to get alignment
        let delimiter_line = self.consume_line();
        for cell in Self::table_row_cells(delimiter_line) {
            if let Some(alignment) = delimiter_alignment(cell) {
                align.push(alignment);
            }
        }
        let column_count = align.len();

        // Build the table AST directly instead of first collecting row slices
        // into short-lived heap Vecs. Each consumed source line is parsed into
        // arena-backed cells immediately, which keeps the table path linear in
        // the input and avoids throwaway row containers.
        let mut children: Vec<'a, TableRow<'a>> = self.allocator.new_vec();
        children.push(self.parse_table_row(header_line, header_start, column_count)?);

        // Parse body rows
        loop {
            if self.is_at_end() {
                break;
            }

            // A blank line or another block-level construct terminates the
            // table. Ordinary lines remain data rows even without a pipe.
            let Some(trimmed_start) = self.first_non_whitespace_in_line(self.position) else {
                break;
            };
            if self.probe_line_without_table(self.position, trimmed_start).starts_block {
                break;
            }

            let row_start = self.position;
            let row_line = self.consume_line();
            children.push(self.parse_table_row(row_line, row_start, column_count)?);
        }

        let span = Span::new(start as u32, self.position as u32);
        Ok(Some(Node::Table(self.allocator.boxed(Table { align, children, span }))))
    }

    /// Parses a table row into arena-backed AST cells without temporary heap
    /// collection.
    ///
    /// The row iterator yields borrowed cell slices from the original line.
    /// Inline parsing then writes cell children into the parser arena, so no
    /// intermediate `Vec<&str>` or owned cell text is needed.
    pub(super) fn parse_table_row(
        &self,
        line: &'a str,
        line_start: usize,
        column_count: usize,
    ) -> ParseResult<TableRow<'a>> {
        profile_span_detail!("parser::table_row");
        // Every row is truncated or padded to the delimiter's exact width.
        // Reserve it before parsing inline children: those arena allocations
        // otherwise prevent the cell vector from growing in place, leaving
        // each discarded backing buffer live until the arena is dropped.
        let mut cells: Vec<'a, TableCell<'a>> = self.allocator.new_vec_with_capacity(column_count);
        let line_end = line_start + line.len();
        for (cell_content, cell_start, cell_end) in
            Self::table_row_cells_with_offsets(line).take(column_count)
        {
            let cell_content = unescape_table_pipes(self.allocator, cell_content);
            let cell_children = if let Some(source_offsets) = &cell_content.source_offsets {
                let mut children = self.parse_inline_block(cell_content.content, 0)?;
                let source_offset = (line_start + cell_start) as u32;
                for child in &mut children {
                    remap_table_cell_inline_spans(child, source_offset, source_offsets);
                }
                children
            } else {
                self.parse_inline_block(cell_content.content, line_start + cell_start)?
            };
            cells.push(TableCell {
                children: cell_children,
                span: Span::new((line_start + cell_start) as u32, (line_start + cell_end) as u32),
            });
        }
        while cells.len() < column_count {
            cells.push(TableCell {
                children: self.allocator.new_vec(),
                span: Span::new(line_end as u32, line_end as u32),
            });
        }
        Ok(TableRow { children: cells, span: Span::new(line_start as u32, line_end as u32) })
    }

    /// Iterates table row cells from a line.
    ///
    /// Leading/trailing pipes are syntax delimiters, not empty cells in this
    /// parser's table model, so they are stripped once before splitting.
    pub(super) fn table_row_cells(line: &'a str) -> impl Iterator<Item = &'a str> {
        Self::table_row_cells_with_offsets(line).map(|(cell, _, _)| cell)
    }

    fn table_row_cells_with_offsets(
        line: &'a str,
    ) -> impl Iterator<Item = (&'a str, usize, usize)> {
        let trimmed = line.trim();
        let trimmed_start = line.len() - line.trim_start().len();
        let mut content_start = trimmed_start;
        let mut content_end = trimmed_start + trimmed.len();
        if line[content_start..content_end].starts_with('|') {
            content_start += 1;
        }
        if content_start < content_end
            && line[content_start..content_end].ends_with('|')
            && !is_escaped_table_pipe(
                &line.as_bytes()[content_start..content_end],
                content_end - content_start - 1,
            )
        {
            content_end -= 1;
        }
        let content = &line[content_start..content_end];
        let bytes = content.as_bytes();
        let mut cell_start = 0;

        std::iter::from_fn(move || {
            if cell_start > bytes.len() {
                return None;
            }

            let mut search_start = cell_start;
            while let Some(relative) = memchr(b'|', &bytes[search_start..]) {
                let pipe = search_start + relative;
                if !is_escaped_table_pipe(bytes, pipe) {
                    let raw = &content[cell_start..pipe];
                    let (cell, start, end) = trim_cell(raw, content_start + cell_start);
                    cell_start = pipe + 1;
                    return Some((cell, start, end));
                }
                search_start = pipe + 1;
            }

            let raw = &content[cell_start..];
            let (cell, start, end) = trim_cell(raw, content_start + cell_start);
            cell_start = bytes.len() + 1;
            Some((cell, start, end))
        })
    }
}

fn trim_cell(cell: &str, offset: usize) -> (&str, usize, usize) {
    let trimmed = cell.trim();
    let start = offset + cell.len() - cell.trim_start().len();
    let end = start + trimmed.len();
    (trimmed, start, end)
}

fn delimiter_alignment(cell: &str) -> Option<AlignKind> {
    let trimmed = cell.trim();
    let left = trimmed.starts_with(':');
    let right = trimmed.ends_with(':');
    let hyphens = trimmed.strip_prefix(':').unwrap_or(trimmed);
    let hyphens = hyphens.strip_suffix(':').unwrap_or(hyphens);

    if hyphens.is_empty() || !hyphens.bytes().all(|byte| byte == b'-') {
        return None;
    }

    Some(match (left, right) {
        (true, true) => AlignKind::Center,
        (true, false) => AlignKind::Left,
        (false, true) => AlignKind::Right,
        (false, false) => AlignKind::None,
    })
}
