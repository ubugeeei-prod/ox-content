//! `crates/ox_content_parser/src/parser/mdx_jsx/braces.rs`: the brace,
//! string and comment skips under JSX tags and MDX expressions.
//!
//! Each body is the production body plus ghost code (`tools/scripts/
//! check-creusot-mirrors.mjs` checks that they match once the ghost code is
//! taken out). `steps` counts loop iterations, the nested skips' included.
//! Every iteration does a constant amount of work besides the calls it
//! counts, so the bounds below are bounds on running time.
//!
//! Proven for every input of every length: each skip terminates, never
//! indexes out of bounds or overflows, and runs at most one iteration per
//! byte between `start` and where it stops.

use creusot_std::prelude::*;

/// How far a skip that returned `end` read: to `end` when it found one, to
/// the end of the input when it did not.
#[logic(open)]
pub fn reached(bytes: Seq<u8>, end: Option<usize>) -> Int {
    pearlite! {
        match end {
            Some(end) => end@,
            None => bytes.len(),
        }
    }
}

/// Rust never makes a slice of bytes longer than `isize::MAX`, so an
/// offset one past an index cannot overflow. Creusot does not assume it.
#[logic(open)]
pub fn fits(bytes: Seq<u8>) -> bool {
    pearlite! { bytes.len() <= isize::MAX@ }
}

/// A skip that succeeds stops strictly after `start`, inside the input.
#[logic(open)]
pub fn advances(bytes: Seq<u8>, start: Int, end: Option<usize>) -> bool {
    pearlite! {
        match end {
            Some(end) => start < end@ && end@ <= bytes.len(),
            None => true,
        }
    }
}

#[check(terminates)]
#[ensures(advances(bytes@, start@, result))]
#[ensures(start@ < bytes@.len() ==> ^*steps - **steps + start@ + 1 <= reached(bytes@, result))]
#[ensures(start@ >= bytes@.len() ==> ^*steps == **steps)]
pub fn skip_quoted(bytes: &[u8], start: usize, mut steps: Ghost<&mut Int>) -> Option<usize> {
    let quote = *bytes.get(start)?;
    let mut cursor = start + 1;
    let entry = snapshot!(steps);
    #[variant(bytes@.len() - cursor@)]
    #[invariant(start@ < cursor@ && cursor@ <= bytes@.len())]
    #[invariant(^*steps == ^**entry)]
    #[invariant(**steps - ***entry + start@ + 1 <= cursor@)]
    while cursor < bytes.len() {
        ghost! { **steps = **steps + 1int };
        if bytes[cursor] == b'\\' && cursor + 1 < bytes.len() {
            cursor += 2;
            continue;
        }
        if bytes[cursor] == quote {
            return Some(cursor + 1);
        }
        cursor += 1;
    }
    None
}

#[check(terminates)]
#[requires(start@ + 2 <= bytes@.len())]
#[ensures(start@ + 2 <= result@ && result@ <= bytes@.len())]
#[ensures(^*steps - **steps + start@ + 2 == result@)]
pub fn skip_line_comment(bytes: &[u8], start: usize, mut steps: Ghost<&mut Int>) -> usize {
    let mut cursor = start + 2;
    let entry = snapshot!(steps);
    #[variant(bytes@.len() - cursor@)]
    #[invariant(start@ + 2 <= cursor@ && cursor@ <= bytes@.len())]
    #[invariant(^*steps == ^**entry)]
    #[invariant(**steps - ***entry + start@ + 2 == cursor@)]
    while cursor < bytes.len() && !matches!(bytes[cursor], b'\n' | b'\r') {
        ghost! { **steps = **steps + 1int };
        cursor += 1;
    }
    cursor
}

#[check(terminates)]
#[requires(fits(bytes@))]
#[requires(start@ + 2 <= bytes@.len())]
#[ensures(advances(bytes@, start@, result))]
#[ensures(^*steps - **steps + start@ + 2 <= reached(bytes@, result))]
pub fn skip_block_comment(bytes: &[u8], start: usize, mut steps: Ghost<&mut Int>) -> Option<usize> {
    let mut cursor = start + 2;
    let entry = snapshot!(steps);
    #[variant(bytes@.len() - cursor@)]
    #[invariant(start@ + 2 <= cursor@ && cursor@ <= bytes@.len())]
    #[invariant(^*steps == ^**entry)]
    #[invariant(**steps - ***entry + start@ + 2 <= cursor@)]
    while cursor + 1 < bytes.len() {
        ghost! { **steps = **steps + 1int };
        if bytes[cursor] == b'*' && bytes[cursor + 1] == b'/' {
            return Some(cursor + 2);
        }
        cursor += 1;
    }
    None
}

/// The whole walk, nested skips included, is one step per byte at most.
#[requires(fits(bytes@))]
#[ensures(advances(bytes@, start@, result))]
#[ensures(start@ < bytes@.len() ==> ^*steps - **steps + start@ <= reached(bytes@, result))]
#[ensures(start@ >= bytes@.len() ==> ^*steps == **steps)]
pub fn skip_braces(bytes: &[u8], start: usize, mut steps: Ghost<&mut Int>) -> Option<usize> {
    if bytes.get(start) != Some(&b'{') {
        return None;
    }
    let mut cursor = start;
    let mut depth = 0u32;
    let entry = snapshot!(steps);
    #[variant(bytes@.len() - cursor@)]
    #[invariant(start@ <= cursor@ && cursor@ <= bytes@.len())]
    #[invariant(^*steps == ^**entry)]
    #[invariant(**steps - ***entry + start@ <= cursor@)]
    while cursor < bytes.len() {
        ghost! { **steps = **steps + 1int };
        match bytes[cursor] {
            b'"' | b'\'' | b'`' => cursor = skip_quoted(bytes, cursor, ghost!(&mut **steps))?,
            b'/' if bytes.get(cursor + 1) == Some(&b'/') => {
                cursor = skip_line_comment(bytes, cursor, ghost!(&mut **steps));
            }
            b'/' if bytes.get(cursor + 1) == Some(&b'*') => {
                cursor = skip_block_comment(bytes, cursor, ghost!(&mut **steps))?;
            }
            b'{' => {
                depth = depth.saturating_add(1);
                cursor += 1;
            }
            b'}' => {
                depth = depth.saturating_sub(1);
                cursor += 1;
                if depth == 0 {
                    return Some(cursor);
                }
            }
            _ => cursor += 1,
        }
    }
    None
}
