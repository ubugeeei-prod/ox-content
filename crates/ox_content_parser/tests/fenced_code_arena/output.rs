//! Exact output for fenced code blocks whose opening fence is indented: the
//! body is whatever lies between the fences, with up to the opener's
//! indentation taken off each line.

use super::{render, tree};

#[test]
fn content_lines_lose_at_most_the_opening_indentation() {
    for (source, expected) in [
        // Indented like the fence, less, and more.
        (" ```\n aaa\naaa\n  bbb\n ```\n", "<pre><code>aaa\naaa\n bbb\n</code></pre>"),
        (
            "  ```\naaa\n aaa\n  aaa\n   aaa\n    aaa\n  ```\n",
            "<pre><code>aaa\naaa\naaa\n aaa\n  aaa\n</code></pre>",
        ),
        ("   ```\n   aaa\n    aaa\n  aaa\n   ```\n", "<pre><code>aaa\n aaa\naaa\n</code></pre>"),
        // Blank lines stay, and a tab is never split to pay for a column.
        (" ~~~\n aaa\n\n  bbb\n ~~~\n", "<pre><code>aaa\n\n bbb\n</code></pre>"),
        (" ```\n\taaa\n \tbbb\n ```\n", "<pre><code>\taaa\n\tbbb\n</code></pre>"),
        ("  ```\n \taaa\n  ```\n", "<pre><code>\taaa\n</code></pre>"),
    ] {
        assert_eq!(render(source), expected, "for {source:?}");
    }
}

#[test]
fn the_closing_fence_is_found_where_it_was() {
    for (source, expected) in [
        // Any indentation up to three spaces closes; four is content.
        (" ```\naaa\n```\n", "<pre><code>aaa\n</code></pre>"),
        (" ```\naaa\n   ```\n", "<pre><code>aaa\n</code></pre>"),
        (" ```\naaa\n    ```\n ```\n", "<pre><code>aaa\n   ```\n</code></pre>"),
        // At least as long as the opener, and the same character.
        (" ````\naaa\n ```\n `````\nrest\n", "<pre><code>aaa\n```\n</code></pre>\n<p>rest</p>"),
        (" ```\naaa\n ~~~\n ```\nrest\n", "<pre><code>aaa\n~~~\n</code></pre>\n<p>rest</p>"),
        (" ~~~\naaa\n ```\n ~~~\nrest\n", "<pre><code>aaa\n```\n</code></pre>\n<p>rest</p>"),
        (" ```\naaa\n ``` \t\nrest\n", "<pre><code>aaa\n</code></pre>\n<p>rest</p>"),
    ] {
        assert_eq!(render(source), expected, "for {source:?}");
    }
}

#[test]
fn text_after_the_closing_run_keeps_its_current_meaning() {
    // Pinned as it stands: under an indented opener a long enough run of the
    // fence character ends the block whatever follows it on the line, while
    // under an unindented opener the same line is content. Aligning the two
    // is a behavior change and should be made on purpose, not by a refactor.
    assert_eq!(render(" ```\naaa\n ``` x\nrest\n"), "<pre><code>aaa\n</code></pre>\n<p>rest</p>");
    assert_eq!(
        render("```\naaa\n``` x\nrest\n```\n"),
        "<pre><code>aaa\n``` x\nrest\n</code></pre>"
    );
}

#[test]
fn unclosed_and_empty_blocks_keep_their_bodies() {
    for (source, expected) in [
        // The last line only ends in `\n` when the source gave it an ending.
        (" ```\naaa", "<pre><code>aaa</code></pre>"),
        (" ```\naaa\n", "<pre><code>aaa\n</code></pre>"),
        (" ```\n aaa\n\n", "<pre><code>aaa\n\n</code></pre>"),
        (" ```", "<pre><code></code></pre>"),
        (" ```\n", "<pre><code></code></pre>"),
        (" ```\n ```\n", "<pre><code></code></pre>"),
        (" ```\n\n ```\n", "<pre><code>\n</code></pre>"),
    ] {
        assert_eq!(render(source), expected, "for {source:?}");
    }
}

#[test]
fn line_endings_inside_the_block_become_newlines() {
    for (source, expected) in [
        (
            " ```\r\n aaa\r\n  bbb\r\n ```\r\nrest\r\n",
            "<pre><code>aaa\n bbb\n</code></pre>\n<p>rest</p>",
        ),
        (" ```\r aaa\r ```\rrest\r", "<pre><code>aaa\n</code></pre>\n<p>rest</p>"),
        (" ```\n aaa\r\n bbb", "<pre><code>aaa\nbbb</code></pre>"),
    ] {
        assert_eq!(render(source), expected, "for {source:?}");
    }
}

#[test]
fn indented_fences_next_to_other_blocks_and_inside_containers() {
    for (source, expected) in [
        ("x\n ```\nx\n ```\nx\n", "<p>x</p>\n<pre><code>x\n</code></pre>\n<p>x</p>"),
        (
            "x\n ```\n ```\nx\n ```\n ```\n",
            "<p>x</p>\n<pre><code></code></pre>\n<p>x</p>\n<pre><code></code></pre>",
        ),
        ("   ~~~\n   ~~~\n   ~~~\n   ~~~\n", "<pre><code></code></pre>\n<pre><code></code></pre>"),
        // Nothing inside the block is a block of its own.
        (
            " ```\n- a\n> b\n# c\n\n[x]: /u\n ```\n[y][x]\n",
            "<pre><code>- a\n&gt; b\n# c\n\n[x]: /u\n</code></pre>\n<p>[y][x]</p>",
        ),
        // Indented relative to a quote's or a list item's content.
        (
            ">  ```\n>  aaa\n>   bbb\n>  ```\n> rest\n",
            "<blockquote>\n<pre><code>aaa\n bbb\n</code></pre>\n<p>rest</p>\n</blockquote>",
        ),
        (
            ">  ```\n>  aaa\nrest\n",
            "<blockquote>\n<pre><code>aaa\n</code></pre>\n</blockquote>\n<p>rest</p>",
        ),
        (
            "-  ```\n   aaa\n    bbb\n   ```\n- next\n",
            "<ul>\n<li>\n<pre><code>aaa\n bbb\n</code></pre>\n</li>\n<li>next</li>\n</ul>",
        ),
        (
            "- a\n\n   ```\n   aaa\n   ```\n",
            "<ul>\n<li><p>a</p>\n<pre><code>aaa\n</code></pre>\n</li>\n</ul>",
        ),
    ] {
        assert_eq!(render(source), expected, "for {source:?}");
    }
}

#[test]
fn the_node_keeps_its_info_string_and_span() {
    // HTML alone would not show a span that stopped short of the closing
    // fence, or one that swallowed the line after it.
    assert_eq!(
        tree("  ```rust title=\"x\"\n  fn main() {}\n  ```\nrest\n"),
        "Document [0..46]\n  \
         CodeBlock lang=Some(\"rust\") meta=Some(\"title=\\\"x\\\"\") \
         value=\"fn main() {}\\n\" [0..41]\n  \
         Paragraph [41..46]\n    Text \"rest\" [41..45]\n"
    );
    // Unclosed: the block and its span run to the end of input.
    assert_eq!(
        tree(" ~~~ a`b\n x\n"),
        "Document [0..12]\n  CodeBlock lang=Some(\"a`b\") meta=None value=\"x\\n\" [0..12]\n"
    );
    // A closing fence on an unterminated last line, after CRLF content.
    assert_eq!(
        tree("x\n ```\r\n y\r\n ```"),
        "Document [0..16]\n  Paragraph [0..2]\n    Text \"x\" [0..1]\n  \
         CodeBlock lang=None meta=None value=\"y\\n\" [2..16]\n"
    );
}
