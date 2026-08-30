//! `{.class}` / `{#id}` blocks rewrite the HTML in one forward pass.
//!
//! That pass tracks two things that used to be one variable: how much of
//! the input has been written out, and where to look for the next block.
//! Conflating them meant a block the pass declined moved the write mark
//! forward without writing anything, so a later rewrite copied from the
//! moved position and silently dropped the text in between — and an
//! enclosing tag that had already been written still looked rewritable,
//! which sliced backwards and aborted the process.

use ox_content_transform::transformer::MarkdownTransformer;
use ox_content_transform::{AttrsOptions, TransformOptions};

fn transform(markdown: &str) -> String {
    let transformer = MarkdownTransformer::from_options(&TransformOptions {
        gfm: Some(true),
        attributes: Some(AttrsOptions { enabled: Some(true) }),
        ..Default::default()
    });
    transformer.transform(markdown).html.trim().to_string()
}

#[test]
fn two_declined_and_applied_blocks_in_one_paragraph_keep_every_word() {
    assert_eq!(
        transform("aaa *x*{.c} bbb ![i](u){.z} ccc\n"),
        "<p>aaa <em>x</em>{.c} bbb <img src=\"u\" alt=\"i\" class=\"z\"> ccc</p>"
    );
    assert_eq!(
        transform("lead *x*{.c} tail `code`{.k} end\n"),
        "<p>lead <em>x</em>{.c} tail <code class=\"k\">code</code> end</p>"
    );
}

#[test]
fn a_declined_block_does_not_stop_the_paragraph_from_being_targeted() {
    assert_eq!(
        transform("text *x*{.c} more {.p}\n"),
        "<p class=\"p\">text <em>x</em>{.c} more</p>"
    );
}

#[test]
fn a_declined_block_mid_line_still_leaves_the_trailing_one_working() {
    // Emphasis in the middle of a line is not a target, so `{.c}` stays
    // literal — but `{.d}` sits at the end of the paragraph, which is one,
    // and the paragraph tag has to still be rewritable at that point.
    assert_eq!(
        transform("*x*{.c} and *y*{.d}\n"),
        "<p class=\"d\"><em>x</em>{.c} and <em>y</em></p>"
    );
    assert_eq!(
        transform("**b**{.x} **c**{.y}\n"),
        "<p class=\"y\"><strong>b</strong>{.x} <strong>c</strong></p>"
    );
}

#[test]
fn several_applied_blocks_in_one_paragraph_all_take_effect() {
    assert_eq!(
        transform("[l](u){.m} [n](o){.p}\n"),
        "<p><a href=\"u\" class=\"m\">l</a> <a href=\"o\" class=\"p\">n</a></p>"
    );
    assert_eq!(
        transform("![a](b){.i} and ![c](d){.j}\n"),
        "<p><img src=\"b\" alt=\"a\" class=\"i\"> and <img src=\"d\" alt=\"c\" class=\"j\"></p>"
    );
    assert_eq!(
        transform("`one`{.k} then `two`{.l}\n"),
        "<p><code class=\"k\">one</code> then <code class=\"l\">two</code></p>"
    );
}

#[test]
fn single_block_behaviour_is_unchanged() {
    assert_eq!(transform("*x*{.c}\n"), "<p class=\"c\"><em>x</em></p>");
    assert_eq!(transform("lead *x*{.c}\n"), "<p class=\"c\">lead <em>x</em></p>");
    assert_eq!(transform("*x*{.c} tail\n"), "<p><em>x</em>{.c} tail</p>");
    assert_eq!(transform("para one {.p}\n"), "<p class=\"p\">para one</p>");
    assert_eq!(transform("# One {#a}\n"), "<h1 id=\"a\">One</h1>");
    assert_eq!(
        transform("# One {#a}\n\n# Two {#b}\n"),
        "<h1 id=\"a\">One</h1>\n<h1 id=\"b\">Two</h1>"
    );
}

/// xorshift64 — deterministic, so a failure reproduces from its seed.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

#[test]
fn no_arrangement_of_blocks_loses_a_word() {
    // Every word carries its own marker, so anything the pass drops is
    // visible. Blocks are mixed across targets it accepts (images, code,
    // links, the paragraph itself) and ones it declines (emphasis in the
    // middle of a line), because a decline is what moved the write mark.
    let targets: [&str; 8] = [
        "*e{n}*",
        "**s{n}**",
        "`c{n}`",
        "[l{n}](u{n})",
        "![i{n}](s{n})",
        "w{n}",
        "<kbd>k{n}</kbd>",
        "~~d{n}~~",
    ];
    let blocks: [&str; 6] = ["{.c}", "{#i}", "{.a .b}", "{}", "{ }", "{.x #y}"];

    for seed in 1..20_000u64 {
        let mut rng = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
        let count = 1 + rng.below(5);
        let mut markdown = String::new();
        let mut words = Vec::new();
        for index in 0..count {
            let marker = format!("z{seed}x{index}");
            words.push(marker.clone());
            markdown.push_str(&marker);
            markdown.push(' ');
            markdown
                .push_str(&targets[rng.below(targets.len())].replace("{n}", &index.to_string()));
            if rng.below(4) != 0 {
                markdown.push_str(blocks[rng.below(blocks.len())]);
            }
            markdown.push(' ');
        }
        let tail = format!("z{seed}tail");
        words.push(tail.clone());
        markdown.push_str(&tail);
        markdown.push('\n');

        let html = transform(&markdown);
        for word in &words {
            assert!(
                html.contains(word.as_str()),
                "seed {seed} lost {word:?}\n  in:  {markdown:?}\n  out: {html:?}"
            );
        }
    }
}
