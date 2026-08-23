//! Re-applying type cross-reference links after highlighting.
//!
//! The docs generator writes a member type as
//! `<code>…<a href="…">NavGroup</a>[] | null</code>`. Highlighting reads the
//! text through those wrappers and replaces the element's children, which
//! used to drop every `<a>`. The ranges are recovered from the original
//! markup and laid back over the highlighted tokens.

use super::scan::find_tag_end;
use super::text::{CodeLink, next_decoded, recover_code};

/// Wrap highlighted inner HTML with the `<a>` ranges from `original_element`.
pub(super) fn reapply_links(original_element: &str, highlighted_inner: &str) -> String {
    let Some(recovered) = recover_code(original_element) else {
        return highlighted_inner.to_string();
    };
    if recovered.links.is_empty() {
        return highlighted_inner.to_string();
    }
    apply_links(highlighted_inner, &recovered.links)
}

fn apply_links(html: &str, links: &[CodeLink]) -> String {
    let mut out = String::with_capacity(html.len() + links.len() * 32);
    let mut text_pos = 0;
    let mut index = 0;
    let mut open: Option<&CodeLink> = None;

    while index < html.len() {
        if html.as_bytes()[index] == b'<' {
            close_link(&mut out, &mut open);
            if let Some(end) = find_tag_end(html, index) {
                out.push_str(&html[index..end]);
                index = end;
            } else {
                emit_text(&mut out, &html[index..], &mut text_pos, links, &mut open);
                break;
            }
        } else {
            let next = html[index..].find('<').map_or(html.len(), |relative| index + relative);
            emit_text(&mut out, &html[index..next], &mut text_pos, links, &mut open);
            index = next;
        }
    }
    close_link(&mut out, &mut open);
    out
}

fn emit_text<'a>(
    out: &mut String,
    raw: &str,
    text_pos: &mut usize,
    links: &'a [CodeLink],
    open: &mut Option<&'a CodeLink>,
) {
    let mut index = 0;
    while index < raw.len() {
        let (raw_len, decoded_len) = next_decoded(&raw[index..]);
        if raw_len == 0 {
            break;
        }
        let link = link_at(links, *text_pos);
        if !same_href(link, *open) {
            close_link(out, open);
            if let Some(link) = link {
                out.push_str("<a href=\"");
                out.push_str(&link.href);
                out.push_str("\">");
            }
            *open = link;
        }
        out.push_str(&raw[index..index + raw_len]);
        *text_pos += decoded_len;
        index += raw_len;
    }
}

fn link_at(links: &[CodeLink], pos: usize) -> Option<&CodeLink> {
    links.iter().find(|link| pos >= link.start && pos < link.end)
}

fn same_href(left: Option<&CodeLink>, right: Option<&CodeLink>) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => left.href == right.href,
        (None, None) => true,
        _ => false,
    }
}

fn close_link(out: &mut String, open: &mut Option<&CodeLink>) {
    if open.take().is_some() {
        out.push_str("</a>");
    }
}

#[cfg(test)]
mod tests {
    use super::reapply_links;
    use crate::highlight::highlight_code_blocks;

    #[test]
    fn wraps_a_linked_identifier_inside_its_token() {
        let original =
            r#"<code class="language-ts"><a href="./x.html">NavGroup</a>[] | null</code>"#;
        let highlighted = r#"<span class="tok">NavGroup</span><span>[] | null</span>"#;
        assert_eq!(
            reapply_links(original, highlighted),
            r#"<span class="tok"><a href="./x.html">NavGroup</a></span><span>[] | null</span>"#
        );
    }

    #[test]
    fn splits_a_span_when_the_link_covers_only_a_prefix() {
        let original =
            r#"<code class="language-ts"><a href="./x.html">NavGroup</a>[] | null</code>"#;
        assert_eq!(
            reapply_links(original, r"<span>NavGroup[] | null</span>"),
            r#"<span><a href="./x.html">NavGroup</a>[] | null</span>"#
        );
    }

    #[test]
    fn wraps_each_token_when_a_link_spans_two_of_them() {
        let original = r#"<code class="language-ts"><a href="./foo.html">Foo.Bar</a></code>"#;
        assert_eq!(
            reapply_links(original, r"<span>Foo</span><span>.Bar</span>"),
            r#"<span><a href="./foo.html">Foo</a></span><span><a href="./foo.html">.Bar</a></span>"#
        );
    }

    #[test]
    fn leaves_unlinked_source_alone() {
        let original = r#"<code class="language-ts">string</code>"#;
        let highlighted = r"<span>string</span>";
        assert_eq!(reapply_links(original, highlighted), highlighted);
    }

    #[test]
    fn keeps_the_link_when_the_document_pass_highlights_a_member_type() {
        let html = "<p><code class=\"mt language-ts\"><a href=\"./x.html\">NavGroup</a>[] | null</code></p>";
        let result = highlight_code_blocks(
            html,
            |_| true,
            |code, _| {
                assert_eq!(code, "NavGroup[] | null");
                Some(format!("<pre><code><span>{code}</span></code></pre>"))
            },
        );

        assert!(result.skipped.is_empty());
        assert!(
            result.html.contains("<a href=\"./x.html\">NavGroup</a>"),
            "the type cross-reference must survive highlighting: {}",
            result.html
        );
    }
}
