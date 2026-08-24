//! Island payload serialization when MDX JSX is present.
//!
//! Named components become `data-ox-island` placeholders. Literal props are
//! JSON values; `{expression}` and `{...spread}` store source and are never
//! evaluated. Markdown children render as HTML inside the island. Pages
//! without components stay JS-free. `mdx=false` is unchanged.

use ox_content_allocator::Allocator;
use ox_content_parser::{Parser, ParserOptions};
use serde_json::Value;

use crate::html::HtmlRenderer;

const ISLAND_JSON_SCRIPT: &str = "<script type=\"application/json\">";

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
    let start = html.find(ISLAND_JSON_SCRIPT)? + ISLAND_JSON_SCRIPT.len();
    let rest = &html[start..];
    let end = rest.find("</script>")?;
    Some(serde_json::from_str(&rest[..end]).unwrap_or_else(|error| {
        panic!("island script is not JSON ({error}): {:?}\n{html}", &rest[..end])
    }))
}

/// Markdown children of a named island, after the optional JSON payload.
fn island_inner<'a>(html: &'a str, name: &str) -> &'a str {
    let marker = format!("data-ox-island=\"{name}\"");
    let start = html.find(&marker).unwrap_or_else(|| panic!("missing island {name}:\n{html}"));
    let after_gt =
        html[start..].find('>').unwrap_or_else(|| panic!("unclosed island tag {name}:\n{html}"));
    let mut body = start + after_gt + 1;
    if let Some(rest) = html[body..].strip_prefix(ISLAND_JSON_SCRIPT)
        && let Some(end) = rest.find("</script>")
    {
        body += ISLAND_JSON_SCRIPT.len() + end + "</script>".len();
    }
    let close_rel = html[body..]
        .rfind("</div>")
        .or_else(|| html[body..].rfind("</span>"))
        .unwrap_or_else(|| panic!("missing island close for {name}:\n{html}"));
    &html[body..body + close_rel]
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

#[test]
fn disabled_mdx_does_not_parse_jsx_children() {
    let html = render_with(
        "<Callout>\n\n# Title\n\nHello **world**.\n\n</Callout>\n",
        ParserOptions::default(),
    );
    assert!(!html.contains("data-ox-island"), "mdx=false must not wrap children:\n{html}");
    assert!(!html.contains("ox-island"), "mdx=false has no island chrome:\n{html}");
    assert!(html.contains("<h1"), "heading still renders as document HTML:\n{html}");
    assert!(html.contains("<strong>world</strong>"), "strong is not dropped:\n{html}");
}

#[test]
fn flow_component_renders_heading_and_strong_inside_island() {
    let html = render_mdx("<Callout>\n\n# Title\n\nHello **world**.\n\n</Callout>\n");
    assert!(html.contains("data-ox-island=\"Callout\""), "expected a Callout island:\n{html}");
    let inner = island_inner(&html, "Callout");
    assert!(inner.contains("<h1"), "heading must render inside the island:\n{html}");
    assert!(inner.contains("id=\"title\""), "heading id is preserved:\n{html}");
    assert!(
        inner.contains("<strong>world</strong>"),
        "strong must be real HTML, not escaped:\n{html}"
    );
    assert!(inner.contains("Hello"), "prose must not be dropped:\n{html}");
    let after = html.rsplit_once("</div>").map_or("", |(_, rest)| rest);
    assert!(!after.contains("<h1"), "heading must not leak after the island:\n{html}");
    assert!(!after.contains("<strong>"), "strong must not leak after the island:\n{html}");
}

#[test]
fn nested_list_inside_component() {
    let html = render_mdx("<Callout>\n\n- parent\n  - child\n- sibling\n\n</Callout>\n");
    let inner = island_inner(&html, "Callout");
    assert!(inner.contains("<ul>"), "list must render inside the island:\n{html}");
    assert!(inner.contains("<li>parent"), "parent item is inside the island:\n{html}");
    assert!(inner.contains("<li>child"), "nested item is inside the island:\n{html}");
    assert!(inner.contains("<li>sibling"), "sibling item is inside the island:\n{html}");
    let nested_lists = inner.matches("<ul>").count();
    assert!(nested_lists >= 2, "nested list must stay nested:\n{html}");
}

#[test]
fn fence_inside_component_is_code_not_island() {
    let html = render_mdx("<Callout>\n\n```js\nconst x = <Alert />;\n```\n\n</Callout>\n");
    let inner = island_inner(&html, "Callout");
    assert!(inner.contains("<pre>"), "fence must become a code block:\n{html}");
    assert!(inner.contains("language-js"), "fence language is preserved:\n{html}");
    assert!(inner.contains("const x"), "fence text is not dropped:\n{html}");
    assert!(!inner.contains("data-ox-island=\"Alert\""), "fence JSX is not an island:\n{html}");
    assert!(
        inner.contains("&lt;Alert") || inner.contains("&lt;Alert /&gt;"),
        "fence contents are escaped code, not a component:\n{html}"
    );
}

#[test]
fn nested_jsx_inside_markdown_children() {
    let html = render_mdx("<Callout>\n\n# Title\n\n<Badge title=\"hi\" />\n\n</Callout>\n");
    let inner = island_inner(&html, "Callout");
    assert!(inner.contains("<h1"), "markdown heading stays inside Callout:\n{html}");
    assert!(inner.contains("data-ox-island=\"Badge\""), "nested PascalCase is an island:\n{html}");
    assert!(
        html.contains("data-ox-island=\"Callout\""),
        "outer component is still an island:\n{html}"
    );
    assert!(
        inner.contains("<script type=\"application/json\">"),
        "nested island payload is not tagfiltered:\n{html}"
    );
    assert_eq!(island_payload(&html)["props"]["title"], "hi", "nested props still parse:\n{html}");
}

#[test]
fn hostile_child_text_escaped() {
    let html = render_mdx("<Callout>\n<script>alert(1)</script>\n</Callout>\n");
    assert!(html.contains("data-ox-island=\"Callout\""), "wrapper still emits:\n{html}");
    assert!(!html.contains("<script>"), "raw script must not appear:\n{html}");
    assert!(!html.contains("<script "), "raw script tag must not appear:\n{html}");
    let inner = island_inner(&html, "Callout");
    assert!(inner.contains("alert(1)"), "hostile child text is not dropped:\n{html}");
    assert!(inner.contains("&lt;script"), "script tag is escaped:\n{html}");

    let nested = render_mdx("<Callout>\n\n- <script>alert(1)</script>\n\n</Callout>\n");
    assert!(!nested.contains("<script>"), "list child script is not raw:\n{nested}");
    assert!(
        island_inner(&nested, "Callout").contains("<ul>"),
        "list still renders around escaped HTML:\n{nested}"
    );
}

#[test]
fn unclosed_component_does_not_swallow_file() {
    let html = render_mdx("<Callout>\n\n# Still here\n\nAfter the unclosed tag.\n");
    assert!(!html.contains("data-ox-island"), "unclosed opener is not an island:\n{html}");
    assert!(html.contains("<h1"), "heading after the opener still renders:\n{html}");
    assert!(html.contains("Still here"), "heading text is not swallowed:\n{html}");
    assert!(html.contains("After the unclosed tag."), "trailing prose is not swallowed:\n{html}");
}

#[test]
fn page_without_components_still_js_free() {
    let prose = render_mdx("# Hello\n\nJust prose with **bold**.\n\n- a\n- b\n");
    assert!(!prose.contains("<script"), "prose pages stay JS-free:\n{prose}");
    assert!(!prose.contains("data-ox-island"), "prose is not an island:\n{prose}");

    let fragment = render_mdx("<>\n\n# Title\n\nHello **world**.\n\n</>\n");
    assert!(!fragment.contains("<script"), "fragments are not islands:\n{fragment}");
    assert!(!fragment.contains("data-ox-island"), "nameless wrappers emit no island:\n{fragment}");
    assert!(fragment.contains("<h1"), "fragment markdown children still render:\n{fragment}");
    assert!(
        fragment.contains("<strong>world</strong>"),
        "fragment phrasing is real HTML:\n{fragment}"
    );
}
