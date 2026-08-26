//! XSS-safe island payloads from MDX JSX attributes.
//!
//! Literal props become JSON values. `{expression}` and `{...spread}` store
//! source text and are never evaluated. The serialized JSON unicode-escapes
//! `<`, `>`, and `&` so it is safe inside a non-executed script or an
//! HTML-escaped attribute.

use ox_content_ast::{MdxJsxAttributeEntry, MdxJsxAttributeValue};
use serde_json::{Map, Value};

/// Structured island payload: literals, expression sources, and spreads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct IslandPayload {
    pub(super) props: Map<String, Value>,
    pub(super) expressions: Map<String, Value>,
    pub(super) spreads: Vec<Value>,
}

impl IslandPayload {
    pub(super) fn is_empty(&self) -> bool {
        self.props.is_empty() && self.expressions.is_empty() && self.spreads.is_empty()
    }

    pub(super) fn into_json_value(self) -> Value {
        let mut object = Map::new();
        object.insert("expressions".into(), Value::Object(sorted_map(self.expressions)));
        object.insert("props".into(), Value::Object(sorted_map(self.props)));
        object.insert("spreads".into(), Value::Array(self.spreads));
        Value::Object(object)
    }
}

fn sorted_map(map: Map<String, Value>) -> Map<String, Value> {
    let mut entries = map.into_iter().collect::<Vec<_>>();
    entries.sort_by(|(left, _), (right, _)| left.cmp(right));

    let mut sorted = Map::new();
    for (key, value) in entries {
        sorted.insert(key, value);
    }
    sorted
}

pub(super) fn collect_island_payload(attributes: &[MdxJsxAttributeEntry<'_>]) -> IslandPayload {
    let mut payload =
        IslandPayload { props: Map::new(), expressions: Map::new(), spreads: Vec::new() };

    for entry in attributes {
        match entry {
            MdxJsxAttributeEntry::Attribute(attribute) => match &attribute.value {
                None => {
                    payload.props.insert(attribute.name.to_owned(), Value::Bool(true));
                }
                Some(MdxJsxAttributeValue::Literal(value)) => {
                    payload
                        .props
                        .insert(attribute.name.to_owned(), Value::String((*value).to_owned()));
                }
                Some(MdxJsxAttributeValue::Expression(expr)) => {
                    match serde_json::from_str::<Value>(expr.value.trim()) {
                        Ok(value) => {
                            payload.props.insert(attribute.name.to_owned(), value);
                        }
                        Err(_) => {
                            payload.expressions.insert(
                                attribute.name.to_owned(),
                                Value::String(expr.value.to_owned()),
                            );
                        }
                    }
                }
            },
            MdxJsxAttributeEntry::Expression(expr) => {
                payload.spreads.push(Value::String(expr.value.to_owned()));
            }
        }
    }

    payload
}

/// JSON text that cannot break out of `<script>` or HTML attributes.
pub(super) fn stringify_xss_safe(value: &Value) -> String {
    let json = serde_json::to_string(value).unwrap_or_else(|_| "{}".to_owned());
    escape_json_for_html(&json)
}

pub(super) fn escape_json_for_html(json: &str) -> String {
    let mut out = String::with_capacity(json.len());
    for ch in json.chars() {
        match ch {
            '<' => out.push_str("\\u003c"),
            '>' => out.push_str("\\u003e"),
            '&' => out.push_str("\\u0026"),
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            ch => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use ox_content_allocator::Allocator;
    use ox_content_ast::{
        MdxJsxAttribute, MdxJsxAttributeEntry, MdxJsxAttributeValue,
        MdxJsxAttributeValueExpression, MdxJsxExpressionAttribute, Span,
    };

    use super::{IslandPayload, collect_island_payload, stringify_xss_safe};

    #[test]
    fn literals_and_json_expressions_become_props() {
        let allocator = Allocator::new();
        let mut attributes = allocator.new_vec();
        attributes.push(MdxJsxAttributeEntry::Attribute(MdxJsxAttribute {
            name: "title",
            value: Some(MdxJsxAttributeValue::Literal("hi")),
            span: Span::new(0, 1),
        }));
        attributes.push(MdxJsxAttributeEntry::Attribute(MdxJsxAttribute {
            name: "count",
            value: Some(MdxJsxAttributeValue::Expression(MdxJsxAttributeValueExpression {
                value: "42",
                span: Span::new(0, 1),
            })),
            span: Span::new(0, 1),
        }));
        attributes.push(MdxJsxAttributeEntry::Attribute(MdxJsxAttribute {
            name: "disabled",
            value: None,
            span: Span::new(0, 1),
        }));

        let payload = collect_island_payload(&attributes);
        assert_eq!(payload.props["title"], "hi");
        assert_eq!(payload.props["count"], 42);
        assert_eq!(payload.props["disabled"], true);
        assert!(payload.expressions.is_empty());
    }

    #[test]
    fn non_json_expression_and_spread_store_source() {
        let allocator = Allocator::new();
        let mut attributes = allocator.new_vec();
        attributes.push(MdxJsxAttributeEntry::Attribute(MdxJsxAttribute {
            name: "onClick",
            value: Some(MdxJsxAttributeValue::Expression(MdxJsxAttributeValueExpression {
                value: "alert(1)",
                span: Span::new(0, 1),
            })),
            span: Span::new(0, 1),
        }));
        attributes.push(MdxJsxAttributeEntry::Expression(MdxJsxExpressionAttribute {
            value: "...cardProps",
            span: Span::new(0, 1),
        }));

        let payload = collect_island_payload(&attributes);
        assert_eq!(payload.expressions["onClick"], "alert(1)");
        assert_eq!(payload.spreads, vec![serde_json::json!("...cardProps")]);
        assert!(payload.props.is_empty());
    }

    #[test]
    fn serializes_payload_keys_in_a_stable_order() {
        let mut payload = IslandPayload {
            props: serde_json::Map::new(),
            expressions: serde_json::Map::new(),
            spreads: Vec::new(),
        };
        payload.props.insert("title".into(), serde_json::json!("Docs"));
        payload.props.insert("data-kind".into(), serde_json::json!("guide"));
        payload.expressions.insert("count".into(), serde_json::json!("count"));

        let json = stringify_xss_safe(&payload.into_json_value());

        assert_eq!(
            json,
            r#"{"expressions":{"count":"count"},"props":{"data-kind":"guide","title":"Docs"},"spreads":[]}"#
        );
    }

    #[test]
    fn stringify_escapes_script_breakout() {
        let value = serde_json::json!({
            "props": { "title": "</script><script>alert(1)" },
            "expressions": {},
            "spreads": []
        });
        let json = stringify_xss_safe(&value);
        assert!(!json.contains("</script>"), "raw </script> must not appear: {json}");
        assert!(!json.contains('<'), "raw < must not appear: {json}");
        assert!(json.contains("\\u003c"), "expected unicode-escaped markup: {json}");
    }
}
