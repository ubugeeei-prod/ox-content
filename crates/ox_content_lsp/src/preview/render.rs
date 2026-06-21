use ox_content_allocator::Allocator;
use ox_content_parser::{ParseError, Parser, ParserOptions};
use ox_content_renderer::HtmlRenderer;
use serde::{Deserialize, Serialize};

use crate::document::TextDocumentState;
use crate::frontmatter::parse_frontmatter;
use crate::preview::html::wrap_preview_html;
use crate::preview::text::preview_title;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PreviewPayload {
    pub html: String,
    pub title: String,
}

pub fn render_preview(source: &str) -> Result<PreviewPayload, ParseError> {
    let document = TextDocumentState::new(source.to_string());
    let frontmatter = parse_frontmatter(&document);
    let block = frontmatter.block;
    // Render the markdown body that follows the frontmatter, not the YAML
    // inside it. `block_end_offset` points just past the closing `---` line.
    let content = block.as_ref().map_or(source, |block| &source[block.block_end_offset..]);

    // Preview rendering is latency-sensitive and reparses the current document
    // often. Use the same source-length arena heuristic as NAPI so editor
    // previews do not benchmark a zero-capacity allocator.
    let allocator = Allocator::for_source_len(content.len());
    let parser = Parser::with_options(&allocator, content, ParserOptions::gfm());
    let ast = parser.parse()?;
    let mut renderer = HtmlRenderer::new();
    let body = renderer.render(&ast);
    let title = preview_title(block.as_ref(), &ast.children)
        .unwrap_or_else(|| "Ox Content Preview".to_string());

    Ok(PreviewPayload { title: title.clone(), html: wrap_preview_html(&title, &body) })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(name: &str, payload: &PreviewPayload) {
        let serialized = format!("title: {}\n---\n{}", payload.title, payload.html);
        insta::with_settings!({
            snapshot_path => "snapshots",
            prepend_module_to_snapshot => false,
            omit_expression => true,
        }, {
            insta::assert_snapshot!(name, serialized);
        });
    }

    #[test]
    fn render_plain_markdown_uses_first_heading_as_title() {
        let payload = render_preview("# Hello World\n\nA paragraph.\n").unwrap();
        snap("render_plain_markdown_uses_first_heading_as_title", &payload);
    }

    #[test]
    fn render_with_frontmatter_uses_frontmatter_title() {
        let source =
            "---\ntitle: From Frontmatter\nlayout: doc\n---\n\n# Heading Should Not Win\n\nbody\n";
        let payload = render_preview(source).unwrap();
        snap("render_with_frontmatter_uses_frontmatter_title", &payload);
    }

    #[test]
    fn render_without_heading_or_frontmatter_falls_back_to_default_title() {
        let payload = render_preview("just a paragraph\n").unwrap();
        snap("render_without_heading_or_frontmatter_falls_back_to_default_title", &payload);
    }

    #[test]
    fn render_strips_frontmatter_from_body() {
        // The rendered HTML body must not contain the frontmatter block — it
        // should be stripped before parsing so editors never display the YAML.
        let payload = render_preview("---\ntitle: Hi\n---\n\n# Body Title\n").unwrap();
        snap("render_strips_frontmatter_from_body", &payload);
    }

    #[test]
    fn render_gfm_features_round_trip() {
        let source = concat!(
            "# Features\n",
            "\n",
            "- [x] task done\n",
            "- [ ] task pending\n",
            "\n",
            "| col | val |\n",
            "| :-- | --: |\n",
            "| a   | 1   |\n",
            "\n",
            "~~strike~~ and `code` and **bold**.\n",
        );
        let payload = render_preview(source).unwrap();
        snap("render_gfm_features_round_trip", &payload);
    }
}
