//! Code block rendering and line annotation output.
//!
//! Metadata parsing is handled by `html::code_annotations`; this module decides how the
//! resulting line states become `<pre><code>` attributes, wrapper spans, line numbers,
//! and compatibility classes.

use compact_str::CompactString;
use ox_content_ast::CodeBlock;
use smallvec::SmallVec;

use super::super::code_annotations::{
    CodeAnnotationKind, CodeBlockRenderState, CodeLineRenderState, MetaTokenKind,
    apply_annotation_numbers, apply_btree_annotations, normalize_code_block_info,
    parse_code_annotations, parse_line_numbers, parse_vitepress_inline_annotations,
    split_code_block_meta,
};
use super::super::heading::slugify_heading_into;
use super::HtmlRenderer;

impl HtmlRenderer {
    /// Builds the normalized render state for one fenced code block.
    ///
    /// The renderer only pays the expensive annotation parsers when annotation
    /// output is enabled and the configured syntax can produce them. Plain code
    /// blocks become a simple line list, while VitePress inline annotations and
    /// meta annotations share the same `CodeLineRenderState` vector so later
    /// rendering walks each line once.
    pub(in crate::html::renderer) fn build_code_block_state(
        &self,
        code_block: &CodeBlock<'_>,
        block_index: usize,
    ) -> CodeBlockRenderState {
        crate::profile_span!("renderer::code_block_state");
        let info = normalize_code_block_info(code_block.lang, code_block.meta);
        let syntax = self.options.code_annotation_syntax;
        let mut lines = if self.options.code_annotations && syntax.includes_vitepress() {
            parse_vitepress_inline_annotations(code_block.value)
        } else {
            code_block
                .value
                .split('\n')
                .map(|line| CodeLineRenderState {
                    value: line.to_string(),
                    annotations: SmallVec::new(),
                })
                .collect()
        };

        let mut title = None;
        let mut line_links = false;
        let mut line_link_prefix = None;
        let mut wrap_lines = None;
        let mut line_numbers_start = if self.options.code_annotations
            && syntax.includes_vitepress()
            && self.options.code_annotation_default_line_numbers
        {
            Some(1)
        } else {
            None
        };

        if self.options.code_annotations && !info.meta.is_empty() {
            if syntax.includes_attribute() {
                let annotations = parse_code_annotations(
                    info.meta.as_str(),
                    self.options.code_annotation_meta_key(),
                );
                apply_btree_annotations(&mut lines, &annotations);
            }

            if syntax.includes_vitepress() {
                for token in split_code_block_meta(info.meta.as_str()) {
                    match token.kind {
                        MetaTokenKind::Braces => {
                            let line_numbers = parse_line_numbers(token.value);
                            apply_annotation_numbers(
                                &mut lines,
                                &line_numbers,
                                CodeAnnotationKind::Highlight,
                            );
                        }
                        MetaTokenKind::Brackets => {
                            if title.is_none() && !token.value.trim().is_empty() {
                                title = Some(CompactString::from(token.value.trim()));
                            }
                        }
                        MetaTokenKind::Raw => {
                            if token.value == ":line-numbers" {
                                line_numbers_start = Some(1);
                            } else if let Some(start) =
                                token.value.strip_prefix(":line-numbers=").and_then(|value| {
                                    value
                                        .trim()
                                        .parse::<usize>()
                                        .ok()
                                        .filter(|line_number| *line_number > 0)
                                })
                            {
                                line_numbers_start = Some(start);
                            } else if token.value == ":no-line-numbers" {
                                line_numbers_start = None;
                            } else if matches!(token.value, ":wrap" | ":wrap-lines") {
                                wrap_lines = Some(true);
                            } else if matches!(
                                token.value,
                                ":no-wrap" | ":nowrap" | ":no-wrap-lines"
                            ) {
                                wrap_lines = Some(false);
                            } else if token.value == ":line-links" {
                                line_links = true;
                            } else if let Some(prefix) =
                                token.value.strip_prefix(":line-links=").and_then(slug_meta_value)
                            {
                                line_link_prefix = Some(prefix);
                            }
                        }
                    }
                }
            }
        }

        if line_links && line_link_prefix.is_none() {
            line_link_prefix = Some(default_line_link_prefix(
                title.as_deref(),
                info.language.as_deref(),
                block_index,
            ));
        }

        let copy_source = if self.options.code_annotations
            && syntax.includes_vitepress()
            && !line_values_match_source(&lines, code_block.value)
        {
            Some(code_block.value.to_string())
        } else {
            None
        };

        CodeBlockRenderState {
            language: info.language,
            title,
            line_numbers_start,
            line_link_prefix,
            wrap_lines,
            copy_source,
            lines,
        }
    }

    /// Emits annotated code lines from precomputed render state.
    ///
    /// Class names use `SmallVec` because typical lines have only one or two
    /// classes, but focus/highlight/diff combinations can add a few more. This
    /// keeps the common case stack-backed while still preserving de-duplication
    /// when multiple annotations imply the same class.
    pub(in crate::html::renderer) fn write_code_lines(&mut self, state: &CodeBlockRenderState) {
        crate::profile_span!("renderer::write_code_lines");
        let has_focus = state.has_focus();

        for (index, line) in state.lines.iter().enumerate() {
            let line_number = index + 1;
            let mut class_names: SmallVec<[&str; 8]> = SmallVec::new();
            class_names.push("line");
            class_names.push("ox-code-line");

            for annotation in &line.annotations {
                let class_name = annotation.class_name();
                if !class_names.contains(&class_name) {
                    class_names.push(class_name);
                }
                for extra_class_name in annotation.extra_class_names() {
                    if !class_names.contains(extra_class_name) {
                        class_names.push(extra_class_name);
                    }
                }
            }

            if has_focus && !line.annotations.contains(&CodeAnnotationKind::Focus) {
                class_names.push("ox-code-line--dimmed");
            }

            self.write("<span class=\"");
            self.write(&class_names.join(" "));
            self.write("\" data-line=\"");
            self.write_display(line_number);
            self.write("\"");

            let visible_line_number =
                state.line_numbers_start.map_or(line_number, |start| start + index);
            if let Some(prefix) = state.line_link_prefix.as_deref() {
                self.write(" id=\"");
                self.write_attribute_escaped(prefix);
                self.write("-L");
                self.write_display(visible_line_number);
                self.write("\" data-line-anchor=\"");
                self.write_attribute_escaped(prefix);
                self.write("-L");
                self.write_display(visible_line_number);
                self.write("\"");
            }

            if let Some(start) = state.line_numbers_start {
                self.write(" data-line-number=\"");
                self.write_display(start + index);
                self.write("\"");
            }

            self.write(">");
            self.write_escaped(&line.value);
            self.write("</span>");

            if index + 1 < state.lines.len() {
                self.write("\n");
            }
        }
    }
}

fn slug_meta_value(value: &str) -> Option<CompactString> {
    let value = unquote_meta_value(value);
    let mut slug = String::with_capacity(value.len());
    slugify_heading_into(value, &mut slug);
    (slug != "section").then(|| CompactString::from(slug))
}

fn unquote_meta_value(value: &str) -> &str {
    let value = value.trim();
    if value.len() >= 2 {
        let bytes = value.as_bytes();
        if matches!(
            (bytes.first(), bytes.last()),
            (Some(b'"'), Some(b'"')) | (Some(b'\''), Some(b'\''))
        ) {
            return &value[1..value.len() - 1];
        }
    }
    value
}

fn default_line_link_prefix(
    title: Option<&str>,
    language: Option<&str>,
    block_index: usize,
) -> CompactString {
    let title = title.filter(|value| !value.trim().is_empty());
    let mut source = String::with_capacity(title.map_or(32, |title| title.len() + 5));
    source.push_str("code-");
    if let Some(title) = title {
        source.push_str(title);
    } else {
        source.push_str(language.unwrap_or("block"));
        source.push('-');
        push_usize(block_index, &mut source);
    }

    let mut slug = String::with_capacity(source.len());
    slugify_heading_into(&source, &mut slug);
    CompactString::from(slug)
}

fn push_usize(mut value: usize, output: &mut String) {
    let mut digits = [0_u8; 20];
    let mut index = digits.len();
    loop {
        index -= 1;
        digits[index] = b'0' + (value % 10) as u8;
        value /= 10;
        if value == 0 {
            break;
        }
    }
    for digit in &digits[index..] {
        output.push(char::from(*digit));
    }
}

fn line_values_match_source(lines: &[CodeLineRenderState], source: &str) -> bool {
    let mut source_lines = source.split('\n');
    for line in lines {
        if source_lines.next() != Some(line.value.as_str()) {
            return false;
        }
    }
    source_lines.next().is_none()
}
