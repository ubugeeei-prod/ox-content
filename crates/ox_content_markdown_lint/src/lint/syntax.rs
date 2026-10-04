use ox_content_allocator::Allocator;
use ox_content_ast as walk;
use ox_content_ast::{
    CodeBlock, Heading, Html, Image, Link, Paragraph, Span, TableCell, Text, Visit,
};
use ox_content_parser::{Parser, ParserOptions};

use super::{
    InternalMarkdownLintOptions, MarkdownLintDiagnostic,
    controls::Controls,
    source::{Source, frontmatter_end},
    structure::Structure,
};

pub(super) struct Syntax {
    pub visible: Vec<Span>,
    pub blocks: Vec<Span>,
    pub skipped: Vec<Span>,
    pub breaks: Vec<Span>,
    pub hidden: Vec<Span>,
    urls: Vec<Span>,
    pub diagnostics: Vec<MarkdownLintDiagnostic>,
    pub controls: Controls,
}

impl Syntax {
    pub fn analyze(source: &Source<'_>, options: &InternalMarkdownLintOptions) -> Self {
        let base = frontmatter_end(source.text);
        let allocator = Allocator::for_source_len(source.text.len() - base);
        let mut parser_options = ParserOptions::gfm();
        parser_options.mdx = options.mdx;
        let mut result = Self {
            visible: Vec::new(),
            blocks: Vec::new(),
            skipped: Vec::new(),
            breaks: Vec::new(),
            hidden: Vec::new(),
            urls: Vec::new(),
            diagnostics: Vec::new(),
            controls: Controls::default(),
        };
        if base > 0 {
            result.skipped.push(Span::new(0, base as u32));
        }
        match Parser::with_options(&allocator, &source.text[base..], parser_options).parse() {
            Ok(document) => {
                let structure = Structure::new(source, &options.rules);
                Analyzer { base: base as u32, source, syntax: &mut result, structure }
                    .visit_document(&document);
            }
            Err(error) => {
                let span = error.span();
                let mut diagnostic = source.diagnostic(
                    "markdown-parse",
                    error.to_string(),
                    base + span.start as usize,
                    base + span.end as usize,
                );
                diagnostic.severity = "error".into();
                result.diagnostics.push(diagnostic);
                // A failed MDX parse must not accidentally lint JavaScript as prose.
            }
        }
        result.visible.sort_unstable_by_key(|span| span.start);
        {
            result.urls.sort_unstable_by_key(|v| v.start);
            let urls = &result.urls;
            if !urls.is_empty() {
                let mut index = 0;
                let mut visible = Vec::with_capacity(result.visible.len());
                for span in result.visible.drain(..) {
                    let mut start = span.start as usize;
                    let end = span.end as usize;
                    while urls.get(index).is_some_and(|v| v.end as usize <= start) {
                        index += 1;
                    }
                    let mut current = index;
                    while let Some(url) = urls.get(current).filter(|v| (v.start as usize) < end) {
                        if url.start as usize > start {
                            visible.push(Span::new(
                                start as u32,
                                (url.start as usize).min(end) as u32,
                            ));
                        }
                        start = start.max(url.end as usize);
                        current += 1;
                    }
                    if start < end {
                        visible.push(Span::new(start as u32, end as u32));
                    }
                }
                result.visible = visible;
            }
        }
        result.skipped.sort_unstable_by_key(|span| span.start);
        result.breaks.sort_unstable_by_key(|span| span.start);
        result.hidden.sort_unstable_by_key(|span| span.start);
        result
    }

    pub fn mask_line<'b>(&self, line: &'b str, offset: usize, buffer: &'b mut String) -> &'b str {
        let mut index = self.visible.partition_point(|span| span.end as usize <= offset);
        let end = offset + line.len();
        if self
            .visible
            .get(index)
            .is_some_and(|span| span.start as usize <= offset && span.end as usize >= end)
        {
            return line;
        }
        buffer.clear();
        let mut cursor = offset;
        while let Some(span) = self.visible.get(index).filter(|span| (span.start as usize) < end) {
            let start = (span.start as usize).max(cursor);
            let visible_end = (span.end as usize).min(end);
            if visible_end <= cursor {
                index += 1;
                continue;
            }
            if start > cursor {
                buffer.extend(
                    line[cursor - offset..start - offset]
                        .chars()
                        .map(|c| if c.is_whitespace() { c } else { ' ' }),
                );
            }
            buffer.push_str(&line[start - offset..visible_end - offset]);
            cursor = visible_end;
            index += 1;
        }
        if cursor < end {
            buffer.extend(
                line[cursor - offset..].chars().map(|c| if c.is_whitespace() { c } else { ' ' }),
            );
        }
        buffer
    }

    pub fn skipped_at(&self, offset: usize) -> bool {
        let index = self.skipped.partition_point(|span| span.start as usize <= offset);
        index.checked_sub(1).is_some_and(|i| offset < self.skipped[i].end as usize)
    }
}

struct Analyzer<'r, 's> {
    base: u32,
    source: &'r Source<'s>,
    syntax: &'r mut Syntax,
    structure: Structure<'r, 's>,
}

impl Analyzer<'_, '_> {
    fn span(&self, span: Span) -> Span {
        Span::new(span.start + self.base, span.end + self.base)
    }
}

impl<'a> Visit<'a> for Analyzer<'_, '_> {
    fn visit_text(&mut self, node: &Text<'a>) {
        let span = self.span(node.span);
        let Some(raw) = self.source.text.get(span.start as usize..span.end as usize) else {
            return;
        };
        if raw.contains('&')
            && raw != node.value
            && let Some(pattern) = super::patterns::ENTITY_PATTERN.as_ref()
        {
            let mut start = 0;
            for entity in pattern.find_iter(raw) {
                if entity.start() > start {
                    self.syntax.visible.push(Span::new(
                        span.start + start as u32,
                        span.start + entity.start() as u32,
                    ));
                }
                self.syntax.hidden.push(Span::new(
                    span.start + entity.start() as u32,
                    span.start + entity.end() as u32,
                ));
                start = entity.end();
            }
            if start < raw.len() {
                self.syntax.visible.push(Span::new(span.start + start as u32, span.end));
            }
        } else {
            self.syntax.visible.push(span);
        }
    }
    fn visit_break(&mut self, node: &ox_content_ast::Break) {
        self.syntax.breaks.push(self.span(node.span));
    }

    fn visit_paragraph(&mut self, node: &Paragraph<'a>) {
        self.syntax.blocks.push(self.span(node.span));
        walk::walk_paragraph(self, node);
    }

    fn visit_table_cell(&mut self, node: &TableCell<'a>) {
        self.syntax.blocks.push(self.span(node.span));
        walk::walk_table_cell(self, node);
    }

    fn visit_heading(&mut self, node: &Heading<'a>) {
        self.structure.heading(node, self.span(node.span), &mut self.syntax.diagnostics);
        self.syntax.blocks.push(self.span(node.span));
        walk::walk_heading(self, node);
    }

    fn visit_code_block(&mut self, node: &CodeBlock<'a>) {
        let span = self.span(node.span);
        self.structure.code(node, span, &mut self.syntax.diagnostics);
        self.syntax.skipped.push(span);
    }

    fn visit_html(&mut self, node: &Html<'a>) {
        let span = self.span(node.span);
        self.syntax.controls.collect(self.source, span);
        self.syntax.hidden.push(span);
        let line = self.source.line_index(span.start as usize);
        let prefix = &self.source.text[self.source.lines[line]..span.start as usize];
        if prefix.trim().is_empty() {
            self.syntax.skipped.push(span);
        }
    }

    fn visit_definition(&mut self, node: &ox_content_ast::Definition<'a>) {
        self.syntax.skipped.push(self.span(node.span));
    }
    fn visit_math_block(&mut self, node: &ox_content_ast::MathBlock<'a>) {
        self.syntax.skipped.push(self.span(node.span));
    }
    fn visit_mdxjs_esm(&mut self, node: &ox_content_ast::MdxjsEsm<'a>) {
        self.syntax.skipped.push(self.span(node.span));
    }
    fn visit_mdx_flow_expression(&mut self, node: &ox_content_ast::MdxFlowExpression<'a>) {
        self.syntax.skipped.push(self.span(node.span));
    }

    fn visit_inline_code(&mut self, node: &ox_content_ast::InlineCode<'a>) {
        self.syntax.hidden.push(self.span(node.span));
    }
    fn visit_inline_math(&mut self, node: &ox_content_ast::InlineMath<'a>) {
        self.syntax.hidden.push(self.span(node.span));
    }
    fn visit_mdx_text_expression(&mut self, node: &ox_content_ast::MdxTextExpression<'a>) {
        self.syntax.hidden.push(self.span(node.span));
    }
    fn visit_link(&mut self, node: &Link<'a>) {
        self.structure.link(node, self.span(node.span), &mut self.syntax.diagnostics);
        let span = self.span(node.span);
        let raw = &self.source.text[span.start as usize..span.end as usize];
        if raw.strip_prefix('[').is_some_and(|label| {
            label.starts_with("http://")
                || label.starts_with("https://")
                || label.starts_with("www.")
                || label.starts_with("mailto:")
        }) {
            self.syntax.hidden.push(span);
            return;
        }
        // Autolink labels are URLs, not vocabulary. Include punctuation
        // split off by GFM only for bare URLs, never for bracketed links.
        if node.children.len() == 1
            && matches!(&node.children[0], ox_content_ast::Node::Text(text) if text.value == node.url || text.value.starts_with("https://") || text.value.starts_with("http://") || text.value.starts_with("www.") || text.value.starts_with("mailto:"))
        {
            self.syntax.hidden.push(span);
            let start = span.start as usize;
            let raw = &self.source.text[start..];
            if raw.starts_with("https://") || raw.starts_with("http://") || raw.starts_with("www.")
            {
                let end = start + raw.find(char::is_whitespace).unwrap_or(raw.len());
                self.syntax.urls.push(Span::new(span.start, end as u32));
            }
            return;
        }
        walk::walk_link(self, node);
    }

    fn visit_image(&mut self, node: &Image<'a>) {
        self.structure.image(node, self.span(node.span), &mut self.syntax.diagnostics);
    }
}
