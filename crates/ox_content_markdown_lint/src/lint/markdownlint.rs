mod blocks;
mod catalog;
mod config;
mod controls;
mod emphasis;
mod headings;
mod inline;
mod lists;
mod references;
mod syntax;
mod tables;
mod whitespace;

pub use catalog::MarkdownlintRule;
pub use config::MarkdownlintConfig;

/// Metadata for the supported markdownlint 0.41.1 rule set.
pub fn markdownlint_rules() -> &'static [MarkdownlintRule] {
    catalog::RULES
}
pub(super) use config::Settings;
pub(super) use syntax::Document;

use super::{MarkdownLintDiagnostic, MarkdownLintFix, source::Source};
use ox_content_ast::Span;

pub(super) fn collect(
    source: &Source<'_>,
    document: &Document,
    settings: &Settings,
    output: &mut Vec<MarkdownLintDiagnostic>,
) {
    let mut context = Context { source, document, settings, output };
    controls::run(&mut context);
}

fn rules(context: &mut Context<'_, '_>) {
    headings::check(context);
    blocks::check(context);
    lists::check(context);
    whitespace::check(context);
    inline::check(context);
    references::check(context);
    tables::check(context);
}

struct Context<'a, 's> {
    source: &'a Source<'s>,
    document: &'a Document,
    settings: &'a Settings,
    output: &'a mut Vec<MarkdownLintDiagnostic>,
}

impl<'a, 's> Context<'a, 's> {
    fn report(&mut self, number: usize, span: Span) -> Option<&mut MarkdownLintDiagnostic> {
        if !self.settings.on(number) {
            return None;
        }
        let rule = catalog::RULES.iter().find(|rule| rule.number == number)?;
        let mut value = self.source.diagnostic(
            rule.id,
            rule.description.into(),
            span.start as usize,
            span.end as usize,
        );
        value.severity.clear();
        value.severity.push_str(self.settings.severity(number));
        self.output.push(value);
        self.output.last_mut()
    }
    fn replace(&mut self, rule: usize, span: Span, edit: Span, text: &str) {
        if let Some(value) = self.report(rule, span) {
            value.fix =
                Some(MarkdownLintFix { start: edit.start, end: edit.end, text: text.into() });
        }
    }
    fn line(&self, index: usize) -> &'s str {
        let start = self.source.lines.get(index).copied().unwrap_or(self.source.text.len());
        let end = self.source.lines.get(index + 1).copied().unwrap_or(self.source.text.len());
        self.source.text[start..end].trim_end_matches(['\n', '\r'])
    }
    fn line_span(&self, index: usize) -> Span {
        let start = self.source.lines.get(index).copied().unwrap_or(self.source.text.len());
        Span::new(start as u32, (start + self.line(index).len()) as u32)
    }
    fn line_count(&self) -> usize {
        self.source.lines.len() - usize::from(self.source.text.ends_with('\n'))
    }
    fn blank(&self, index: usize) -> bool {
        self.line(index).trim().trim_start_matches('>').trim().is_empty()
    }
    fn code_at(&self, offset: usize) -> Option<&'a syntax::Code> {
        let index = self.document.codes.partition_point(|code| code.span.start as usize <= offset);
        index
            .checked_sub(1)
            .and_then(|i| self.document.codes.get(i))
            .filter(|code| offset < code.span.end as usize)
    }
    fn raw(&self, span: Span) -> &'s str {
        &self.source.text[span.start as usize..span.end as usize]
    }
    fn new_line(&self) -> &'static str {
        if self.source.text.contains("\r\n") { "\r\n" } else { "\n" }
    }
    fn around(&mut self, rule: usize, span: Span, above: usize, below: usize) {
        let first = self.source.line_index(span.start as usize);
        let last = self.source.line_index(span.end.saturating_sub(1) as usize);
        if first > 0 && (1..=above).any(|n| first >= n && !self.blank(first - n)) {
            self.report(rule, self.line_span(first));
        }
        if last + 1 < self.line_count()
            && (1..=below).any(|n| last + n < self.line_count() && !self.blank(last + n))
        {
            self.report(rule, self.line_span(last));
        }
    }
}

fn prefix(text: &str) -> &str {
    let mut text = text.trim_start_matches([' ', '\t']);
    while let Some(rest) = text.strip_prefix('>') {
        text = rest.strip_prefix(' ').unwrap_or(rest).trim_start_matches([' ', '\t']);
    }
    text
}

fn contains(spans: &[Span], position: u32) -> bool {
    at(spans, position, |&span| span).is_some()
}

fn at<T>(values: &[T], position: u32, span: impl Fn(&T) -> Span) -> Option<&T> {
    let index = values.partition_point(|value| span(value).start <= position);
    index
        .checked_sub(1)
        .and_then(|index| values.get(index))
        .filter(|value| position < span(value).end)
}
