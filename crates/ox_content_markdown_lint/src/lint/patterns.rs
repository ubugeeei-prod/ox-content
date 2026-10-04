use std::sync::LazyLock;

use regex::Regex;

pub(super) static LATIN_WORD_PATTERN: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"(?u)\p{Latin}+(?:['’-]\p{Latin}+)*").ok());
pub(super) static CJK_RUN_PATTERN: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"(?u)[\p{Han}\p{Hiragana}\p{Katakana}ー]+").ok());

pub(super) static ENTITY_PATTERN: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"&(?:#[xX][0-9a-fA-F]+|#[0-9]+|[a-zA-Z][a-zA-Z0-9]+);").ok());
