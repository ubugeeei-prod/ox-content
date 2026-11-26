//! The per-text-node search for autolink candidates, earliest first.
//!
//! Split out of `scan.rs` so that file stays the needle definitions and the
//! block-level pre-flight: this one is the search strategy and nothing else.

use memchr::memchr;

use super::domain::{DomainRun, is_host_at};
use super::scan::{
    SCHEME_FINDER, WWW_FINDER, email_domain_end, local_part_start, scheme_prefix_at,
    scheme_prefix_len, url_end, valid_boundary,
};
use super::{AutolinkScan, Candidate};
use crate::parser::inline::marker_scan::ForwardScan;

/// Yields the autolinks of one text node in order.
///
/// A link splits its node, and what follows the link is searched as a string
/// of its own: its first byte counts as start-of-text, and an email's local
/// part cannot reach back across it. That used to be done literally — one
/// fresh search per remaining suffix — and every one of those searches
/// validated from scratch whatever it met on the way to its answer, a
/// rejected needle a few hundred bytes ahead once for every link in front of
/// it. Two shapes turned that into O(n²), each growing x16 or more for every
/// x4 of input:
///
/// - `www.a.b_` repeated is one run of domain bytes holding a needle every
///   eight bytes, and each needle walked the whole run to be rejected for
///   its last segment: 125 KB took 5.4 s (see [`DomainRun`]).
/// - `www.a.b@c.d(` repeated has a `www.` candidate at the same byte as every
///   email. The email wins the tie, but the URL was measured to its end —
///   the next whitespace, here the end of the node — before losing: 125 KB
///   took 0.76 s.
///
/// One scan per node now keeps what each of the three finders has already
/// established, so the search after a link does not walk the same bytes
/// again, and a URL is measured only once it has won. Everything a finder
/// remembers holds for any later suffix, because the only two things that
/// depend on where the suffix starts are settled separately on every call:
/// a candidate sitting exactly on that first byte, and the email whose `@`
/// is the first one left.
pub(super) struct CandidateScan<'v> {
    value: &'v str,
    may_have_www: bool,
    /// First byte not yet consumed by a link: the start of the suffix the
    /// next search reads.
    floor: usize,
    at_sign: ForwardScan,
    head: EmailHead,
    email: Ahead,
    www: Ahead,
    www_run: DomainRun,
    scheme: Ahead,
}

/// What one finder found ahead: the first valid candidate whose `key` — the
/// position that orders that finder's candidates — is at or after `origin`.
///
/// Position-independent the same way [`ForwardScan`] is: nothing valid lies
/// between `origin` and `key`, so the answer is unchanged for a search that
/// starts anywhere in `origin..=key`.
#[derive(Clone, Copy)]
struct Ahead {
    origin: usize,
    /// `usize::MAX` when nothing valid is left.
    key: usize,
    start: usize,
    /// End of an email; first host byte of a URL, whose end is not known
    /// until it wins.
    end: usize,
}

impl Ahead {
    /// `usize::MAX` cannot be a real origin, so the first lookup always
    /// searches.
    const UNSEARCHED: Self = Self { origin: usize::MAX, key: 0, start: 0, end: 0 };

    const fn nothing_from(origin: usize) -> Self {
        Self { origin, key: usize::MAX, start: 0, end: 0 }
    }

    const fn covers(&self, from: usize) -> bool {
        self.origin <= from && from <= self.key
    }

    fn found(&self) -> Option<(usize, usize)> {
        (self.key != usize::MAX).then_some((self.start, self.end))
    }
}

/// The first `@` of the remaining text, which is the only one whose local
/// part can run back to where that text starts.
///
/// Both halves are kept: a link found in front of this `@` moves the start
/// of the text without changing either walk.
#[derive(Clone, Copy)]
struct EmailHead {
    at: usize,
    /// Where the local part stopped. When the walk stopped at the start of
    /// the text, a later start still bounds it: every byte in between is a
    /// local-part byte.
    local_start: usize,
    domain_end: Option<usize>,
}

enum Winner {
    Email { end: usize },
    Url { domain_start: usize, href_prefix: &'static str },
}

impl<'v> CandidateScan<'v> {
    pub(super) const fn new(value: &'v str, scan: AutolinkScan) -> Self {
        Self {
            value,
            may_have_www: scan.may_have_www,
            floor: 0,
            at_sign: ForwardScan::new(),
            head: EmailHead { at: usize::MAX, local_start: 0, domain_end: None },
            email: Ahead::UNSEARCHED,
            www: Ahead::UNSEARCHED,
            www_run: DomainRun::NONE,
            scheme: Ahead::UNSEARCHED,
        }
    }

    /// The earliest valid candidate in what is left of the node, which is
    /// then consumed up to the candidate's end.
    pub(super) fn next(&mut self) -> Option<Candidate> {
        crate::profile_span_detail!("parser::gfm_autolink_scan");
        let floor = self.floor;
        if floor >= self.value.len() {
            return None;
        }

        // The earliest start wins. Two kinds can only start on the same byte
        // when an email's local part begins with `www.`, and the email has
        // always taken that tie.
        let mut best = self.email(floor).map(|(start, end)| (start, Winner::Email { end }));
        let www = if self.may_have_www { self.www(floor) } else { None };
        for (found, href_prefix) in [(www, "http://"), (self.scheme(floor), "")] {
            if let Some((start, domain_start)) = found
                && best.as_ref().is_none_or(|(current, _)| start < *current)
            {
                best = Some((start, Winner::Url { domain_start, href_prefix }));
            }
        }

        let (start, winner) = best?;
        let candidate = match winner {
            Winner::Email { end } => Candidate { start, end, href_prefix: "mailto:" },
            Winner::Url { domain_start, href_prefix } => {
                Candidate { start, end: url_end(self.value, start, domain_start), href_prefix }
            }
        };
        self.floor = candidate.end;
        Some(candidate)
    }

    /// Start and end of the first valid email at or after `floor`.
    fn email(&mut self, floor: usize) -> Option<(usize, usize)> {
        let value = self.value;
        let bytes = value.as_bytes();
        let at = self.at_sign.hit(floor, |from| {
            memchr(b'@', &bytes[from..]).map_or(bytes.len(), |offset| from + offset)
        });
        if at == bytes.len() {
            return None;
        }

        if self.head.at != at {
            self.head = EmailHead {
                at,
                local_start: local_part_start(bytes, floor, at),
                domain_end: email_domain_end(bytes, at),
            };
        }
        let start = self.head.local_start.max(floor);
        if let Some(end) = self.head.domain_end
            && start < at
            && (start == floor || valid_boundary(value, start))
        {
            return Some((start, end));
        }

        if !self.email.covers(at + 1) {
            self.email = find_email(value, at + 1);
        }
        self.email.found()
    }

    /// Start and first host byte of the first valid `www.` URL at or after
    /// `floor`.
    fn www(&mut self, floor: usize) -> Option<(usize, usize)> {
        let bytes = self.value.as_bytes();
        if bytes[floor..].starts_with(b"www.") && is_host_at(bytes, floor + 4) {
            return Some((floor, floor + 4));
        }
        if !self.www.covers(floor + 1) {
            self.www = self.find_www(floor + 1);
        }
        self.www.found()
    }

    /// The first valid `www.` URL that starts at or after `origin`, behind a
    /// boundary character.
    fn find_www(&mut self, origin: usize) -> Ahead {
        let bytes = self.value.as_bytes();
        let mut from = origin;
        while let Some(offset) = bytes.get(from..).and_then(|rest| WWW_FINDER.find(rest)) {
            let at = from + offset;
            let domain_start = at + 4;
            if valid_boundary(self.value, at) {
                if !self.www_run.covers(domain_start) {
                    self.www_run = DomainRun::starting_at(bytes, domain_start);
                }
                if self.www_run.is_host_from(domain_start) {
                    return Ahead { origin, key: at, start: at, end: domain_start };
                }
            }
            from = domain_start;
        }
        Ahead::nothing_from(origin)
    }

    /// Start and first host byte of the first valid `scheme://` URL at or
    /// after `floor`.
    fn scheme(&mut self, floor: usize) -> Option<(usize, usize)> {
        let value = self.value;
        let bytes = value.as_bytes();
        if let Some(prefix_len) = scheme_prefix_at(bytes, floor)
            && is_host_at(bytes, floor + prefix_len)
        {
            return Some((floor, floor + prefix_len));
        }
        if !self.scheme.covers(floor + 1) {
            self.scheme = find_scheme(value, floor + 1);
        }
        self.scheme.found()
    }
}

/// The first valid email whose `@` is at or after `origin`, which is one
/// past an earlier `@`: that `@` ends every local part behind it, so none of
/// them can reach the start of the text.
fn find_email(value: &str, origin: usize) -> Ahead {
    let bytes = value.as_bytes();
    let mut from = origin;
    while let Some(offset) = bytes.get(from..).and_then(|rest| memchr(b'@', rest)) {
        let at = from + offset;
        let start = local_part_start(bytes, origin, at);
        if start < at
            && valid_boundary(value, start)
            && let Some(end) = email_domain_end(bytes, at)
        {
            return Ahead { origin, key: at, start, end };
        }
        from = at + 1;
    }
    Ahead::nothing_from(origin)
}

/// The first valid `scheme://` URL that starts at or after `origin`, behind
/// a boundary character.
///
/// A `://` ends its own run of domain bytes, so no two of these candidates
/// share a host and each is judged on its own.
fn find_scheme(value: &str, origin: usize) -> Ahead {
    let bytes = value.as_bytes();
    let mut from = origin;
    while let Some(offset) = bytes.get(from..).and_then(|rest| SCHEME_FINDER.find(rest)) {
        let at = from + offset;
        let domain_start = at + 3;
        if let Some(prefix_len) = scheme_prefix_len(bytes, at) {
            // `domain_start - prefix_len` steps back over the scheme name
            // to the first byte of the candidate.
            let start = domain_start - prefix_len;
            if start >= origin && valid_boundary(value, start) && is_host_at(bytes, domain_start) {
                return Ahead { origin, key: start, start, end: domain_start };
            }
        }
        from = domain_start;
    }
    Ahead::nothing_from(origin)
}
