use super::{
    InternalMarkdownLintOptions, MarkdownLintDiagnostic, MarkdownLintFix, source::Source,
    syntax::Syntax,
};

pub(super) fn collect(
    source: &Source<'_>,
    syntax: &Syntax,
    options: &InternalMarkdownLintOptions,
    output: &mut Vec<MarkdownLintDiagnostic>,
) {
    let rules = &options.text_rules;
    let lengths = rules.sentence_length.is_some() || rules.max_ten.is_some();
    let symbols = rules.no_exclamation_question_mark.unwrap_or(false);
    let todo = rules.no_todo.unwrap_or(false);
    if !lengths && !symbols && !todo && options.terminology.is_none() {
        return;
    }

    // One forward cursor per paragraph; no sentence strings or token allocations.
    if lengths {
        for block in &syntax.blocks {
            let mut visible = syntax.visible.partition_point(|v| v.end <= block.start);
            let mut hidden = syntax.hidden.partition_point(|v| v.end <= block.start);
            let mut sentence = Sentence::default();
            let mut quote = 0_u32;
            let mut previous = '\0';
            let text = &source.text[block.start as usize..block.end as usize];
            let mut chars = text.char_indices().peekable();
            while let Some((offset, c)) = chars.next() {
                let offset = block.start as usize + offset;
                while syntax.hidden.get(hidden).is_some_and(|v| v.end as usize <= offset) {
                    hidden += 1;
                }
                if syntax.hidden.get(hidden).is_some_and(|v| v.start as usize <= offset) {
                    continue;
                }
                while syntax.visible.get(visible).is_some_and(|v| v.end as usize <= offset) {
                    visible += 1;
                }
                let is_visible =
                    syntax.visible.get(visible).is_some_and(|v| v.start as usize <= offset);
                if !is_visible && !c.is_whitespace() {
                    continue;
                }
                if sentence.start.is_none() && c.is_whitespace() {
                    continue;
                }
                sentence.start.get_or_insert(offset);
                sentence.length += 1;
                if !c.is_whitespace() {
                    sentence.end = offset + c.len_utf8();
                    sentence.trimmed_length = sentence.length;
                }
                if c == '、' {
                    sentence.commas += 1;
                }
                match c {
                    '「' | '『' | '（' | '(' => quote = quote.saturating_add(1),
                    '」' | '』' | '）' | ')' => quote = quote.saturating_sub(1),
                    _ => {}
                }
                let decimal = c == '.'
                    && previous.is_ascii_digit()
                    && chars.peek().is_some_and(|(_, next)| next.is_ascii_digit());
                if quote == 0 && matches!(c, '。' | '.' | '!' | '?' | '！' | '？') && !decimal {
                    sentence.report(source, options, output);
                    sentence = Sentence::default();
                }
                previous = c;
            }
            sentence.report(source, options, output);
        }
    }

    for span in &syntax.visible {
        let start = span.start as usize;
        let text = &source.text[start..span.end as usize];
        if symbols {
            for (offset, c) in
                text.char_indices().filter(|(_, c)| matches!(c, '!' | '?' | '！' | '？'))
            {
                output.push(source.diagnostic(
                    "no-exclamation-question-mark",
                    "Avoid exclamation and question marks in formal prose.".into(),
                    start + offset,
                    start + offset + c.len_utf8(),
                ));
            }
        }
        if todo {
            for (offset, _) in text.match_indices("TODO") {
                if word_boundary(text, offset, offset + 4) {
                    output.push(source.diagnostic(
                        "no-todo",
                        "Resolve this TODO before publishing.".into(),
                        start + offset,
                        start + offset + 4,
                    ));
                }
            }
        }
        if let (Some(matcher), Some(terms)) = (&options.terminology, &rules.terminology) {
            for matched in matcher.find_iter(text) {
                let term = &terms[matched.pattern().as_usize()];
                if term.term.is_ascii() && !word_boundary(text, matched.start(), matched.end()) {
                    continue;
                }
                let mut value = source.diagnostic(
                    "terminology",
                    format!("Use \"{}\" instead of \"{}\".", term.replacement, term.term),
                    start + matched.start(),
                    start + matched.end(),
                );
                value.suggestions = Some(vec![term.replacement.clone()]);
                // Replacement must remain literal prose and cannot create Markdown syntax.
                if safe_replacement(&term.replacement) {
                    value.fix = Some(MarkdownLintFix {
                        start: (start + matched.start()) as u32,
                        end: (start + matched.end()) as u32,
                        text: term.replacement.clone(),
                    });
                }
                output.push(value);
            }
        }
    }
}

fn word_boundary(text: &str, start: usize, end: usize) -> bool {
    let is_word = |c: char| c.is_ascii_alphanumeric() || c == '_';
    !text[..start].chars().next_back().is_some_and(is_word)
        && !text[end..].chars().next().is_some_and(is_word)
}

#[derive(Default)]
struct Sentence {
    start: Option<usize>,
    end: usize,
    length: u32,
    trimmed_length: u32,
    commas: u32,
}

impl Sentence {
    fn report(
        &self,
        source: &Source<'_>,
        options: &InternalMarkdownLintOptions,
        output: &mut Vec<MarkdownLintDiagnostic>,
    ) {
        let Some(start) = self.start else {
            return;
        };
        if let Some(max) = options.text_rules.sentence_length
            && self.trimmed_length > max
        {
            output.push(source.diagnostic(
                "sentence-length",
                format!("Sentence has {} characters; maximum is {max}.", self.trimmed_length),
                start,
                self.end,
            ));
        }
        if let Some(max) = options.text_rules.max_ten
            && self.commas > max
        {
            output.push(source.diagnostic(
                "max-ten",
                format!("Sentence has {} Japanese commas; maximum is {max}.", self.commas),
                start,
                self.end,
            ));
        }
    }
}

fn safe_replacement(text: &str) -> bool {
    if text.is_empty()
        || text.trim() != text
        || text.contains([
            '\n', '\r', '\t', '`', '[', ']', '*', '_', '<', '>', '{', '}', '\\', '#', '|', '!',
            '~', '&',
        ])
    {
        return false;
    }
    if text.chars().all(|c| matches!(c, '-' | '=' | ' ')) {
        return false;
    }
    let after_digits = text.trim_start_matches(|c: char| c.is_ascii_digit());
    if after_digits.len() < text.len()
        && (after_digits.starts_with(". ") || after_digits.starts_with(") "))
    {
        return false;
    }
    !text.starts_with("- ") && !text.starts_with("+ ")
}
