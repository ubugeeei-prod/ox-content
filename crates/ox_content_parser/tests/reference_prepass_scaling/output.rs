//! Exact output for small documents around the pre-pass chunk: every
//! candidate in a chunk now reads from one shared copy (or from the source
//! itself), so these pin what each of them resolved to when it was handed a
//! copy of its own.

use ox_content_parser::ParserOptions;

use super::{render, render_with};

#[test]
fn definitions_interrupted_by_each_kind_of_block_still_resolve() {
    for (source, expected) in [
        (
            "[a]: /a\n***\n[b]: /b\n\n[1][a] [2][b]\n",
            "<hr>\n<p><a href=\"/a\">1</a> <a href=\"/b\">2</a></p>",
        ),
        (
            "[a]: /a\n# h\n[b]: /b\n\n[1][a] [2][b]\n",
            "<h1 id=\"h\">h</h1>\n<p><a href=\"/a\">1</a> <a href=\"/b\">2</a></p>",
        ),
        (
            "[a]: /a\n```\ncode\n```\n[b]: /b\n\n[1][a] [2][b]\n",
            "<pre><code>code\n</code></pre>\n<p><a href=\"/a\">1</a> <a href=\"/b\">2</a></p>",
        ),
        (
            "[a]: /a\nh\n---\n[b]: /b\n\n[1][a] [2][b]\n",
            "<h2 id=\"h\">h</h2>\n<p><a href=\"/a\">1</a> <a href=\"/b\">2</a></p>",
        ),
        (
            "[a]: /a\nh\n===\n[b]: /b\n\n[1][a] [2][b]\n",
            "<h1 id=\"h\">h</h1>\n<p><a href=\"/a\">1</a> <a href=\"/b\">2</a></p>",
        ),
        // A failed candidate between two definitions.
        (
            "[a]: /a\n[y]\n***\n[b]: /b\n\n[1][a] [2][b] [y]\n",
            "<p>[y]</p>\n<hr>\n<p><a href=\"/a\">1</a> <a href=\"/b\">2</a> [y]</p>",
        ),
        // The first definition of a label wins, wherever the second one is.
        ("[a]: /first\n***\n[a]: /second\n\n[1][a]\n", "<hr>\n<p><a href=\"/first\">1</a></p>"),
        // References ahead of the definitions, and no final line ending.
        (
            "[1][a] [2][b] [3][c]\n\n[a]: /a\n***\n[b]: /b\n# h\n[c]: /c",
            "<p><a href=\"/a\">1</a> <a href=\"/b\">2</a> <a href=\"/c\">3</a></p>\n<hr>\n<h1 id=\"h\">h</h1>",
        ),
        // Up to three spaces of indentation; four make it code.
        (
            " [a]: /a\n***\n  [b]: /b\n***\n   [c]: /c\n***\n    [d]: /d\n\n[1][a] [2][b] [3][c] [4][d]\n",
            "<hr>\n<hr>\n<hr>\n<pre><code>[d]: /d\n</code></pre>\n<p><a href=\"/a\">1</a> <a href=\"/b\">2</a> <a href=\"/c\">3</a> [4][d]</p>",
        ),
    ] {
        assert_eq!(render(source), expected, "for {source:?}");
    }
}

#[test]
fn lines_that_cannot_start_a_definition_still_do_not() {
    for (source, expected) in [
        // A definition cannot interrupt a paragraph, a lazy quote line, or a
        // list item's text, and the inside of a code block is never one.
        ("text\n[a]: /a\n\n[1][a]\n", "<p>text\n[a]: /a</p>\n<p>[1][a]</p>"),
        (
            "[a]: /a\ntext\n[b]: /b\n\n[1][a] [2][b]\n",
            "<p>text\n[b]: /b</p>\n<p><a href=\"/a\">1</a> [2][b]</p>",
        ),
        (
            "[a]: /a\n> quote\n[b]: /b\n\n[1][a] [2][b]\n",
            "<blockquote>\n<p>quote\n[b]: /b</p>\n</blockquote>\n<p><a href=\"/a\">1</a> [2][b]</p>",
        ),
        (
            "[a]: /a\n- item\n[b]: /b\n\n[1][a] [2][b]\n",
            "<ul>\n<li>item\n[b]: /b</li>\n</ul>\n<p><a href=\"/a\">1</a> [2][b]</p>",
        ),
        (
            "[a]: /a\n    code\n[b]: /b\n\n[1][a] [2][b]\n",
            "<pre><code>code\n</code></pre>\n<p><a href=\"/a\">1</a> [2][b]</p>",
        ),
        (
            "[a]: /a\n<div>\n[b]: /b\n</div>\n\n[1][a] [2][b]\n",
            "<div>\n[b]: /b\n</div>\n<p><a href=\"/a\">1</a> [2][b]</p>",
        ),
        ("```\n[a]: /a\n```\n\n[1][a]\n", "<pre><code>[a]: /a\n</code></pre>\n<p>[1][a]</p>"),
        (
            "[a]: /a\n~~~\n[b]: /b\n~~~\n[c]: /c\n\n[1][a] [2][b] [3][c]\n",
            "<pre><code>[b]: /b\n</code></pre>\n<p><a href=\"/a\">1</a> [2][b] <a href=\"/c\">3</a></p>",
        ),
        (
            "[a]: /a\n```\n[b]: /b\n\n[1][a] [2][b]\n",
            "<pre><code>[b]: /b\n\n[1][a] [2][b]\n</code></pre>",
        ),
        (
            "> ```\n> [a]: /a\n> ```\n> [b]: /b\n\n[1][a] [2][b]\n",
            "<blockquote>\n<pre><code>[a]: /a\n</code></pre>\n</blockquote>\n<p>[1][a] <a href=\"/b\">2</a></p>",
        ),
    ] {
        assert_eq!(render(source), expected, "for {source:?}");
    }
}

#[test]
fn titles_and_labels_that_span_lines_are_read_from_the_shared_chunk() {
    for (source, expected) in [
        (
            "[a]: /a\n  'title'\n***\n[b]: /b \"t\"\n\n[1][a] [2][b]\n",
            "<hr>\n<p><a href=\"/a\" title=\"title\">1</a> <a href=\"/b\" title=\"t\">2</a></p>",
        ),
        (
            "[a\nb]: /ab\n***\n[c]:\n/c\n\n[1][a b] [2][c]\n",
            "<hr>\n<p><a href=\"/ab\">1</a> <a href=\"/c\">2</a></p>",
        ),
        // An unclosed title is not a title, and takes its definition with it.
        (
            "[a]: /a 'open\n***\n[b]: /b\n\n[1][a] [2][b]\n",
            "<p>[a]: /a &#39;open</p>\n<hr>\n<p>[1][a] <a href=\"/b\">2</a></p>",
        ),
        // A title may run across the lines between two definitions.
        (
            "[a]: /a\n'\n***\n[b]: /b\n'\n\n[1][a] [2][b]\n",
            "<p><a href=\"/a\" title=\"\n***\n[b]: /b\n\">1</a> [2][b]</p>",
        ),
        // A line ending inside a title is a `\n` in the link, whatever the
        // source used.
        (
            "[a]: /a \"t1\r\nt2\"\r\n***\r\n[b]: /b\r\n\r\n[1][a] [2][b]\r\n",
            "<hr>\n<p><a href=\"/a\" title=\"t1\nt2\">1</a> <a href=\"/b\">2</a></p>",
        ),
        (
            "[a]: /a 't1\rt2'\r***\r[b]: /b\r\r[1][a] [2][b]\r",
            "<hr>\n<p><a href=\"/a\" title=\"t1\nt2\">1</a> <a href=\"/b\">2</a></p>",
        ),
    ] {
        assert_eq!(render(source), expected, "for {source:?}");
    }
}

#[test]
fn definitions_inside_block_quotes_and_lists_still_resolve() {
    for (source, expected) in [
        ("> [a]: /a\n\n[1][a]\n", "<blockquote>\n</blockquote>\n<p><a href=\"/a\">1</a></p>"),
        (
            "> [a]: /a\n> ***\n> [b]: /b\n\n[1][a] [2][b]\n",
            "<blockquote>\n<hr>\n</blockquote>\n<p><a href=\"/a\">1</a> <a href=\"/b\">2</a></p>",
        ),
        (
            "> > [a]: /a\n> > # h\n> > [b]: /b\n\n[1][a] [2][b]\n",
            "<blockquote>\n<blockquote>\n<h1 id=\"h\">h</h1>\n</blockquote>\n</blockquote>\n<p><a href=\"/a\">1</a> <a href=\"/b\">2</a></p>",
        ),
        (
            "   > [a]: /a\n   > ***\n   > [b]: /b\n\n[1][a] [2][b]\n",
            "<blockquote>\n<hr>\n</blockquote>\n<p><a href=\"/a\">1</a> <a href=\"/b\">2</a></p>",
        ),
        (
            "> [a]: /a\n> \"title\"\n> ***\n> [b]: /b\n\n[1][a] [2][b]\n",
            "<blockquote>\n<hr>\n</blockquote>\n<p><a href=\"/a\" title=\"title\">1</a> <a href=\"/b\">2</a></p>",
        ),
        // A bare `>` is a blank line inside the quote and ends the chunk.
        (
            "> [a]: /a\n>\n> [b]: /b\n\n[1][a] [2][b]\n",
            "<blockquote>\n</blockquote>\n<p><a href=\"/a\">1</a> <a href=\"/b\">2</a></p>",
        ),
        // Quoted and unquoted lines in one chunk.
        (
            "[a]: /a\n> [b]: /b\n***\n> [c]: /c\n\n[1][a] [2][b] [3][c]\n",
            "<blockquote>\n</blockquote>\n<hr>\n<blockquote>\n</blockquote>\n<p><a href=\"/a\">1</a> <a href=\"/b\">2</a> <a href=\"/c\">3</a></p>",
        ),
        (
            "> [a]: /a\n***\n[b]: /b\n> ***\n> [c]: /c\n\n[1][a] [2][b] [3][c]\n",
            "<blockquote>\n</blockquote>\n<hr>\n<blockquote>\n<hr>\n</blockquote>\n<p><a href=\"/a\">1</a> <a href=\"/b\">2</a> <a href=\"/c\">3</a></p>",
        ),
        // In a list item's continuation paragraphs, quoted or not.
        (
            "- item\n\n  [a]: /a\n  ***\n  [b]: /b\n\n[1][a] [2][b]\n",
            "<ul>\n<li><p>item</p>\n<hr>\n</li>\n</ul>\n<p><a href=\"/a\">1</a> <a href=\"/b\">2</a></p>",
        ),
        (
            "1. item\n\n   [a]: /a\n\n[1][a]\n",
            "<ol>\n<li><p>item</p>\n</li>\n</ol>\n<p><a href=\"/a\">1</a></p>",
        ),
        (
            "- item\n\n  > [a]: /a\n  > ***\n  > [b]: /b\n\n[1][a] [2][b]\n",
            "<ul>\n<li><p>item</p>\n<blockquote>\n<hr>\n</blockquote>\n</li>\n</ul>\n<p><a href=\"/a\">1</a> <a href=\"/b\">2</a></p>",
        ),
        // On the marker line itself it is the item's text, as before.
        ("- [a]: /a\n\n[1][a]\n", "<ul>\n<li>[a]: /a</li>\n</ul>\n<p>[1][a]</p>"),
    ] {
        assert_eq!(render(source), expected, "for {source:?}");
    }
}

#[test]
fn footnotes_and_definitions_in_one_chunk_both_register() {
    let note =
        "<div id=\"fn-n\" class=\"footnote\">\n<p>note</p>\n<a href=\"#fnref-n\">↩</a>\n</div>";
    let reference = "<sup><a href=\"#fn-n\" id=\"fnref-n\">n</a></sup>";
    for (source, expected) in [
        (
            "[^n]: note\n***\n[a]: /a\n\n[1][a] and[^n]\n",
            format!("{note}\n<hr>\n<p><a href=\"/a\">1</a> and{reference}</p>"),
        ),
        (
            "[a]: /a\n[^n]: note\n***\n[b]: /b\n\n[1][a] [2][b] and[^n]\n",
            format!(
                "{note}\n<hr>\n<p><a href=\"/a\">1</a> <a href=\"/b\">2</a> and{reference}</p>"
            ),
        ),
        // The footnote line follows a definition's title line, which the
        // reference side skips as part of the definition.
        (
            "[a]: /a\n   \"title\"\n[^n]: note\n\n[1][a] and[^n]\n",
            format!("{note}\n<p><a href=\"/a\" title=\"title\">1</a> and{reference}</p>"),
        ),
    ] {
        assert_eq!(render_with(source, ParserOptions::gfm()), expected, "for {source:?}");
    }
}
