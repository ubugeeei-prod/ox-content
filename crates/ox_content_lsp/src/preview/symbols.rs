use ox_content_allocator::Allocator;
use ox_content_ast::Node;
use ox_content_parser::{ParseError, Parser, ParserOptions};
use tower_lsp::lsp_types::{DocumentSymbol, SymbolKind};

use crate::document::TextDocumentState;
use crate::frontmatter::parse_frontmatter;
use crate::preview::text::heading_text;

pub fn document_symbols(
    source: &str,
    document: &TextDocumentState,
) -> Result<Vec<DocumentSymbol>, ParseError> {
    let frontmatter = parse_frontmatter(document);
    let (content, base_offset) = if let Some(block) = frontmatter.block {
        // Skip past the frontmatter (incl. closing `---\n`) so headings
        // inside the markdown body are picked up, and so the reported
        // ranges line up with what the editor opened.
        (&source[block.block_end_offset..], block.block_end_offset)
    } else {
        (source, 0)
    };

    let allocator = Allocator::new();
    let parser = Parser::with_options(&allocator, content, ParserOptions::gfm());
    let ast = parser.parse()?;
    let mut symbols = Vec::new();
    collect_symbols(&ast.children, document, base_offset, &mut symbols);
    Ok(symbols)
}

fn collect_symbols(
    nodes: &[Node<'_>],
    document: &TextDocumentState,
    base_offset: usize,
    symbols: &mut Vec<DocumentSymbol>,
) {
    for node in nodes {
        match node {
            Node::Heading(heading) => {
                symbols.push(symbol_for_heading(heading, document, base_offset));
            }
            Node::BlockQuote(block) => {
                collect_symbols(&block.children, document, base_offset, symbols);
            }
            Node::List(list) => {
                for item in &list.children {
                    collect_symbols(&item.children, document, base_offset, symbols);
                }
            }
            _ => {}
        }
    }
}

fn symbol_for_heading(
    heading: &ox_content_ast::Heading<'_>,
    document: &TextDocumentState,
    base_offset: usize,
) -> DocumentSymbol {
    let start = base_offset + heading.span.start as usize;
    let end = base_offset + heading.span.end as usize;
    let range = document.range_from_offsets(start, end);

    #[allow(deprecated)]
    DocumentSymbol {
        name: heading_text(heading),
        detail: Some({
            let mut detail = String::from("h");
            detail.push_str(&heading.depth.to_string());
            detail
        }),
        kind: SymbolKind::STRING,
        range,
        selection_range: range,
        tags: None,
        deprecated: None,
        children: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write as _;

    fn outline(symbols: &[DocumentSymbol]) -> String {
        let mut out = String::new();
        for symbol in symbols {
            let detail = symbol.detail.as_deref().unwrap_or("?");
            let range = symbol.range;
            if writeln!(
                &mut out,
                "{} [{}:{}..{}:{}] {}",
                detail,
                range.start.line,
                range.start.character,
                range.end.line,
                range.end.character,
                symbol.name,
            )
            .is_err()
            {
                out.push_str("[formatting failed]\n");
            }
        }
        out
    }

    fn collect(source: &str) -> Vec<DocumentSymbol> {
        let document = TextDocumentState::new(source.to_string());
        document_symbols(source, &document).expect("parse should not fail")
    }

    #[test]
    fn flat_outline_for_atx_headings() {
        let source = "# h1\n## h2 deeper\n### h3 deepest\n";
        let result = outline(&collect(source));
        insta::with_settings!({
            snapshot_path => "snapshots",
            prepend_module_to_snapshot => false,
            omit_expression => true,
        }, {
            insta::assert_snapshot!("flat_outline_for_atx_headings", result);
        });
    }

    #[test]
    fn outline_offsets_account_for_frontmatter() {
        // Heading positions are reported relative to the original document,
        // not the frontmatter-stripped slice the parser sees. The Position
        // must point at the actual line in the source the editor opened.
        let source = "---\ntitle: Doc\n---\n\n# After Frontmatter\n";
        let result = outline(&collect(source));
        insta::with_settings!({
            snapshot_path => "snapshots",
            prepend_module_to_snapshot => false,
            omit_expression => true,
        }, {
            insta::assert_snapshot!("outline_offsets_account_for_frontmatter", result);
        });
    }

    #[test]
    fn outline_skips_non_heading_blocks() {
        // Code fences, blockquotes (without nested headings), paragraphs, and
        // tables should never appear as document symbols.
        let source = concat!(
            "# Top\n",
            "\n",
            "Paragraph.\n",
            "\n",
            "```rs\nfn main() {}\n```\n",
            "\n",
            "> just a quote\n",
            "\n",
            "## Below\n",
        );
        let result = outline(&collect(source));
        insta::with_settings!({
            snapshot_path => "snapshots",
            prepend_module_to_snapshot => false,
            omit_expression => true,
        }, {
            insta::assert_snapshot!("outline_skips_non_heading_blocks", result);
        });
    }
}
