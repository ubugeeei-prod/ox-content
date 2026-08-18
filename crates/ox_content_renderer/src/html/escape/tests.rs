//! Differential tests: every escape path must agree with a plain
//! byte-at-a-time reference, including the vector classifiers.

use super::*;

/// Straightforward byte-at-a-time escaper the SWAR scanner must match.
///
/// Every escaped byte is ASCII, so copying the rest through verbatim at
/// the byte level keeps multi-byte sequences intact.
fn reference(s: &str, flags: &[u8; 256], table: &[&'static str; 256]) -> String {
    let mut out: Vec<u8> = Vec::new();
    for byte in s.bytes() {
        if flags[byte as usize] != 0 {
            out.extend_from_slice(table[byte as usize].as_bytes());
        } else {
            out.push(byte);
        }
    }
    String::from_utf8(out).expect("escaping only rewrites ASCII bytes")
}

fn check(s: &str) {
    let mut text = String::new();
    write_escaped_into(&mut text, s);
    assert_eq!(text, reference(s, &ESCAPE_FLAG, &ESCAPE_TABLE), "text escape: {s:?}");

    let mut url = String::new();
    write_url_escaped_into(&mut url, s);
    assert_eq!(url, reference(s, &URL_ESCAPE_FLAG, &URL_ESCAPE_TABLE), "url escape: {s:?}");
}

#[test]
fn matches_reference_on_fixtures() {
    for case in [
        "",
        "a",
        "&",
        "<>",
        "plain ascii text with no escapes at all",
        "&<>\"'",
        "a&b<c>d\"e'f",
        "exactly-8b",
        "seven77",
        "&&&&&&&&&&&&&&&&",
        "trailing escape &",
        "& leading escape",
        // The folded masks admit no extra bytes, but these neighbours are
        // the ones a sloppy fold would leak: 0x21 0x23 0x25 0x3D 0x3F.
        "!#%=?",
        "!#%=? &<>\"' !#%=?",
    ] {
        check(case);
    }
}

#[test]
fn matches_reference_on_borrow_propagation_shapes() {
    // `has_zero` can flag a 0x01 lane that borrowed from a real match
    // below it. These interleavings put such bytes directly after a match
    // inside the same word, which is where a wrong lane index would show.
    for filler in ["!", "#", "%", "=", "?", "\x01", "\x00"] {
        for needle in ["&", "<", ">", "\"", "'", " "] {
            for lead in 0..9 {
                let mut s = "x".repeat(lead);
                s.push_str(needle);
                for _ in 0..8 {
                    s.push_str(filler);
                }
                s.push_str(needle);
                check(&s);
            }
        }
    }
}

#[test]
fn matches_reference_across_the_overlapping_tail_read() {
    // The sub-word tail is covered by re-reading the last eight bytes
    // with the already-cleared lanes masked off, which is exactly where
    // a masked-off match can borrow into the lane above it. The pairs
    // below are the ones that can do it: after the mask folds, `<`/`>`
    // leave a `0x01` lane on a following `=` or `?`, and `"` on a
    // following `#`. One is placed at every offset of every length so
    // each lands on both sides of the tail boundary.
    for pair in ["<=", "<?", ">=", ">?", "\"#"] {
        for len in 2..40usize {
            for at in 0..len - 1 {
                let mut s = "x".repeat(len);
                s.replace_range(at..at + 2, pair);
                check(&s);
            }
        }
    }
}

#[test]
fn matches_reference_for_every_byte_value_at_every_offset() {
    // The vector paths classify from nibble pairs rather than the flag
    // table, so a wrong entry would admit or drop a byte the table
    // disagrees with. Check all 256 values at every offset of a buffer
    // long enough to cross the 16-byte step and land in the overlapping
    // tail, and at lengths below one vector so the word path is covered
    // too.
    for value in 0..=255u8 {
        for len in [1usize, 7, 8, 15, 16, 17, 33] {
            for at in 0..len {
                let mut buffer = vec![b'x'; len];
                buffer[at] = value;
                check(&String::from_utf8_lossy(&buffer));
            }
        }
    }
}

#[test]
fn matches_reference_on_pseudorandom_bytes() {
    // xorshift over the printable-plus-needles range; deterministic so a
    // failure is reproducible.
    let mut state = 0x2545_F491_4F6C_DD1Du64;
    let alphabet: Vec<u8> = (0x20u8..0x7f).chain(*b"&<>\"' ").collect();
    for len in 0..200 {
        let mut s = String::with_capacity(len);
        for _ in 0..len {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            s.push(alphabet[(state % alphabet.len() as u64) as usize] as char);
        }
        check(&s);
    }
}

#[test]
fn matches_reference_on_multibyte_utf8() {
    check("日本語のテキスト");
    check("emoji 🎉 and <tags> mixed");
    check("café & naïve — \"quoted\"");
}
