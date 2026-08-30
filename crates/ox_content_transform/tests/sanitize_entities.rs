//! The sanitizer runs over already-escaped HTML, so an attribute value
//! reaching it is escaped text, not the value itself. It used to escape
//! that text a second time — turning a query string's `&amp;` into a
//! literal `&amp;` — and to run its URL screening against the escaped
//! form, where `&#106;avascript:` reads as a harmless relative path that
//! the browser then decodes back into a scheme.
//!
//! Values are now decoded once, screened, and escaped once on the way out.

use ox_content_transform::sanitize::sanitize_html;
use ox_content_transform::transformer::MarkdownTransformer;
use ox_content_transform::{SanitizeOptions, TransformOptions};

fn sanitize(html: &str) -> String {
    sanitize_html(html, None)
}

fn transform_sanitized(markdown: &str) -> String {
    let transformer = MarkdownTransformer::from_options(&TransformOptions {
        gfm: Some(true),
        sanitize: Some(SanitizeOptions::default()),
        ..Default::default()
    });
    transformer.transform(markdown).html.trim().to_string()
}

#[test]
fn a_query_string_keeps_its_ampersands() {
    assert_eq!(
        transform_sanitized("[q](/search?a=1&b=2)"),
        "<p><a href=\"/search?a=1&amp;b=2\">q</a></p>"
    );
    assert_eq!(sanitize("<a href=\"/x?a=1&amp;b=2\">t</a>"), "<a href=\"/x?a=1&amp;b=2\">t</a>");
}

#[test]
fn text_attributes_keep_their_escapes() {
    assert_eq!(
        transform_sanitized("![alt \"x\" & y](/i.png \"t & u\")"),
        "<p><img src=\"/i.png\" alt=\"alt &quot;x&quot; &amp; y\" title=\"t &amp; u\"></p>"
    );
    assert_eq!(
        sanitize("<a title=\"&lt;b&gt; &amp;amp; &quot;q&quot;\">t</a>"),
        "<a title=\"&lt;b&gt; &amp;amp; &quot;q&quot;\">t</a>"
    );
}

#[test]
fn sanitizing_twice_changes_nothing() {
    // Every escape the sanitizer emits has to survive its own next pass;
    // otherwise a value drifts a little further from the author's every
    // time the HTML is handled again.
    for html in [
        "<a href=\"/x?a=1&amp;b=2\" title=\"a &amp; b\">t</a>",
        "<img src=\"/i.png\" alt=\"&quot;q&quot; &amp; &lt;b&gt;\">",
        "<a href=\"/a&#63;b\">t</a>",
        "<pre data-ox-code-source=\"line 1&#10;line 2\"><code>x</code></pre>",
    ] {
        let once = sanitize(html);
        assert_eq!(sanitize(&once), once, "not idempotent for {html}");
    }
}

#[test]
fn a_line_break_in_an_attribute_stays_encoded() {
    assert_eq!(
        sanitize("<pre data-ox-code-source=\"line 1&#10;line 2\"><code>x</code></pre>"),
        "<pre data-ox-code-source=\"line 1&#10;line 2\"><code>x</code></pre>"
    );
    assert_eq!(sanitize("<a title=\"a\nb\">t</a>"), "<a title=\"a&#10;b\">t</a>");
}

#[test]
fn an_encoded_scheme_is_refused_rather_than_escaped() {
    // A browser decodes these back into `javascript:`, so screening has to
    // see them decoded. Dropping the attribute is the only safe answer.
    for html in [
        "<a href=\"&#106;avascript:alert(1)\">t</a>",
        "<a href=\"&#74;avaScript:alert(1)\">t</a>",
        "<a href=\"&#x6a;avascript:alert(1)\">t</a>",
        "<a href=\"javascript&#58;alert(1)\">t</a>",
        "<a href=\"&#106;&#97;&#118;&#97;script&#58;alert(1)\">t</a>",
    ] {
        assert_eq!(sanitize(html), "<a>t</a>", "for {html}");
    }
    assert_eq!(sanitize("<img src=\"&#106;avascript:alert(1)\">"), "<img>");
}

#[test]
fn a_reference_the_decoder_does_not_know_stays_inert() {
    // `&colon;` would decode to `:` in a browser, so it must not reach the
    // page as written; escaping the `&` keeps it literal text, which is
    // what it was before decoding existed.
    assert_eq!(
        sanitize("<a href=\"javascript&colon;alert(1)\">t</a>"),
        "<a href=\"javascript&amp;colon;alert(1)\">t</a>"
    );
    assert_eq!(
        sanitize("<img alt=\"&unknown; &notreal\">"),
        "<img alt=\"&amp;unknown; &amp;notreal\">"
    );
}

#[test]
fn an_encoded_path_decodes_to_the_path_the_author_wrote() {
    assert_eq!(sanitize("<a href=\"/a&#63;b\">t</a>"), "<a href=\"/a?b\">t</a>");
    assert_eq!(sanitize("<a href=\"/a&#35;frag\">t</a>"), "<a href=\"/a#frag\">t</a>");
    // U+0000 is not a character a URL may carry; the parser maps it to the
    // replacement character and so does this.
    assert_eq!(sanitize("<a href=\"&#0;/x\">t</a>"), "<a href=\"\u{FFFD}/x\">t</a>");
}

#[test]
fn plain_urls_and_schemes_behave_as_before() {
    assert_eq!(sanitize("<a href=\"https://x/p?a=1\">t</a>"), "<a href=\"https://x/p?a=1\">t</a>");
    assert_eq!(sanitize("<a href=\"mailto:a@b.c\">t</a>"), "<a href=\"mailto:a@b.c\">t</a>");
    assert_eq!(sanitize("<a href=\"javascript:alert(1)\">t</a>"), "<a>t</a>");
    assert_eq!(sanitize("<a href=\"vbscript:x\">t</a>"), "<a>t</a>");
    assert_eq!(sanitize("<a href=\"data:text/html,x\">t</a>"), "<a>t</a>");
}
