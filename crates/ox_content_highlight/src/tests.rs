use super::*;
use crate::test_support::{assert_text_has_capture_token, visible_text};

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
    assert!(supports("fish"));
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
    assert!(supports("sql"));
    for alias in ["graphql", "gql"] {
        assert!(supports(alias), "{alias} should be supported");
    }
    for alias in ["dockerfile", "docker", "containerfile"] {
        assert!(supports(alias), "{alias} should be supported");
    }
    for alias in ["ruby", "rb"] {
        assert!(supports(alias), "{alias} should be supported");
    }
    assert!(supports("php"));
    assert!(supports("nix"));
    for alias in ["nu", "nushell"] {
        assert!(supports(alias), "{alias} should be supported");
    }
    for alias in ["csharp", "cs"] {
        assert!(supports(alias), "{alias} should be supported");
    }
    assert!(supports("swift"));
    for alias in ["kotlin", "kt"] {
        assert!(supports(alias), "{alias} should be supported");
    }
    assert!(supports("glsl"));
    assert!(supports("diff"));
    assert!(supports("patch"));
    assert!(supports("less"));
    for alias in ["xml", "svg", "xsl", "rss", "plist"] {
        assert!(supports(alias), "{alias} should be supported");
    }
    assert!(supports("lua"));
    for alias in ["hcl", "terraform", "tf"] {
        assert!(supports(alias), "{alias} should be supported");
    }
    for alias in ["makefile", "make", "mk"] {
        assert!(supports(alias), "{alias} should be supported");
    }
    assert!(supports("cmake"));
    for alias in ["vimscript", "vim"] {
        assert!(supports(alias), "{alias} should be supported");
    }
    for alias in ["powershell", "pwsh", "ps1"] {
        assert!(supports(alias), "{alias} should be supported");
    }
    for alias in ["haskell", "hs", "elixir", "ex", "exs", "scala", "sbt"] {
        assert!(supports(alias), "{alias} should be supported");
    }
    for alias in ["zig", "r", "rscript"] {
        assert!(supports(alias), "{alias} should be supported");
    }
    assert!(!supports("assembly"));
    assert!(!supports("asm"));
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
    assert_eq!(
        highlight_to_html("let x = true\n", "nu"),
        highlight_to_html("let x = true\n", "nushell")
    );
    assert_eq!(
        highlight_to_html("echo \"hi\"\n", "vim"),
        highlight_to_html("echo \"hi\"\n", "vimscript")
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
fn added_grammars_tokenize_and_escape() {
    for (code, lang) in [
        ("SELECT name FROM t WHERE a < b AND c = 'x & y';\n", "sql"),
        ("type Query { hello: String }\n# a < b & c\n", "gql"),
        ("FROM alpine\nRUN echo \"a < b & c > d\"\n", "docker"),
        ("def f(x)\n  x < 1 && \"a & b\"\nend\n", "rb"),
        ("<?php echo \"a < b & c > d\";\n", "php"),
        ("let x = \"a < b & c\"; in x\n", "nix"),
        ("let x = \"a < b & c\"\n", "nu"),
        ("class A { string F() { return \"a < b & c\"; } }\n", "cs"),
        ("let s = \"a < b & c\"\n", "swift"),
        ("fun main() { val s = \"a < b & c\" }\n", "kt"),
        ("void main() { bool b = 1.0 < 2.0 && true; }\n", "glsl"),
        ("--- a/x\n+++ b/x\n-a < b & c\n+a < b & d\n", "diff"),
        ("@c: red;\n.a::after { content: \"a < b & c\"; }\n", "less"),
        ("<?xml version=\"1.0\"?>\n<a b=\"c\">x &lt; y &amp; z</a>\n", "xml"),
        ("echo \"a < b & c\" | string upper\n", "fish"),
        ("local s = \"a < b & c\"\n", "lua"),
        ("variable \"x\" {\n  default = \"a < b & c\"\n}\n", "terraform"),
        ("# a < b & c\nall:\n\techo hi\n", "makefile"),
        ("set(NAME \"a < b & c\")\n", "cmake"),
        ("echo \"a < b & c\"\n", "vimscript"),
        ("Write-Host \"a < b & c\"\n", "powershell"),
        ("-- a < b & c\nmain = putStrLn \"hi\"\n", "haskell"),
        ("# a < b & c\ndefmodule M do\nend\n", "elixir"),
        ("// a < b & c\nobject M { val s = \"hi\" }\n", "scala"),
        ("// a < b & c\nconst x = 1;\n", "zig"),
        ("# a < b & c\nx <- 1\n", "r"),
    ] {
        let html = highlight_to_html(code, lang).expect(lang);
        assert_eq!(visible_text(&html), code, "lang={lang}");
        assert!(html.contains("--octc-syntax-token-"), "lang={lang} html={html}");
        assert!(html.contains("&lt;"), "lang={lang} html={html}");
        assert!(html.contains("&amp;"), "lang={lang} html={html}");
        if code.contains('>') {
            assert!(html.contains("&gt;"), "lang={lang} html={html}");
        }
        assert!(!html.contains("a < b"), "lang={lang} html={html}");
    }
}

#[test]
fn nix_keeps_specific_captures_over_generic_identifiers() {
    let code = r#"{
  programs.nix-secure-enclave-key = {
    enable = true;
    identities.git-signing = {
      keyFile = "~/.ssh/id_enclave_key";
      source = import ./module.nix;
    };
  };
}
"#;

    let html = highlight_to_html(code, "nix").expect("nix is supported");
    assert_eq!(visible_text(&html), code);
    assert_text_has_capture_token(&html, "programs", "property");
    assert_text_has_capture_token(&html, "nix-secure-enclave-key", "property");
    assert_text_has_capture_token(&html, "enable", "property");
    assert_text_has_capture_token(&html, "identities", "property");
    assert_text_has_capture_token(&html, "git-signing", "property");
    assert_text_has_capture_token(&html, "keyFile", "property");
    assert_text_has_capture_token(&html, "true", "constant.builtin");
    assert_text_has_capture_token(&html, "import", "function.builtin");
    assert_text_has_capture_token(&html, "./module.nix", "string.special");
    assert_text_has_capture_token(&html, "\"~/.ssh/id_enclave_key\"", "string");
}

#[test]
fn nix_covers_comments_keywords_parameters_calls_paths_and_interpolation() {
    let code = r#"let
  makePath = name: ./packages/${name};
in with builtins; {
  url = https://example.com/pkg;
  names = map (name: makePath name) [ "core" ];
  # production corpus shape
}
"#;

    let html = highlight_to_html(code, "nix").expect("nix is supported");
    assert_eq!(visible_text(&html), code);
    assert_text_has_capture_token(&html, "let", "keyword");
    assert_text_has_capture_token(&html, "in", "keyword");
    assert_text_has_capture_token(&html, "with", "keyword");
    assert_text_has_capture_token(&html, "makePath", "property");
    assert_text_has_capture_token(&html, "name", "variable.parameter");
    assert_text_has_capture_token(&html, "builtins", "constant.builtin");
    assert_text_has_capture_token(&html, "map", "function.builtin");
    assert_text_has_capture_token(&html, "https://example.com/pkg", "text.uri");
    assert_text_has_capture_token(&html, "# production corpus shape", "comment");
    assert_text_has_capture_token(&html, "${", "punctuation.special");
}

#[test]
fn nushell_highlights_native_language_constructs() {
    let code = r#"let expensive = open --raw usage.json
  | where cost > 10
  | get project
  | uniq

def summarise [rows: list<record>] {
  $rows | group-by project
  { project: "core", active: true }
  ls err> errors.log
  # production corpus shape < &
}
"#;

    let html = highlight_to_html(code, "nu").expect("nu is supported");
    assert_eq!(visible_text(&html), code);
    assert_text_has_capture_token(&html, "let", "keyword");
    assert_text_has_capture_token(&html, "def", "keyword");
    assert_text_has_capture_token(&html, "open", "function.builtin");
    assert_text_has_capture_token(&html, "raw", "attribute");
    assert_text_has_capture_token(&html, "|", "operator");
    assert_text_has_capture_token(&html, ">", "operator");
    assert_text_has_capture_token(&html, "err>", "operator");
    assert_text_has_capture_token(&html, "expensive", "variable.parameter");
    assert_text_has_capture_token(&html, "rows", "variable.parameter");
    assert_text_has_capture_token(&html, "project", "property");
    assert_text_has_capture_token(&html, "list", "type");
    assert_text_has_capture_token(&html, "record", "type");
    assert_text_has_capture_token(&html, "\"core\"", "string");
    assert_text_has_capture_token(&html, "true", "constant.builtin");
    assert_text_has_capture_token(&html, "# production corpus shape < &", "comment");
}

#[test]
fn empty_input_produces_a_well_formed_block() {
    let html = highlight_to_html("", "ts").expect("supported");
    assert_eq!(
        html,
        "<pre class=\"ox-highlight css-variables\" style=\"background-color:var(--octc-syntax-background, #0d1117);color:var(--octc-syntax-foreground, #e6edf3)\" tabindex=\"0\"><code><span class=\"line\"></span></code></pre>"
    );
}
