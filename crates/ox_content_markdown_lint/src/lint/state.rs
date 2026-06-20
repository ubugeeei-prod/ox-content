use super::*;

pub(super) struct MarkdownLintState {
    pub(super) diagnostics: Vec<MarkdownLintDiagnostic>,
    pub(super) masked_lines: Vec<String>,
}

pub(super) fn collect_markdown_lint_state(
    source: &str,
    normalized_options: &InternalMarkdownLintOptions,
    dictionary: &DictionaryBundle,
) -> MarkdownLintState {
    let mut diagnostics = Vec::new();
    let mut masked_lines = Vec::new();
    let mut seen_headings = FxHashMap::default();

    let mut blank_line_streak = 0_u32;
    let mut html_comment_open = false;
    let mut in_fence = false;
    let mut fence_char = '\0';
    let mut fence_length = 0_usize;
    let mut frontmatter_open = false;
    let mut frontmatter_checked = false;
    let mut previous_heading_depth = 0_usize;

    for (index, raw_line) in source.split('\n').enumerate() {
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        let line_number = index + 1;
        let trimmed = line.trim();

        if !frontmatter_checked {
            frontmatter_checked = true;
            if trimmed == "---" {
                frontmatter_open = true;
                masked_lines.push(create_skipped_line_mask(line));
                continue;
            }
        }

        if frontmatter_open {
            if trimmed == "---" || trimmed == "..." {
                frontmatter_open = false;
            }
            masked_lines.push(create_skipped_line_mask(line));
            continue;
        }

        if html_comment_open {
            if line.contains("-->") {
                html_comment_open = false;
            }
            masked_lines.push(create_skipped_line_mask(line));
            continue;
        }

        if !in_fence && trimmed.starts_with("<!--") {
            if !trimmed.contains("-->") {
                html_comment_open = true;
            }
            masked_lines.push(create_skipped_line_mask(line));
            continue;
        }

        if in_fence {
            if is_fence_close(line, fence_char, fence_length) {
                in_fence = false;
                fence_char = '\0';
                fence_length = 0;
            }
            masked_lines.push(create_skipped_line_mask(line));
            continue;
        }

        if let Some(fence_pattern) = FENCE_PATTERN.as_ref() {
            if let Some(fence_match) = fence_pattern.find(line) {
                let fence = &line[fence_match.start()..fence_match.end()];
                in_fence = true;
                fence_char = fence.chars().next().unwrap_or('\0');
                fence_length = fence.chars().count();
                masked_lines.push(create_skipped_line_mask(line));
                continue;
            }
        }

        if normalized_options.rules.trailing_spaces {
            let trailing_length = get_trailing_whitespace_length(line);
            if trailing_length > 0 {
                let line_length = count_code_points(line);
                let start_column = line_length.saturating_sub(trailing_length) + 1;
                diagnostics.push(create_diagnostic(
                    "trailing-spaces",
                    "Trailing whitespace is not allowed.".to_string(),
                    line_number,
                    start_column,
                    line_length + 1,
                    None,
                    None,
                ));
            }
        }

        if trimmed.is_empty() {
            blank_line_streak += 1;
            if blank_line_streak > normalized_options.rules.max_consecutive_blank_lines {
                let limit = normalized_options.rules.max_consecutive_blank_lines;
                diagnostics.push(create_diagnostic(
                    "max-consecutive-blank-lines",
                    format!(
                        "More than {limit} blank line{} in a row.",
                        if limit == 1 { "" } else { "s" }
                    ),
                    line_number,
                    1,
                    1,
                    None,
                    None,
                ));
            }
            masked_lines.push(create_skipped_line_mask(line));
            continue;
        }

        blank_line_streak = 0;

        if let Some(captures) = HEADING_PATTERN.as_ref().and_then(|pattern| pattern.captures(line))
        {
            let depth = captures.get(1).map_or(0, |value| value.as_str().chars().count());
            let heading_text = captures
                .get(2)
                .map(|value| collapse_whitespace(&get_visible_text(value.as_str())))
                .unwrap_or_default();
            let normalized_heading = normalize_latin_word(&heading_text);

            if normalized_options.rules.heading_increment
                && previous_heading_depth > 0
                && depth > previous_heading_depth + 1
            {
                diagnostics.push(create_diagnostic(
                    "heading-increment",
                    format!("Heading depth jumps from h{previous_heading_depth} to h{depth}."),
                    line_number,
                    1,
                    depth + 1,
                    None,
                    None,
                ));
            }

            previous_heading_depth = depth;

            if normalized_options.rules.duplicate_headings && !normalized_heading.is_empty() {
                if let Some(first_seen_line) = seen_headings.get(&normalized_heading) {
                    diagnostics.push(create_diagnostic(
                        "duplicate-heading",
                        format!("Heading text duplicates the heading on line {first_seen_line}."),
                        line_number,
                        1,
                        count_code_points(line) + 1,
                        None,
                        None,
                    ));
                } else {
                    seen_headings.insert(normalized_heading, line_number);
                }
            }
        }

        if REFERENCE_DEFINITION_PATTERN.as_ref().is_some_and(|pattern| pattern.is_match(line))
            || is_indented_code_block_line(line)
        {
            masked_lines.push(create_skipped_line_mask(line));
            continue;
        }

        let masked_line = mask_markdown_line(line);
        masked_lines.push(masked_line.clone());

        if normalized_options.rules.repeated_punctuation {
            diagnostics.extend(collect_repeated_punctuation_diagnostics(line_number, &masked_line));
        }

        let tokens = collect_tokens(&masked_line, &normalized_options.languages, dictionary);

        if normalized_options.rules.repeated_words {
            let mut previous_comparable_token: Option<&Token> = None;

            for token in &tokens {
                if should_ignore_repeated_word_token(token) {
                    continue;
                }

                if let Some(previous_token) = previous_comparable_token {
                    if normalize_comparable_word(&previous_token.text)
                        == normalize_comparable_word(&token.text)
                    {
                        diagnostics.push(create_diagnostic(
                            "repeated-word",
                            format!("Repeated word \"{}\" looks accidental.", token.text),
                            line_number,
                            token.start + 1,
                            token.end + 1,
                            Some(token.language.clone()),
                            None,
                        ));
                    }
                }

                previous_comparable_token = Some(token);
            }
        }

        if normalized_options.rules.spellcheck {
            for token in &tokens {
                if !should_spellcheck_token(token, dictionary) || is_known_token(token, dictionary)
                {
                    continue;
                }

                let suggestions = if token.language == "ja" || token.language == "zh" {
                    None
                } else {
                    let values =
                        suggest_latin_words(&token.text, &dictionary.latin_suggestion_words);
                    if values.is_empty() {
                        None
                    } else {
                        Some(values)
                    }
                };

                diagnostics.push(create_diagnostic(
                    "spellcheck",
                    format!("Unknown {} word \"{}\".", token.language, token.text),
                    line_number,
                    token.start + 1,
                    token.end + 1,
                    Some(token.language.clone()),
                    suggestions,
                ));
            }
        }
    }

    MarkdownLintState { diagnostics, masked_lines }
}
