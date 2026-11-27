//! Kani harnesses: `LexMemo`'s quote and backtick skips return what the
//! `braces.rs` scans they are named after return, for every input up to
//! `LEN` bytes and for every memo state the earlier calls of a harness leave
//! behind. (`brace_end` is left out: CBMC does not get through its hash map
//! in reasonable time even at four bytes.)
//!
//! `memchr` is stubbed with a plain loop: the SIMD search finds the same
//! byte, and CBMC cannot reason through NEON or SSE2 in reasonable time.

use super::super::braces;
use super::LexMemo;

const LEN: usize = 6;

fn memchr_stub(needle: u8, haystack: &[u8]) -> Option<usize> {
    let mut index = 0;
    while index < haystack.len() {
        if haystack[index] == needle {
            return Some(index);
        }
        index += 1;
    }
    None
}

fn any_bytes(buffer: &[u8; LEN]) -> &[u8] {
    let len: usize = kani::any();
    kani::assume(len <= LEN);
    &buffer[..len]
}

fn any_start(bytes: &[u8], holds: impl Fn(u8) -> bool) -> usize {
    let start: usize = kani::any();
    kani::assume(start < bytes.len() && holds(bytes[start]));
    start
}

#[kani::proof]
#[kani::unwind(8)]
#[kani::stub(memchr::memchr::memchr, memchr_stub)]
fn backticks_end_matches_skip_backticks() {
    let buffer: [u8; LEN] = kani::any();
    let bytes = any_bytes(&buffer);
    let mut memo = LexMemo::new(4);
    for _ in 0..2 {
        let start = any_start(bytes, |byte| byte == b'`');
        assert_eq!(memo.backticks_end(bytes, start), braces::skip_backticks(bytes, start));
    }
}

#[kani::proof]
#[kani::unwind(8)]
fn quoted_end_matches_skip_quoted() {
    let buffer: [u8; LEN] = kani::any();
    let bytes = any_bytes(&buffer);
    let mut memo = LexMemo::new(4);
    for _ in 0..2 {
        let start = any_start(bytes, |byte| matches!(byte, b'"' | b'\'' | b'`'));
        assert_eq!(memo.quoted_end(bytes, start), braces::skip_quoted(bytes, start));
    }
}
