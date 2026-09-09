#![allow(clippy::literal_string_with_formatting_args)]
use super::{Case, case};
use ox_content_transform::*;

pub fn cases() -> Vec<Case> {
    let mut cases = vec![];
    macro_rules! flag {
        ($field:ident, $source:expr, $marker:expr) => {
            cases.push(case(
                stringify!($field),
                $source,
                $marker,
                TransformOptions { $field: Some(true), ..Default::default() },
            ));
        };
    }
    macro_rules! feature {
        ($field:ident, $source:expr, $marker:expr) => {
            cases.push(case(
                stringify!($field),
                $source,
                $marker,
                TransformOptions { $field: Some(Default::default()), ..Default::default() },
            ));
        };
    }
    flag!(
        gfm,
        "~~deleted~~ https://example.com\n\n- [x] Task\n\n| A | B |\n|---|---|\n|1|2|\n\n",
        "<table>"
    );
    flag!(mdx, "<Demo label=\"hello\" />\n\n", "Demo");
    flag!(footnotes, "Sentence[^note].\n\n[^note]: Note text.\n\n", "footnote");
    flag!(task_lists, "- [x] Done\n- [ ] Pending\n\n", "checkbox");
    flag!(tables, "| A | B |\n|---|---|\n|1|2|\n\n", "<table>");
    flag!(strikethrough, "~~removed~~ unchanged\n\n", "<del>");
    flag!(autolinks, "https://example.com/docs\n\n", "href=");
    flag!(superscript, "x^2^ + y^3^\n\n", "<sup>");
    flag!(subscript, "H~2~O\n\n", "<sub>");
    flag!(smart_punctuation, "\"quoted\" -- --- ...\n\n", "…");
    flag!(heading_attributes, "## Heading {#custom .wide}\n\n", "custom");
    flag!(cjk_emphasis, "日本語**強調**です。\n\n", "<strong>");
    flag!(source_spans, "## Heading\n\nParagraph.\n\n", "data-source");
    flag!(heading_permalinks, "## Heading\n\n", "header-anchor");
    flag!(
        code_annotations,
        "```ts annotate=highlight:1\nconst a = 1;\nconst b = 2;\n```\n\n",
        "highlighted"
    );
    feature!(wiki_links, "[[Guide]] and [[API|Reference]]\n\n", "href=");
    feature!(emoji_shortcodes, ":rocket: :smile: :unknown: :heart:\n\n", "🚀");
    feature!(math, "$x^2$\n\n$$\ny = x + 1\n$$\n\n", "math");
    feature!(attributes, "A [link](/guide){.wide target=_blank}\n\n", "wide");
    feature!(badges, "Status {badge:tip}Stable{/badge}\n\n", "ox-badge");
    feature!(not_by_ai, "Authored <NotByAI />.\n\n", "ox-not-by-ai");
    feature!(keyboard_keys, "Press {kbd:Ctrl+K} then {kbd:Enter}.\n\n", "<kbd");
    feature!(definition_lists, "Term\n: Definition with **emphasis**.\n\n", "<dl");
    feature!(magic_links, "{link:https://example.com}\n\n", "ox-magic-link");
    feature!(images, "![A caption](/image.png \"Caption\")\n\n", "<figure");
    feature!(
        sanitize,
        "<p>Safe <a href=\"https://example.com\">link</a><script>bad()</script></p>\n\n",
        "Safe"
    );
    for (name, count, first_use_only) in [
        ("abbreviations_8", 8, false),
        ("abbreviations_256", 256, false),
        ("abbreviations_first_use", 256, true),
    ] {
        let mut terms = (0..count)
            .map(|i| (format!("TERM{i}"), format!("Term number {i}")))
            .collect::<rustc_hash::FxHashMap<_, _>>();
        terms.insert("API".into(), "Application programming interface".into());
        cases.push(case(
            name,
            "The API uses TERM0 and TERM7. API and unknown prose continue.\n\n",
            "<abbr",
            TransformOptions {
                abbreviations: Some(AbbreviationsOptions {
                    enabled: Some(true),
                    terms: Some(terms),
                    first_use_only: Some(first_use_only),
                }),
                ..Default::default()
            },
        ));
    }
    cases.push(case(
        "abbreviations_inline",
        "*[API]: Application programming interface\n\nThe API is used.\n\n",
        "<abbr",
        TransformOptions {
            abbreviations: Some(AbbreviationsOptions::default()),
            ..Default::default()
        },
    ));
    cases
}
