use ox_content_allocator::Allocator;
use ox_content_ast::Node;
use ox_content_parser::{Parser, ParserOptions};

use super::check;

// --- Mixed / regression ---

#[test]
fn snapshot_kitchen_sink_document() {
    check(
        "kitchen_sink_document",
        concat!(
            "# Project README\n",
            "\n",
            "A short blurb with **bold**, *italic*, and `code`.\n",
            "\n",
            "## Install\n",
            "\n",
            "```bash\n",
            "npm install ox-content\n",
            "```\n",
            "\n",
            "## Usage\n",
            "\n",
            "- Step one\n",
            "- Step two\n",
            "  - Sub-step a\n",
            "  - Sub-step b\n",
            "\n",
            "> Tip: read the docs.\n",
            "\n",
            "| col | val |\n",
            "| --- | --- |\n",
            "| a   | 1   |\n",
            "| b   | 2   |\n",
            "\n",
            "See the [website](https://example.com).\n",
        ),
        ParserOptions::gfm(),
    );
}

// --- Sanity guard: pretty-printer never emits an empty document for input that contains
//     at least one block-level construct. This is a structural invariant we want to defend
//     against silent regressions where the parser swallows valid content. ---

#[test]
fn snapshot_first_block_node_kinds_for_basic_constructs() {
    let cases: &[(&str, &str)] = &[
        ("para_kind", "hello"),
        ("heading_kind", "# h"),
        ("list_kind", "- a"),
        ("blockquote_kind", "> q"),
        ("code_kind", "```\nfn\n```"),
        ("hr_kind", "---"),
        ("table_kind", "| a |\n| - |\n| 1 |"),
    ];

    let mut report = String::new();
    let allocator = Allocator::new();
    for (label, source) in cases {
        let doc = Parser::with_options(&allocator, source, ParserOptions::gfm())
            .parse()
            .expect("parser should not fail");
        report.push_str(label);
        report.push_str(": ");
        report.push_str(node_kind(doc.children.first()));
        report.push('\n');
    }

    insta::with_settings!({
        snapshot_path => "../snapshots/parser",
        prepend_module_to_snapshot => false,
        omit_expression => true,
    }, {
        insta::assert_snapshot!("first_block_node_kinds_for_basic_constructs", report);
    });
}

fn node_kind(node: Option<&Node<'_>>) -> &'static str {
    match node {
        None => "<none>",
        Some(Node::Paragraph(_)) => "Paragraph",
        Some(Node::Heading(_)) => "Heading",
        Some(Node::ThematicBreak(_)) => "ThematicBreak",
        Some(Node::BlockQuote(_)) => "BlockQuote",
        Some(Node::List(_)) => "List",
        Some(Node::ListItem(_)) => "ListItem",
        Some(Node::CodeBlock(_)) => "CodeBlock",
        Some(Node::Html(_)) => "Html",
        Some(Node::Table(_)) => "Table",
        Some(Node::Text(_)) => "Text",
        Some(Node::Emphasis(_)) => "Emphasis",
        Some(Node::Strong(_)) => "Strong",
        Some(Node::InlineCode(_)) => "InlineCode",
        Some(Node::Break(_)) => "Break",
        Some(Node::Link(_)) => "Link",
        Some(Node::Image(_)) => "Image",
        Some(Node::Delete(_)) => "Delete",
        Some(Node::FootnoteReference(_)) => "FootnoteReference",
        Some(Node::Definition(_)) => "Definition",
        Some(Node::FootnoteDefinition(_)) => "FootnoteDefinition",
    }
}
