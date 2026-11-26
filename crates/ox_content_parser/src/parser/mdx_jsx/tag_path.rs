//! One remembered walk over the tags of a parser source.
//!
//! `find_matching_close` walks from an opening tag to the end of its input
//! and only then reports that nothing closed it. The caller keeps the tag
//! as literal text and moves on to the next opener, which repeats the walk
//! from three bytes further in: 31 KiB of `<A>` took 1.8 s and 125 KiB took
//! 20.7 s. Nested elements paid the same way when they did close, once per
//! level.
//!
//! The walk itself does not depend on which tag name is being matched, so
//! one pass can pair every opener it meets with its closer, per name, the
//! way the depth counter in `find_matching_close` would. [`TagPath`] keeps
//! that pass: an opener that lies on it reads its answer back, and only an
//! opener the pass stepped over, or one beyond it, starts a new pass.

use std::cell::OnceCell;

use compact_str::CompactString;
use memchr::{memchr3, memrchr};
use rustc_hash::FxHashMap;
use smallvec::SmallVec;

use super::close::{self, TagSkip};
use super::lex::Lex;
use super::lex_memo::LexMemo;

const NONE: u32 = u32::MAX;
const NOT_AN_OPENER: u32 = u32::MAX - 1;

pub(super) enum Close {
    Found(usize, usize),
    Missing,
    /// The remembered walk does not settle this slice; scan it directly.
    Unknown,
}

struct Tag {
    start: u32,
    end: u32,
    /// Index of the closing tag, `NONE` while unmatched, or `NOT_AN_OPENER`.
    partner: u32,
}

#[derive(Default)]
pub(super) struct TagPath {
    /// Offset of the last `</` in the source. No element closes after it.
    last_close: OnceCell<Option<usize>>,
    /// Where the walk resumes. Everything before it, back to the first
    /// tag, has been stepped through.
    frontier: usize,
    /// The walk is past `last_close`; unmatched openers stay unmatched.
    complete: bool,
    /// Tags on the walk, in source order.
    tags: SmallVec<[Tag; 4]>,
    /// Brace groups and code spans the walk stepped over, in source order.
    skips: SmallVec<[(u32, u32); 4]>,
    /// Innermost open tag per name, as an index into `open`. Entries carry
    /// the walk they belong to so a new walk does not have to clear them.
    innermost: FxHashMap<CompactString, (u32, u32)>,
    /// Open tags as (enclosing open tag of the same name, tag index).
    open: SmallVec<[(u32, u32); 4]>,
    walk: u32,
}

impl TagPath {
    /// `find_matching_close` for the opener spanning `open_start..open_end`,
    /// limited to `source[..limit]`.
    pub(super) fn matching_close(
        &mut self,
        memo: &mut LexMemo,
        source: &str,
        open_start: usize,
        open_end: usize,
        limit: usize,
    ) -> Close {
        let bytes = source.as_bytes();
        let Some(last_close) = self.last_close(source).filter(|at| *at >= open_end) else {
            return Close::Missing;
        };
        let mut lex = Lex::shared(bytes, memo, bytes, 0);
        let index = if let Some(index) = self.tag_at(open_start) {
            index
        } else {
            self.restart(open_start);
            self.step(&mut lex, source, last_close);
            let Some(index) = self.tag_at(open_start) else {
                return Close::Unknown;
            };
            index
        };
        if self.tags[index].end as usize != open_end || self.tags[index].partner == NOT_AN_OPENER {
            return Close::Unknown;
        }
        while self.tags[index].partner == NONE && !self.complete && self.frontier < limit {
            self.step(&mut lex, source, last_close);
        }
        if let Some(close) = self.tags.get(self.tags[index].partner as usize)
            && close.end as usize <= limit
        {
            return Close::Found(close.start as usize, close.end as usize);
        }
        // Nothing on the walk closes the opener before `limit`. The slice
        // takes the same steps up to its end unless one of them crosses it.
        if self.crosses(limit) { Close::Unknown } else { Close::Missing }
    }

    /// Whether any closing tag starts at or after `from`. When none does,
    /// no opener that ends there can be closed.
    pub(super) fn closes_after(&self, source: &str, from: usize) -> bool {
        self.last_close(source).is_some_and(|at| at >= from)
    }

    /// Whether the walk has stepped onto the tag that starts at `start`.
    pub(super) fn covers(&self, start: usize) -> bool {
        self.tag_at(start).is_some()
    }

    /// How many walks have been started.
    #[cfg(test)]
    pub(super) fn walks(&self) -> u32 {
        self.walk
    }

    fn last_close(&self, source: &str) -> Option<usize> {
        *self.last_close.get_or_init(|| {
            let bytes = source.as_bytes();
            let mut end = bytes.len();
            while let Some(slash) = memrchr(b'/', &bytes[..end]) {
                if slash > 0 && bytes[slash - 1] == b'<' {
                    return Some(slash - 1);
                }
                end = slash;
            }
            None
        })
    }

    fn tag_at(&self, start: usize) -> Option<usize> {
        self.tags.binary_search_by_key(&(start as u32), |tag| tag.start).ok()
    }

    fn restart(&mut self, at: usize) {
        self.frontier = at;
        self.complete = false;
        self.tags.clear();
        self.skips.clear();
        self.open.clear();
        if self.walk == u32::MAX {
            self.innermost.clear();
        }
        self.walk = self.walk.wrapping_add(1);
    }

    /// Takes the step `find_matching_close` takes from the frontier.
    fn step(&mut self, lex: &mut Lex<'_>, source: &str, last_close: usize) {
        let bytes = source.as_bytes();
        let next = memchr3(b'{', b'`', b'<', &bytes[self.frontier..]);
        let Some(at) = next.map(|found| self.frontier + found).filter(|at| *at <= last_close)
        else {
            self.complete = true;
            return;
        };
        let end = match bytes[at] {
            b'{' => lex.skip_braces(at),
            b'`' => lex.skip_backticks(at),
            _ => close::scan_tag_skip(lex, source, at).map(|tag| self.push_tag(&tag)),
        };
        let Some(end) = end else {
            self.frontier = at + 1;
            return;
        };
        if bytes[at] != b'<' {
            self.skips.push((at as u32, end as u32));
        }
        self.frontier = end;
    }

    /// Records a tag and pairs it the way a per-name depth counter would.
    fn push_tag(&mut self, tag: &TagSkip<'_>) -> usize {
        let index = self.tags.len() as u32;
        let name = tag.name.unwrap_or("");
        let mut partner = NOT_AN_OPENER;
        if tag.closing {
            if let Some((walk, innermost)) = self.innermost.get_mut(name)
                && *walk == self.walk
                && let Some(&(enclosing, opener)) = self.open.get(*innermost as usize)
            {
                self.tags[opener as usize].partner = index;
                *innermost = enclosing;
            }
        } else if !tag.self_closing {
            let slot = match self.innermost.get_mut(name) {
                Some(slot) => slot,
                None => self.innermost.entry(name.into()).or_insert((self.walk, NONE)),
            };
            if slot.0 != self.walk {
                *slot = (self.walk, NONE);
            }
            self.open.push((slot.1, index));
            slot.1 = (self.open.len() - 1) as u32;
            partner = NONE;
        }
        self.tags.push(Tag { start: tag.start as u32, end: tag.end as u32, partner });
        tag.end
    }

    /// Whether a recorded step starts before `limit` and ends after it.
    fn crosses(&self, limit: usize) -> bool {
        let spans = |start: u32, end: u32| (start as usize) < limit && limit < end as usize;
        let tag = self.tags.partition_point(|tag| (tag.start as usize) < limit);
        let skip = self.skips.partition_point(|skip| (skip.0 as usize) < limit);
        tag.checked_sub(1).is_some_and(|at| spans(self.tags[at].start, self.tags[at].end))
            || skip.checked_sub(1).is_some_and(|at| spans(self.skips[at].0, self.skips[at].1))
    }
}
