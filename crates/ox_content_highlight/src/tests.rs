use super::*;

#[test]
fn unknown_language_is_left_to_the_caller() {
    assert!(highlight_to_html("hello", "brainfuck").is_none());
    assert!(!supports("brainfuck"));
}

#[test]
fn aliases_resolve_to_the_same_grammar() {
    for alias in ["typescript", "ts", "cts", "mts"] {
        assert!(supports(alias), "{alias} should be supported");
    }
    for alias in ["bash", "sh", "shell", "zsh"] {
        assert!(supports(alias), "{alias} should be supported");
    }
    for alias in ["jsonc", "json5", "webmanifest"] {
        assert!(supports(alias), "{alias} should be supported");
    }
    for alias in ["vue", "svelte", "astro", "angular"] {
        assert!(supports(alias), "{alias} should be supported");
    }
    for alias in ["flow", "javascriptreact", "typescriptreact"] {
        assert!(supports(alias), "{alias} should be supported");
    }
    assert!(supports("mdx"));
    assert_eq!(
        highlight_to_html("const a = 1;", "ts"),
        highlight_to_html("const a = 1;", "typescript")
    );
    assert_eq!(highlight_to_html("{\"a\":1}", "jsonc"), highlight_to_html("{\"a\":1}", "json"));
    assert_eq!(
        highlight_to_html("<script>const a = 1;</script>\n", "vue"),
        highlight_to_html("<script>const a = 1;</script>\n", "html")
    );
    assert_eq!(
        highlight_to_html("export const C = () => <p />;\n", "typescriptreact"),
        highlight_to_html("export const C = () => <p />;\n", "tsx")
    );
}

#[test]
fn language_matching_ignores_case() {
    assert!(supports("TypeScript"));
    assert!(supports("JSON"));
}

#[test]
fn wraps_the_block_the_way_the_theme_expects() {
    let html = highlight_to_html("const a = 1;\n", "ts").expect("supported");
    assert!(html.starts_with("<pre class=\"ox-highlight css-variables\" style=\"background-color:var(--octc-syntax-background, #0d1117);color:var(--octc-syntax-foreground, #e6edf3)\" tabindex=\"0\"><code>"));
    assert!(html.ends_with("</code></pre>"));
    assert!(html.contains("<span class=\"line\">"));
}

#[test]
fn keywords_and_comments_get_their_own_variables() {
    let html = highlight_to_html("// note\nconst a = 1;\n", "ts").expect("supported");
    assert!(html.contains("--octc-syntax-token-comment"), "{html}");
    assert!(html.contains("--octc-syntax-token-keyword"), "{html}");
}

#[test]
fn every_line_is_its_own_span() {
    let html = highlight_to_html("const a = 1;\nconst b = 2;\n", "ts").expect("supported");
    assert_eq!(html.matches("<span class=\"line\">").count(), 3, "{html}");
    // Two content lines plus the trailing empty one the line-number CSS
    // counts against.
    assert!(html.contains("</span>\n<span class=\"line\"></span></code></pre>"), "{html}");
}

#[test]
fn source_without_a_trailing_newline_still_closes_its_line() {
    let html = highlight_to_html("const a = 1;", "ts").expect("supported");
    assert_eq!(html.matches("<span class=\"line\">").count(), 1, "{html}");
    assert!(html.ends_with("</span></code></pre>"), "{html}");
}

#[test]
fn html_significant_bytes_are_escaped() {
    let html = highlight_to_html("const s = \"a < b & c > d\";\n", "ts").expect("supported");
    assert!(html.contains("&lt;"), "{html}");
    assert!(html.contains("&amp;"), "{html}");
    assert!(html.contains("&gt;"), "{html}");
    assert!(!html.contains("a < b"), "{html}");
}

/// Strips the generated markup and unescapes, leaving what a reader sees.
///
/// Highlighting may split a token across any number of spans, so the only
/// stable contract is that the visible text still equals the input. Asserting
/// on span boundaries instead would pin an implementation detail of whichever
/// grammar happens to be in use.
fn visible_text(html: &str) -> String {
    let mut text = String::new();
    let mut rest = html;
    while let Some(open) = rest.find('<') {
        text.push_str(&rest[..open]);
        let Some(close) = rest[open..].find('>') else { break };
        rest = &rest[open + close + 1..];
    }
    text.push_str(rest);
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
}

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
    assert!(closing.contains("--octc-syntax-token-function"), "{html}");
    assert!(closing.contains("--octc-syntax-token-punctuation"), "{html}");
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

#[test]
fn plain_text_renders_without_tokenizing() {
    // `text` is the third most common fence tag in the documentation corpus.
    // Declining it would send those pages to the fallback highlighter purely
    // to render prose, which is the cost this avoids.
    for lang in [
        "text",
        "plaintext",
        "txt",
        "plain",
        "dotenv",
        "env",
        ".env",
        "gitignore",
        "dockerignore",
        "npmrc",
        "yarnrc",
        "properties",
        "ini",
        "conf",
        "config",
        "TEXT",
    ] {
        assert!(supports(lang), "{lang} should be supported");
    }

    let html = highlight_to_html("a < b\nline two\n", "text").expect("supported");
    assert!(html.starts_with("<pre class=\"ox-highlight css-variables\""), "{html}");
    assert!(html.contains("&lt;"), "{html}");
    // Nothing is tokenized, so no token variable may appear.
    assert!(!html.contains("--octc-syntax-token-"), "{html}");
    assert_eq!(html.matches("<span class=\"line\">").count(), 3, "{html}");
}

#[test]
fn plain_text_keeps_the_input_verbatim() {
    for code in ["a < b & c > d\n", "\ttabbed\n\n", "🎉 ünï\n", "no newline"] {
        let html = highlight_to_html(code, "text").expect("supported");
        assert_eq!(visible_text(&html), code);
    }
}

#[test]
fn empty_input_produces_a_well_formed_block() {
    let html = highlight_to_html("", "ts").expect("supported");
    assert_eq!(
        html,
        "<pre class=\"ox-highlight css-variables\" style=\"background-color:var(--octc-syntax-background, #0d1117);color:var(--octc-syntax-foreground, #e6edf3)\" tabindex=\"0\"><code><span class=\"line\"></span></code></pre>"
    );
}
