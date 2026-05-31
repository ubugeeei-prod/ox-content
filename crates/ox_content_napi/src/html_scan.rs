//! Small byte-level scanning helpers shared by the static HTML embed transforms
//! ([`crate::tabs`], [`crate::youtube`]).
//!
//! These transforms walk already-rendered HTML looking for a handful of literal
//! tag markers (`<youtube`, `<tabs`, `</tab>`, …). Every marker begins with `<`,
//! which is sparse in real documents, so the search is dominated by skipping
//! over long runs of prose. [`find_ci`] uses SIMD `memchr` to jump straight to
//! the next candidate `<` instead of running a case-insensitive comparison at
//! every byte — the same trick the renderer's autolink scanner relies on.

/// Case-insensitive ASCII substring search in `haystack[from..]`, returning an
/// absolute byte offset.
///
/// `needle` must be ASCII (it always is at the call sites). The first byte is
/// located with `memchr`, so the cost of skipping non-matching bytes is a tight
/// SIMD scan rather than a per-position `eq_ignore_ascii_case`.
pub fn find_ci(haystack: &str, from: usize, needle: &str) -> Option<usize> {
    let hay = haystack.as_bytes();
    let pat = needle.as_bytes();
    if pat.is_empty() || from > hay.len() || hay.len() - from < pat.len() {
        return None;
    }
    // Last index at which a full `pat`-length match can still start.
    let last_start = hay.len() - pat.len();
    let rest = &pat[1..];

    // Match the first byte case-insensitively. The markers here all begin with
    // `<` (non-alphabetic, so `lo == hi` and this collapses to a single
    // `memchr`), but handle the alphabetic case too so the helper stays a
    // correct case-insensitive search.
    let lo = pat[0].to_ascii_lowercase();
    let hi = pat[0].to_ascii_uppercase();

    let mut base = from;
    while base <= last_start {
        let window = &hay[base..=last_start];
        let rel = if lo == hi {
            memchr::memchr(pat[0], window)
        } else {
            memchr::memchr2(lo, hi, window)
        }?;
        let i = base + rel;
        if hay[i + 1..i + pat.len()].eq_ignore_ascii_case(rest) {
            return Some(i);
        }
        base = i + 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::find_ci;

    fn naive(haystack: &str, from: usize, needle: &str) -> Option<usize> {
        let hay = haystack.as_bytes();
        let pat = needle.as_bytes();
        if pat.is_empty() || from > hay.len() || hay.len() - from < pat.len() {
            return None;
        }
        (from..=hay.len() - pat.len()).find(|&i| hay[i..i + pat.len()].eq_ignore_ascii_case(pat))
    }

    #[test]
    fn matches_naive_reference_across_cases() {
        let cases = [
            ("<p>hi</p><YouTube id>", "<youtube"),
            ("plain prose, no tags at all", "<tabs"),
            ("<TABS><TAB>x</TAB></TABS>", "</tab>"),
            ("a<b<c<youtube>", "<youtube"),
            ("", "<tabs"),
            ("<youtube", "<youtube"),
        ];
        for (hay, needle) in cases {
            for from in 0..=hay.len() {
                assert_eq!(
                    find_ci(hay, from, needle),
                    naive(hay, from, needle),
                    "mismatch for needle {needle:?} in {hay:?} from {from}"
                );
            }
        }
    }

    #[test]
    fn returns_none_when_window_too_small() {
        assert_eq!(find_ci("<yo", 0, "<youtube"), None);
        assert_eq!(find_ci("<youtube", 1, "<youtube"), None);
    }
}
