use super::{source::Source, syntax::Syntax, *};

pub(super) struct MarkdownLintState {
    pub diagnostics: Vec<MarkdownLintDiagnostic>,
    pub masked_document: String,
}

pub(super) fn collect_markdown_lint_state(
    source: &str,
    options: &InternalMarkdownLintOptions,
    dictionary: &DictionaryBundle,
    include_mask: bool,
) -> MarkdownLintState {
    let source = Source::new(source);
    let mut syntax = Syntax::analyze(&source, options);
    let mut diagnostics = std::mem::take(&mut syntax.diagnostics);
    let mut masked_document =
        if include_mask { String::with_capacity(source.text.len()) } else { String::new() };
    let mut blank_streak = 0_u32;
    let mut offset = 0;
    let mut mask_buffer = String::new();
    let mut tokens = Vec::new();
    for (index, raw) in source.text.split_inclusive('\n').enumerate() {
        let line = raw.strip_suffix('\n').unwrap_or(raw);
        let line = line.strip_suffix('\r').unwrap_or(line);
        let line_number = index + 1;
        let skipped = syntax.skipped_at(offset + line.len().saturating_sub(1));
        if !skipped {
            let trailing = get_trailing_whitespace_length(line);
            let break_offset = offset + line.len();
            let hard_break = syntax
                .breaks
                .binary_search_by_key(&(break_offset as u32), |span| span.start)
                .is_ok();
            if options.rules.trailing_spaces && trailing > 0 {
                let start = offset + line.len() - trailing;
                let preserve = if hard_break && line.ends_with("  ") { 2 } else { 0 };
                if trailing > preserve {
                    let end = offset + line.len() - preserve;
                    let mut value = source.diagnostic(
                        "trailing-spaces",
                        "Trailing whitespace is not allowed.".into(),
                        start,
                        end,
                    );
                    value.fix = Some(MarkdownLintFix {
                        start: start as u32,
                        end: end as u32,
                        text: String::new(),
                    });
                    diagnostics.push(value);
                }
            }
            if line.trim().is_empty() {
                blank_streak += 1;
                if blank_streak > options.rules.max_consecutive_blank_lines {
                    let limit = options.rules.max_consecutive_blank_lines;
                    let mut value = source.diagnostic(
                        "max-consecutive-blank-lines",
                        format!(
                            "More than {limit} blank line{} in a row.",
                            if limit == 1 { "" } else { "s" }
                        ),
                        offset,
                        offset,
                    );
                    value.fix = Some(MarkdownLintFix {
                        start: offset as u32,
                        end: (offset + raw.len()) as u32,
                        text: String::new(),
                    });
                    diagnostics.push(value);
                }
            } else {
                blank_streak = 0;
            }
        } else {
            blank_streak = 0;
        }
        if skipped && !include_mask {
            offset += raw.len();
            continue;
        }
        let masked = syntax.mask_line(line, offset, &mut mask_buffer);
        if !skipped {
            if options.rules.repeated_punctuation {
                collect_repeated_punctuation_diagnostics(
                    line,
                    masked,
                    offset,
                    &source,
                    &mut diagnostics,
                );
            }
            if options.rules.spellcheck || options.rules.repeated_words {
                collect_word_diagnostics(
                    &source,
                    line,
                    line_number,
                    masked,
                    options,
                    dictionary,
                    &mut tokens,
                    &mut diagnostics,
                );
            }
        }
        if include_mask {
            let spellcheck = syntax.controls.spellcheck_enabled(line_number, options);
            // CSpell uses UTF-16 offsets; masking must preserve astral widths.
            for (original, visible) in line.chars().zip(masked.chars()) {
                if spellcheck && !skipped && visible != ' ' {
                    masked_document.push(visible);
                } else {
                    for _ in 0..original.len_utf16() {
                        masked_document.push(' ');
                    }
                }
            }
            if raw.ends_with("\r\n") {
                masked_document.push('\r');
            }
            if raw.ends_with('\n') {
                masked_document.push('\n');
            }
        }
        offset += raw.len();
    }
    if options.rules.structure.final_newline.unwrap_or(false)
        && !source.text.is_empty()
        && !source.text.ends_with('\n')
    {
        let end = source.text.len();
        let mut value = source.diagnostic(
            "final-newline",
            "Document should end with a newline.".into(),
            end,
            end,
        );
        let newline = if source.text.contains("\r\n") { "\r\n" } else { "\n" };
        value.fix =
            Some(MarkdownLintFix { start: end as u32, end: end as u32, text: newline.into() });
        diagnostics.push(value);
    }
    prose::collect(&source, &syntax, options, &mut diagnostics);
    syntax.controls.apply(&mut diagnostics, options);
    MarkdownLintState { diagnostics, masked_document }
}

fn collect_word_diagnostics(
    source: &Source<'_>,
    line: &str,
    line_number: usize,
    masked: &str,
    options: &InternalMarkdownLintOptions,
    dictionary: &DictionaryBundle,
    tokens: &mut Vec<Token>,
    output: &mut Vec<MarkdownLintDiagnostic>,
) {
    collect_tokens(masked, options, dictionary, tokens);
    let mut previous: Option<(&Token, usize)> = None;
    let mut cursor = line.char_indices().peekable();
    let mut scalar = 0;
    let base = source.lines[line_number - 1];
    for token in tokens.iter() {
        while scalar < token.start {
            cursor.next();
            scalar += 1;
        }
        let start = base + cursor.peek().map_or(line.len(), |(offset, _)| *offset);
        while scalar < token.end {
            cursor.next();
            scalar += 1;
        }
        let end = base + cursor.peek().map_or(line.len(), |(offset, _)| *offset);
        if options.rules.repeated_words && !should_ignore_repeated_word_token(token) {
            if let Some((prior, prior_end)) = previous {
                // Only actual whitespace separates duplicates. Masked syntax
                // and punctuation are not separators for this rule.
                let separator = &source.text[prior_end..start];
                if token.start > prior.end
                    && separator.chars().all(char::is_whitespace)
                    && comparable_words_equal(&prior.text, &token.text)
                {
                    let mut value = source.diagnostic(
                        "repeated-word",
                        format!("Repeated word \"{}\" looks accidental.", token.text),
                        start,
                        end,
                    );
                    value.language = Some(token.language.to_string());
                    let delete_start = prior_end;
                    value.fix = Some(MarkdownLintFix {
                        start: delete_start as u32,
                        end: end as u32,
                        text: String::new(),
                    });
                    output.push(value);
                }
            }
            previous = Some((token, end));
        }
        if options.rules.spellcheck
            && should_spellcheck_token(token, dictionary)
            && !is_known_token(token, dictionary)
        {
            let suggestions = if token.language == "ja" || token.language == "zh" {
                None
            } else {
                let values = suggest_latin_words(&token.text, &dictionary.latin_suggestion_words);
                if values.is_empty() { None } else { Some(values) }
            };
            let mut value = source.diagnostic(
                "spellcheck",
                format!("Unknown {} word \"{}\".", token.language, token.text),
                start,
                end,
            );
            value.language = Some(token.language.to_string());
            value.suggestions = suggestions;
            output.push(value);
        }
    }
}

fn comparable_words_equal(left: &str, right: &str) -> bool {
    if left.is_ascii() && right.is_ascii() {
        left.eq_ignore_ascii_case(right)
    } else {
        normalize_comparable_word(left) == normalize_comparable_word(right)
    }
}
