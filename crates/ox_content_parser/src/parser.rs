//! Markdown parser implementation.

use ox_content_allocator::{Allocator, Vec};
use ox_content_ast::{
    AlignKind, BlockQuote, Document, Html, Image, Link, List, ListItem, Node, Paragraph, Span,
    Table, TableCell, TableRow, Text,
};

use crate::error::{ParseError, ParseResult};

/// Parser options.
#[derive(Debug, Clone, Default)]
pub struct ParserOptions {
    /// Enable GFM (GitHub Flavored Markdown) extensions.
    pub gfm: bool,
    /// Enable footnotes.
    pub footnotes: bool,
    /// Enable task lists.
    pub task_lists: bool,
    /// Enable tables.
    pub tables: bool,
    /// Enable strikethrough.
    pub strikethrough: bool,
    /// Enable autolinks.
    pub autolinks: bool,
    /// Maximum nesting depth for block elements.
    pub max_nesting_depth: usize,
}

impl ParserOptions {
    /// Creates new parser options with GFM extensions enabled.
    #[must_use]
    pub fn gfm() -> Self {
        Self {
            gfm: true,
            footnotes: true,
            task_lists: true,
            tables: true,
            strikethrough: true,
            autolinks: true,
            max_nesting_depth: 100,
        }
    }
}

/// Markdown parser.
pub struct Parser<'a> {
    /// Arena allocator.
    allocator: &'a Allocator,
    /// Source text.
    source: &'a str,
    /// Parser options.
    options: ParserOptions,
    /// Current position in the source.
    position: usize,
    /// Current nesting depth.
    nesting_depth: usize,
}

struct ParsedListItem<'a> {
    ordered: bool,
    start: Option<u32>,
    content: &'a str,
    content_offset: usize,
    checked: Option<bool>,
}

impl<'a> Parser<'a> {
    /// Creates a new parser with default options.
    #[must_use]
    pub fn new(allocator: &'a Allocator, source: &'a str) -> Self {
        Self { allocator, source, options: ParserOptions::default(), position: 0, nesting_depth: 0 }
    }

    /// Creates a new parser with the specified options.
    #[must_use]
    pub fn with_options(allocator: &'a Allocator, source: &'a str, options: ParserOptions) -> Self {
        Self { allocator, source, options, position: 0, nesting_depth: 0 }
    }

    /// Parses the source into a document AST.
    pub fn parse(mut self) -> ParseResult<Document<'a>> {
        let mut children = self.allocator.new_vec();

        while !self.is_at_end() {
            if let Some(node) = self.parse_block()? {
                children.push(node);
            }
        }

        let span = Span::new(0, self.source.len() as u32);
        Ok(Document { children, span })
    }

    /// Checks if we've reached the end of input.
    fn is_at_end(&self) -> bool {
        self.position >= self.source.len()
    }

    /// Returns the remaining source.
    fn remaining(&self) -> &'a str {
        &self.source[self.position..]
    }

    /// Peeks at the current character.
    fn peek(&self) -> Option<char> {
        self.remaining().chars().next()
    }

    /// Advances by one character.
    fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.position += ch.len_utf8();
        Some(ch)
    }

    /// Skips whitespace characters.
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == ' ' || ch == '\t' {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Skips blank lines.
    fn skip_blank_lines(&mut self) {
        while !self.is_at_end() {
            let start = self.position;
            self.skip_whitespace();
            if self.peek() == Some('\n') {
                self.advance();
            } else {
                self.position = start;
                break;
            }
        }
    }

    /// Parses a block element.
    fn parse_block(&mut self) -> ParseResult<Option<Node<'a>>> {
        self.skip_blank_lines();

        if self.is_at_end() {
            return Ok(None);
        }

        // Check nesting depth
        if self.nesting_depth > self.options.max_nesting_depth {
            return Err(ParseError::NestingTooDeep {
                span: Span::new(self.position as u32, self.position as u32),
                max_depth: self.options.max_nesting_depth,
            });
        }

        let start = self.position;
        let line = self.remaining().lines().next().unwrap_or("");
        let trimmed = line.trim_start();
        let first = trimmed.as_bytes().first().copied();

        // Try to parse different block types
        match first {
            Some(b'#') if self.try_parse_heading() => return self.parse_heading(start),
            Some(b'-' | b'*') => {
                if self.try_parse_thematic_break() {
                    return self.parse_thematic_break(start);
                }
                if self.try_parse_list() {
                    return self.parse_list(start);
                }
            }
            Some(b'_') if self.try_parse_thematic_break() => {
                return self.parse_thematic_break(start);
            }
            Some(b'>') => return self.parse_block_quote(start),
            Some(b'`' | b'~') if self.try_parse_fenced_code() => {
                return self.parse_fenced_code(start);
            }
            Some(b'<') if self.try_parse_html_block() => return self.parse_html_block(start),
            Some(b'+' | b'0'..=b'9') if self.try_parse_list() => {
                return self.parse_list(start);
            }
            _ => {}
        }

        if self.options.tables && line.contains('|') && self.try_parse_table() {
            return self.parse_table(start);
        }

        // Default: parse as paragraph
        self.parse_paragraph(start)
    }

    fn line_starts_block(&self) -> bool {
        let line = self.remaining().lines().next().unwrap_or("");
        let trimmed = line.trim_start();
        let first = trimmed.as_bytes().first().copied();

        let starts_block = match first {
            Some(b'#') => self.try_parse_heading(),
            Some(b'-' | b'*') => self.try_parse_thematic_break() || self.try_parse_list(),
            Some(b'_') => self.try_parse_thematic_break(),
            Some(b'>') => self.try_parse_block_quote(),
            Some(b'`' | b'~') => self.try_parse_fenced_code(),
            Some(b'<') => self.try_parse_html_block(),
            Some(b'+' | b'0'..=b'9') => self.try_parse_list(),
            _ => false,
        };

        starts_block || (self.options.tables && line.contains('|') && self.try_parse_table())
    }

    fn try_parse_html_block(&self) -> bool {
        let line = self.remaining().lines().next().unwrap_or("");
        Self::parse_html_block_tag_name(line).is_some() || line.trim_start().starts_with("<!--")
    }

    fn parse_html_block(&mut self, start: usize) -> ParseResult<Option<Node<'a>>> {
        let line = self.remaining().lines().next().unwrap_or("");

        if line.trim_start().starts_with("<!--") {
            loop {
                let consumed = self.consume_line();
                if consumed.contains("-->") || self.is_at_end() {
                    break;
                }
            }

            let span = Span::new(start as u32, self.position as u32);
            let value = &self.source[start..self.position];
            return Ok(Some(Node::Html(Html { value, span })));
        }

        let Some(tag_name) = Self::parse_html_block_tag_name(line) else {
            return Ok(None);
        };

        if Self::is_type1_html_block_tag(&tag_name) {
            let closing_tag = format!("</{tag_name}");
            loop {
                let consumed = self.consume_line();
                if consumed.to_ascii_lowercase().contains(&closing_tag) || self.is_at_end() {
                    break;
                }
            }
        } else {
            self.consume_line();
            while !self.is_at_end() {
                let next = self.remaining().lines().next().unwrap_or("");
                if next.trim().is_empty() {
                    break;
                }
                self.consume_line();
            }
        }

        let span = Span::new(start as u32, self.position as u32);
        let value = &self.source[start..self.position];
        Ok(Some(Node::Html(Html { value, span })))
    }

    fn parse_html_block_tag_name(line: &str) -> Option<String> {
        let trimmed = line.trim_start();
        let after_open = trimmed.strip_prefix('<')?;
        let after_slash = after_open.strip_prefix('/').unwrap_or(after_open);
        let mut tag_len = 0;

        for byte in after_slash.as_bytes() {
            if byte.is_ascii_alphanumeric() || *byte == b'-' {
                tag_len += 1;
            } else {
                break;
            }
        }

        if tag_len == 0 {
            return None;
        }

        let tag_name = &after_slash[..tag_len];
        let next = after_slash.as_bytes().get(tag_len).copied();

        if let Some(byte) = next {
            if !matches!(byte, b' ' | b'\t' | b'>' | b'/') {
                return None;
            }
        }

        if !Self::is_supported_html_block_tag(tag_name) {
            return None;
        }

        Some(tag_name.to_ascii_lowercase())
    }

    fn is_supported_html_block_tag(tag_name: &str) -> bool {
        [
            "article",
            "aside",
            "blockquote",
            "details",
            "dialog",
            "div",
            "figcaption",
            "figure",
            "footer",
            "header",
            "main",
            "nav",
            "ol",
            "p",
            "pre",
            "script",
            "section",
            "style",
            "summary",
            "table",
            "tbody",
            "td",
            "tfoot",
            "th",
            "thead",
            "textarea",
            "tr",
            "ul",
        ]
        .iter()
        .any(|candidate| tag_name.eq_ignore_ascii_case(candidate))
    }

    fn is_type1_html_block_tag(tag_name: &str) -> bool {
        ["pre", "script", "style", "textarea"]
            .iter()
            .any(|candidate| tag_name.eq_ignore_ascii_case(candidate))
    }

    /// Checks if the current position starts a block quote.
    fn try_parse_block_quote(&self) -> bool {
        let remaining = self.remaining();
        let line = remaining.lines().next().unwrap_or("");
        let trimmed = line.trim_start();
        trimmed.starts_with('>')
    }

    /// Parses a block quote.
    fn parse_block_quote(&mut self, start: usize) -> ParseResult<Option<Node<'a>>> {
        self.nesting_depth += 1;

        // Collect lines belonging to this block quote and strip the `>` prefix.
        let mut inner = String::new();

        loop {
            if self.is_at_end() {
                break;
            }

            let line_start = self.position;
            self.skip_whitespace();

            // Blank line ends the block quote
            if self.peek() == Some('\n') || self.is_at_end() {
                self.position = line_start;
                break;
            }

            self.position = line_start;

            let remaining = self.remaining();
            let line = remaining.lines().next().unwrap_or("");
            let trimmed = line.trim_start();

            if let Some(after_gt) = trimmed.strip_prefix('>') {
                // Strip the optional single space after `>`
                let stripped = after_gt.strip_prefix(' ').unwrap_or(after_gt);
                inner.push_str(stripped);
                inner.push('\n');

                // Advance past this line
                self.position += line.len();
                if self.peek() == Some('\n') {
                    self.advance();
                }
            } else {
                // Line doesn't start with `>`, block quote ends
                break;
            }
        }

        // Recursively parse the inner content
        let inner_str = self.allocator.alloc_str(&inner);
        let sub_parser = Parser::with_options(self.allocator, inner_str, self.options.clone());
        let sub_doc = sub_parser.parse()?;

        self.nesting_depth -= 1;

        let span = Span::new(start as u32, self.position as u32);
        Ok(Some(Node::BlockQuote(BlockQuote { children: sub_doc.children, span })))
    }

    /// Checks if the current position starts a list.
    fn try_parse_list(&self) -> bool {
        let remaining = self.remaining();
        let line = remaining.lines().next().unwrap_or("");
        let trimmed = line.trim_start();

        // Unordered list: starts with -, *, or + followed by space
        if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
            return true;
        }

        // Ordered list: starts with digit(s) followed by . or ) and space
        let mut chars = trimmed.chars().peekable();
        let mut has_digit = false;
        while let Some(ch) = chars.peek() {
            if ch.is_ascii_digit() {
                has_digit = true;
                chars.next();
            } else {
                break;
            }
        }
        if has_digit {
            if let Some(ch) = chars.next() {
                if (ch == '.' || ch == ')') && chars.peek() == Some(&' ') {
                    return true;
                }
            }
        }

        false
    }

    /// Calculates the indentation level (number of spaces) of the current line.
    fn calc_indentation(&self, start: usize) -> usize {
        let mut indent = 0;
        let bytes = self.source.as_bytes();
        for byte in bytes.iter().skip(start) {
            match byte {
                b' ' => indent += 1,
                b'\t' => indent += 4, // Assume tab is 4 spaces
                _ => break,
            }
        }
        indent
    }

    fn parse_task_list_prefix(&self, content: &'a str) -> Option<(bool, usize)> {
        if !self.options.task_lists || content.len() < 3 {
            return None;
        }

        if (content.starts_with("[x]") || content.starts_with("[X]"))
            && (content.len() == 3 || content.starts_with("[x] ") || content.starts_with("[X] "))
        {
            return Some((true, usize::from(content.len() > 3) + 3));
        }

        if content.starts_with("[ ]") && (content.len() == 3 || content.starts_with("[ ] ")) {
            return Some((false, usize::from(content.len() > 3) + 3));
        }

        None
    }

    fn parse_list_item_line(&self, line_start: usize) -> Option<ParsedListItem<'a>> {
        let remaining = &self.source[line_start..];
        let line = remaining.lines().next().unwrap_or("");
        let trimmed = line.trim_start();
        let trimmed_offset = line_start + (line.len() - trimmed.len());

        if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
            let mut content = &trimmed[2..];
            let mut content_offset = trimmed_offset + 2;
            let mut checked = None;

            if let Some((done, consumed)) = self.parse_task_list_prefix(content) {
                checked = Some(done);
                content = &content[consumed..];
                content_offset += consumed;
            }

            return Some(ParsedListItem {
                ordered: false,
                start: None,
                content,
                content_offset,
                checked,
            });
        }

        let bytes = trimmed.as_bytes();
        let mut marker_end = 0;
        while marker_end < bytes.len() && bytes[marker_end].is_ascii_digit() {
            marker_end += 1;
        }

        if marker_end == 0 || marker_end + 1 >= bytes.len() {
            return None;
        }

        let marker = bytes[marker_end];
        if !matches!(marker, b'.' | b')') || bytes[marker_end + 1] != b' ' {
            return None;
        }

        Some(ParsedListItem {
            ordered: true,
            start: trimmed[..marker_end].parse().ok(),
            content: &trimmed[marker_end + 2..],
            content_offset: trimmed_offset + marker_end + 2,
            checked: None,
        })
    }

    fn line_at(&self, line_start: usize) -> &'a str {
        self.source[line_start..].lines().next().unwrap_or("")
    }

    fn next_line_start(&self, line_start: usize) -> usize {
        let line = self.line_at(line_start);
        let next = line_start + line.len();
        if self.source.as_bytes().get(next) == Some(&b'\n') {
            next + 1
        } else {
            next
        }
    }

    fn strip_indent_columns(line: &str, columns: usize) -> &str {
        let mut consumed = 0;
        let mut byte_index = 0;

        for ch in line.chars() {
            let width = if ch == ' ' {
                1
            } else if ch == '\t' {
                4
            } else {
                break;
            };

            if consumed + width > columns {
                break;
            }

            consumed += width;
            byte_index += ch.len_utf8();
        }

        &line[byte_index..]
    }

    /// Parses a list (ordered or unordered).
    fn parse_list(&mut self, start: usize) -> ParseResult<Option<Node<'a>>> {
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
            let Some(item) = self.parse_list_item_line(line_start) else {
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
            let mut item_source = String::new();
            item_source.push_str(item.content);
            if consumed_newline {
                item_source.push('\n');
            }
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
                    if self
                        .parse_list_item_line(continuation_start)
                        .is_some_and(|next| next.ordered == ordered)
                    {
                        break;
                    }

                    break;
                }

                let stripped = Self::strip_indent_columns(continuation_line, content_indent);
                item_source.push_str(stripped);
                item_source.push('\n');
                self.position = continuation_next;
                item_end = self.position;
            }

            let item_source = self.allocator.alloc_str(&item_source);
            let sub_parser =
                Parser::with_options(self.allocator, item_source, self.options.clone());
            let sub_doc = sub_parser.parse()?;
            let mut item_children = sub_doc.children;
            for child in &mut item_children {
                Self::offset_node_spans(child, item.content_offset as u32);
            }

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

    fn offset_span(span: &mut Span, offset: u32) {
        span.start += offset;
        span.end += offset;
    }

    fn offset_node_spans(node: &mut Node<'a>, offset: u32) {
        match node {
            Node::Paragraph(node) => {
                Self::offset_span(&mut node.span, offset);
                for child in &mut node.children {
                    Self::offset_node_spans(child, offset);
                }
            }
            Node::Heading(node) => {
                Self::offset_span(&mut node.span, offset);
                for child in &mut node.children {
                    Self::offset_node_spans(child, offset);
                }
            }
            Node::ThematicBreak(node) => Self::offset_span(&mut node.span, offset),
            Node::BlockQuote(node) => {
                Self::offset_span(&mut node.span, offset);
                for child in &mut node.children {
                    Self::offset_node_spans(child, offset);
                }
            }
            Node::List(node) => {
                Self::offset_span(&mut node.span, offset);
                for child in &mut node.children {
                    Self::offset_list_item_spans(child, offset);
                }
            }
            Node::ListItem(node) => Self::offset_list_item_spans(node, offset),
            Node::CodeBlock(node) => Self::offset_span(&mut node.span, offset),
            Node::Html(node) => Self::offset_span(&mut node.span, offset),
            Node::Table(node) => {
                Self::offset_span(&mut node.span, offset);
                for row in &mut node.children {
                    Self::offset_span(&mut row.span, offset);
                    for cell in &mut row.children {
                        Self::offset_span(&mut cell.span, offset);
                        for child in &mut cell.children {
                            Self::offset_node_spans(child, offset);
                        }
                    }
                }
            }
            Node::Text(node) => Self::offset_span(&mut node.span, offset),
            Node::Emphasis(node) => {
                Self::offset_span(&mut node.span, offset);
                for child in &mut node.children {
                    Self::offset_node_spans(child, offset);
                }
            }
            Node::Strong(node) => {
                Self::offset_span(&mut node.span, offset);
                for child in &mut node.children {
                    Self::offset_node_spans(child, offset);
                }
            }
            Node::InlineCode(node) => Self::offset_span(&mut node.span, offset),
            Node::Break(node) => Self::offset_span(&mut node.span, offset),
            Node::Link(node) => {
                Self::offset_span(&mut node.span, offset);
                for child in &mut node.children {
                    Self::offset_node_spans(child, offset);
                }
            }
            Node::Image(node) => Self::offset_span(&mut node.span, offset),
            Node::Delete(node) => {
                Self::offset_span(&mut node.span, offset);
                for child in &mut node.children {
                    Self::offset_node_spans(child, offset);
                }
            }
            Node::FootnoteReference(node) => Self::offset_span(&mut node.span, offset),
            Node::Definition(node) => Self::offset_span(&mut node.span, offset),
            Node::FootnoteDefinition(node) => {
                Self::offset_span(&mut node.span, offset);
                for child in &mut node.children {
                    Self::offset_node_spans(child, offset);
                }
            }
        }
    }

    fn offset_list_item_spans(list_item: &mut ListItem<'a>, offset: u32) {
        Self::offset_span(&mut list_item.span, offset);
        for child in &mut list_item.children {
            Self::offset_node_spans(child, offset);
        }
    }

    /// Checks if the current position starts a heading.
    fn try_parse_heading(&self) -> bool {
        let remaining = self.remaining();
        let mut chars = remaining.chars().peekable();
        let mut hash_count = 0;

        while chars.peek() == Some(&'#') {
            chars.next();
            hash_count += 1;
            if hash_count > 6 {
                return false;
            }
        }

        hash_count > 0 && matches!(chars.peek(), Some(' ') | Some('\t') | Some('\n') | None)
    }

    /// Checks if the current position starts a thematic break.
    fn try_parse_thematic_break(&self) -> bool {
        let remaining = self.remaining();
        let line = remaining.lines().next().unwrap_or("");
        let trimmed = line.trim();

        if trimmed.len() < 3 {
            return false;
        }

        let first = trimmed.chars().next().unwrap();
        if !matches!(first, '-' | '*' | '_') {
            return false;
        }

        trimmed.chars().all(|c| c == first || c == ' ' || c == '\t')
            && trimmed.chars().filter(|&c| c == first).count() >= 3
    }

    /// Checks if the current position starts a fenced code block.
    fn try_parse_fenced_code(&self) -> bool {
        let line = self.remaining().lines().next().unwrap_or("");
        if Self::indentation_columns(line) > 3 {
            return false;
        }

        let trimmed = line.trim_start();
        trimmed.starts_with("```") || trimmed.starts_with("~~~")
    }

    fn indentation_columns(line: &str) -> usize {
        let mut indent = 0;
        for ch in line.chars() {
            match ch {
                ' ' => indent += 1,
                '\t' => indent += 4,
                _ => break,
            }
        }
        indent
    }

    /// Checks if the current position starts a table.
    fn try_parse_table(&self) -> bool {
        let remaining = self.remaining();
        let lines: std::vec::Vec<&str> = remaining.lines().take(2).collect();

        if lines.len() < 2 {
            return false;
        }

        // First line must start with | or contain |
        let first_line = lines[0].trim();
        if !first_line.starts_with('|') && !first_line.contains('|') {
            return false;
        }

        // Second line must be the delimiter row (contains | and -)
        let second_line = lines[1].trim();
        if !second_line.contains('|') || !second_line.contains('-') {
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

    /// Parses a heading.
    fn parse_heading(&mut self, start: usize) -> ParseResult<Option<Node<'a>>> {
        let mut depth = 0u8;
        while self.peek() == Some('#') {
            depth += 1;
            self.advance();
        }

        self.skip_whitespace();

        let content_start = self.position;
        let mut content_end = content_start;

        // Read until end of line
        while let Some(ch) = self.peek() {
            if ch == '\n' {
                break;
            }
            self.advance();
            content_end = self.position;
        }

        // Skip trailing hashes and whitespace
        let content = self.source[content_start..content_end].trim_end();
        let content = content.trim_end_matches('#').trim_end();

        // Consume newline
        if self.peek() == Some('\n') {
            self.advance();
        }

        let span = Span::new(start as u32, self.position as u32);

        // Parse inline content
        let children = if !content.is_empty() {
            self.parse_inline(content, content_start)?
        } else {
            self.allocator.new_vec()
        };

        Ok(Some(Node::Heading(ox_content_ast::Heading { depth, children, span })))
    }

    /// Parses a thematic break.
    fn parse_thematic_break(&mut self, start: usize) -> ParseResult<Option<Node<'a>>> {
        // Skip to end of line
        while let Some(ch) = self.peek() {
            self.advance();
            if ch == '\n' {
                break;
            }
        }

        let span = Span::new(start as u32, self.position as u32);
        Ok(Some(Node::ThematicBreak(ox_content_ast::ThematicBreak { span })))
    }

    /// Parses a fenced code block.
    fn parse_fenced_code(&mut self, start: usize) -> ParseResult<Option<Node<'a>>> {
        let opening_indent = self.calc_indentation(start).min(3);
        for _ in 0..opening_indent {
            if self.peek() == Some(' ') {
                self.advance();
            }
        }

        let fence_char = self.peek().unwrap();
        let mut fence_len = 0;

        while self.peek() == Some(fence_char) {
            fence_len += 1;
            self.advance();
        }

        // Parse info string (language)
        self.skip_whitespace();
        let info_start = self.position;
        while let Some(ch) = self.peek() {
            if ch == '\n' {
                break;
            }
            self.advance();
        }
        let info = self.source[info_start..self.position].trim();
        let (lang, meta) = if info.is_empty() {
            (None, None)
        } else if let Some(space_idx) = info.find(' ') {
            (Some(&info[..space_idx]), Some(&info[space_idx + 1..]))
        } else {
            (Some(info), None)
        };

        // Skip newline after info string
        if self.peek() == Some('\n') {
            self.advance();
        }

        let mut value = String::new();

        loop {
            if self.is_at_end() {
                break;
            }

            let line_start = self.position;
            let line = self.line_at(line_start);
            let line_indent = Self::indentation_columns(line);

            if line_indent <= 3 {
                self.position = line_start;
                let indent_to_skip = self.calc_indentation(line_start).min(3);
                for _ in 0..indent_to_skip {
                    if self.peek() == Some(' ') {
                        self.advance();
                    }
                }

                // Check for closing fence
                let mut closing_fence_len = 0;
                while self.peek() == Some(fence_char) {
                    closing_fence_len += 1;
                    self.advance();
                }

                if closing_fence_len >= fence_len {
                    // Skip rest of line
                    while let Some(ch) = self.peek() {
                        if ch == '\n' {
                            self.advance();
                            break;
                        }
                        self.advance();
                    }
                    break;
                }
            }

            // Not a closing fence, reset and consume line
            self.position = line_start;
            let next_line = self.next_line_start(line_start);
            let stripped = Self::strip_indent_columns(line, opening_indent);
            value.push_str(stripped);
            if self.source.as_bytes().get(line_start + line.len()) == Some(&b'\n') {
                value.push('\n');
            }
            self.position = next_line;
        }

        let value = self.allocator.alloc_str(&value);
        let span = Span::new(start as u32, self.position as u32);

        Ok(Some(Node::CodeBlock(ox_content_ast::CodeBlock { lang, meta, value, span })))
    }

    /// Parses a table.
    fn parse_table(&mut self, start: usize) -> ParseResult<Option<Node<'a>>> {
        let mut rows: std::vec::Vec<std::vec::Vec<&str>> = std::vec::Vec::new();
        let mut align: Vec<'a, AlignKind> = self.allocator.new_vec();

        // Parse header row
        let header_line = self.consume_line();
        let header_cells = Self::parse_table_row_cells(header_line);
        rows.push(header_cells);

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
            let row_cells = Self::parse_table_row_cells(row_line);
            rows.push(row_cells);
        }

        // Build the table AST
        let mut children: Vec<'a, TableRow<'a>> = self.allocator.new_vec();

        for row_cells in rows {
            let mut cells: Vec<'a, TableCell<'a>> = self.allocator.new_vec();
            for cell_content in row_cells {
                let cell_children = self.parse_inline(cell_content, 0)?;
                let cell = TableCell { children: cell_children, span: Span::new(0, 0) };
                cells.push(cell);
            }
            let row = TableRow { children: cells, span: Span::new(0, 0) };
            children.push(row);
        }

        let span = Span::new(start as u32, self.position as u32);
        Ok(Some(Node::Table(Table { align, children, span })))
    }

    /// Consumes a line and returns it.
    fn consume_line(&mut self) -> &'a str {
        let start = self.position;
        while let Some(ch) = self.peek() {
            self.advance();
            if ch == '\n' {
                break;
            }
        }
        self.source[start..self.position].trim_end_matches('\n')
    }

    /// Parses table row cells from a line.
    fn parse_table_row_cells(line: &'a str) -> std::vec::Vec<&'a str> {
        let trimmed = line.trim();
        let trimmed = trimmed.strip_prefix('|').unwrap_or(trimmed);
        let trimmed = trimmed.strip_suffix('|').unwrap_or(trimmed);
        trimmed.split('|').map(str::trim).collect()
    }

    /// Parses a paragraph.
    fn parse_paragraph(&mut self, start: usize) -> ParseResult<Option<Node<'a>>> {
        let mut content_end = start;

        loop {
            if self.is_at_end() {
                break;
            }

            // Check for blank line (paragraph end)
            let line_start = self.position;
            self.skip_whitespace();
            if self.peek() == Some('\n') || self.is_at_end() {
                self.position = line_start;
                break;
            }

            self.position = line_start;

            // Check for block-level element that would end paragraph
            if self.line_starts_block() {
                break;
            }

            // Consume line
            while let Some(ch) = self.peek() {
                self.advance();
                if ch == '\n' {
                    break;
                }
            }
            content_end = self.position;
        }

        let content = self.source[start..content_end].trim();
        if content.is_empty() {
            return Ok(None);
        }

        let span = Span::new(start as u32, content_end as u32);

        // Parse inline content
        let children = self.parse_inline(content, start)?;

        Ok(Some(Node::Paragraph(Paragraph { children, span })))
    }

    /// Parses inline content.
    fn parse_inline(&self, content: &'a str, offset: usize) -> ParseResult<Vec<'a, Node<'a>>> {
        let mut children = self.allocator.new_vec();
        let mut pos = 0;
        let bytes = content.as_bytes();

        while pos < content.len() {
            let start = pos;

            // Look for special characters
            while pos < content.len() {
                let ch = bytes[pos];
                if matches!(ch, b'*' | b'_' | b'`' | b'[' | b'!' | b'~' | b'\\' | b'<') {
                    break;
                }
                pos += 1;
            }

            // Emit text before special character
            if pos > start {
                let text_content = &content[start..pos];
                let text = Text {
                    value: text_content,
                    span: Span::new((offset + start) as u32, (offset + pos) as u32),
                };
                children.push(Node::Text(text));
            }

            if pos >= content.len() {
                break;
            }

            // Handle special characters
            let ch = bytes[pos];
            match ch {
                b'\\' if pos + 1 < content.len() && bytes[pos + 1] == b'\n' => {
                    let break_node = ox_content_ast::Break {
                        span: Span::new((offset + pos) as u32, (offset + pos + 2) as u32),
                    };
                    children.push(Node::Break(break_node));
                    pos += 2;
                }
                b'<' => {
                    if let Some((html, end)) = Self::parse_inline_html(content, pos, offset) {
                        children.push(Node::Html(html));
                        pos = end;
                    } else {
                        let text = Text {
                            value: "<",
                            span: Span::new((offset + pos) as u32, (offset + pos + 1) as u32),
                        };
                        children.push(Node::Text(text));
                        pos += 1;
                    }
                }
                b'\\' if pos + 1 < content.len() => {
                    // Escape sequence
                    pos += 1;
                    let escaped = &content[pos..pos + 1];
                    let text = Text {
                        value: escaped,
                        span: Span::new((offset + pos - 1) as u32, (offset + pos + 1) as u32),
                    };
                    children.push(Node::Text(text));
                    pos += 1;
                }
                b'~' if self.options.strikethrough
                    && pos + 1 < content.len()
                    && bytes[pos + 1] == b'~' =>
                {
                    let inner_start = pos + 2;
                    let mut inner_end = inner_start;
                    let mut found = false;

                    while inner_end + 1 < content.len() {
                        if bytes[inner_end] == b'~' && bytes[inner_end + 1] == b'~' {
                            found = true;
                            break;
                        }
                        inner_end += 1;
                    }

                    if found {
                        let inner_content = &content[inner_start..inner_end];
                        let inner_children =
                            self.parse_inline(inner_content, offset + inner_start)?;
                        let span =
                            Span::new((offset + pos) as u32, (offset + inner_end + 2) as u32);
                        let delete = ox_content_ast::Delete { children: inner_children, span };
                        children.push(Node::Delete(delete));
                        pos = inner_end + 2;
                    } else {
                        let text = Text {
                            value: &content[pos..pos + 2],
                            span: Span::new((offset + pos) as u32, (offset + pos + 2) as u32),
                        };
                        children.push(Node::Text(text));
                        pos += 2;
                    }
                }
                b'*' | b'_' => {
                    // Emphasis or Strong
                    let marker = ch;
                    let mut count = 1;
                    while pos + count < content.len() && bytes[pos + count] == marker {
                        count += 1;
                    }

                    // Simple logic: find next matching sequence of same length
                    let inner_start = pos + count;
                    let mut inner_end = inner_start;
                    let mut found = false;

                    while inner_end < content.len() {
                        if bytes[inner_end] == marker {
                            let mut end_count = 1;
                            while inner_end + end_count < content.len()
                                && bytes[inner_end + end_count] == marker
                            {
                                end_count += 1;
                            }

                            if end_count >= count {
                                found = true;
                                break;
                            }
                            inner_end += end_count;
                        } else {
                            inner_end += 1;
                        }
                    }

                    if found {
                        let inner_content = &content[inner_start..inner_end];
                        // Recursively parse inner content
                        let inner_children =
                            self.parse_inline(inner_content, offset + inner_start)?;

                        let span =
                            Span::new((offset + pos) as u32, (offset + inner_end + count) as u32);

                        if count >= 2 {
                            // Strong
                            let strong = ox_content_ast::Strong { children: inner_children, span };
                            children.push(Node::Strong(strong));
                            pos = inner_end + count;
                        } else {
                            // Emphasis
                            let emphasis =
                                ox_content_ast::Emphasis { children: inner_children, span };
                            children.push(Node::Emphasis(emphasis));
                            pos = inner_end + count;
                        }
                    } else {
                        // Treat as text
                        let text = Text {
                            value: &content[pos..pos + count],
                            span: Span::new((offset + pos) as u32, (offset + pos + count) as u32),
                        };
                        children.push(Node::Text(text));
                        pos += count;
                    }
                }
                b'`' => {
                    // Inline code
                    pos += 1;
                    let code_start = pos;
                    while pos < content.len() && bytes[pos] != b'`' {
                        pos += 1;
                    }
                    if pos < content.len() {
                        let code_content = &content[code_start..pos];
                        let inline_code = ox_content_ast::InlineCode {
                            value: code_content,
                            span: Span::new(
                                (offset + code_start - 1) as u32,
                                (offset + pos + 1) as u32,
                            ),
                        };
                        children.push(Node::InlineCode(inline_code));
                        pos += 1;
                    } else {
                        // No closing backtick, treat as text
                        let text = Text {
                            value: &content[code_start - 1..],
                            span: Span::new(
                                (offset + code_start - 1) as u32,
                                (offset + content.len()) as u32,
                            ),
                        };
                        children.push(Node::Text(text));
                    }
                }
                b'[' => {
                    // Link: [text](url)
                    let link_start = pos;
                    pos += 1;
                    let text_start = pos;

                    // Find closing ]
                    let mut bracket_depth = 1;
                    while pos < content.len() && bracket_depth > 0 {
                        match bytes[pos] {
                            b'[' => bracket_depth += 1,
                            b']' => bracket_depth -= 1,
                            _ => {} // Ignore other characters
                        }
                        if bracket_depth > 0 {
                            pos += 1;
                        }
                    }

                    if pos < content.len()
                        && bytes[pos] == b']'
                        && pos + 1 < content.len()
                        && bytes[pos + 1] == b'('
                    {
                        let link_text = &content[text_start..pos];
                        pos += 2; // skip ](

                        // Find closing )
                        let url_start = pos;
                        let mut paren_depth = 1;
                        while pos < content.len() && paren_depth > 0 {
                            match bytes[pos] {
                                b'(' => paren_depth += 1,
                                b')' => paren_depth -= 1,
                                _ => {} // Ignore other characters
                            }
                            if paren_depth > 0 {
                                pos += 1;
                            }
                        }

                        if pos < content.len() && bytes[pos] == b')' {
                            let url = &content[url_start..pos];
                            pos += 1; // skip )

                            // Parse link text as inline content
                            let link_children =
                                self.parse_inline(link_text, offset + text_start)?;

                            let link = Link {
                                url,
                                title: None,
                                children: link_children,
                                span: Span::new(
                                    (offset + link_start) as u32,
                                    (offset + pos) as u32,
                                ),
                            };
                            children.push(Node::Link(link));
                        } else {
                            // Invalid link, treat as text
                            let text = Text {
                                value: &content[link_start..pos],
                                span: Span::new(
                                    (offset + link_start) as u32,
                                    (offset + pos) as u32,
                                ),
                            };
                            children.push(Node::Text(text));
                        }
                    } else {
                        // Not a link, just a [
                        let text = Text {
                            value: "[",
                            span: Span::new(
                                (offset + link_start) as u32,
                                (offset + link_start + 1) as u32,
                            ),
                        };
                        children.push(Node::Text(text));
                        pos = link_start + 1;
                    }
                }
                b'!' => {
                    // Image: ![alt](url)
                    if pos + 1 < content.len() && bytes[pos + 1] == b'[' {
                        let image_start = pos;
                        pos += 2; // skip ![
                        let alt_start = pos;

                        // Find closing ]
                        let mut bracket_depth = 1;
                        while pos < content.len() && bracket_depth > 0 {
                            match bytes[pos] {
                                b'[' => bracket_depth += 1,
                                b']' => bracket_depth -= 1,
                                _ => {}
                            }
                            if bracket_depth > 0 {
                                pos += 1;
                            }
                        }

                        if pos < content.len()
                            && bytes[pos] == b']'
                            && pos + 1 < content.len()
                            && bytes[pos + 1] == b'('
                        {
                            let alt_text = &content[alt_start..pos];
                            pos += 2; // skip ](

                            // Find closing )
                            let url_start = pos;
                            let mut paren_depth = 1;
                            while pos < content.len() && paren_depth > 0 {
                                match bytes[pos] {
                                    b'(' => paren_depth += 1,
                                    b')' => paren_depth -= 1,
                                    _ => {}
                                }
                                if paren_depth > 0 {
                                    pos += 1;
                                }
                            }

                            if pos < content.len() && bytes[pos] == b')' {
                                let url = &content[url_start..pos];
                                pos += 1; // skip )

                                let image = Image {
                                    url,
                                    alt: alt_text,
                                    title: None,
                                    span: Span::new(
                                        (offset + image_start) as u32,
                                        (offset + pos) as u32,
                                    ),
                                };
                                children.push(Node::Image(image));
                            } else {
                                // Invalid image, treat as text
                                let text = Text {
                                    value: &content[image_start..pos],
                                    span: Span::new(
                                        (offset + image_start) as u32,
                                        (offset + pos) as u32,
                                    ),
                                };
                                children.push(Node::Text(text));
                            }
                        } else {
                            // Not an image, just ![
                            let text = Text {
                                value: "![",
                                span: Span::new(
                                    (offset + image_start) as u32,
                                    (offset + image_start + 2) as u32,
                                ),
                            };
                            children.push(Node::Text(text));
                            pos = image_start + 2;
                        }
                    } else {
                        // Just a !, treat as text
                        let text = Text {
                            value: "!",
                            span: Span::new((offset + pos) as u32, (offset + pos + 1) as u32),
                        };
                        children.push(Node::Text(text));
                        pos += 1;
                    }
                }
                _ => {
                    // Other special characters
                    let text = Text {
                        value: &content[pos..pos + 1],
                        span: Span::new((offset + pos) as u32, (offset + pos + 1) as u32),
                    };
                    children.push(Node::Text(text));
                    pos += 1;
                }
            }
        }

        Ok(children)
    }

    fn parse_inline_html(content: &'a str, pos: usize, offset: usize) -> Option<(Html<'a>, usize)> {
        let bytes = content.as_bytes();

        if content[pos..].starts_with("<!--") {
            let end = content[pos + 4..].find("-->").map(|found| pos + 4 + found + 3)?;
            return Some((
                Html {
                    value: &content[pos..end],
                    span: Span::new((offset + pos) as u32, (offset + end) as u32),
                },
                end,
            ));
        }

        let closing = bytes.get(pos + 1) == Some(&b'/');
        let tag_start = if closing { pos + 2 } else { pos + 1 };
        if !bytes.get(tag_start).is_some_and(u8::is_ascii_alphabetic) {
            return None;
        }

        let mut tag_end = tag_start + 1;
        while tag_end < bytes.len()
            && (bytes[tag_end].is_ascii_alphanumeric() || bytes[tag_end] == b'-')
        {
            tag_end += 1;
        }

        let mut cursor = tag_end;
        if closing {
            while cursor < bytes.len() && matches!(bytes[cursor], b' ' | b'\t') {
                cursor += 1;
            }
            if bytes.get(cursor) != Some(&b'>') {
                return None;
            }
            let end = cursor + 1;
            return Some((
                Html {
                    value: &content[pos..end],
                    span: Span::new((offset + pos) as u32, (offset + end) as u32),
                },
                end,
            ));
        }

        if !matches!(bytes.get(cursor), Some(b' ' | b'\t' | b'/' | b'>')) {
            return None;
        }

        let mut quote = None;
        while cursor < bytes.len() {
            let byte = bytes[cursor];
            if let Some(quote_byte) = quote {
                if byte == quote_byte {
                    quote = None;
                }
            } else if matches!(byte, b'"' | b'\'') {
                quote = Some(byte);
            } else if byte == b'>' {
                let end = cursor + 1;
                return Some((
                    Html {
                        value: &content[pos..end],
                        span: Span::new((offset + pos) as u32, (offset + end) as u32),
                    },
                    end,
                ));
            } else if byte == b'\n' {
                return None;
            }
            cursor += 1;
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_image() {
        let allocator = Allocator::new();
        let doc = Parser::new(&allocator, "![Alt text](/path/to/image.png)").parse().unwrap();
        assert_eq!(doc.children.len(), 1);
        match &doc.children[0] {
            Node::Paragraph(p) => {
                assert_eq!(p.children.len(), 1);
                match &p.children[0] {
                    Node::Image(img) => {
                        assert_eq!(img.alt, "Alt text");
                        assert_eq!(img.url, "/path/to/image.png");
                    }
                    _ => panic!("expected image, got {:?}", &p.children[0]),
                }
            }
            _ => panic!("expected paragraph"),
        }
    }

    #[test]
    fn test_parse_heading() {
        let allocator = Allocator::new();
        // Use "# " with trailing space - our parser requires space/tab/newline after #
        let doc = Parser::new(&allocator, "# Hello\n").parse().unwrap();
        assert_eq!(doc.children.len(), 1);
        match &doc.children[0] {
            Node::Heading(h) => {
                assert_eq!(h.depth, 1);
            }
            _ => panic!("expected heading"),
        }
    }

    #[test]
    fn test_parse_paragraph() {
        let allocator = Allocator::new();
        let doc = Parser::new(&allocator, "Hello world").parse().unwrap();
        assert_eq!(doc.children.len(), 1);
        assert!(matches!(&doc.children[0], Node::Paragraph(_)));
    }

    #[test]
    fn test_parse_thematic_break() {
        let allocator = Allocator::new();
        let doc = Parser::new(&allocator, "---").parse().unwrap();
        assert_eq!(doc.children.len(), 1);
        assert!(matches!(&doc.children[0], Node::ThematicBreak(_)));
    }

    #[test]
    fn test_parse_fenced_code() {
        let allocator = Allocator::new();
        let doc = Parser::new(
            &allocator,
            "```rust\nfn main() {}
```",
        )
        .parse()
        .unwrap();
        assert_eq!(doc.children.len(), 1);
        match &doc.children[0] {
            Node::CodeBlock(cb) => {
                assert_eq!(cb.lang, Some("rust"));
            }
            _ => panic!("expected code block"),
        }
    }

    #[test]
    fn test_parse_inline_code() {
        let allocator = Allocator::new();
        let doc = Parser::new(&allocator, "Use `code` here").parse().unwrap();
        assert_eq!(doc.children.len(), 1);
        match &doc.children[0] {
            Node::Paragraph(p) => {
                assert!(p.children.iter().any(|n| matches!(n, Node::InlineCode(_))));
            }
            _ => panic!("expected paragraph"),
        }
    }

    #[test]
    fn test_parse_strikethrough() {
        let allocator = Allocator::new();
        let doc =
            Parser::with_options(&allocator, "~~done~~", ParserOptions::gfm()).parse().unwrap();
        assert_eq!(doc.children.len(), 1);
        match &doc.children[0] {
            Node::Paragraph(p) => {
                assert!(matches!(&p.children[0], Node::Delete(_)));
            }
            _ => panic!("expected paragraph"),
        }
    }

    #[test]
    fn test_parse_hard_break() {
        let allocator = Allocator::new();
        let doc = Parser::new(&allocator, "line 1\\\nline 2").parse().unwrap();
        assert_eq!(doc.children.len(), 1);
        match &doc.children[0] {
            Node::Paragraph(p) => {
                assert!(p.children.iter().any(|n| matches!(n, Node::Break(_))));
            }
            _ => panic!("expected paragraph"),
        }
    }

    #[test]
    fn test_parse_table() {
        let allocator = Allocator::new();
        let table_md = "| Header 1 | Header 2 |\n|----------|----------|\n| Cell 1   | Cell 2   |";
        let parser = Parser::with_options(&allocator, table_md, ParserOptions::gfm());
        let doc = parser.parse().unwrap();
        assert_eq!(doc.children.len(), 1);
        match &doc.children[0] {
            Node::Table(t) => {
                assert_eq!(t.children.len(), 2); // header + 1 body row
            }
            _ => panic!("expected table, got {:?}", &doc.children[0]),
        }
    }

    #[test]
    fn test_parse_unordered_list() {
        let allocator = Allocator::new();
        let list_md = "- Item 1\n- Item 2\n- Item 3";
        let parser = Parser::new(&allocator, list_md);
        let doc = parser.parse().unwrap();
        assert_eq!(doc.children.len(), 1);
        match &doc.children[0] {
            Node::List(list) => {
                assert!(!list.ordered);
                assert_eq!(list.children.len(), 3);
            }
            _ => panic!("expected list, got {:?}", &doc.children[0]),
        }
    }

    #[test]
    fn test_parse_ordered_list() {
        let allocator = Allocator::new();
        let list_md = "1. First\n2. Second\n3. Third";
        let parser = Parser::new(&allocator, list_md);
        let doc = parser.parse().unwrap();
        assert_eq!(doc.children.len(), 1);
        match &doc.children[0] {
            Node::List(list) => {
                assert!(list.ordered);
                assert_eq!(list.children.len(), 3);
            }
            _ => panic!("expected list, got {:?}", &doc.children[0]),
        }
    }

    #[test]
    fn test_parse_block_quote() {
        let allocator = Allocator::new();
        let doc = Parser::new(&allocator, "> Hello world").parse().unwrap();
        assert_eq!(doc.children.len(), 1);
        match &doc.children[0] {
            Node::BlockQuote(bq) => {
                assert_eq!(bq.children.len(), 1);
                assert!(matches!(&bq.children[0], Node::Paragraph(_)));
            }
            _ => panic!("expected block quote, got {:?}", &doc.children[0]),
        }
    }

    #[test]
    fn test_parse_block_quote_multiline() {
        let allocator = Allocator::new();
        let doc = Parser::new(&allocator, "> line 1\n> line 2").parse().unwrap();
        assert_eq!(doc.children.len(), 1);
        match &doc.children[0] {
            Node::BlockQuote(bq) => {
                assert_eq!(bq.children.len(), 1);
            }
            _ => panic!("expected block quote, got {:?}", &doc.children[0]),
        }
    }

    #[test]
    fn test_parse_nested_block_quote() {
        let allocator = Allocator::new();
        let doc = Parser::new(&allocator, "> > nested").parse().unwrap();
        assert_eq!(doc.children.len(), 1);
        match &doc.children[0] {
            Node::BlockQuote(bq) => {
                assert_eq!(bq.children.len(), 1);
                assert!(matches!(&bq.children[0], Node::BlockQuote(_)));
            }
            _ => panic!("expected block quote, got {:?}", &doc.children[0]),
        }
    }
}
