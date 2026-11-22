use super::*;

fn target() -> ApiDocEntry {
    test_entry("Target", "interface", "/repo/src/target.ts", "Target.")
}

fn rendered(value: &str, options: &MarkdownDocsOptions) -> String {
    let mut entry = greet();
    entry.tags = remarks(value);
    page(&module(vec![entry, target()]), options, "greet")
}

#[test]
fn unclosed_blocks_stay_inside_their_section() {
    for (value, closer) in [
        ("Note:\n\n```ts\nconst a = 1", "```"),
        ("~~~ts", "~~~"),
        ("````md\nx\n```", "````"),
        ("<!-- TODO", "-->"),
        ("<pre>x\n\ny", "</pre>"),
        ("<SCRIPT>x", "</script>"),
        ("<?php x", "?>"),
        ("<!DOCTYPE x", ">"),
    ] {
        let page = rendered(value, &markdown_typedoc_options());
        let closed = join3(value, "\n", closer);
        assert!(page.contains(&join2(&closed, "\n\n## Signature")), "{value}:\n{page}");
    }
}

#[test]
fn detail_blocks_keep_their_boundary_when_inline_parsing_is_skipped() {
    let body = "[Target]\n".repeat(70);
    for (opener, closer) in [(" <!--", "-->"), ("<PRE>", "</pre>"), ("   ~~~ts", "~~~")] {
        let value = join3(opener, "\n", body.trim_end());
        let expected = join3(&value, "\n", closer);
        for input in [&value, &expected] {
            let page = rendered(input, &markdown_typedoc_options());
            assert!(page.contains(&join2(&expected, "\n\n## Signature")), "{page}");
        }
    }
}

#[test]
fn a_fence_line_indented_too_far_does_not_close_it() {
    let page = rendered("```\ncode\n    ```", &markdown_typedoc_options());

    assert!(page.contains("code\n    ```\n```\n\n## Signature"), "{page}");
}

#[test]
fn html_shows_no_closer_the_author_did_not_write() {
    for value in ["<!-- TODO", "<pre>x", "```ts\nconst a = 1"] {
        let page = rendered(value, &html_typedoc_options());
        assert!(!page.contains("--&gt;"), "{value}:\n{page}");
        assert!(!page.contains("&lt;/pre&gt;"), "{value}:\n{page}");
        assert!(!page.contains("```"), "{value}:\n{page}");
    }
}

#[test]
fn closed_blocks_are_left_alone() {
    for value in [
        "```ts\nconst a = 1\n```",
        "<pre>x</pre>",
        "<!-- note -->",
        "<pre>x</style>",
        "<preview>",
        "<pre-wrap>",
        "<script-x>",
    ] {
        let page = rendered(value, &markdown_typedoc_options());
        assert!(page.contains(&join2(value, "\n\n## Signature")), "{value}:\n{page}");
    }
}

#[test]
fn markdown_link_syntax_keeps_its_source() {
    let mut entry = greet();
    entry.tags = remarks(concat!(
        "See [the docs][Target] and ![a `[Target]` b](u).\n\n",
        "- [X] done\n\n",
        "<pre>[Target]</pre>\n\n",
        "[Target]: https://example.com/guide",
    ));
    let symbol = test_entry("X", "interface", "/repo/src/x.ts", "X.");
    let page = page(&module(vec![entry, target(), symbol]), &markdown_typedoc_options(), "greet");

    assert!(page.contains("[the docs][Target]"));
    assert!(page.contains("![a `[Target]` b](u)"));
    assert!(page.contains("- [X] done"));
    assert!(page.contains("<pre>[Target]</pre>"));
    assert!(page.contains("[Target]: https://example.com/guide"));
    assert!(!page.contains("../interfaces/"));
}

#[test]
fn html_detail_headings_have_no_ids() {
    let page = rendered("## Notes\n\n## greet", &html_typedoc_options());

    assert!(page.contains("<h2>Notes</h2>"));
    assert!(page.contains("<h2>greet</h2>"));
    assert!(!page.contains("id=\"notes\""));
    assert_eq!(page.matches("id=\"greet\"").count(), 1);
}

#[test]
fn html_module_details_escape_raw_html() {
    let docs = vec![ApiDocModule {
        file: "mod".to_string(),
        description: "A module.".to_string(),
        tags: details("<img src=x onerror=alert(1)>", "<script>alert(1)</script>"),
        entries: vec![test_entry("run", "function", "/repo/src/run.ts", "Run.")],
        ..ApiDocModule::default()
    }];

    let markdown = generate_markdown(&docs, &html_typedoc_options());
    let index = markdown.get("mod/index.md").expect("the module page");
    assert_in_order(
        index,
        &["A module.", "<h2>Description</h2>", "<h2>Remarks</h2>", "</div>\n\n"],
    );
    assert!(index.contains("&lt;script&gt;"));
    assert!(!index.contains("<script>"));
    assert!(!index.contains("<img"));
}

#[test]
fn code_around_inline_tags_stays_protected() {
    for (value, kept) in [
        ("Use `{@link` and `[Target]` then }.", "`[Target]` then }."),
        ("{@see `[Target]`} and more.", "{@see `[Target]`} and"),
    ] {
        let page = rendered(value, &markdown_typedoc_options());
        assert!(page.contains(kept), "{value}:\n{page}");
    }

    let page = rendered("{@link Target | `[Target]`}", &markdown_typedoc_options());
    assert!(page.contains("[`[Target]`](../interfaces/Target.md)"));
}

#[test]
fn an_unterminated_link_tag_does_not_reach_into_a_later_block() {
    let code = "```ts\nconst a = {b: 1}; const c = [Target]\n```";
    let page = rendered(&join2("{@link Foo\n\n", code), &markdown_typedoc_options());
    assert!(page.contains(code), "{page}");

    let code = "```ts\nconst x = a | b; const y = {c: 1}\n```";
    let page = rendered(&join3("{@link Foo\n", code, "\n\nAfter"), &markdown_typedoc_options());
    assert!(page.contains(code), "{page}");
    assert!(page.contains("After\n\n## Signature"), "{page}");
}

#[test]
fn code_at_the_start_of_a_continuation_line_stays_protected() {
    let page = rendered("- item\n  `[Target]` and [Target]", &markdown_typedoc_options());

    assert!(page.contains("`[Target]` and [Target](../interfaces/Target.md)"), "{page}");
}

#[test]
fn snake_case_does_not_trip_the_nesting_guard() {
    let value = join3("[Target] ", &"a_b ".repeat(1_100), "\n\n```ts\nconst u = [Target]");
    let page = rendered(&value, &markdown_typedoc_options());

    assert!(page.contains("[Target](../interfaces/Target.md) a_b"), "{page}");
    assert!(page.contains("const u = [Target]\n```\n\n## Signature"), "{page}");
}

#[test]
fn html_code_lines_of_spaces_keep_the_html_block() {
    let page = rendered("```ts\nfirst()\n  \nsecond()\n```", &html_typedoc_options());

    assert!(page.contains("first()&#10;  \nsecond()"));
}

#[test]
fn an_indented_first_line_stays_code() {
    let value = "    const a = [Target]\n    const b = 2";
    let page = rendered(value, &markdown_typedoc_options());

    assert!(page.contains(value));
    assert!(!page.contains("(../interfaces/Target.md)"));
}

#[test]
fn deep_inline_nesting_never_reaches_the_parser() {
    let brackets = join3(&"[".repeat(20_000), "a", &"]".repeat(20_000));
    let images = "![".repeat(20_000);
    let emphasis = join3(&"*a ".repeat(20_000), "b", &" c*".repeat(20_000));
    let underscores = join3(&"_a ".repeat(20_000), "b", &" c_".repeat(20_000));
    let escaped = join3(&"[\\]".repeat(20_000), "a", &"]".repeat(20_000));
    let in_code = join3(&"[`]`".repeat(20_000), "a", &"]".repeat(20_000));
    let in_html = join3(&"[<a b=\"]\">".repeat(20_000), "a", &"]".repeat(20_000));
    let in_a_title = "[x[a](b \"]\")".repeat(20_000);
    let nested_images = join2(&"![a".repeat(40), &"]".repeat(40));
    let at_the_bracket_limit = join3(&"[".repeat(63), "a", &"]".repeat(63));
    let at_the_emphasis_limit = join3(&"*a ".repeat(512), "b", &" c*".repeat(512));
    for value in [
        brackets,
        images,
        emphasis,
        underscores,
        escaped,
        in_code,
        in_html,
        in_a_title,
        nested_images,
        at_the_bracket_limit,
        at_the_emphasis_limit,
    ] {
        let value = join2(&value, " [Target]");
        assert!(rendered(&value, &markdown_typedoc_options()).contains("## Remarks"));
        assert!(rendered(&value, &html_typedoc_options()).contains("<h4>Remarks</h4>"));
    }
}

#[test]
fn a_long_code_sample_stays_code_past_the_nesting_guard() {
    let value = join2("```ts\n", &"const a = [Target];\n".repeat(200));
    let page = rendered(value.trim_end(), &markdown_typedoc_options());

    assert!(!page.contains("../interfaces/"));
    assert!(page.contains("const a = [Target];\n```\n\n## Signature"), "{page}");

    let html = rendered(value.trim_end(), &html_typedoc_options());
    assert!(html.contains("<code class=\"language-text\">```ts\nconst a = [Target];"), "{html}");
}

#[test]
fn text_too_deep_to_parse_stays_as_written() {
    for value in [
        join2(&"[".repeat(100), " [Target] {@link Target}"),
        join2(&">".repeat(120), " [Target] {@link Target}"),
    ] {
        let page = rendered(&value, &markdown_typedoc_options());
        assert!(page.contains(&value), "{page}");
    }
}

#[test]
fn html_text_the_parser_rejects_stays_escaped() {
    let quotes = ">".repeat(120);
    let value = join3(&quotes, " x\n\n", "[click](javascript:alert(1)) <script>x</script>");
    let page = rendered(&value, &html_typedoc_options());

    assert!(!page.contains("href=\"javascript:"));
    assert!(!page.contains("<script>x"));
    assert!(page.contains("&lt;script&gt;x"));
}

#[test]
fn a_tab_after_a_list_marker_keeps_prose_links() {
    let page = rendered("-\t`a` [Target]", &markdown_typedoc_options());

    assert!(page.contains("`a` [Target](../interfaces/Target.md)"));
}
