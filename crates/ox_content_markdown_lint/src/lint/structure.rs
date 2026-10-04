use super::{
    FxHashMap, InternalMarkdownLintRules, MarkdownLintDiagnostic, normalize_latin_word,
    source::Source,
};
use ox_content_ast::{CodeBlock, Heading, Image, InlineCode, Link, Span, Text, Visit};

pub(super) struct Structure<'r, 's> {
    source: &'r Source<'s>,
    rules: &'r InternalMarkdownLintRules,
    seen: FxHashMap<String, usize>,
    previous: u8,
    h1: bool,
}

impl<'r, 's> Structure<'r, 's> {
    pub fn new(source: &'r Source<'s>, rules: &'r InternalMarkdownLintRules) -> Self {
        Self { source, rules, seen: FxHashMap::default(), previous: 0, h1: false }
    }

    fn report(
        &self,
        output: &mut Vec<MarkdownLintDiagnostic>,
        rule: &str,
        message: String,
        span: Span,
    ) {
        output.push(self.source.diagnostic(rule, message, span.start as usize, span.end as usize));
    }

    pub fn heading(
        &mut self,
        node: &Heading<'_>,
        span: Span,
        output: &mut Vec<MarkdownLintDiagnostic>,
    ) {
        let mut label = Label(super::CompactString::default());
        label.visit_heading(node);
        let label = normalize_latin_word(&super::collapse_whitespace(&label.0));
        let rules = &self.rules.structure;
        if label.is_empty() && rules.empty_headings.unwrap_or(true) {
            self.report(output, "empty-heading", "Heading must contain visible text.".into(), span);
        }
        if self.previous == 0 && node.depth != 1 && rules.first_heading_h1.unwrap_or(false) {
            self.report(output, "first-heading-h1", "The first heading should be h1.".into(), span);
        }
        if node.depth == 1 {
            if self.h1 && rules.single_h1.unwrap_or(false) {
                self.report(
                    output,
                    "single-h1",
                    "Document contains more than one h1 heading.".into(),
                    span,
                );
            }
            self.h1 = true;
        }
        if self.rules.heading_increment && self.previous > 0 && node.depth > self.previous + 1 {
            let start = span.start as usize;
            let line_end =
                self.source.text[start..].find('\n').map_or(span.end as usize, |i| start + i);
            let end = if self.source.text[start..].starts_with('#') {
                start + usize::from(node.depth)
            } else {
                line_end
            };
            output.push(self.source.diagnostic(
                "heading-increment",
                format!("Heading depth jumps from h{} to h{}.", self.previous, node.depth),
                start,
                end,
            ));
        }
        self.previous = node.depth;
        if self.rules.duplicate_headings && !label.is_empty() {
            if let Some(first) = self.seen.get(&label) {
                self.report(
                    output,
                    "duplicate-heading",
                    format!("Heading text duplicates the heading on line {first}."),
                    span,
                );
            } else {
                self.seen.insert(label, self.source.position(span.start as usize).0);
            }
        }
    }

    pub fn code(&self, node: &CodeBlock<'_>, span: Span, output: &mut Vec<MarkdownLintDiagnostic>) {
        let raw = &self.source.text[span.start as usize..span.end as usize];
        let first = raw.lines().next().unwrap_or("").trim_start_matches([' ', '\t', '>']);
        let Some(character @ ('`' | '~')) = first.chars().next() else {
            return;
        };
        let length = first.bytes().take_while(|b| *b == character as u8).count();
        if length < 3 {
            return;
        }
        let opening =
            Span::new(span.start, span.start + raw.lines().next().unwrap_or("").len() as u32);
        if self.rules.structure.code_fence_language.unwrap_or(false)
            && node.lang.is_none_or(str::is_empty)
        {
            self.report(
                output,
                "code-fence-language",
                "Fenced code block should declare a language.".into(),
                opening,
            );
        }
        let last = raw.lines().last().unwrap_or("").trim_start_matches([' ', '\t', '>']);
        let closing = last.bytes().take_while(|b| *b == character as u8).count();
        let closed =
            raw.lines().count() > 1 && closing >= length && last[closing..].trim().is_empty();
        if self.rules.structure.code_fence_closed.unwrap_or(true) && !closed {
            self.report(
                output,
                "code-fence-closed",
                "Fenced code block is not closed.".into(),
                opening,
            );
        }
    }

    pub fn link(&self, node: &Link<'_>, span: Span, output: &mut Vec<MarkdownLintDiagnostic>) {
        if !self.rules.structure.empty_links.unwrap_or(true) {
            return;
        }
        let mut label = Label(super::CompactString::default());
        label.visit_link(node);
        if node.url.trim().is_empty() || label.0.trim().is_empty() {
            self.report(
                output,
                "empty-link",
                "Link must have a destination and visible label.".into(),
                span,
            );
        }
    }

    pub fn image(&self, node: &Image<'_>, span: Span, output: &mut Vec<MarkdownLintDiagnostic>) {
        if self.rules.structure.image_alt.unwrap_or(false) && node.alt.trim().is_empty() {
            self.report(
                output,
                "image-alt",
                "Image should have descriptive alternative text.".into(),
                span,
            );
        }
    }
}

struct Label(super::CompactString);
impl<'a> Visit<'a> for Label {
    fn visit_text(&mut self, node: &Text<'a>) {
        self.0.push_str(node.value);
    }
    fn visit_inline_code(&mut self, node: &InlineCode<'a>) {
        self.0.push_str(node.value);
    }
    fn visit_image(&mut self, node: &Image<'a>) {
        self.0.push_str(node.alt);
    }
    fn visit_break(&mut self, _: &ox_content_ast::Break) {
        self.0.push(' ');
    }
}
