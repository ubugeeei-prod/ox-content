//! The host of a bare URL candidate.
//!
//! A `www.` or `scheme://` candidate is a link only when the domain bytes
//! after its prefix form a valid domain (GFM spec "Autolinks (extension)"):
//! at least two dot-separated segments, the last two of them non-empty and
//! free of underscores, with trailing dots left to the surrounding sentence.

use memchr::memrchr;

/// One unbroken run of domain bytes, and what its tail says about a host
/// that starts anywhere inside it.
///
/// The verdict used to be reached by walking the run from the candidate and
/// splitting it on dots, once per candidate. Every `www.` inside one run
/// shares the run's end, and only the last two segments are ever judged, so
/// a line of `www.a.b_` — one run, a needle every eight bytes, each rejected
/// for the underscore at the very end — paid for the whole run each time:
/// 125 KB took 5.4 s and grew x16 for every x4 of input. The tail is read
/// once per run here and answers every candidate in it.
#[derive(Clone, Copy)]
pub(super) struct DomainRun {
    /// Where the walk that measured this run began. The run may reach
    /// further back; nothing in front of this was looked at.
    origin: usize,
    /// One past the run's last domain byte.
    end: usize,
    /// The dot between the last two segments, once trailing dots are set
    /// aside. `None` when there is no dot left to split on.
    last_dot: Option<usize>,
    /// First byte of the second-to-last segment, as far as the walk saw it.
    second_start: usize,
    /// The last underscore in those two segments.
    underscore: Option<usize>,
}

impl DomainRun {
    /// A run no position falls in, for a memo that has not been filled yet.
    pub(super) const NONE: Self =
        Self { origin: usize::MAX, end: 0, last_dot: None, second_start: 0, underscore: None };

    /// Measures the run of domain bytes that starts at `origin`.
    pub(super) fn starting_at(bytes: &[u8], origin: usize) -> Self {
        let mut end = origin.min(bytes.len());
        while end < bytes.len() && is_domain_byte(bytes[end]) {
            end += 1;
        }
        let origin = origin.min(end);

        let mut trimmed = end;
        while trimmed > origin && bytes[trimmed - 1] == b'.' {
            trimmed -= 1;
        }
        let last_dot = memrchr(b'.', &bytes[origin..trimmed]).map(|found| origin + found);
        let second_start = last_dot
            .and_then(|dot| memrchr(b'.', &bytes[origin..dot]))
            .map_or(origin, |found| origin + found + 1);
        let underscore =
            memrchr(b'_', &bytes[second_start..trimmed]).map(|found| second_start + found);

        Self { origin, end, last_dot, second_start, underscore }
    }

    /// True when the domain bytes from `start` on are this same run, so its
    /// tail is theirs too.
    pub(super) const fn covers(&self, start: usize) -> bool {
        self.origin <= start && start <= self.end
    }

    /// Whether the host that begins at `start` — a position this run
    /// [`covers`](Self::covers) — is a valid domain.
    ///
    /// The host needs the run's last dot inside it with something on both
    /// sides, and no underscore from the later of its own first byte and the
    /// second-to-last segment's on.
    pub(super) fn is_host_from(&self, start: usize) -> bool {
        self.last_dot.is_some_and(|dot| start < dot && self.second_start < dot)
            && self.underscore.is_none_or(|underscore| underscore < start)
    }
}

const fn is_domain_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.')
}

/// Whether the domain bytes at `start` form a valid domain, for a candidate
/// that shares its run with no other.
pub(super) fn is_host_at(bytes: &[u8], start: usize) -> bool {
    DomainRun::starting_at(bytes, start).is_host_from(start)
}
