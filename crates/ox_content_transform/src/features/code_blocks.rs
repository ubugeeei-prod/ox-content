#![allow(clippy::redundant_pub_crate)]

use compact_str::CompactString;
use rustc_hash::FxHashSet;

use crate::{CodeBlockLintOptions, DocsTestOptions};

use super::segments::{is_closing_fence, parse_opening_fence};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtractedCodeBlock {
    pub language: String,
    pub meta: String,
    pub code: String,
    pub start_line: u32,
    pub end_line: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodeBlockDiagnostic {
    pub rule_id: String,
    pub severity: String,
    pub message: String,
    pub line: u32,
    pub column: u32,
    pub end_line: u32,
    pub end_column: u32,
    pub language: Option<String>,
}

pub fn extract_code_blocks(source: &str) -> Vec<ExtractedCodeBlock> {
    let mut blocks = Vec::new();
    let mut in_fence = false;
    let mut fence_char = b'\0';
    let mut fence_len = 0usize;
    let mut language = CompactString::default();
    let mut meta = CompactString::default();
    let mut code = String::new();
    let mut start_line = 0u32;
    let mut line_number = 0u32;

    for line in source.lines() {
        line_number += 1;
        if in_fence {
            if is_closing_fence(line, fence_char, fence_len) {
                blocks.push(ExtractedCodeBlock {
                    language: std::mem::take(&mut language).into_string(),
                    meta: std::mem::take(&mut meta).into_string(),
                    code: String::from(code.trim_end_matches('\n')),
                    start_line,
                    end_line: line_number.saturating_sub(1),
                });
                in_fence = false;
                fence_char = b'\0';
                fence_len = 0;
                code.clear();
            } else {
                code.push_str(line);
                code.push('\n');
            }
            continue;
        }

        if let Some(open) = parse_opening_fence(line) {
            in_fence = true;
            fence_char = open.fence_char;
            fence_len = open.fence_len;
            language = open.language;
            meta = open.meta;
            start_line = line_number + 1;
        }
    }

    blocks
}

pub fn lint_code_blocks(
    source: &str,
    options: Option<&CodeBlockLintOptions>,
) -> Vec<CodeBlockDiagnostic> {
    let Some(options) = options else {
        return Vec::new();
    };
    if options.enabled == Some(false) {
        return Vec::new();
    }

    let languages = options.languages.as_ref().map(|values| {
        values.iter().map(|value| value.to_ascii_lowercase()).collect::<FxHashSet<_>>()
    });
    let require_language = options.require_language.unwrap_or(false);
    let trailing_spaces = options.trailing_spaces.unwrap_or(true);
    let mut diagnostics = Vec::new();

    for block in extract_code_blocks(source) {
        if require_language && block.language.is_empty() {
            diagnostics.push(CodeBlockDiagnostic {
                rule_id: "code-block-language".to_string(),
                severity: "warning".to_string(),
                message: "Code block is missing a language identifier.".to_string(),
                line: block.start_line.saturating_sub(1),
                column: 1,
                end_line: block.start_line.saturating_sub(1),
                end_column: 1,
                language: None,
            });
        }

        if let Some(languages) = &languages {
            if !block.language.is_empty()
                && !languages.contains(&block.language.to_ascii_lowercase())
            {
                continue;
            }
        }

        if trailing_spaces {
            for (index, line) in block.code.lines().enumerate() {
                let trimmed = line.trim_end_matches([' ', '\t']);
                if trimmed.len() != line.len() {
                    diagnostics.push(CodeBlockDiagnostic {
                        rule_id: "code-block-trailing-spaces".to_string(),
                        severity: "warning".to_string(),
                        message: "Code block line has trailing whitespace.".to_string(),
                        line: block.start_line + index as u32,
                        column: (trimmed.len() + 1) as u32,
                        end_line: block.start_line + index as u32,
                        end_column: (line.len() + 1) as u32,
                        language: if block.language.is_empty() {
                            None
                        } else {
                            Some(block.language.clone())
                        },
                    });
                }
            }
        }
    }

    diagnostics
}

pub fn extract_docs_tests(
    source: &str,
    options: Option<&DocsTestOptions>,
) -> Vec<ExtractedCodeBlock> {
    let Some(options) = options else {
        return Vec::new();
    };
    if options.enabled == Some(false) {
        return Vec::new();
    }

    let languages = options.languages.as_ref().map_or_else(
        || ["js", "jsx", "ts", "tsx", "mjs", "mts"].into_iter().map(ToString::to_string).collect(),
        |values| values.iter().map(|value| value.to_ascii_lowercase()).collect::<FxHashSet<_>>(),
    );
    let require_meta = options.require_meta.unwrap_or(true);

    extract_code_blocks(source)
        .into_iter()
        .filter(|block| languages.contains(&block.language.to_ascii_lowercase()))
        .filter(|block| {
            !require_meta
                || block.meta.split_whitespace().any(|token| {
                    matches!(token, "test" | "runnable" | "vitest" | "docs-test")
                        || token.starts_with("test=")
                })
        })
        .collect()
}
