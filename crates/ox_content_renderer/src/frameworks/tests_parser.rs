use super::parser::{HtmlAttribute, HtmlElement, HtmlFragmentParser, HtmlNode};

#[test]
fn parses_nested_fragment_and_decodes_entities() {
    let nodes = HtmlFragmentParser::new("<p title=\"Tom &amp; Jerry\">A &lt; B</p>").parse();
    assert_eq!(
        nodes,
        vec![HtmlNode::Element(HtmlElement {
            tag_name: "p".into(),
            attributes: vec![HtmlAttribute {
                name: "title".into(),
                value: Some("Tom & Jerry".into())
            }],
            children: vec![HtmlNode::Text("A < B".into())],
        })]
    );
}

#[test]
fn parser_skips_comments_declarations_and_processing_instructions() {
    let nodes = HtmlFragmentParser::new("<!doctype html><!--x--><?pi?><p>Body</p>").parse();
    assert_eq!(
        nodes,
        vec![HtmlNode::Element(HtmlElement {
            tag_name: "p".into(),
            attributes: vec![],
            children: vec![HtmlNode::Text("Body".into())],
        })]
    );
}

#[test]
fn parser_keeps_malformed_tags_as_text() {
    let nodes = HtmlFragmentParser::new("<p>before <broken after").parse();
    assert_eq!(
        nodes,
        vec![HtmlNode::Element(HtmlElement {
            tag_name: "p".into(),
            attributes: vec![],
            children: vec![HtmlNode::Text("before <broken after".into())],
        })]
    );
}

#[test]
fn parser_reads_boolean_unquoted_and_single_quoted_attributes() {
    let nodes = HtmlFragmentParser::new("<input disabled data-id=abc title='A &amp; B'>").parse();
    assert_eq!(
        nodes,
        vec![HtmlNode::Element(HtmlElement {
            tag_name: "input".into(),
            attributes: vec![
                HtmlAttribute { name: "disabled".into(), value: None },
                HtmlAttribute { name: "data-id".into(), value: Some("abc".into()) },
                HtmlAttribute { name: "title".into(), value: Some("A & B".into()) },
            ],
            children: vec![],
        })]
    );
}

#[test]
fn parser_closes_mismatched_nested_elements_at_matching_ancestor() {
    let nodes = HtmlFragmentParser::new("<div><p><em>x</p>y</div>").parse();
    assert_eq!(
        nodes,
        vec![HtmlNode::Element(HtmlElement {
            tag_name: "div".into(),
            attributes: vec![],
            children: vec![
                HtmlNode::Element(HtmlElement {
                    tag_name: "p".into(),
                    attributes: vec![],
                    children: vec![HtmlNode::Element(HtmlElement {
                        tag_name: "em".into(),
                        attributes: vec![],
                        children: vec![HtmlNode::Text("x".into())],
                    })],
                }),
                HtmlNode::Text("y".into()),
            ],
        })]
    );
}
