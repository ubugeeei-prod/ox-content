use compact_str::CompactString;
use ox_content_ast::{self as ast, Node, Span, Visit};

#[derive(Default)]
pub(in crate::lint) struct Document {
    pub(super) headings: Vec<Heading>,
    pub(super) codes: Vec<Code>,
    pub(super) lists: Vec<List>,
    pub(super) list_items: Vec<Span>,
    pub(super) quotes: Vec<Span>,
    pub(super) paragraphs: Vec<Span>,
    pub(super) html: Vec<Span>,
    pub(super) comments: Vec<Span>,
    pub(super) html_contents: Vec<Span>,
    pub(super) list_ranges: Vec<Span>,
    pub(super) thematic: Vec<Span>,
    pub(super) emphasis: Vec<(Span, bool)>,
    pub(super) inline_code: Vec<Span>,
    pub(super) links: Vec<Link>,
    pub(super) tables: Vec<Span>,
    pub(super) definitions: Vec<Definition>,
    pub(super) text: Vec<Span>,
    pub(super) breaks: Vec<Span>,
}
pub(super) struct Heading {
    pub(super) span: Span,
    pub(super) level: u8,
    pub(super) text: CompactString,
    pub(super) slug_text: CompactString,
    pub(super) content: Span,
}
pub(super) struct Code {
    pub(super) span: Span,
    pub(super) language: CompactString,
    pub(super) body_lines: usize,
    pub(super) fenced: bool,
}
pub(super) struct List {
    pub(super) span: Span,
    pub(super) ordered: bool,
    pub(super) items: std::ops::Range<usize>,
}
pub(super) struct Link {
    pub(super) span: Span,
    pub(super) url: CompactString,
    pub(super) label: CompactString,
    pub(super) image: bool,
}
pub(super) struct Definition {
    pub(super) span: Span,
    pub(super) label: CompactString,
}

impl Document {
    pub(in crate::lint) fn finish(&mut self, source: &str) {
        for list in &self.lists {
            if let Some(previous) = self.list_ranges.last_mut()
                && list.span.start <= previous.end
            {
                previous.end = previous.end.max(list.span.end);
            } else {
                self.list_ranges.push(list.span);
            }
        }
        let mut tags: Vec<(&str, u32)> = Vec::new();
        for &span in &self.html {
            let raw = &source[span.start as usize..span.end as usize];
            let mut offset = 0;
            while let Some(start) = raw[offset..].find("<!--") {
                let start = offset + start;
                let Some(end) = raw[start + 4..].find("-->") else {
                    break;
                };
                offset = start + 4 + end + 3;
                self.comments
                    .push(Span::new(span.start + start as u32, span.start + offset as u32));
            }
            let Some(tag) = raw.strip_prefix('<') else {
                continue;
            };
            let closing = tag.starts_with('/');
            let tag = tag.strip_prefix('/').unwrap_or(tag);
            let length =
                tag.bytes().take_while(|c| c.is_ascii_alphanumeric() || *c == b'-').count();
            if length == 0 {
                continue;
            }
            let tag = &tag[..length];
            if closing {
                if let Some(index) =
                    tags.iter().rposition(|(name, _)| name.eq_ignore_ascii_case(tag))
                {
                    let start = tags[index].1;
                    if index == 0 {
                        self.html_contents.push(Span::new(start, span.start));
                    }
                    tags.truncate(index);
                }
            } else if raw.ends_with('>') && !raw.ends_with("/>") {
                tags.push((tag, span.end));
            }
        }
    }

    pub(in crate::lint) fn record(&mut self, node: &Node<'_>, base: u32, source: &str) {
        let span = |v: Span| Span::new(v.start + base, v.end + base);
        match node {
            Node::Heading(v) => {
                let mut label = Label::default();
                label.visit_heading(v);
                let mut text = CompactString::default();
                for child in &v.children {
                    if matches!(child, Node::Html(_)) {
                        continue;
                    }
                    let child = span(child.span());
                    if let Some(raw) = source.get(child.start as usize..child.end as usize) {
                        for c in raw.chars() {
                            text.push(if c == '\n' || c == '\r' { ' ' } else { c });
                        }
                    }
                }
                self.headings.push(Heading {
                    span: span(v.span),
                    level: v.depth,
                    text,
                    slug_text: label.0,
                    content: Span::new(
                        v.children.first().map_or(v.span.start + base, |v| v.span().start + base),
                        v.children.last().map_or(v.span.start + base, |v| v.span().end + base),
                    ),
                });
            }
            Node::CodeBlock(v) => {
                let range = span(v.span);
                let raw = &source[range.start as usize..range.end as usize];
                let first = super::prefix(raw.lines().next().unwrap_or(""));
                let marker = first.bytes().next().unwrap_or(0);
                let body_lines = v.value.lines().count();
                let fenced = matches!(marker, b'`' | b'~')
                    && first.bytes().take_while(|&c| c == marker).count() >= 3
                    && body_lines < raw.lines().count();
                self.codes.push(Code {
                    span: range,
                    language: v.lang.unwrap_or("").into(),
                    body_lines,
                    fenced,
                });
            }
            Node::List(v) => {
                let start = self.list_items.len();
                self.list_items.extend(v.children.iter().map(|item| span(item.span)));
                self.lists.push(List {
                    span: span(v.span),
                    ordered: v.ordered,
                    items: start..self.list_items.len(),
                });
            }
            Node::BlockQuote(v) => self.quotes.push(span(v.span)),
            Node::Paragraph(v) => self.paragraphs.push(span(v.span)),
            Node::Html(v) => self.html.push(span(v.span)),
            Node::ThematicBreak(v) => self.thematic.push(span(v.span)),
            Node::Emphasis(v) => self.emphasis.push((span(v.span), false)),
            Node::Strong(v) => self.emphasis.push((span(v.span), true)),
            Node::InlineCode(v) => self.inline_code.push(span(v.span)),
            Node::Link(v) => {
                let mut label = Label::default();
                label.visit_link(v);
                self.links.push(Link {
                    span: span(v.span),
                    url: v.url.into(),
                    label: label.0,
                    image: false,
                });
            }
            Node::Image(v) => self.links.push(Link {
                span: span(v.span),
                url: v.url.into(),
                label: v.alt.into(),
                image: true,
            }),
            Node::Table(v) => self.tables.push(span(v.span)),
            Node::Definition(v) => {
                self.definitions
                    .push(Definition { span: span(v.span), label: v.identifier.into() });
            }
            Node::Text(v) => self.text.push(span(v.span)),
            Node::Break(v) => self.breaks.push(span(v.span)),
            _ => {}
        }
    }
}

#[derive(Default)]
struct Label(CompactString);
impl<'a> Visit<'a> for Label {
    fn visit_text(&mut self, node: &ast::Text<'a>) {
        self.0.push_str(node.value);
    }
    fn visit_inline_code(&mut self, node: &ast::InlineCode<'a>) {
        self.0.push_str(node.value);
    }
    fn visit_image(&mut self, _: &ast::Image<'a>) {}
    fn visit_break(&mut self, _: &ast::Break) {
        self.0.push(' ');
    }
}
