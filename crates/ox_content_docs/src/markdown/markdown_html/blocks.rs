use regex::Regex;
use std::sync::OnceLock;

use super::super::{cached_regex, RegexCache};
use super::inline::{render_code_block_html, render_inline_html};
use crate::string_builder::{join3, StringBuilder};

// The four block-start regexes are cached once and shared between the
// value-returning helpers, which need captures, and `is_markdown_block_start`,
// which only needs a boolean and therefore uses allocation-free `is_match`.
fn fence_re() -> Option<&'static Regex> {
    static FENCE_RE: RegexCache = OnceLock::new();
    cached_regex(&FENCE_RE, r"^```([\w-]+)?\s*$")
}

fn heading_re() -> Option<&'static Regex> {
    static HEADING_RE: RegexCache = OnceLock::new();
    cached_regex(&HEADING_RE, r"^(#{1,6})\s+(.*)$")
}

fn ordered_re() -> Option<&'static Regex> {
    static ORDERED_RE: RegexCache = OnceLock::new();
    cached_regex(&ORDERED_RE, r"^\d+\.\s+(.*)$")
}

fn unordered_re() -> Option<&'static Regex> {
    static UNORDERED_RE: RegexCache = OnceLock::new();
    cached_regex(&UNORDERED_RE, r"^[-*+]\s+(.*)$")
}

fn is_fence_start(line: &str) -> Option<String> {
    fence_re()?
        .captures(line.trim())
        .map(|captures| captures.get(1).map_or("text", |value| value.as_str()).to_string())
}

fn heading_match(line: &str) -> Option<(usize, String)> {
    heading_re()?.captures(line.trim()).map(|captures| {
        (
            captures.get(1).map_or(1, |value| value.as_str().len()).min(6),
            captures.get(2).map_or("", |value| value.as_str()).trim().to_string(),
        )
    })
}

fn ordered_list_item(line: &str) -> Option<String> {
    ordered_re()?
        .captures(line.trim())
        .and_then(|captures| captures.get(1).map(|value| value.as_str().to_string()))
}

fn unordered_list_item(line: &str) -> Option<String> {
    unordered_re()?
        .captures(line.trim())
        .and_then(|captures| captures.get(1).map(|value| value.as_str().to_string()))
}

fn is_markdown_block_start(line: &str) -> bool {
    let trimmed = line.trim();
    fence_re().is_some_and(|re| re.is_match(trimmed))
        || heading_re().is_some_and(|re| re.is_match(trimmed))
        || ordered_re().is_some_and(|re| re.is_match(trimmed))
        || unordered_re().is_some_and(|re| re.is_match(trimmed))
}

pub(super) fn render_markdown_blocks_html(text: &str) -> String {
    // This renderer handles the small Markdown subset embedded in generated
    // API descriptions. It walks the line slice once and emits blocks as soon
    // as they are recognized.
    static ORDERED_CONTINUATION_RE: RegexCache = OnceLock::new();
    static UNORDERED_CONTINUATION_RE: RegexCache = OnceLock::new();

    let lines: Vec<&str> =
        text.split('\n').map(|line| line.strip_suffix('\r').unwrap_or(line)).collect();
    let mut blocks = Vec::new();
    let mut index = 0;
    let ordered_continuation_re = cached_regex(&ORDERED_CONTINUATION_RE, r"^ {0,1}\d+\.\s+");
    let unordered_continuation_re = cached_regex(&UNORDERED_CONTINUATION_RE, r"^[-*+]\s+");

    while index < lines.len() {
        let line = lines[index];
        let trimmed = line.trim();

        if trimmed.is_empty() {
            index += 1;
            continue;
        }

        if let Some(language) = is_fence_start(line) {
            let mut code_lines = Vec::new();
            index += 1;

            while index < lines.len() && !lines[index].trim().starts_with("```") {
                code_lines.push(lines[index]);
                index += 1;
            }

            if index < lines.len() {
                index += 1;
            }

            blocks.push(render_code_block_html(&code_lines.join("\n"), &language));
            continue;
        }

        if let Some((level, content)) = heading_match(line) {
            let content = render_inline_html(&content);
            let mut block = StringBuilder::with_capacity(content.len() + 9);
            block.push_str("<h");
            block.push_usize(level);
            block.push_char('>');
            block.push_str(&content);
            block.push_str("</h");
            block.push_usize(level);
            block.push_char('>');
            blocks.push(block.into_string());
            index += 1;
            continue;
        }

        if let Some(first_item) = ordered_list_item(line) {
            let mut items = Vec::new();
            let mut current = Some(first_item);

            while index < lines.len() {
                let Some(item_text) = current.take().or_else(|| ordered_list_item(lines[index]))
                else {
                    break;
                };

                let mut item_lines = Vec::new();
                item_lines.push(String::from(item_text.trim()));
                index += 1;

                while index < lines.len() {
                    let continuation = lines[index];
                    let continuation_trimmed = continuation.trim();

                    if continuation_trimmed.is_empty()
                        || is_markdown_block_start(continuation)
                        || ordered_continuation_re
                            .is_some_and(|re| re.is_match(continuation_trimmed))
                    {
                        break;
                    }

                    item_lines.push(continuation_trimmed.to_string());
                    index += 1;
                }

                items.push(join3("<li>", &render_inline_html(&item_lines.join(" ")), "</li>"));

                if index < lines.len() && lines[index].trim().is_empty() {
                    break;
                }
            }

            blocks.push(join3("<ol>\n", &items.join("\n"), "\n</ol>"));
            continue;
        }

        if let Some(first_item) = unordered_list_item(line) {
            let mut items = Vec::new();
            let mut current = Some(first_item);

            while index < lines.len() {
                let Some(item_text) = current.take().or_else(|| unordered_list_item(lines[index]))
                else {
                    break;
                };

                let mut item_lines = Vec::new();
                item_lines.push(String::from(item_text.trim()));
                index += 1;

                while index < lines.len() {
                    let continuation = lines[index];
                    let continuation_trimmed = continuation.trim();

                    if continuation_trimmed.is_empty()
                        || is_markdown_block_start(continuation)
                        || unordered_continuation_re
                            .is_some_and(|re| re.is_match(continuation_trimmed))
                    {
                        break;
                    }

                    item_lines.push(continuation_trimmed.to_string());
                    index += 1;
                }

                items.push(join3("<li>", &render_inline_html(&item_lines.join(" ")), "</li>"));

                if index < lines.len() && lines[index].trim().is_empty() {
                    break;
                }
            }

            blocks.push(join3("<ul>\n", &items.join("\n"), "\n</ul>"));
            continue;
        }

        let mut paragraph_lines = Vec::new();
        paragraph_lines.push(String::from(trimmed));
        index += 1;

        while index < lines.len() {
            let next_line = lines[index];
            let next_trimmed = next_line.trim();

            if next_trimmed.is_empty() || is_markdown_block_start(next_line) {
                break;
            }

            paragraph_lines.push(next_trimmed.to_string());
            index += 1;
        }

        blocks.push(join3("<p>", &render_inline_html(&paragraph_lines.join(" ")), "</p>"));
    }

    join3("<div class=\"ox-api-entry__prose\">\n", &blocks.join("\n"), "\n</div>")
}
