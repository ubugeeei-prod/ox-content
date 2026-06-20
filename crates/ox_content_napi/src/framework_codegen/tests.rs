use std::collections::HashMap;

use serde_json::json;

use super::{
    escape_svelte_markup,
    parser::{HtmlAttribute, HtmlElement, HtmlFragmentParser, HtmlNode},
    render_framework_component_code, JsFrameworkComponentIsland,
};

#[test]
fn parses_nested_fragment_and_decodes_entities() {
    let nodes = HtmlFragmentParser::new("<p title=\"Tom &amp; Jerry\">A &lt; B</p>").parse();
    assert_eq!(
        nodes,
        vec![HtmlNode::Element(HtmlElement {
            tag_name: "p".to_string(),
            attributes: vec![HtmlAttribute {
                name: "title".to_string(),
                value: Some("Tom & Jerry".to_string())
            }],
            children: vec![HtmlNode::Text("A < B".to_string())],
        })]
    );
}

#[test]
fn renders_react_create_element_code() {
    let code = render_framework_component_code(
        [
            r#"<section class="lead" for="name" data-id="42" aria-label="Intro">"#,
            r#"<p style="font-weight: bold; --brand: red;">Hello <strong>world</strong></p>"#,
            "</section>",
        ]
        .join(""),
        "react".to_string(),
        None,
    )
    .unwrap();

    assert!(code.contains("createElement('div', { className: 'ox-content' }"));
    assert!(code.contains(r#""className": "lead""#));
    assert!(code.contains(r#""htmlFor": "name""#));
    assert!(code.contains(r#""data-id": "42""#));
    assert!(code.contains(r#""aria-label": "Intro""#));
    assert!(code.contains(r#""style": { "fontWeight": "bold", "--brand": "red" }"#));
    assert!(code.contains(r#"createElement("strong", null, "world")"#));
}

#[test]
fn renders_vue_h_code() {
    let code = render_framework_component_code(
        r#"<label class="field" for="name"><span>Name</span><input disabled type="text"></label>"#
            .to_string(),
        "vue".to_string(),
        None,
    )
    .unwrap();

    assert!(code.contains("h('div', { class: 'ox-content' }"));
    assert!(code.contains(r#"h("label", { "class": "field", "for": "name" }"#));
    assert!(code.contains(r#"h("span", null, "Name")"#));
    assert!(code.contains(r#"h("input", { "disabled": true, "type": "text" })"#));
}

#[test]
fn renders_framework_islands_with_deterministic_props() {
    let mut props = HashMap::new();
    props.insert("tone".to_string(), json!("info"));
    props.insert("active".to_string(), json!(true));
    let islands = vec![JsFrameworkComponentIsland {
        id: "ox-island-0".to_string(),
        name: "Alert".to_string(),
        props,
        content: Some("Read docs".to_string()),
    }];

    let code = render_framework_component_code(
        r#"<p>Before</p><div data-ox-island="Alert" data-ox-id="ox-island-0"></div>"#.to_string(),
        "react".to_string(),
        Some(islands),
    )
    .unwrap();

    assert!(
        code.contains(r#"createElement(Alert, { "active": true, "tone": "info" }, "Read docs")"#)
    );
}

#[test]
fn escapes_svelte_expression_delimiters() {
    assert_eq!(
        escape_svelte_markup("<p>{count} and }</p>".to_string()),
        "<p>&#123;count&#125; and &#125;</p>"
    );
}
