//! GitHub-style block quote callout markers.
//!
//! The renderer recognizes markers such as `[!NOTE]` only at the beginning of a
//! block quote paragraph. This module keeps the marker grammar and presentation labels
//! together so block rendering can stay focused on emitting HTML.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CalloutKind {
    Note,
    Tip,
    Important,
    Warning,
    Caution,
}

impl CalloutKind {
    pub(super) fn parse_marker(value: &str) -> Option<(Self, &str)> {
        let marker = value.strip_prefix("[!")?;
        let end = marker.find(']')?;
        // Allocation-free: the previous `to_ascii_uppercase().as_str()`
        // path allocated a fresh `String` for every `[!FOO]`-prefixed
        // text run that reached this branch. `eq_ignore_ascii_case`
        // compares the trimmed slice in place against each known label.
        let name = marker[..end].trim();
        let kind = if name.eq_ignore_ascii_case("NOTE") {
            Self::Note
        } else if name.eq_ignore_ascii_case("TIP") {
            Self::Tip
        } else if name.eq_ignore_ascii_case("IMPORTANT") {
            Self::Important
        } else if name.eq_ignore_ascii_case("WARNING") {
            Self::Warning
        } else if name.eq_ignore_ascii_case("CAUTION") {
            Self::Caution
        } else {
            return None;
        };

        Some((kind, marker[end + 1..].trim_start_matches(char::is_whitespace)))
    }

    pub(super) fn class_name(self) -> &'static str {
        match self {
            Self::Note => "note",
            Self::Tip => "tip",
            Self::Important => "important",
            Self::Warning => "warning",
            Self::Caution => "caution",
        }
    }

    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Note => "Note",
            Self::Tip => "Tip",
            Self::Important => "Important",
            Self::Warning => "Warning",
            Self::Caution => "Caution",
        }
    }
}
