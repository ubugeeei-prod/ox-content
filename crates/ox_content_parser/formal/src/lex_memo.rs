//! `crates/ox_content_parser/src/parser/mdx_jsx/lex_memo.rs`: the memoized
//! skip over a backtick code span in an MDX expression.
//!
//! Only the two fields `backticks_end` reads are modelled; the rest of
//! `LexMemo` is untouched by it. `memchr` is the plain loop it stands for.
//!
//! Proven for every input of every length:
//!
//! - `backticks_end` returns the end of the first closer of the opener at
//!   `start` ([`first_closer`]), which is what `braces::skip_backticks`
//!   returns, whatever earlier calls left in the memo;
//! - one call reads each byte after `start` at most once;
//! - after a miss, the caller's retry one backtick further into the same
//!   run is answered without reading anything, as long as the shorter
//!   opener is still at least as long as every run after it.

use creusot_std::prelude::*;

use crate::braces::fits;

#[logic(open)]
pub fn tick(bytes: Seq<u8>, i: Int) -> bool {
    pearlite! { 0 <= i && i < bytes.len() && bytes[i] == 96u8 }
}

/// `open` backticks at `at` that are not followed by another.
#[logic(open)]
pub fn closes_at(bytes: Seq<u8>, open: Int, at: Int) -> bool {
    pearlite! {
        0 <= at && at + open <= bytes.len()
            && (forall<i: Int> at <= i && i < at + open ==> tick(bytes, i))
            && !tick(bytes, at + open)
    }
}

/// `start` begins a run of exactly `open` backticks.
#[logic(open)]
pub fn opener(bytes: Seq<u8>, start: Int, open: Int) -> bool {
    pearlite! {
        0 < open && start + open <= bytes.len()
            && (forall<i: Int> start <= i && i < start + open ==> tick(bytes, i))
            && !tick(bytes, start + open)
    }
}

/// `end` closes the first closer for `open` at or after `from`, or is
/// `None` when nothing from there closes it.
#[logic(open)]
pub fn first_closer(bytes: Seq<u8>, from: Int, open: Int, end: Option<usize>) -> bool {
    pearlite! {
        match end {
            Some(end) => from <= end@ - open
                && closes_at(bytes, open, end@ - open)
                && forall<at: Int> from <= at && at < end@ - open ==> !closes_at(bytes, open, at),
            None => forall<at: Int> from <= at ==> !closes_at(bytes, open, at),
        }
    }
}

/// `LexMemo`'s backtick fields.
pub struct LexMemo {
    /// A backtick offset and the end of the run it sits in.
    pub backtick_run: (usize, usize),
    /// A run end and the shortest opener known to find no closer after it.
    pub backtick_miss: (usize, usize),
}

impl LexMemo {
    /// What the fields claim about the source `bytes`.
    #[logic(open)]
    pub fn wf(self, bytes: Seq<u8>) -> bool {
        pearlite! {
            self.backtick_run.0@ <= self.backtick_run.1@
                && self.backtick_run.1@ <= bytes.len()
                && (forall<i: Int> self.backtick_run.0@ <= i && i < self.backtick_run.1@
                    ==> tick(bytes, i))
                && (self.backtick_run.0@ < self.backtick_run.1@
                    ==> !tick(bytes, self.backtick_run.1@))
                && (self.backtick_miss.0@ > 0 ==> forall<o: Int, at: Int>
                    o >= self.backtick_miss.1@ && at >= self.backtick_miss.0@
                    ==> !closes_at(bytes, o, at))
        }
    }

    /// `skip_backticks(bytes, start)` for a `start` that holds a backtick.
    ///
    /// The closer is the first later run at least as long as the opener.
    #[requires(fits(bytes@))]
    #[requires((*self).wf(bytes@))]
    #[requires(tick(bytes@, start@))]
    #[ensures((^self).wf(bytes@))]
    #[ensures(forall<open: Int> opener(bytes@, start@, open)
        ==> first_closer(bytes@, start@ + open, open, result))]
    #[ensures(^*steps - **steps + start@ <= bytes@.len())]
    pub fn backticks_end(
        &mut self,
        bytes: &[u8],
        start: usize,
        mut steps: Ghost<&mut Int>,
    ) -> Option<usize> {
        let initial = snapshot!(**steps);
        let (from, known_end) = self.backtick_run;
        let run_end = if from <= start && start < known_end {
            known_end
        } else {
            let end = backtick_run_end(bytes, start, ghost!(&mut **steps));
            self.backtick_run = (start, end);
            end
        };
        let open = run_end - start;
        proof_assert!(opener(bytes@, start@, open@));
        proof_assert!(forall<o: Int> opener(bytes@, start@, o) ==> o == open@);
        // After a miss the caller retries one backtick further into the
        // same run, which only shortens the opener.
        if self.backtick_miss.0 == run_end && open >= self.backtick_miss.1 {
            return None;
        }
        let mut cursor = run_end;
        let mut longest = 0;
        let entry = snapshot!(steps);
        #[variant(bytes@.len() - cursor@)]
        #[invariant(run_end@ <= cursor@ && cursor@ <= bytes@.len())]
        #[invariant(!tick(bytes@, cursor@))]
        #[invariant(longest@ < open@)]
        #[invariant(^*steps == ^**entry)]
        #[invariant(**steps - ***entry + run_end@ <= cursor@)]
        #[invariant(***entry - *initial + start@ <= run_end@)]
        #[invariant((*self).wf(bytes@))]
        #[invariant(forall<o: Int, at: Int> o > longest@ && run_end@ <= at && at < cursor@
            ==> !closes_at(bytes@, o, at))]
        while let Some(found) = memchr(b'`', &bytes[cursor..], ghost!(&mut **steps)) {
            let end = backtick_run_end(bytes, cursor + found, ghost!(&mut **steps));
            let len = end - (cursor + found);
            proof_assert!(forall<j: Int> cursor@ <= j && j < cursor@ + found@ ==> !tick(bytes@, j));
            if len >= open {
                proof_assert!(closes_at(bytes@, open@, end@ - open@));
                return Some(end);
            }
            longest = longest.max(len);
            cursor = end;
        }
        proof_assert!(forall<j: Int> cursor@ <= j && j < bytes@.len() ==> !tick(bytes@, j));
        proof_assert!(forall<o: Int, at: Int> o > longest@ && at >= run_end@
            ==> !closes_at(bytes@, o, at));
        self.backtick_miss = (run_end, longest + 1);
        None
    }
}

/// End of the backtick run that holds `start`.
#[check(terminates)]
#[requires(fits(bytes@))]
#[requires(tick(bytes@, start@))]
#[ensures(start@ < result@ && result@ <= bytes@.len())]
#[ensures(forall<i: Int> start@ <= i && i < result@ ==> tick(bytes@, i))]
#[ensures(!tick(bytes@, result@))]
#[ensures(^*steps - **steps + start@ == result@)]
fn backtick_run_end(bytes: &[u8], start: usize, mut steps: Ghost<&mut Int>) -> usize {
    let mut end = start;
    let entry = snapshot!(steps);
    #[variant(bytes@.len() - end@)]
    #[invariant(start@ <= end@ && end@ <= bytes@.len())]
    #[invariant(forall<i: Int> start@ <= i && i < end@ ==> tick(bytes@, i))]
    #[invariant(^*steps == ^**entry)]
    #[invariant(**steps - ***entry + start@ == end@)]
    while end < bytes.len() && bytes[end] == b'`' {
        ghost! { **steps = **steps + 1int };
        end += 1;
    }
    end
}

/// `memchr::memchr`: the first `needle` in `haystack`, one step per byte
/// it passes. (That the SIMD search reads each byte a constant number of
/// times is assumed, not proven.)
#[check(terminates)]
#[ensures(match result {
    Some(i) => i@ < haystack@.len() && haystack@[i@] == needle
        && (forall<j: Int> 0 <= j && j < i@ ==> haystack@[j] != needle)
        && ^*steps - **steps == i@,
    None => (forall<j: Int> 0 <= j && j < haystack@.len() ==> haystack@[j] != needle)
        && ^*steps - **steps == haystack@.len(),
})]
fn memchr(needle: u8, haystack: &[u8], mut steps: Ghost<&mut Int>) -> Option<usize> {
    let mut index = 0;
    let entry = snapshot!(steps);
    #[variant(haystack@.len() - index@)]
    #[invariant(index@ <= haystack@.len())]
    #[invariant(forall<j: Int> 0 <= j && j < index@ ==> haystack@[j] != needle)]
    #[invariant(^*steps == ^**entry)]
    #[invariant(**steps - ***entry == index@)]
    while index < haystack.len() {
        if haystack[index] == needle {
            return Some(index);
        }
        ghost! { **steps = **steps + 1int };
        index += 1;
    }
    None
}
