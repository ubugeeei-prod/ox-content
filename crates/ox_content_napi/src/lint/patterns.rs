use std::sync::LazyLock;

use regex::Regex;

pub(super) static HEADING_PATTERN: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"^\s{0,3}(#{1,6})[ \t]+(.+?)\s*#*\s*$").ok());
pub(super) static FENCE_PATTERN: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"^\s{0,3}(`{3,}|~{3,})").ok());
pub(super) static LIST_PREFIX_PATTERN: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"^\s{0,3}(?:#{1,6}[ \t]+|>\s?|(?:[-*+]|(?:\d+[.)]))[ \t]+)").ok());
pub(super) static REFERENCE_DEFINITION_PATTERN: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"^\s{0,3}\[[^\]]+\]:").ok());
pub(super) static URL_PATTERN: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"(?iu)\b(?:https?://|mailto:|www\.)\S+").ok());
pub(super) static HTML_TAG_PATTERN: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"(?u)</?[\p{L}!][^>]*>").ok());
pub(super) static FOOTNOTE_PATTERN: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"\[\^[^\]]+\]").ok());
pub(super) static LATIN_WORD_PATTERN: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"(?u)\p{Latin}+(?:['’-]\p{Latin}+)*").ok());
pub(super) static CJK_RUN_PATTERN: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"(?u)[\p{Han}\p{Hiragana}\p{Katakana}ー]+").ok());
