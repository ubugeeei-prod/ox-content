use rustc_hash::FxHashMap;
use serde_json::json;

use super::{
    escape_svelte_markup, render_framework_code, render_framework_component_code,
    FrameworkCodegenError, FrameworkCodegenMode, FrameworkCodegenTarget, FrameworkComponentIsland,
};

#[test]
fn renders_react_create_element_code() {
    let code = render_framework_component_code(
        &[
            r#"<section class="lead" for="name" data-id="42" aria-label="Intro">"#,
            r#"<p style="font-weight: bold; --brand: red;">Hello <strong>world</strong></p>"#,
            "</section>",
        ]
        .join(""),
        FrameworkCodegenTarget::React,
        &[],
    );

    insta::assert_snapshot!(code);
}

#[test]
fn renders_vue_h_code() {
    let code = render_framework_component_code(
        r#"<label class="field" for="name"><span>Name</span><input disabled type="text"></label>"#,
        FrameworkCodegenTarget::Vue,
        &[],
    );

    insta::assert_snapshot!(code);
}

#[test]
fn renders_void_and_self_closing_elements_without_children() {
    let code = render_framework_component_code(
        r#"<p>Image <img src="/logo.png"><br/><input disabled></p>"#,
        FrameworkCodegenTarget::React,
        &[],
    );

    assert!(code.contains(r#"createElement("img", { "src": "/logo.png" })"#));
    assert!(code.contains(r#"createElement("br", null)"#));
    assert!(code.contains(r#"createElement("input", { "disabled": true })"#));
}

#[test]
fn renders_escaped_js_string_literals_for_text_and_attributes() {
    let code = render_framework_component_code(
        "<p title=\"quote &quot; slash \\\">line\nnext</p>",
        FrameworkCodegenTarget::React,
        &[],
    );

    assert!(code.contains(r#""title": "quote \" slash \\""#));
    assert!(code.contains(r#""line\nnext""#));
}

#[test]
fn renders_react_data_and_aria_camel_case_as_attributes() {
    let code = render_framework_component_code(
        r#"<button dataTestId="save" ariaLabel="Save">Save</button>"#,
        FrameworkCodegenTarget::React,
        &[],
    );

    assert!(code.contains(r#""data-test-id": "save""#));
    assert!(code.contains(r#""aria-label": "Save""#));
}

#[test]
fn renders_vue_single_child_without_array_and_multiple_children_with_array() {
    let single = render_framework_component_code(
        "<p><span>one</span></p>",
        FrameworkCodegenTarget::Vue,
        &[],
    );
    let many = render_framework_component_code(
        "<p><span>one</span><span>two</span></p>",
        FrameworkCodegenTarget::Vue,
        &[],
    );

    assert!(single.contains(r#"h("p", null, h("span", null, "one"))"#));
    assert!(many.contains(r#"h("p", null, [h("span", null, "one"), h("span", null, "two")])"#));
}

#[test]
fn renders_framework_islands_with_deterministic_props() {
    let mut props = FxHashMap::default();
    props.insert("tone".to_string(), json!("info"));
    props.insert("active".to_string(), json!(true));
    let islands = vec![FrameworkComponentIsland {
        id: "ox-island-0".to_string(),
        name: "Alert".to_string(),
        props,
        content: Some("Read docs".to_string()),
    }];

    let code = render_framework_component_code(
        r#"<p>Before</p><div data-ox-island="Alert" data-ox-id="ox-island-0"></div>"#,
        FrameworkCodegenTarget::React,
        &islands,
    );

    insta::assert_snapshot!(code);
}

#[test]
fn renders_islands_from_camel_case_marker_and_skips_marker_props() {
    let mut props = FxHashMap::default();
    props.insert("count".into(), json!(3));
    let islands = vec![FrameworkComponentIsland {
        id: "ox-island-0".into(),
        name: "Counter".into(),
        props,
        content: None,
    }];

    let code = render_framework_component_code(
        r#"<div dataOxId="ox-island-0" data-ox-island="Counter"></div>"#,
        FrameworkCodegenTarget::Vue,
        &islands,
    );

    assert_eq!(code, r#"h('div', { class: 'ox-content' }, h(Counter, { "count": 3 }))"#);
}

#[test]
fn renders_nested_json_props_in_stable_top_level_order() {
    let mut props = FxHashMap::default();
    props.insert("z".into(), json!([3, 2, 1]));
    props.insert("a".into(), json!({ "nested": true }));
    let islands = vec![FrameworkComponentIsland {
        id: "ox-island-0".into(),
        name: "Widget".into(),
        props,
        content: Some("child".into()),
    }];

    let code = render_framework_component_code(
        r#"<div data-ox-id="ox-island-0"></div>"#,
        FrameworkCodegenTarget::React,
        &islands,
    );

    assert!(
        code.contains(r#"createElement(Widget, { "a": {"nested":true}, "z": [3,2,1] }, "child")"#)
    );
}

#[test]
fn renders_inner_html_component_modules() {
    let react = render_framework_code(
        "<p>Hello</p>",
        FrameworkCodegenTarget::React,
        FrameworkCodegenMode::InnerHtml,
        &[],
    )
    .unwrap();
    let vue = render_framework_code(
        "<p>Hello</p>",
        FrameworkCodegenTarget::Vue,
        FrameworkCodegenMode::InnerHtml,
        &[],
    )
    .unwrap();

    assert!(react.contains("dangerouslySetInnerHTML: { __html: rawHtml }"));
    assert!(vue.contains("innerHTML: rawHtml"));
}

#[test]
fn escapes_raw_html_literals_without_changing_runtime_html() {
    let html = "</script><p title=\"line\nnext\">{ ok }</p>";
    let react = render_framework_code(
        html,
        FrameworkCodegenTarget::React,
        FrameworkCodegenMode::InnerHtml,
        &[],
    )
    .unwrap();
    let vue = render_framework_code(
        html,
        FrameworkCodegenTarget::Vue,
        FrameworkCodegenMode::InnerHtml,
        &[],
    )
    .unwrap();
    let svelte = render_framework_code(
        html,
        FrameworkCodegenTarget::Svelte,
        FrameworkCodegenMode::InnerHtml,
        &[],
    )
    .unwrap();

    assert!(!react.contains("</script>"));
    assert!(!vue.contains("</script>"));
    assert_eq!(svelte.matches("</script>").count(), 1);
    assert!(react.contains(r#"\x3C/script>\x3Cp title=\"line\nnext\">{ ok }\x3C/p>"#));
    assert!(vue.contains(r#"\x3C/script>\x3Cp title=\"line\nnext\">{ ok }\x3C/p>"#));
    assert!(svelte.contains(r#"\x3C/script>\x3Cp title=\"line\nnext\">{ ok }\x3C/p>"#));
}

#[test]
fn renders_component_and_render_function_modules() {
    let component = render_framework_code(
        "<p>Hello</p>",
        FrameworkCodegenTarget::React,
        FrameworkCodegenMode::Component,
        &[],
    )
    .unwrap();
    let render_function = render_framework_code(
        "<p>Hello</p>",
        FrameworkCodegenTarget::Vue,
        FrameworkCodegenMode::RenderFunction,
        &[],
    )
    .unwrap();

    assert!(component.contains("export default function MarkdownContent()"));
    assert!(component.contains(r#"createElement("p", null, "Hello")"#));
    assert!(render_function.contains("export function renderMarkdownContent()"));
    assert!(render_function.contains(r"return h('div', { class: 'ox-content' }"));
}

#[test]
fn renders_svelte_component_modes() {
    let component = render_framework_code(
        "<p>{count}</p>",
        FrameworkCodegenTarget::Svelte,
        FrameworkCodegenMode::Component,
        &[],
    )
    .unwrap();
    let inner_html = render_framework_code(
        "<p>{count}</p>",
        FrameworkCodegenTarget::Svelte,
        FrameworkCodegenMode::InnerHtml,
        &[],
    )
    .unwrap();

    assert!(component.contains("<p>&#123;count&#125;</p>"));
    assert!(inner_html.contains("{@html rawHtml}"));
}

#[test]
fn rejects_svelte_render_function_mode() {
    let error = render_framework_code(
        "<p>Hello</p>",
        FrameworkCodegenTarget::Svelte,
        FrameworkCodegenMode::RenderFunction,
        &[],
    )
    .unwrap_err();

    assert_eq!(
        error,
        FrameworkCodegenError::UnsupportedModeForTarget {
            mode: FrameworkCodegenMode::RenderFunction,
            target: FrameworkCodegenTarget::Svelte,
        }
    );
}

#[test]
fn escapes_svelte_expression_delimiters() {
    assert_eq!(escape_svelte_markup("<p>{count} and }</p>"), "<p>&#123;count&#125; and &#125;</p>");
}
