//! Unquoted attribute values used to end at the first `/`, so the
//! sanitizer emptied or truncated exactly the values that matter most:
//! `href=/docs/page` became `href=""`, and `href=https://x/page` became
//! `href="https:"`. HTML ends an unquoted value at whitespace, and the
//! trailing `/` of `<img src=a.png/>` belongs to the value.

use ox_content_transform::sanitize::sanitize_html;

fn sanitize(html: &str) -> String {
    sanitize_html(html, None)
}

#[test]
fn unquoted_paths_survive() {
    assert_eq!(sanitize("<a href=/docs/page>x</a>"), "<a href=\"/docs/page\">x</a>");
    assert_eq!(sanitize("<img src=/a/b.png>"), "<img src=\"/a/b.png\">");
    assert_eq!(sanitize("<a href=../up/one>x</a>"), "<a href=\"../up/one\">x</a>");
    assert_eq!(sanitize("<a href=#frag>x</a>"), "<a href=\"#frag\">x</a>");
}

#[test]
fn unquoted_absolute_urls_survive() {
    assert_eq!(
        sanitize("<a href=https://example.com/page>x</a>"),
        "<a href=\"https://example.com/page\">x</a>"
    );
    assert_eq!(
        sanitize("<a href=https://example.com/>x</a>"),
        "<a href=\"https://example.com/\">x</a>"
    );
    assert_eq!(sanitize("<a href=/a/b/c/>x</a>"), "<a href=\"/a/b/c/\">x</a>");
}

#[test]
fn several_unquoted_attributes_all_survive() {
    assert_eq!(
        sanitize("<a class=lead href=/y title=t>x</a>"),
        "<a class=\"lead\" href=\"/y\" title=\"t\">x</a>"
    );
    assert_eq!(sanitize("<a href=/y disabled>x</a>"), "<a href=\"/y\" disabled>x</a>");
}

#[test]
fn a_standalone_trailing_slash_still_closes_the_tag() {
    assert_eq!(sanitize("<br/>"), "<br />");
    assert_eq!(sanitize("<br />"), "<br />");
    assert_eq!(sanitize("<hr/>"), "<hr />");
    assert_eq!(sanitize("<img src=\"a.png\"/>"), "<img src=\"a.png\" />");
    assert_eq!(sanitize("<img src='a.png'/>"), "<img src=\"a.png\" />");
    assert_eq!(sanitize("<img src=a.png />"), "<img src=\"a.png\" />");
}

#[test]
fn a_slash_glued_to_an_unquoted_value_belongs_to_the_value() {
    // What a browser does with the same markup: the value runs to the
    // whitespace or `>`, so `a.png/` is the source, not a close marker.
    assert_eq!(sanitize("<img src=a.png/>"), "<img src=\"a.png/\">");
}

#[test]
fn unquoted_values_are_still_screened() {
    assert_eq!(sanitize("<img src=javascript:alert(1)>"), "<img>");
    assert_eq!(sanitize("<img src=/a onerror=alert(1)>"), "<img src=\"/a\">");
    assert_eq!(sanitize("<a href=vbscript:x>t</a>"), "<a>t</a>");
    // A `/` before the colon still reads as a relative path, as it must.
    assert_eq!(sanitize("<a href=/a:b>t</a>"), "<a href=\"/a:b\">t</a>");
}

#[test]
fn quoted_values_are_unchanged_by_the_new_scan() {
    assert_eq!(sanitize("<a href=\"/docs/page\">x</a>"), "<a href=\"/docs/page\">x</a>");
    assert_eq!(sanitize("<a href='/single/quoted'>x</a>"), "<a href=\"/single/quoted\">x</a>");
    assert_eq!(sanitize("<a href=\"javascript:alert(1)\">x</a>"), "<a>x</a>");
}

#[test]
fn a_broken_tag_still_collapses_to_something_inert() {
    // The classic quote-confusion payload: whatever the sanitizer makes of
    // it, no event handler and no second tag may reach the output.
    let out = sanitize("<a \"><img src=x onerror=alert(1)>\">");
    assert!(!out.contains("onerror"), "{out}");
    assert!(!out.contains("<img"), "{out}");
}
