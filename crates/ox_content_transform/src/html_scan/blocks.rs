//! Where the blocks of an HTML embed transform close, settled for the whole
//! document at once.
//!
//! [`crate::tabs`] and [`crate::pm`] expand `<tabs>…</tabs>` and `<pm>…</pm>`
//! blocks. Each opener searched for its matching closer by counting nested
//! openers and closers, and each step of that count searched afresh for both,
//! so one search over a run of nested openers was O(n²) and the transform,
//! which searches again from every opener that does not close, O(n³): 64 KiB
//! of `<tabs><tab label="a">x</tab>` before one `</tabs>` took 118 s. A start
//! tag with an unclosed quote likewise walked to the end once per opener.
//!
//! Every opener's count runs over the same stream of tokens — openers, each
//! skipped to the end of its start tag, and closers — from the token after
//! it, so one pass over the stream with a stack matches every opener at once.

/// Where a quote-aware scan for the `>` that ends a start tag stops, from
/// every position: built the first time a scan runs out of document, after
/// which every scan would.
#[derive(Default)]
pub struct TagCloses {
    from: Option<Vec<u32>>,
}

const NONE: u32 = u32::MAX;

impl TagCloses {
    /// The `>` outside quotes at or after `from`, for a scan that starts
    /// outside quotes there.
    pub fn close_from(&mut self, bytes: &[u8], from: usize) -> Option<usize> {
        if let Some(index) = &self.from {
            return index.get(from).copied().filter(|at| *at != NONE).map(|at| at as usize);
        }
        let found = scan(bytes, from);
        if found.is_none() && u32::try_from(bytes.len()).is_ok_and(|len| len < NONE) {
            self.from = Some(build(bytes));
        }
        found
    }
}

fn scan(bytes: &[u8], from: usize) -> Option<usize> {
    let mut quote: Option<u8> = None;
    for (at, &byte) in bytes.iter().enumerate().skip(from) {
        match quote {
            Some(open) if byte == open => quote = None,
            Some(_) => {}
            None if byte == b'"' || byte == b'\'' => quote = Some(byte),
            None if byte == b'>' => return Some(at),
            None => {}
        }
    }
    None
}

/// The scan's only state is whether it is inside quotes and which kind, so
/// its answer from every position, in each state, comes from one walk back
/// from the end.
fn build(bytes: &[u8]) -> Vec<u32> {
    // From the current position: outside quotes, inside `"`, inside `'`.
    let mut close = [NONE; 3];
    let mut from = vec![NONE; bytes.len() + 1];
    for at in (0..bytes.len()).rev() {
        let after = close;
        close = match bytes[at] {
            b'"' => [after[1], after[0], after[2]],
            b'\'' => [after[2], after[1], after[0]],
            #[allow(clippy::cast_possible_truncation)]
            b'>' => [at as u32, after[1], after[2]],
            _ => after,
        };
        from[at] = close[0];
    }
    from
}

/// What [`BlockMatches::close_of`] knows about an opener.
pub enum Match {
    Closes(usize),
    Unclosed,
    /// Not an opener of the token stream; the caller has to search.
    Unknown,
}

/// The closer that matches each opener of a document, by its position.
#[derive(Default)]
pub struct BlockMatches {
    matches: Option<Vec<(usize, Option<usize>)>>,
}

impl BlockMatches {
    /// The closer matching the opener at `open_at`. `find_open` and
    /// `find_close` return the first opener or closer at or after a
    /// position; `skip_open` where the stream resumes after an opener.
    pub fn close_of(
        &mut self,
        open_at: usize,
        close_len: usize,
        find_open: impl Fn(usize) -> Option<usize>,
        find_close: impl Fn(usize) -> Option<usize>,
        mut skip_open: impl FnMut(usize) -> usize,
    ) -> Match {
        let matches = self.matches.get_or_insert_with(|| {
            let mut matches = Vec::new();
            let mut stack = Vec::new();
            let (mut open, mut close) = (find_open(0), find_close(0));
            let mut search;
            while let Some(at) = open {
                // The same choice the per-opener count makes: an opener
                // first when it comes first, else a closer, and nothing once
                // closers run out but openers to record.
                if let Some(close_at) = close.filter(|close_at| *close_at <= at) {
                    if let Some(index) = stack.pop() {
                        let entry: &mut (usize, Option<usize>) = &mut matches[index];
                        entry.1 = Some(close_at);
                    }
                    search = close_at + close_len;
                } else {
                    stack.push(matches.len());
                    matches.push((at, None));
                    search = skip_open(at);
                }
                if open.is_some_and(|at| at < search) {
                    open = find_open(search);
                }
                if close.is_some_and(|at| at < search) {
                    close = find_close(search);
                }
            }
            // Closers after the last opener match what is still open.
            while let Some(close_at) = close {
                let Some(index) = stack.pop() else { break };
                matches[index].1 = Some(close_at);
                close = find_close(close_at + close_len);
            }
            matches
        });
        match matches.binary_search_by_key(&open_at, |(at, _)| *at) {
            Ok(index) => matches[index].1.map_or(Match::Unclosed, Match::Closes),
            Err(_) => Match::Unknown,
        }
    }
}
