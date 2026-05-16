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
        (&source[block.content_start_offset..block.content_end_offset], block.content_start_offset)
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
        detail: Some(format!("h{}", heading.depth)),
        kind: SymbolKind::STRING,
        range,
        selection_range: range,
        tags: None,
        deprecated: None,
        children: None,
    }
}
