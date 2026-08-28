use ox_content_allocator::Allocator;
use ox_content_parser::{Parser, ParserOptions};

use super::from_mdast_json;
use crate::mdast::to_mdast_json;

/// Parse `source`, serialize, rebuild, serialize again.
///
/// Two identical payloads mean the round trip kept every field the
/// serializer writes, without asserting on JSON spelling twice.
fn round_trip(source: &str) -> (String, String) {
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, ParserOptions::gfm())
        .parse()
        .expect("source parses");
    let json = to_mdast_json(&document);

    let rebuilt_allocator = Allocator::new();
    let rebuilt = from_mdast_json(&rebuilt_allocator, &json).expect("payload rebuilds");

    let again = to_mdast_json(&rebuilt);
    (json, again)
}

#[test]
fn every_block_survives_the_round_trip() {
    let source = "\
# Heading

Paragraph with *emphasis*, **strong**, ~~struck~~, `code`, a [link](/a 'Title'),
an ![image](/b.png 'Pic'), and a footnote.[^1]

> Quote

- [ ] todo
- [x] done

1. first
2. second

```rust title=demo
fn main() {}
```

| Left | Center | Right | Plain |
| :--- | :----: | ----: | ----- |
| a    | b      | c     | d     |

---

<div>raw html</div>

[ref]: /ref 'Ref title'

[^1]: Footnote body.
";

    let (json, again) = round_trip(source);
    assert_eq!(json, again);
}

#[test]
fn mdx_nodes_survive_the_round_trip() {
    let allocator = Allocator::new();
    let source = "\
import Card from './Card'

export const x = 1

<Card title=\"Hi\" count={2} {...rest}>

Markdown *inside*.

</Card>

Inline <Badge kind=\"tip\" /> and {expression}.

{blockExpression}
";
    let document = Parser::with_options(
        &allocator,
        source,
        ParserOptions { mdx: true, ..ParserOptions::gfm() },
    )
    .parse()
    .expect("source parses");
    let json = to_mdast_json(&document);

    let rebuilt_allocator = Allocator::new();
    let rebuilt = from_mdast_json(&rebuilt_allocator, &json).expect("payload rebuilds");

    assert_eq!(json, to_mdast_json(&rebuilt));
    assert!(json.contains("mdxJsxFlowElement"), "{json}");
    assert!(json.contains("mdxJsxExpressionAttribute"), "{json}");
    assert!(json.contains("mdxTextExpression"), "{json}");
    assert!(json.contains("mdxjsEsm"), "{json}");
}

#[test]
fn a_hand_written_tree_rebuilds() {
    let allocator = Allocator::new();
    let json = r#"{"type":"root","children":[
        {"type":"heading","depth":2,"children":[{"type":"text","value":"Hi"}]},
        {"type":"paragraph","children":[{"type":"text","value":"Body"}]}
    ]}"#;

    let document = from_mdast_json(&allocator, json).expect("payload rebuilds");

    assert_eq!(document.children.len(), 2);
    assert_eq!(
        to_mdast_json(&document),
        r#"{"type":"root","children":[{"type":"heading","depth":2,"children":[{"type":"text","value":"Hi"}]},{"type":"paragraph","children":[{"type":"text","value":"Body"}]}]}"#
    );
}

#[test]
fn missing_children_are_read_as_empty() {
    let allocator = Allocator::new();

    let document = from_mdast_json(&allocator, r#"{"type":"root"}"#).expect("payload rebuilds");

    assert!(document.children.is_empty());
}

#[test]
fn a_rewrite_that_invents_a_node_type_is_reported() {
    let allocator = Allocator::new();
    let json = r#"{"type":"root","children":[{"type":"directive","name":"note"}]}"#;

    let error = from_mdast_json(&allocator, json).expect_err("unknown types are rejected");

    assert!(error.message().contains("directive"), "{}", error.message());
}

#[test]
fn a_malformed_payload_is_reported_rather_than_guessed_at() {
    let allocator = Allocator::new();

    for (json, expected) in [
        ("not json", "expected"),
        ("[]", "not an object"),
        (r#"{"type":"paragraph"}"#, "not \"root\""),
        (r#"{"children":[]}"#, "no \"type\""),
        (r#"{"type":"root","children":[{"type":"text"}]}"#, "\"value\""),
        (r#"{"type":"root","children":[{"type":"heading"}]}"#, "\"depth\""),
        (r#"{"type":"root","children":"nope"}"#, "not an array"),
    ] {
        let error = from_mdast_json(&allocator, json).expect_err(json);
        assert!(error.message().contains(expected), "{json}: {}", error.message());
    }
}

#[test]
fn an_out_of_range_heading_depth_is_clamped_like_the_parser() {
    let allocator = Allocator::new();
    let json = r#"{"type":"root","children":[{"type":"heading","depth":0,"children":[]}]}"#;

    let document = from_mdast_json(&allocator, json).expect("payload rebuilds");

    assert!(to_mdast_json(&document).contains("\"depth\":1"), "{}", to_mdast_json(&document));
}
