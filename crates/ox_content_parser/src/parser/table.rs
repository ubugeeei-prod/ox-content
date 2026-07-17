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
        children.push(self.parse_table_row(header_line, column_count)?);

        // Parse body rows
        loop {
            if self.is_at_end() {
                break;
            }

            // A blank line or another block-level construct terminates the
            // table. Ordinary lines remain data rows even without a pipe.
            if self.first_non_whitespace_in_line(self.position).is_none()
                || self.line_starts_block()
            {
                break;
            }

            let row_line = self.consume_line();
            children.push(self.parse_table_row(row_line, column_count)?);
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
    pub(super) fn parse_table_row(
        &self,
        line: &'a str,
        column_count: usize,
    ) -> ParseResult<TableRow<'a>> {
        let mut cells: Vec<'a, TableCell<'a>> = self.allocator.new_vec();
        for cell_content in Self::table_row_cells(line).take(column_count) {
            let cell_content = self.unescape_table_pipes(cell_content);
            let cell_children = self.parse_inline(cell_content, 0)?;
            cells.push(TableCell { children: cell_children, span: Span::new(0, 0) });
        }
        while cells.len() < column_count {
            cells.push(TableCell { children: self.allocator.new_vec(), span: Span::new(0, 0) });
        }
        Ok(TableRow { children: cells, span: Span::new(0, 0) })
    }

    /// Removes the table-level escape from pipes before inline parsing.
    ///
    /// GFM treats `\|` as a literal pipe even inside code spans. Normal inline
    /// parsing already handles the escape in prose, but code spans preserve
    /// backslashes, so the table parser must consume it first.
    fn unescape_table_pipes(&self, content: &'a str) -> &'a str {
        let bytes = content.as_bytes();
        if !bytes
            .iter()
            .enumerate()
            .any(|(index, &byte)| byte == b'|' && is_escaped_table_pipe(bytes, index))
        {
            return content;
        }

        let mut unescaped =
            ox_content_allocator::String::with_capacity_in(content.len(), self.allocator.bump());
        let mut copied_through = 0;
        let mut search_start = 0;
        while let Some(relative) = memchr(b'|', &bytes[search_start..]) {
            let pipe = search_start + relative;
            if is_escaped_table_pipe(bytes, pipe) {
                unescaped.push_str(&content[copied_through..pipe - 1]);
                unescaped.push('|');
                copied_through = pipe + 1;
            }
            search_start = pipe + 1;
        }
        unescaped.push_str(&content[copied_through..]);
        unescaped.into_bump_str()
    }

    /// Iterates table row cells from a line.
    ///
    /// Leading/trailing pipes are syntax delimiters, not empty cells in this
    /// parser's table model, so they are stripped once before splitting.
    pub(super) fn table_row_cells(line: &'a str) -> impl Iterator<Item = &'a str> {
        let trimmed = line.trim();
        let trimmed = trimmed.strip_prefix('|').unwrap_or(trimmed);
        let trimmed = if trimmed.ends_with('|')
            && !is_escaped_table_pipe(trimmed.as_bytes(), trimmed.len() - 1)
        {
            &trimmed[..trimmed.len() - 1]
        } else {
            trimmed
        };
        let bytes = trimmed.as_bytes();
        let mut cell_start = 0;

        std::iter::from_fn(move || {
            if cell_start > bytes.len() {
                return None;
            }

            let mut search_start = cell_start;
            while let Some(relative) = memchr(b'|', &bytes[search_start..]) {
                let pipe = search_start + relative;
                if !is_escaped_table_pipe(bytes, pipe) {
                    let cell = trimmed[cell_start..pipe].trim();
                    cell_start = pipe + 1;
                    return Some(cell);
                }
                search_start = pipe + 1;
            }

            let cell = trimmed[cell_start..].trim();
            cell_start = bytes.len() + 1;
            Some(cell)
        })
    }
}

fn is_escaped_table_pipe(bytes: &[u8], pipe: usize) -> bool {
    bytes[..pipe].iter().rev().take_while(|&&byte| byte == b'\\').count() % 2 == 1
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
