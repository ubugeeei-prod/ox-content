use crate::tui::style::{THEMES, clean, clip, pad, paint, width, wrap};

const SAMPLES: &[&str] = &[
    "",
    "plain ascii words in a sentence",
    "日本語の文章をここに書きます。",
    "mixed 日本語 and English 👩‍💻 text",
    "e\u{301}toile cafe\u{301} combining marks",
    "supercalifragilisticexpialidocious",
    "https://example.com/a/very/long/url/without/any/spaces/at/all",
    "  leading and trailing  ",
    "tab\tseparated\tcells",
    "🇯🇵🇺🇸 flags 👨‍👩‍👧‍👦 family",
    "first line\nsecond line\n\nfourth line",
];

#[test]
fn clean_removes_controls_but_keeps_layout_whitespace() {
    for (input, expected) in [
        ("plain", "plain"),
        ("a\tb\nc", "a\tb\nc"),
        ("a\x1b[31mb\x1b[0m", "a[31mb[0m"),
        ("bell\x07 null\x00 del\x7f", "bell null del"),
        ("c1\u{9b}31m \u{85}next", "c131m next"),
        ("carriage\rreturn", "carriagereturn"),
        ("日本語\u{200d}👩‍💻", "日本語\u{200d}👩‍💻"),
    ] {
        assert_eq!(clean(input), expected, "{input:?}");
    }
}

#[test]
fn width_counts_terminal_cells() {
    for (text, cells) in [
        ("", 0),
        ("abc", 3),
        ("日本語", 6),
        ("a日b", 4),
        ("e\u{301}", 1),
        ("👩‍💻", 2),
        ("─│╭╰", 4),
        ("☑ ◇ ◆", 5),
    ] {
        assert_eq!(width(text), cells, "{text:?}");
    }
}

#[test]
fn clip_stops_at_grapheme_boundaries_within_the_budget() {
    for (text, columns, expected) in [
        ("hello", 3, "hel"),
        ("hello", 5, "hello"),
        ("hello", 9, "hello"),
        ("hello", 0, ""),
        ("日本語", 6, "日本語"),
        ("日本語", 5, "日本"),
        ("日本語", 3, "日"),
        ("日本語", 1, ""),
        ("a日b", 2, "a"),
        ("👩‍💻x", 1, ""),
        ("👩‍💻x", 2, "👩‍💻"),
        ("e\u{301}x", 1, "e\u{301}"),
        ("ab\u{301}c", 2, "ab\u{301}"),
    ] {
        assert_eq!(clip(text, columns), expected, "{text:?} in {columns}");
    }
}

#[test]
fn clip_never_exceeds_its_budget_and_is_a_prefix() {
    for sample in SAMPLES.iter().map(|sample| sample.replace('\n', " ")) {
        for columns in 0..=width(&sample) + 2 {
            let clipped = clip(&sample, columns);
            assert!(width(&clipped) <= columns, "{sample:?} in {columns}");
            assert!(sample.starts_with(&clipped), "{sample:?} in {columns}");
        }
        assert_eq!(clip(&sample, width(&sample)), sample);
    }
}

#[test]
fn pad_always_fills_exactly_the_requested_cells() {
    for sample in SAMPLES.iter().map(|sample| sample.replace(['\n', '\t'], " ")) {
        for columns in 0..=width(&sample) + 3 {
            let padded = pad(&sample, columns);
            assert_eq!(width(&padded), columns, "{sample:?} in {columns}");
            assert!(padded.starts_with(&clip(&sample, columns)), "{sample:?} in {columns}");
        }
    }
    assert_eq!(pad("ab", 5), "ab   ");
    assert_eq!(pad("日本語", 5), "日本 ");
    assert_eq!(pad("", 2), "  ");
}

#[test]
fn wrap_breaks_between_words_and_inside_long_ones() {
    for (text, columns, expected) in [
        ("hello world", 20, &["hello world"][..]),
        ("hello world", 8, &["hello", "world"]),
        ("one two three four", 9, &["one two", "three", "four"]),
        ("abcdefghij", 4, &["abcd", "efgh", "ij"]),
        ("ab abcdefgh", 4, &["ab", "abcd", "efgh"]),
        ("日本語の文章", 5, &["日本", "語の", "文章"]),
        ("日本語 text", 7, &["日本語", "text"]),
        ("word   ", 10, &["word"]),
        ("", 10, &[""]),
        ("a\nb", 10, &["a", "b"]),
        ("a\n\nb", 10, &["a", "", "b"]),
        ("  indented code", 20, &["  indented code"]),
        ("👩‍💻👩‍💻👩‍💻", 4, &["👩‍💻👩‍💻", "👩‍💻"]),
    ] {
        assert_eq!(wrap(text, columns), expected, "{text:?} in {columns}");
    }
}

#[test]
fn wrap_treats_zero_columns_as_one_and_keeps_wide_glyphs_whole() {
    assert_eq!(wrap("abc", 0), ["a", "b", "c"]);
    assert_eq!(wrap("abc", 1), ["a", "b", "c"]);
    assert_eq!(wrap("日本", 1), ["日", "本"]);
    assert_eq!(wrap("a👩‍💻b", 1), ["a", "👩‍💻", "b"]);
}

#[test]
fn wrap_keeps_every_visible_character_in_order_within_the_width() {
    let visible = |text: &str| text.chars().filter(|ch| !ch.is_whitespace()).collect::<String>();
    for sample in SAMPLES {
        for columns in 1..=40 {
            let lines = wrap(sample, columns);
            assert_eq!(visible(&lines.concat()), visible(sample), "{sample:?} in {columns}");
            // A glyph wider than the page is the only thing allowed to overflow.
            let limit = columns.max(2);
            for line in &lines {
                assert!(width(line) <= limit, "{line:?} from {sample:?} in {columns}");
                assert!(!line.ends_with(' '), "{line:?} from {sample:?} in {columns}");
            }
            assert!(lines.len() >= sample.split('\n').count(), "{sample:?} in {columns}");
        }
    }
}

#[test]
fn paint_is_plain_without_colour_and_themed_with_it() {
    for theme in THEMES {
        assert_eq!(paint("text", theme, false, false), "text");
        assert_eq!(paint("text", theme, false, true), "text");
        assert_eq!(paint("text", theme, true, true), "\x1b[7mtext\x1b[0m");
    }
    for (theme, accent) in [
        ("nord", "136;192;208"),
        ("light", "0;96;128"),
        ("mono", "255;255;255"),
        // Unknown themes fall back to the default accent instead of failing.
        ("unknown", "136;192;208"),
    ] {
        assert_eq!(paint("text", theme, true, false), format!("\x1b[38;2;{accent}mtext\x1b[0m"));
    }
    assert_eq!(paint("", "nord", true, false), "\x1b[38;2;136;192;208m\x1b[0m");
}

#[test]
fn themes_are_listed_with_the_default_first() {
    assert_eq!(THEMES, ["nord", "light", "mono"]);
}
