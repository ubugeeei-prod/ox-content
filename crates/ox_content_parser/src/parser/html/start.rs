use super::super::Parser;

/// Classification of an HTML block opener on the current line.
///
/// `parse_block` already has the current line and its trimmed view in hand.
/// Carrying this compact value into `parse_html_block` avoids reparsing that
/// same line after taking `&mut self`, while still preserving the exact block
/// kind decisions needed by the Markdown HTML block rules implemented here.
#[derive(Clone, Copy)]
pub(in crate::parser) enum HtmlBlockStart {
    /// An HTML comment block beginning with `<!--`.
    Comment,
    /// A processing instruction, declaration, or CDATA block: consumes
    /// lines until one contains the given terminator.
    Terminated(&'static str),
    /// A type-1 raw HTML block whose closing tag can appear after blank lines.
    Type1(Type1HtmlBlockTag),
    /// A regular supported HTML block that ends before the next blank line.
    Other,
}

/// Supported type-1 HTML block tags.
///
/// These are stored as an enum instead of a borrowed `&str` so
/// `parse_html_block` can keep mutating the parser cursor without holding an
/// immutable borrow into `self.source`.
#[derive(Clone, Copy)]
pub(in crate::parser) enum Type1HtmlBlockTag {
    Pre,
    Script,
    Style,
    Textarea,
}

impl Type1HtmlBlockTag {
    pub(super) fn closing_name(self) -> &'static [u8] {
        match self {
            Self::Pre => b"pre",
            Self::Script => b"script",
            Self::Style => b"style",
            Self::Textarea => b"textarea",
        }
    }
}

impl<'a> Parser<'a> {
    /// Classifies a trimmed line as a supported HTML block opener.
    ///
    /// The caller passes `trimmed` because `parse_block` and
    /// `line_starts_block` have already paid for `trim_start()`. Returning the
    /// full block kind lets the parser reuse that dispatch work in the actual
    /// parse step.
    pub(in crate::parser) fn parse_html_block_start(trimmed: &str) -> Option<HtmlBlockStart> {
        if trimmed.starts_with("<!--") {
            return Some(HtmlBlockStart::Comment);
        }
        if trimmed.starts_with("<?") {
            return Some(HtmlBlockStart::Terminated("?>"));
        }
        if trimmed.starts_with("<![CDATA[") {
            return Some(HtmlBlockStart::Terminated("]]>"));
        }
        if trimmed.starts_with("<!")
            && trimmed.as_bytes().get(2).is_some_and(u8::is_ascii_alphabetic)
        {
            return Some(HtmlBlockStart::Terminated(">"));
        }

        let tag_name = Self::parse_html_block_tag_name_from_trimmed(trimmed)?;
        let closing = trimmed.as_bytes().get(1) == Some(&b'/');
        match Self::html_block_start_for_tag(tag_name) {
            // Type 1 starts only on open tags; a lone `</pre>` is a
            // type-7 candidate that must not interrupt paragraphs.
            Some(HtmlBlockStart::Type1(_)) if closing => None,
            other => other,
        }
    }

    /// Type-7 HTML block: a line holding one complete open or closing tag
    /// (validated by the inline tag scanner) followed only by whitespace.
    /// This kind never interrupts a paragraph, so only `parse_block` uses
    /// it — never `line_starts_block`.
    pub(in crate::parser) fn is_html_block_type7_line(trimmed: &'a str) -> bool {
        let Some((_, end)) = Self::parse_inline_html(trimmed, 0, 0) else {
            return false;
        };
        trimmed[end..].trim().is_empty()
    }

    /// Returns the raw tag name from a line already known to begin with `<`.
    ///
    /// This intentionally does not check whether the tag is supported. Keeping
    /// syntax extraction separate from support classification lets
    /// `html_block_start_for_tag` return the exact `HtmlBlockStart` variant in
    /// one pass.
    fn parse_html_block_tag_name_from_trimmed(trimmed: &str) -> Option<&str> {
        let after_open = trimmed.strip_prefix('<')?;
        let after_slash = after_open.strip_prefix('/').unwrap_or(after_open);
        let mut tag_len = 0;

        for byte in after_slash.as_bytes() {
            if byte.is_ascii_alphanumeric() || *byte == b'-' {
                tag_len += 1;
            } else {
                break;
            }
        }

        if tag_len == 0 {
            return None;
        }

        let tag_name = &after_slash[..tag_len];
        let next = after_slash.as_bytes().get(tag_len).copied();

        if let Some(byte) = next {
            if !matches!(byte, b' ' | b'\t' | b'>' | b'/') {
                return None;
            }
        }

        Some(tag_name)
    }

    /// Maps a parsed tag name to the supported HTML block kind.
    ///
    /// The previous implementation walked a fixed slice of tag strings for
    /// every `<...>` opener. Length bucketing keeps this allocation-free and
    /// trims comparisons on generated API docs with many HTML blocks.
    fn html_block_start_for_tag(tag_name: &str) -> Option<HtmlBlockStart> {
        match tag_name.len() {
            3 if tag_name.eq_ignore_ascii_case("pre") => {
                return Some(HtmlBlockStart::Type1(Type1HtmlBlockTag::Pre));
            }
            5 if tag_name.eq_ignore_ascii_case("style") => {
                return Some(HtmlBlockStart::Type1(Type1HtmlBlockTag::Style));
            }
            6 if tag_name.eq_ignore_ascii_case("script") => {
                return Some(HtmlBlockStart::Type1(Type1HtmlBlockTag::Script));
            }
            8 if tag_name.eq_ignore_ascii_case("textarea") => {
                return Some(HtmlBlockStart::Type1(Type1HtmlBlockTag::Textarea));
            }
            _ => {}
        }
        // The CommonMark type-6 tag list (spec 4.6), length-bucketed to
        // keep classification allocation-free.
        let known: &[&str] = match tag_name.len() {
            1 => &["p"],
            2 => &[
                "dd", "dl", "dt", "h1", "h2", "h3", "h4", "h5", "h6", "hr", "li", "ol", "td", "th",
                "tr", "ul",
            ],
            3 => &["col", "dir", "div", "nav"],
            4 => &["base", "body", "form", "head", "html", "link", "main", "menu"],
            5 => &["aside", "frame", "param", "table", "tbody", "tfoot", "thead", "title", "track"],
            6 => &[
                "center", "dialog", "figure", "footer", "header", "iframe", "legend", "option",
                "search",
            ],
            7 => &["address", "article", "caption", "details", "section", "summary"],
            8 => &[
                "basefont", "colgroup", "fieldset", "frameset", "menuitem", "noframes", "optgroup",
            ],
            10 => &["blockquote", "figcaption"],
            _ => &[],
        };
        known
            .iter()
            .any(|name| tag_name.eq_ignore_ascii_case(name))
            .then_some(HtmlBlockStart::Other)
    }
}
