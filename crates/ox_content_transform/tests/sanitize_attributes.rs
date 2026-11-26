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

fn sanitize_with(html: &str, tags: &[&str], attrs: &[&str]) -> String {
    let options = ox_content_transform::SanitizeOptions {
        allowed_tags: Some(tags.iter().map(ToString::to_string).collect()),
        allowed_attributes: Some(attrs.iter().map(ToString::to_string).collect()),
        ..Default::default()
    };
    sanitize_html(html, Some(&options))
}

#[test]
fn added_attributes_that_carry_urls_are_screened_too() {
    // Only `href`, `src`, `action`, `poster` and `srcset` used to be read as
    // URLs, so the same value passed under any other name an allow list
    // added.
    assert_eq!(
        sanitize_with(
            "<svg><a xlink:href=\"javascript:alert(1)\">x</a></svg>",
            &["svg", "a"],
            &["xlink:href"]
        ),
        "<svg><a>x</a></svg>"
    );
    assert_eq!(
        sanitize_with(
            "<button formaction=\"javascript:alert(1)\" ping=\"/a javascript:b\">x</button>",
            &["button"],
            &["formaction", "ping"]
        ),
        "<button>x</button>"
    );
    assert_eq!(
        sanitize_with(
            "<svg><animate attributeName=\"href\" values=\"0;javascript:alert(1)\"/><set to=\"javascript:alert(1)\"/></svg>",
            &["svg", "animate", "set"],
            &["attributename", "values", "to"]
        ),
        "<svg><animate attributename=\"href\" /><set /></svg>"
    );
    assert_eq!(
        sanitize_with(
            "<q cite=\"https://a/\" x-src=\"javascript:b\">x</q>",
            &["q"],
            &["cite", "x-src"]
        ),
        "<q cite=\"https://a/\">x</q>"
    );
}

#[test]
fn values_that_only_look_like_schemes_stay() {
    assert_eq!(
        sanitize_with(
            "<time datetime=\"2024-01-02T10:30\" x-note=\"(min-width: 1px)\">t</time>",
            &["time"],
            &["datetime", "x-note"]
        ),
        "<time datetime=\"2024-01-02T10:30\" x-note=\"(min-width: 1px)\">t</time>"
    );
    assert_eq!(
        sanitize("<a title=\"Note: see below\" class=\"md:flex\">x</a>"),
        "<a title=\"Note: see below\" class=\"md:flex\">x</a>"
    );
}

#[test]
fn a_data_wildcard_does_not_admit_aria_attributes() {
    assert_eq!(
        sanitize_with("<p data-a=\"1\" aria-hidden=\"true\">x</p>", &["p"], &["data-*"]),
        "<p data-a=\"1\">x</p>"
    );
    assert_eq!(
        sanitize_with("<p data-a=\"1\" aria-hidden=\"true\">x</p>", &["p"], &["aria-*"]),
        "<p aria-hidden=\"true\">x</p>"
    );
}

#[test]
fn names_that_would_become_document_properties_are_dropped() {
    assert_eq!(
        sanitize("<img src=\"/a.png\" name=\"cookie\" id=\"x\"><iframe name=\"body\"></iframe>"),
        "<img src=\"/a.png\" id=\"x\"><iframe></iframe>"
    );
    // Anchors, ids and the tab inputs keep theirs.
    assert_eq!(
        sanitize("<a name=\"top\" id=\"top\">x</a><input type=\"radio\" name=\"ox-tabs-1\">"),
        "<a name=\"top\" id=\"top\">x</a><input type=\"radio\" name=\"ox-tabs-1\">"
    );
}
