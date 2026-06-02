use memchr::memchr;
use ox_content_allocator::Vec;
use ox_content_ast::{AlignKind, Node, Span, Table, TableCell, TableRow};

use super::Parser;
use crate::error::ParseResult;
#[allow(unused_imports)]
use crate::profile_span;

impl<'a> Parser<'a> {
    /// Returns true when the next two lines look like a GFM table header.
    ///
    /// Table detection sits behind a cheap `|` guard in block dispatch, but it
    /// still runs on many prose lines containing pipes. Peek the first two
    /// lines directly with `memchr` and inspect slices in place instead of
    /// collecting `lines().take(2)` into a temporary `Vec`.
    pub(super) fn try_parse_table(&self) -> bool {
        let bytes = self.source.as_bytes();
        let p0 = self.position;
        let nl0 = match memchr(b'\n', &bytes[p0..]) {
            Some(off) => p0 + off,
            None => return false,
        };
        let p1 = nl0 + 1;
        if p1 >= bytes.len() {
            return false;
        }
        let nl1 = memchr(b'\n', &bytes[p1..]).map_or(bytes.len(), |off| p1 + off);

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

        // Check delimiter row pattern: |---|---|
        let is_delimiter = second_line.split('|').filter(|s| !s.is_empty()).all(|cell| {
            let trimmed = cell.trim();
            if trimmed.is_empty() {
                return true;
            }
            // Allow :---:, :---, ---:, ---
            trimmed.chars().all(|c| c == '-' || c == ':')
        });

        is_delimiter
    }

    pub(super) fn parse_table(&mut self, start: usize) -> ParseResult<Option<Node<'a>>> {
        profile_span!("parser::parse_table");
        let mut align: Vec<'a, AlignKind> = self.allocator.new_vec();

        // Parse header row
        let header_line = self.consume_line();

        // Parse delimiter row to get alignment
        let delimiter_line = self.consume_line();
        for cell in delimiter_line.split('|').filter(|s| !s.trim().is_empty()) {
            let cell = cell.trim();
            let starts_colon = cell.starts_with(':');
            let ends_colon = cell.ends_with(':');
            let alignment = match (starts_colon, ends_colon) {
                (true, true) => AlignKind::Center,
                (true, false) => AlignKind::Left,
                (false, true) => AlignKind::Right,
                (false, false) => AlignKind::None,
            };
            align.push(alignment);
        }

        // Build the table AST directly instead of first collecting row slices
        // into short-lived heap Vecs. Each consumed source line is parsed into
        // arena-backed cells immediately, which keeps the table path linear in
        // the input and avoids throwaway row containers.
        let mut children: Vec<'a, TableRow<'a>> = self.allocator.new_vec();
        children.push(self.parse_table_row(header_line)?);

        // Parse body rows
        loop {
            if self.is_at_end() {
                break;
            }

            let line_start = self.position;
            self.skip_whitespace();

            // Check for blank line or non-table line
            if self.peek() == Some('\n') || self.is_at_end() {
                self.position = line_start;
                break;
            }

            // Check if line contains | (table continuation)
            let remaining = self.remaining();
            let line = remaining.lines().next().unwrap_or("");
            if !line.contains('|') {
                self.position = line_start;
                break;
            }

            self.position = line_start;
            let row_line = self.consume_line();
            children.push(self.parse_table_row(row_line)?);
        }

        let span = Span::new(start as u32, self.position as u32);
        Ok(Some(Node::Table(Table { align, children, span })))
    }

    /// Parses a table row into arena-backed AST cells without temporary heap
    /// collection.
    ///
    /// The row iterator yields borrowed cell slices from the original line.
    /// Inline parsing then writes cell children into the parser arena, so no
    /// intermediate `Vec<&str>` or owned cell text is needed.
    pub(super) fn parse_table_row(&self, line: &'a str) -> ParseResult<TableRow<'a>> {
        let mut cells: Vec<'a, TableCell<'a>> = self.allocator.new_vec();
        for cell_content in Self::table_row_cells(line) {
            let cell_children = self.parse_inline(cell_content, 0)?;
            cells.push(TableCell { children: cell_children, span: Span::new(0, 0) });
        }
        Ok(TableRow { children: cells, span: Span::new(0, 0) })
    }

    /// Iterates table row cells from a line.
    ///
    /// Leading/trailing pipes are syntax delimiters, not empty cells in this
    /// parser's table model, so they are stripped once before splitting.
    pub(super) fn table_row_cells(line: &'a str) -> impl Iterator<Item = &'a str> {
        let trimmed = line.trim();
        let trimmed = trimmed.strip_prefix('|').unwrap_or(trimmed);
        let trimmed = trimmed.strip_suffix('|').unwrap_or(trimmed);
        trimmed.split('|').map(str::trim)
    }
}
