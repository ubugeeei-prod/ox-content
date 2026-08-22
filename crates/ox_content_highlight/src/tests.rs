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
    assert_eq!(
        highlight_to_html("const a = 1;", "ts"),
        highlight_to_html("const a = 1;", "typescript")
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
    assert!(html.starts_with("<pre class=\"shiki css-variables\" style=\"background-color:var(--octc-shiki-background, #0d1117);color:var(--octc-shiki-foreground, #e6edf3)\" tabindex=\"0\"><code>"));
    assert!(html.ends_with("</code></pre>"));
    assert!(html.contains("<span class=\"line\">"));
}

#[test]
fn keywords_and_comments_get_their_own_variables() {
    let html = highlight_to_html("// note\nconst a = 1;\n", "ts").expect("supported");
    assert!(html.contains("--octc-shiki-token-comment"), "{html}");
    assert!(html.contains("--octc-shiki-token-keyword"), "{html}");
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
fn plain_text_renders_without_tokenizing() {
    // `text` is the third most common fence tag in the documentation corpus.
    // Declining it would send those pages to the fallback highlighter purely
    // to render prose, which is the cost this avoids.
    for lang in ["text", "plaintext", "txt", "plain", "TEXT"] {
        assert!(supports(lang), "{lang} should be supported");
    }

    let html = highlight_to_html("a < b\nline two\n", "text").expect("supported");
    assert!(html.starts_with("<pre class=\"shiki css-variables\""), "{html}");
    assert!(html.contains("&lt;"), "{html}");
    // Nothing is tokenized, so no token variable may appear.
    assert!(!html.contains("--octc-shiki-token-"), "{html}");
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
        "<pre class=\"shiki css-variables\" style=\"background-color:var(--octc-shiki-background, #0d1117);color:var(--octc-shiki-foreground, #e6edf3)\" tabindex=\"0\"><code><span class=\"line\"></span></code></pre>"
    );
}
