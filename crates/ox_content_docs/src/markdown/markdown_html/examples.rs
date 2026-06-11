use super::super::{parse_example_block, ExampleBlock};
use super::blocks::render_markdown_blocks_html;
use super::inline::render_code_block_html;
use crate::string_builder::StringBuilder;

pub(in crate::markdown) fn render_module_examples_html(examples: &[String]) -> String {
    if examples.is_empty() {
        return String::new();
    }

    let mut examples_html = StringBuilder::new();
    for (index, example) in examples.iter().enumerate() {
        if !examples_html.is_empty() {
            examples_html.push_char('\n');
        }
        // Mixed Markdown examples (prose + fenced code) render as HTML blocks; a
        // pure-code example renders as a single highlighted code block. This avoids
        // double-wrapping a Markdown example in another `<pre><code>`.
        let rendered = match parse_example_block(example) {
            ExampleBlock::Code { code, language } => render_code_block_html(code, language),
            ExampleBlock::Markdown(markdown) => render_markdown_blocks_html(markdown),
        };
        examples_html.push_str(
            "<div class=\"ox-api-entry__example\">
<div class=\"ox-api-entry__example-heading\">Example ",
        );
        examples_html.push_usize(index + 1);
        examples_html.push_str(
            "</div>
",
        );
        examples_html.push_str(&rendered);
        examples_html.push_str(
            "
</div>",
        );
    }

    let heading = if examples.len() == 1 { "Example" } else { "Examples" };
    let mut section = StringBuilder::new();
    section.push_str(
        "<div class=\"ox-api-entry__section ox-api-entry__section--examples\">
<h2>",
    );
    section.push_str(heading);
    section.push_str(
        "</h2>
",
    );
    section.push_str(&examples_html.into_string());
    section.push_str(
        "
</div>

",
    );
    section.into_string()
}

/// Appends the examples section, or nothing when empty.
pub(super) fn push_examples_html(body: &mut String, examples: &[String]) {
    if examples.is_empty() {
        return;
    }
    let mut examples_html = StringBuilder::new();
    for (index, example) in examples.iter().enumerate() {
        if !examples_html.is_empty() {
            examples_html.push_char('\n');
        }
        // Mixed Markdown examples (prose + fenced code) render as HTML blocks; a
        // pure-code example renders as a single highlighted code block. This avoids
        // double-wrapping a Markdown example in another `<pre><code>`.
        let rendered = match parse_example_block(example) {
            ExampleBlock::Code { code, language } => render_code_block_html(code, language),
            ExampleBlock::Markdown(markdown) => render_markdown_blocks_html(markdown),
        };
        examples_html.push_str(
            "<div class=\"ox-api-entry__example\">
<div class=\"ox-api-entry__example-heading\">Example ",
        );
        examples_html.push_usize(index + 1);
        examples_html.push_str(
            "</div>
",
        );
        examples_html.push_str(&rendered);
        examples_html.push_str(
            "
</div>",
        );
    }

    body.push_str(
        "<div class=\"ox-api-entry__section ox-api-entry__section--examples\">
<h4>Examples</h4>
",
    );
    body.push_str(&examples_html.into_string());
    body.push_str(
        "
</div>\n",
    );
}
