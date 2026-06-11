use crate::html::HtmlRenderer;
use ox_content_allocator::Allocator;
use ox_content_parser::Parser;

#[test]
fn test_render_nested_list() {
    let allocator = Allocator::new();
    // Indent with 2 spaces for nesting
    let doc = Parser::new(&allocator, "- item 1\n  - sub 1\n- item 2").parse().unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);

    // Normalize newlines for comparison
    let normalized = html.replace('\n', "");
    // We expect:
    // <ul>
    //   <li>
    //     <p>item 1</p>
    //     <ul>
    //       <li><p>sub 1</p></li>
    //     </ul>
    //   </li>
    //   <li><p>item 2</p></li>
    // </ul>
    // Note: The exact placement of <p> tags depends on how we handle list content.
    // Assuming tight list items might not have <p> if we implement loose/tight lists,
    // but currently everything is wrapped in <p> in parse_list implementation (wrapped in Paragraph).

    // Let's just check for the structure <li>...<ul>...</ul>...</li>
    assert!(normalized.contains("<li><p>item 1</p><ul><li><p>sub 1</p></li></ul></li>"));
    assert!(normalized.contains("<li><p>item 2</p></li>"));
}

#[test]
fn test_render_table() {
    let allocator = Allocator::new();
    let parser_options = ox_content_parser::ParserOptions::gfm();
    let doc = Parser::with_options(&allocator, "| head |\n| --- |\n| body |", parser_options)
        .parse()
        .unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);
    assert!(html.contains("<table>"));
    assert!(html.contains("<thead>"));
    assert!(html.contains("<th>head</th>"));
    assert!(html.contains("<tbody>"));
    assert!(html.contains("<td>body</td>"));
}

#[test]
fn test_render_table_no_gfm() {
    let allocator = Allocator::new();
    // Default options have tables: false
    let doc = Parser::new(&allocator, "| head |\n| --- |\n| body |").parse().unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);
    assert!(!html.contains("<table>"));
    assert!(html.contains("| head |"));
}

#[test]
fn test_render_list_with_bold() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "- **bold** text").parse().unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);
    assert!(html.contains("<strong>bold</strong>"));
}

#[test]
fn test_render_task_list() {
    let allocator = Allocator::new();
    let parser_options = ox_content_parser::ParserOptions::gfm();
    let doc = Parser::with_options(&allocator, "- [x] task 1\n- [ ] task 2", parser_options)
        .parse()
        .unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);
    assert!(html.contains("<input type=\"checkbox\" checked disabled> <p>task 1</p>"));
    assert!(html.contains("<input type=\"checkbox\" disabled> <p>task 2</p>"));
}
