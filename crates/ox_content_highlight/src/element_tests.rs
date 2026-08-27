//! Grammar coverage and rendered-output shape.
//!
//! Includes the custom-element cases, which are the shape documentation writes
//! most and the one a JavaScript-family grammar gets wrong: MDX allows
//! adjacent top-level elements and JavaScript does not.

use super::*;
use crate::test_support::visible_text;

#[test]
fn the_visible_text_is_exactly_the_input() {
    for (code, lang) in [
        ("const a = 1;\nlet b = \"x < y & z\";\n", "ts"),
        ("<div class=\"x\">hi</div>\n", "html"),
        ("fn main() { println!(\"{}\", 1 & 2); }\n", "rust"),
        ("echo \"a > b\" | grep -q 'x'\n", "bash"),
        ("{\n  \"a\": \"<b>\"\n}\n", "json"),
        (".a::after { content: \"'\"; }\n", "css"),
        ("export const C = () => <p a=\"1\">t</p>;\n", "tsx"),
        ("def f(x):\n    return x < 1 and \"a\"\n", "python"),
        ("func main() { fmt.Println(\"a & b\") }\n", "go"),
        ("class A { void f() { int x = 1 < 2 ? 1 : 0; } }\n", "java"),
        ("int main(void) { return 1 & 2; }\n", "c"),
        ("template <typename T> T f(T a) { return a; }\n", "cpp"),
        ("key: \"a > b\"\nlist:\n  - 1\n", "yaml"),
        ("# H\n\n*a* **b** `c` [d](http://e)\n", "md"),
        ("[package]\nname = \"demo\"\n", "toml"),
        ("@compute @workgroup_size(64)\nfn main() {\n    let x = 1;\n}\n", "wgsl"),
        ("{\n  // comment\n  \"a\": true\n}\n", "jsonc"),
        ("<script lang=\"ts\">let a = 1;</script>\n", "svelte"),
        ("---\ntitle: Demo\n---\n<h1>{title}</h1>\n", "astro"),
        ("FOO=a < b\n# comment\n", "dotenv"),
        ("SELECT name FROM t WHERE a < b AND c = 'x & y';\n", "sql"),
        ("type Query { hello: String }\n# a < b & c\n", "graphql"),
        ("FROM alpine\nRUN echo \"a < b & c > d\"\n", "dockerfile"),
        ("def f(x)\n  x < 1 && \"a & b\"\nend\n", "ruby"),
        ("<?php echo \"a < b & c > d\";\n", "php"),
        ("let x = \"a < b & c\"; in x\n", "nix"),
        ("class A { string F() { return \"a < b & c\"; } }\n", "csharp"),
        ("let s = \"a < b & c\"\n", "swift"),
        ("fun main() { val s = \"a < b & c\" }\n", "kotlin"),
        ("void main() { bool b = 1.0 < 2.0 && true; }\n", "glsl"),
        ("no trailing newline", "ts"),
        ("\n\n\n", "ts"),
        ("tabs\tand  spaces\n", "ts"),
        ("emoji 🎉 and ünïcödé\n", "ts"),
    ] {
        let html = highlight_to_html(code, lang).expect("supported");
        assert_eq!(visible_text(&html), code, "lang={lang}");
    }
}

#[test]
fn every_advertised_language_actually_builds() {
    for lang in supported_languages() {
        assert!(
            highlight_to_html("x\n", lang).is_some(),
            "{lang} is advertised but produced nothing",
        );
    }
}

#[test]
fn markdown_highlights_block_and_inline_syntax() {
    let html =
        highlight_to_html("# Title\n\nSome *emphasis* and `code` and [link](http://x).\n", "md")
            .expect("supported");

    // Block grammar: the heading.
    assert!(html.contains("--octc-syntax-token-function"), "heading: {html}");
    // Inline grammar, reachable only through the injection.
    assert!(html.contains("--octc-syntax-token-constant"), "emphasis: {html}");
    assert!(html.contains("--octc-syntax-token-string"), "code span: {html}");
    assert!(html.contains("--octc-syntax-token-link"), "link: {html}");
}

#[test]
fn markdown_highlights_a_fenced_block_in_its_own_language() {
    // The fence injection names the language the way the source spelled it,
    // so resolving injections by canonical name only would leave this plain.
    let html = highlight_to_html("```ts\nconst a = 1;\n```\n", "md").expect("supported");

    assert!(html.contains("--octc-syntax-token-keyword"), "{html}");
}

#[test]
fn mdx_highlights_custom_element_closing_tags() {
    let html = highlight_to_html(
        "<WebContainer entry=\"index.html\" title=\"Demo\">\n  npm install\n</WebContainer>\n",
        "mdx",
    )
    .expect("supported");
    let closing = html.split("&lt;/").nth(1).expect("closing tag should be present");

    assert!(closing.contains("WebContainer"), "{html}");
    assert!(closing.contains("--octc-syntax-token-constant"), "{html}");
    assert!(closing.contains("--octc-syntax-token-punctuation"), "{html}");
}

/// MDX allows adjacent top-level elements; JavaScript does not, and reads them
/// as `(<A />) < B / …` because ASI will not break before `<`. Highlighting MDX
/// as JavaScript therefore tokenised every second element as operators.
#[test]
fn adjacent_mdx_elements_tokenize_the_same_way() {
    let one = "<GitHub url=\"https://github.com/a/b/issues/1\" />\n";
    let html = highlight_to_html(&one.repeat(3), "mdx").expect("supported");
    let tokens: Vec<Vec<&str>> = html
        .split("<span class=\"line\">")
        .skip(1)
        .map(|line| {
            line.match_indices("--octc-syntax-token-").map(|(i, _)| &line[i + 20..i + 26]).collect()
        })
        .filter(|line: &Vec<&str>| !line.is_empty())
        .collect();

    assert_eq!(tokens.len(), 3, "{html}");
    assert_eq!(tokens[0], tokens[1], "second element differs: {html}");
    assert_eq!(tokens[1], tokens[2], "third element differs: {html}");
}

#[test]
fn html_highlights_tab_custom_elements() {
    let html = highlight_to_html(
        "<tabs>\n  <tab label=\"Install\">pnpm add -D @ox-content/vite-plugin</tab>\n</tabs>\n",
        "html",
    )
    .expect("supported");
    let closing = html.split("&lt;/").nth(1).expect("closing tab should be present");

    assert!(html.contains("--octc-syntax-token-string"), "{html}");
    assert!(html.contains("tabs"), "{html}");
    assert!(html.contains("tab"), "{html}");
    assert!(closing.contains("--octc-syntax-token-constant"), "{html}");
    assert!(closing.contains("--octc-syntax-token-punctuation"), "{html}");
}
