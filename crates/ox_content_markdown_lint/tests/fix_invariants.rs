//! Safety properties that must hold for every document and every rule profile.
//!
//! The corpus is deliberately hostile: mixed line endings, astral and
//! combining characters, control bytes, unterminated constructs and deep
//! nesting. Nothing here pins a specific diagnostic; it pins that positions
//! stay inside the document, fixes stay on character boundaries and converge,
//! and results do not depend on which entry point or thread produced them.

use ox_content_markdown_lint::*;

const CORPUS: &[&str] = &[
    "",
    "\n",
    " ",
    "\u{feff}",
    "\u{feff}# BOM title\r\n\r\ntext  \r\n",
    "# 日本語の見出し。\n\n日本語の本文です。。 本文 本文\n\n- 項目\n- 項目\n",
    "# 👩‍💻 Title!\n\n👨‍👩‍👧‍👦 family family  \nnext 😀😀 \n",
    "Cafe\u{301} cafe\u{301} e\u{301}\u{301} with with\n",
    "# A\r\n\r\n\r\n\r\ntext  \r\nmore \r\n\r\n\r\n",
    "a\rb\r\rc with with\r\n",
    "a\r\nb \nc\rd",
    "\r\n\r\n\r\n",
    "   ",
    "\t",
    " \n \n",
    "text\n\n\n  \n",
    "text  \n  \n\n",
    "a\n \n \n\t\n\nb  \n   ",
    "\n\n\ntext",
    "  \n#  h  #  \n  \n##   h\n",
    "- a  \n\n  \n- b\n   \n\n\n1. c \n",
    "javascript github Github.com JAVASCRIPT java-script\n",
    "\ttab\n\n\t\tcode \n\ntext\twith\ttabs \n| a\t| b |\n",
    "nul\0byte \n\0\n\u{7}bell bell\n",
    "> - a\n>   - b\n>     > c c\n>\n> 1. x\n> 3. y \n",
    "```rust\nfn main() {}  \n",
    "~~~\n",
    "````md\n```\nwith with\n",
    "---\ntitle: TODO TODO!!\ntags: [a, b]\n---\n\n## Sub\n\ntext \n",
    "---\ntitle: x\n",
    "---\n---\n",
    "import X from './x'\n\nexport const a = 1\n\n<X a={1}>with with {b}</X>\n\n{/* note */}\n",
    "<div>\n\n**bold**\n\n</div>\n\n<!-- comment \n spans --> tail tail\n\n<br/>\n",
    "[a][b] [c][] [d] [ e ](f) ![](g) ![h]()\n\n[b]: <url> \"title\"\n[c]: /x\n[c]: /y\n",
    "| a | b |\n|---|:-:|\n| 1 | 2 | 3 |\n|x|\ntext\n",
    "#\n##  \n### a ###\n####  b  #\n#no space\nSetext!\n===\n\nSetext2\n---\n",
    "** a ** * b * __ c __ _d_ *e* **f** ***g*** __h__\n",
    "`` ` `` ` a ` ``b`` `c\n",
    "a  \nb   \nc \n  \n\n\n\nd",
    "a\u{a0}\n b\u{3000}\n\u{2028}c\u{2029} \n",
    "<!-- oxlint-disable -->\nwith with\n<!-- markdownlint-disable -->\ntext \n<!-- markdownlint-restore -->\n<!-- markdownlint-configure-file {\"MD013\":{\"line_length\":5}} -->\n",
    "مرحبا مرحبا بالعالم \n\n𝒳𝒳 𝒳𝒳 😀😀\n",
    "???\n!!!\n...\n、、、\n",
    "text[^1] [^1]\n\n[^1]: note note\n",
    "* a\n+ b\n- c\n   - d\n1. e\n1. f\n3) g\n",
    "Term\n: definition\n\n***\n___\n- - -\n\n    indented \n",
    "https://example.com/a?b=c!! <https://example.com> www.example.com user@example.com\n",
    "Javascript javascript JAVASCRIPT `Javascript` [Javascript](/Javascript)\n",
];

fn generated() -> Vec<String> {
    vec![
        "word ".repeat(2000),
        "語".repeat(1500) + " 語\n",
        ">".repeat(60) + " deep deep\n",
        "- ".repeat(40) + "x\n",
        "[".repeat(200) + &"]".repeat(200),
        "*".repeat(400) + "\n" + &"_".repeat(400),
        "`".repeat(301) + " x\n",
        "\n".repeat(300),
        "# h\n".repeat(200),
        "| a ".repeat(120) + "|\n" + &"|---".repeat(120) + "|\n",
    ]
}

fn profiles() -> Vec<(&'static str, MarkdownLintOptions)> {
    [
        ("native", r#"{"rules":{"spellcheck":false}}"#),
        ("spellcheck", r#"{"languages":["en","ja","zh","fr","de","pl"]}"#),
        (
            "strict",
            r#"{"rules":{"spellcheck":false,"emptyHeadings":true,"firstHeadingH1":true,"singleH1":true,"codeFenceLanguage":true,"codeFenceClosed":true,"emptyLinks":true,"imageAlt":true,"finalNewline":true,"maxConsecutiveBlankLines":2}}"#,
        ),
        ("markdownlint", r#"{"markdownlint":{}}"#),
        (
            "markdownlint options",
            r##"{"markdownlint":{"MD009":{"br_spaces":0,"strict":true},"MD012":{"maximum":2},"MD013":{"line_length":20,"strict":true},"MD044":{"names":["JavaScript","GitHub"]},"MD043":{"headings":["# A","*"]}}}"##,
        ),
        ("markdownlint no inline", r#"{"markdownlint":true,"noInlineConfig":true}"#),
        (
            "prose",
            r#"{"markdownlint":{"default":false},"rules":{"repeatedWords":true,"repeatedPunctuation":true},"textRules":{"sentenceLength":10,"maxTen":0,"noTodo":true,"noExclamationQuestionMark":true,"terminology":[{"term":"Javascript","replacement":"JavaScript"},{"term":"語","replacement":"言葉"}]},"severities":{"repeated-word":"error","no-todo":"info","sentence-length":"off"}}"#,
        ),
        ("mdx", r#"{"mdx":true,"rules":{"spellcheck":false},"markdownlint":{}}"#),
    ]
    .into_iter()
    .map(|(name, json)| (name, serde_json::from_str(json).unwrap()))
    .collect()
}

fn documents() -> Vec<String> {
    CORPUS.iter().map(ToString::to_string).chain(generated()).collect()
}

fn snapshot(result: &MarkdownLintResult) -> serde_json::Value {
    serde_json::to_value(&result.diagnostics).unwrap()
}

fn label(profile: &str, source: &str) -> String {
    format!("{profile}: {:?}", source.chars().take(60).collect::<String>())
}

#[test]
fn diagnostics_stay_inside_the_document_and_counts_match_severities() {
    for (profile, options) in profiles() {
        let linter = MarkdownLinter::new(Some(options));
        for source in documents() {
            let result = linter.lint(&source);
            let lines = source.split('\n').count() as u32;
            let mut previous = (0, 0, String::new());
            for value in &result.diagnostics {
                let at = label(profile, &source);
                assert!(value.line >= 1 && value.column >= 1, "{at}: {value:?}");
                assert!(value.line <= lines, "{at}: {value:?}");
                assert!(value.end_line <= lines, "{at}: {value:?}");
                assert!(
                    (value.end_line, value.end_column) >= (value.line, value.column),
                    "{at}: {value:?}"
                );
                assert!(!value.rule_id.is_empty() && !value.message.is_empty(), "{at}");
                assert!(matches!(value.severity.as_str(), "error" | "warning" | "info"), "{at}");
                let key = (value.line, value.column, value.rule_id.clone());
                assert!(previous <= key, "{at}: unsorted {previous:?} > {key:?}");
                previous = key;
            }
            let count = |severity: &str| {
                result.diagnostics.iter().filter(|v| v.severity == severity).count() as u32
            };
            assert_eq!(
                (result.error_count, result.warning_count, result.info_count),
                (count("error"), count("warning"), count("info")),
                "{}",
                label(profile, &source)
            );
        }
    }
}

#[test]
fn fixes_are_in_bounds_on_char_boundaries_and_converge_in_one_pass() {
    let mut failures = Vec::new();
    for (profile, options) in profiles() {
        let linter = MarkdownLinter::new(Some(options));
        for source in documents() {
            let at = label(profile, &source);
            let before = linter.lint_without_mask(&source);
            for fix in before.diagnostics.iter().filter_map(|v| v.fix.as_ref()) {
                let (start, end) = (fix.start as usize, fix.end as usize);
                assert!(start <= end && end <= source.len(), "{at}: {fix:?}");
                assert!(source.is_char_boundary(start), "{at}: {fix:?}");
                assert!(source.is_char_boundary(end), "{at}: {fix:?}");
            }
            let fixed = linter.fix(&source);
            let fixable = before.diagnostics.iter().filter(|v| v.fix.is_some()).count();
            assert_eq!(fixed.applied_fixes == 0, fixable == 0, "{at}");
            assert!(fixed.applied_fixes as usize <= fixable, "{at}");
            if fixed.applied_fixes == 0 {
                assert_eq!(fixed.output, source, "{at}");
            }
            // The reported result always describes the returned document.
            assert_eq!(
                snapshot(&fixed.result),
                snapshot(&linter.lint_without_mask(&fixed.output)),
                "{at}"
            );
            let again = linter.fix(&fixed.output);
            if again.applied_fixes != 0 || again.output != fixed.output {
                failures.push(format!("{at}: {:?} -> {:?}", fixed.output, again.output));
            }
        }
    }
    assert!(failures.is_empty(), "fixes did not converge:\n{}", failures.join("\n"));
}

#[test]
fn results_are_identical_across_entry_points_runs_and_batch_sizes() {
    let documents = documents();
    for (profile, options) in profiles() {
        let linter = MarkdownLinter::new(Some(options.clone()));
        let serial: Vec<_> = documents.iter().map(|source| linter.lint(source)).collect();
        for (source, expected) in documents.iter().zip(&serial) {
            let at = label(profile, source);
            assert_eq!(snapshot(&linter.lint(source)), snapshot(expected), "{at}");
            assert_eq!(snapshot(&linter.lint_without_mask(source)), snapshot(expected), "{at}");
            assert!(linter.lint_without_mask(source).masked_document.is_empty(), "{at}");
            assert_eq!(
                serde_json::to_value(lint_markdown(source, Some(options.clone()))).unwrap(),
                serde_json::to_value(expected).unwrap(),
                "{at}"
            );
        }
        // Seven documents stay on the calling thread; the full corpus fans out.
        for batch in [&documents[..7], &documents[..8], &documents[..]] {
            let parallel = lint_markdown_documents(batch, Some(options.clone()));
            assert_eq!(parallel.len(), batch.len());
            for (index, (actual, expected)) in parallel.iter().zip(&serial).enumerate() {
                assert_eq!(
                    serde_json::to_value(actual).unwrap(),
                    serde_json::to_value(expected).unwrap(),
                    "{profile}: document {index} of {}",
                    batch.len()
                );
            }
        }
        assert!(lint_markdown_documents(&[], Some(options)).is_empty());
    }
}

#[test]
fn the_spellcheck_mask_preserves_utf16_length_and_line_structure() {
    for (profile, options) in profiles() {
        let linter = MarkdownLinter::new(Some(options));
        for source in documents() {
            let masked = linter.lint(&source).masked_document;
            let at = label(profile, &source);
            let units = |text: &str| text.encode_utf16().count();
            assert_eq!(units(&masked), units(&source), "{at}");
            assert_eq!(masked.matches('\n').count(), source.matches('\n').count(), "{at}");
            for (original, visible) in source.split('\n').zip(masked.split('\n')) {
                assert_eq!(units(visible), units(original), "{at}");
            }
        }
    }
}
