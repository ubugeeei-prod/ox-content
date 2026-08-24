//! Island payload serialization when MDX JSX is present.
//!
//! Named components become `data-ox-island` placeholders. Literal props are
//! JSON values; `{expression}` and `{...spread}` store source and are never
//! evaluated. Pages without components stay JS-free. `mdx=false` is unchanged.

use ox_content_allocator::Allocator;
use ox_content_parser::{Parser, ParserOptions};
use serde_json::Value;

use crate::html::HtmlRenderer;

fn render_with(source: &str, options: ParserOptions) -> String {
    let allocator = Allocator::new();
    let doc = Parser::with_options(&allocator, source, options)
        .parse()
        .expect("parser should not fail on island render fixtures");
    HtmlRenderer::new().render(&doc)
}

fn render_mdx(source: &str) -> String {
    render_with(source, ParserOptions::mdx())
}

fn html_unescape(value: &str) -> String {
    value
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

fn json_attr(html: &str, name: &str) -> Option<Value> {
    let needle = format!("{name}=\"");
    let start = html.find(&needle)? + needle.len();
    let rest = &html[start..];
    let end = rest.find('"')?;
    let raw = html_unescape(&rest[..end]);
    Some(
        serde_json::from_str(&raw).unwrap_or_else(|error| {
            panic!("attribute {name} is not JSON ({error}): {raw:?}\n{html}")
        }),
    )
}

fn script_payload(html: &str) -> Option<Value> {
    const OPEN: &str = "<script type=\"application/json\">";
    let start = html.find(OPEN)? + OPEN.len();
    let rest = &html[start..];
    let end = rest.find("</script>")?;
    Some(serde_json::from_str(&rest[..end]).unwrap_or_else(|error| {
        panic!("island script is not JSON ({error}): {:?}\n{html}", &rest[..end])
    }))
}

fn island_payload(html: &str) -> Value {
    if let Some(value) = script_payload(html) {
        return value;
    }
    if let Some(value) = json_attr(html, "data-ox-props") {
        if value.get("props").is_some() || value.get("expressions").is_some() {
            return value;
        }
        let mut wrapped = serde_json::Map::new();
        wrapped.insert("props".into(), value);
        if let Some(expressions) = json_attr(html, "data-ox-expressions") {
            wrapped.insert("expressions".into(), expressions);
        }
        if let Some(spreads) = json_attr(html, "data-ox-spreads") {
            wrapped.insert("spreads".into(), spreads);
        }
        return Value::Object(wrapped);
    }
    panic!("expected an island payload in:\n{html}");
}

#[test]
fn page_without_islands_stays_js_free() {
    let html = render_mdx("# Hello\n\nJust prose. Hello {name}.\n\n{count + 1}\n");
    assert!(!html.contains("<script"), "pages without islands stay JS-free:\n{html}");
    assert!(!html.contains("data-ox-island"), "prose is not an island:\n{html}");
    assert!(!html.contains("ox-island"), "no island runtime markers:\n{html}");
}

#[test]
fn island_literal_props_serialize() {
    let html = render_mdx("<Alert title=\"hi\" count={42} disabled />\n");
    assert!(html.contains("data-ox-island=\"Alert\""), "expected an Alert island:\n{html}");
    let payload = island_payload(&html);
    assert_eq!(payload["props"]["title"], "hi", "quoted string is a JSON string:\n{html}");
    assert_eq!(payload["props"]["count"], 42, "{{42}} is a JSON number:\n{html}");
    assert_eq!(payload["props"]["disabled"], true, "boolean attr is true:\n{html}");
}

#[test]
fn island_expression_stores_source_and_does_not_eval() {
    let html = render_mdx("<Alert title={foo} count={count + 1} onClick={alert(1)} />\n");
    let payload = island_payload(&html);
    assert_eq!(payload["expressions"]["title"], "foo", "expression source is stored:\n{html}");
    assert_eq!(
        payload["expressions"]["count"], "count + 1",
        "arithmetic is not evaluated:\n{html}"
    );
    assert_eq!(
        payload["expressions"]["onClick"], "alert(1)",
        "{{alert(1)}} stores source, not a result:\n{html}"
    );
    assert!(
        payload["props"].get("count").is_none(),
        "non-JSON expressions must not appear as literal props:\n{html}"
    );
}

#[test]
fn island_spread_stores_source_and_does_not_eval() {
    let html = render_mdx("<Card title=\"hi\" {...cardProps} {...rest} />\n");
    let payload = island_payload(&html);
    assert_eq!(payload["props"]["title"], "hi");
    assert_eq!(
        payload["spreads"],
        serde_json::json!(["...cardProps", "...rest"]),
        "spreads are a source list, not a merged object:\n{html}"
    );
}

#[test]
fn hostile_expression_source_cannot_break_out() {
    let html =
        render_mdx("<Alert title={\"</script><script>alert(1)\"} label={foo + \"<bar>\"} />\n");
    assert!(!html.contains("</script><script>"), "raw script breakout must not appear:\n{html}");
    assert!(!html.contains("<script>alert"), "alert must not become executable HTML:\n{html}");
    let payload = island_payload(&html);
    let title = payload["props"].get("title").or_else(|| payload["expressions"].get("title"));
    assert_eq!(
        title,
        Some(&Value::String("</script><script>alert(1)".into())),
        "hostile string is recovered from the escaped payload:\n{html}"
    );
    assert_eq!(
        payload["expressions"]["label"], "foo + \"<bar>\"",
        "quotes and < stay in the source string:\n{html}"
    );
}

#[test]
fn mdx_false_does_not_emit_islands() {
    let html =
        render_with("<Alert title=\"hi\" count={42} {...props} />\n", ParserOptions::default());
    assert!(!html.contains("data-ox-island"), "mdx=false must not emit islands:\n{html}");
    assert!(!html.contains("data-ox-props"), "mdx=false must not emit a payload:\n{html}");
    assert!(!html.contains("type=\"application/json\""), "mdx=false stays JS-free:\n{html}");
}
