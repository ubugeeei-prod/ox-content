use std::sync::OnceLock;

use regex::Regex;

pub(super) type RegexCache = OnceLock<Option<Regex>>;

pub(super) fn cached_regex(
    cache: &'static RegexCache,
    pattern: &'static str,
) -> Option<&'static Regex> {
    // Regex construction is expensive and these helpers run throughout doc
    // generation. Cache both success and failure in `OnceLock<Option<_>>` so a
    // bad pattern degrades to the fallback path without recompiling on every
    // call.
    cache.get_or_init(|| Regex::new(pattern).ok()).as_ref()
}
