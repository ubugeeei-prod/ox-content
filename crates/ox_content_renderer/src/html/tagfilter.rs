//! GFM `tagfilter` extension — "Disallowed Raw HTML".
//!
//! GFM neutralizes a small set of raw HTML tags that change how the rest of
//! the document is interpreted (a stray `<style>` or `<xmp>` swallows the
//! markup after it, and `<script>`/`<iframe>` execute or embed). Filtering
//! replaces only the leading `<` with `&lt;`, so the tag becomes visible
//! text while everything else in the raw HTML is preserved.
//!
//! Enabled via [`HtmlRendererOptions::disallow_raw_html`](super::options::HtmlRendererOptions::disallow_raw_html);
//! it is off by default because passing raw HTML through is the documented
//! Markdown behaviour that embeds (`<iframe>` video players, `<style>`
//! blocks) rely on.

/// Tags filtered by the extension, in the order the spec lists them.
const DISALLOWED: [&str; 9] =
    ["title", "textarea", "style", "xmp", "iframe", "noembed", "noframes", "script", "plaintext"];

/// Writes `value` into `out`, escaping the `<` of every disallowed tag.
pub(super) fn write_filtered_into(out: &mut String, value: &str) {
    let bytes = value.as_bytes();
    let mut copied = 0;
    let mut i = 0;

    while let Some(offset) = memchr::memchr(b'<', &bytes[i..]) {
        let lt = i + offset;
        // A closing tag (`</style>`) is filtered exactly like an opening one.
        let name_start = if bytes.get(lt + 1) == Some(&b'/') { lt + 2 } else { lt + 1 };

        if let Some(name) = matching_tag(&value[name_start.min(value.len())..]) {
            out.push_str(&value[copied..lt]);
            out.push_str("&lt;");
            copied = lt + 1;
            i = name_start + name.len();
        } else {
            i = lt + 1;
        }
    }

    out.push_str(&value[copied..]);
}

/// Returns the disallowed tag name at the start of `rest`, if any.
///
/// The match is case-insensitive and must end at a tag boundary so that
/// longer names starting with a disallowed one (`<titlebar>`) pass through.
fn matching_tag(rest: &str) -> Option<&'static str> {
    DISALLOWED.into_iter().find(|tag| {
        let Some(after) = rest.get(..tag.len()) else {
            return false;
        };
        if !after.eq_ignore_ascii_case(tag) {
            return false;
        }
        // End of input counts as a boundary: cmark-gfm filters a trailing
        // `<script` with no `>` just as it filters a complete tag.
        rest.as_bytes()
            .get(tag.len())
            .is_none_or(|byte| byte.is_ascii_whitespace() || matches!(byte, b'>' | b'/'))
    })
}

/// Whether `value` contains anything the filter would rewrite. Lets callers
/// skip allocating a filtered copy for the overwhelmingly common case of
/// raw HTML that holds no disallowed tag.
pub(super) fn needs_filtering(value: &str) -> bool {
    let bytes = value.as_bytes();
    let mut i = 0;
    while let Some(offset) = memchr::memchr(b'<', &bytes[i..]) {
        let lt = i + offset;
        let name_start = if bytes.get(lt + 1) == Some(&b'/') { lt + 2 } else { lt + 1 };
        if matching_tag(&value[name_start.min(value.len())..]).is_some() {
            return true;
        }
        i = lt + 1;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn filter(value: &str) -> String {
        let mut out = String::new();
        write_filtered_into(&mut out, value);
        out
    }

    #[test]
    fn filters_disallowed_tags_case_insensitively() {
        assert_eq!(filter("<strong> <title>"), "<strong> &lt;title>");
        assert_eq!(filter("<XMP> and <xmp>"), "&lt;XMP> and &lt;xmp>");
        assert_eq!(filter("</script>"), "&lt;/script>");
        assert_eq!(filter("<script src=\"a.js\">"), "&lt;script src=\"a.js\">");
    }

    #[test]
    fn leaves_other_markup_untouched() {
        assert_eq!(filter("<em>hi</em>"), "<em>hi</em>");
        assert_eq!(filter("a < b and c > d"), "a < b and c > d");
        // Longer names that merely start with a disallowed tag are allowed.
        assert_eq!(filter("<titlebar> <styles>"), "<titlebar> <styles>");
    }

    #[test]
    fn detects_whether_filtering_is_needed() {
        assert!(needs_filtering("<p><iframe></p>"));
        assert!(!needs_filtering("<p><em>plain</em></p>"));
        assert!(!needs_filtering("no tags at all"));
    }

    #[test]
    fn filters_truncated_trailing_tag() {
        assert_eq!(filter("text <script"), "text &lt;script");
    }
}
