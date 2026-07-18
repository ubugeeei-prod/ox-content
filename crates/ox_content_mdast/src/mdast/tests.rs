use super::to_mdast_json;
use ox_content_allocator::Allocator;
use ox_content_parser::{Parser, ParserOptions};
use serde_json::Value;

fn parse_json(source: &str, options: ParserOptions) -> Value {
    let allocator = Allocator::new();
    let doc = Parser::with_options(&allocator, source, options).parse().unwrap();
    serde_json::from_str(&to_mdast_json(&doc)).unwrap()
}

#[test]
fn serializes_mdast_root_shape() {
    let json = parse_json("# Hello\n\nA [link](https://example.com)", ParserOptions::default());
    assert_eq!(json["type"], "root");
    assert_eq!(json["children"][0]["type"], "heading");
    assert_eq!(json["children"][0]["depth"], 1);
    assert_eq!(json["children"][0]["children"][0]["value"], "Hello");
    assert_eq!(json["children"][1]["children"][1]["type"], "link");
    assert_eq!(json["children"][1]["children"][1]["url"], "https://example.com");
}

#[test]
fn serializes_gfm_nodes() {
    let json = parse_json("- [x] ~~done~~\n\n| head |\n| :--- |\n| body |", ParserOptions::gfm());
    assert_eq!(json["children"][0]["type"], "list");
    assert_eq!(json["children"][0]["children"][0]["checked"], true);
    assert_eq!(json["children"][0]["children"][0]["children"][0]["children"][0]["type"], "delete");
    assert_eq!(json["children"][1]["type"], "table");
    assert_eq!(json["children"][1]["align"][0], "left");
}

#[test]
fn serializes_code_breaks_and_ordered_list_start() {
    let json = parse_json(
        "5. item\n\nline 1\\\nline 2\n\n```ts meta=1\nconsole.log(1)\n```",
        ParserOptions::gfm(),
    );

    assert_eq!(json["children"][0]["type"], "list");
    assert_eq!(json["children"][0]["ordered"], true);
    assert_eq!(json["children"][0]["start"], 5);
    assert_eq!(json["children"][1]["type"], "paragraph");
    assert_eq!(json["children"][1]["children"][1]["type"], "break");
    assert_eq!(json["children"][2]["type"], "code");
    assert_eq!(json["children"][2]["lang"], "ts");
    assert_eq!(json["children"][2]["meta"], "meta=1");
}

#[test]
fn escapes_json_string_bytes_across_chunk_boundaries() {
    // 8-byte-aligned safe prefix, then every escape class: structural,
    // shorthand control, and \u-encoded control — all must round-trip.
    let source = "```\nabcdefgh\"quote\\back\ttab\rcr\x08bs\x0cff\x01ctl and 日本語テキスト\n```";
    let json = parse_json(source, ParserOptions::default());
    assert_eq!(
        json["children"][0]["value"],
        "abcdefgh\"quote\\back\ttab\rcr\x08bs\x0cff\x01ctl and 日本語テキスト\n"
    );
}

#[test]
fn long_escape_free_value_is_preserved_verbatim() {
    let body = "safe prose with no escapes at all ".repeat(40);
    let source = format!("```\n{body}\n```");
    let json = parse_json(&source, ParserOptions::default());
    assert_eq!(json["children"][0]["value"], format!("{body}\n"));
}

#[test]
fn serializes_large_ordered_list_start() {
    // CommonMark caps ordered-list markers at nine digits.
    let json = parse_json("123456789. item", ParserOptions::default());
    assert_eq!(json["children"][0]["type"], "list");
    assert_eq!(json["children"][0]["start"], 123_456_789_u32);
}
