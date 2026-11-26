//! Which side of a `*`/`_` run can open or close emphasis.

/// Flanking classification (CommonMark "Emphasis and strong emphasis").
/// Sequence boundaries count as whitespace.
///
/// `cjk_emphasis` reclassifies East Asian punctuation as an ordinary character
/// here and nowhere else; see [`ParserOptions::cjk_emphasis`].
///
/// [`ParserOptions::cjk_emphasis`]: crate::ParserOptions::cjk_emphasis
pub(super) fn classify_flanking(
    marker: u8,
    prev: Option<char>,
    next: Option<char>,
    cjk_emphasis: bool,
) -> (bool, bool) {
    let is_punct =
        |ch: char| is_punctuation_like(ch) && !(cjk_emphasis && is_east_asian_punctuation(ch));

    let prev_ws = prev.is_none_or(char::is_whitespace);
    let next_ws = next.is_none_or(char::is_whitespace);
    let prev_punct = prev.is_some_and(is_punct);
    let next_punct = next.is_some_and(is_punct);

    let left_flanking = !next_ws && (!next_punct || prev_ws || prev_punct);
    let right_flanking = !prev_ws && (!prev_punct || next_ws || next_punct);

    if marker == b'*' {
        (left_flanking, right_flanking)
    } else {
        (
            left_flanking && (!right_flanking || prev_punct),
            right_flanking && (!left_flanking || next_punct),
        )
    }
}

/// Approximates the spec's Unicode punctuation class (general categories
/// P and S): anything printable that is neither alphanumeric nor
/// whitespace.
fn is_punctuation_like(ch: char) -> bool {
    ch.is_ascii_punctuation()
        || (!ch.is_ascii() && !ch.is_alphanumeric() && !ch.is_whitespace() && !ch.is_control())
}

/// Punctuation that East Asian scripts set directly against the text, with no
/// separating space — the reason CommonMark's flanking rules reject emphasis
/// that Latin text would accept.
///
/// The ranges are the fullwidth and CJK-specific blocks only. Halfwidth ASCII
/// punctuation is deliberately excluded even in CJK text: it is written the
/// same way in every script, so reclassifying it would change how ordinary
/// Latin documents parse.
///
/// U+3000 IDEOGRAPHIC SPACE falls inside the first range but is whitespace, and
/// the flanking rules test whitespace before punctuation, so it is unaffected.
fn is_east_asian_punctuation(ch: char) -> bool {
    matches!(ch,
        // CJK Symbols and Punctuation: 、。〈〉《》「」『』【】〜 and friends.
        '\u{3000}'..='\u{303F}'
        // Vertical forms and CJK Compatibility Forms: vertical/rotated variants.
        | '\u{FE10}'..='\u{FE19}'
        | '\u{FE30}'..='\u{FE4F}'
        // Small Form Variants: small comma, small full stop, small brackets.
        | '\u{FE50}'..='\u{FE6F}'
        // Fullwidth ASCII punctuation, split around the fullwidth digits and
        // letters, which stay alphanumeric.
        | '\u{FF01}'..='\u{FF0F}'
        | '\u{FF1A}'..='\u{FF20}'
        | '\u{FF3B}'..='\u{FF40}'
        | '\u{FF5B}'..='\u{FF65}'
    )
}
