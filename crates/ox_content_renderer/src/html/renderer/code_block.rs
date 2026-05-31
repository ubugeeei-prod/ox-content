//! Code block rendering and line annotation output.
//!
//! Metadata parsing is handled by `html::code_annotations`; this module decides how the
//! resulting line states become `<pre><code>` attributes, wrapper spans, line numbers,
//! and compatibility classes.

use ox_content_ast::CodeBlock;

use super::super::code_annotations::{
    apply_annotation_numbers, apply_btree_annotations, normalize_code_block_info,
    parse_code_annotations, parse_line_numbers, parse_vitepress_inline_annotations,
    split_code_block_meta, CodeAnnotationKind, CodeBlockRenderState, CodeLineRenderState,
    MetaTokenKind,
};
use super::HtmlRenderer;

impl HtmlRenderer {
    pub(in crate::html::renderer) fn build_code_block_state(
        &self,
        code_block: &CodeBlock<'_>,
    ) -> CodeBlockRenderState {
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
                    annotations: Vec::new(),
                })
                .collect()
        };

        let mut title = None;
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
                let annotations =
                    parse_code_annotations(&info.meta, &self.options.code_annotation_meta_key);
                apply_btree_annotations(&mut lines, &annotations);
            }

            if syntax.includes_vitepress() {
                for token in split_code_block_meta(&info.meta) {
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
                                title = Some(token.value.trim().to_string());
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
                            }
                        }
                    }
                }
            }
        }

        CodeBlockRenderState { language: info.language, title, line_numbers_start, lines }
    }

    pub(in crate::html::renderer) fn write_code_lines(&mut self, state: &CodeBlockRenderState) {
        let has_focus = state.has_focus();

        for (index, line) in state.lines.iter().enumerate() {
            let line_number = index + 1;
            let mut class_names: Vec<&str> = vec!["line", "ox-code-line"];

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
