use crate::transformer::MarkdownTransformer;
use crate::{MathOptions, TransformOptions};

fn transform_html(source: &str, options: TransformOptions) -> String {
    MarkdownTransformer::from_options(&options).transform(source).html
}

fn math_on() -> TransformOptions {
    TransformOptions {
        gfm: Some(true),
        math: Some(MathOptions { enabled: Some(true) }),
        ..Default::default()
    }
}

#[test]
fn disabled_by_default() {
    let html = transform_html("$E=mc^2$", TransformOptions::default());
    assert!(html.contains("$E=mc^2$"), "{html}");
    assert!(!html.contains("ox-math"), "{html}");
}

#[test]
fn inline_happy_path() {
    let html = transform_html("$E=mc^2$", math_on());
    assert!(html.contains(r#"class="ox-math ox-math-inline""#), "{html}");
    assert!(html.contains("<mtext>E=mc^2</mtext>"), "{html}");
    assert!(!html.contains("ox-math-block"), "{html}");
}

#[test]
fn block_happy_path() {
    let html = transform_html("$$E=mc^2$$", math_on());
    assert!(html.contains(r#"class="ox-math ox-math-block""#), "{html}");
    assert!(html.contains(r#"<math display="block">"#), "{html}");
    assert!(html.contains("<mtext>E=mc^2</mtext>"), "{html}");
}

#[test]
fn skips_fenced_code() {
    let html = transform_html("```\n$E=mc^2$\n```\n", math_on());
    assert!(!html.contains("ox-math"), "{html}");
    assert!(html.contains("$E=mc^2$"), "{html}");
}

#[test]
fn skips_inline_code() {
    let html = transform_html("Use `$E=mc^2$` in docs.\n", math_on());
    assert!(!html.contains("ox-math"), "{html}");
    assert!(html.contains("<code>$E=mc^2$</code>"), "{html}");
}

#[test]
fn currency_dollar_stays_literal() {
    let html = transform_html("It costs $5.", math_on());
    assert!(!html.contains("ox-math"), "{html}");
    assert!(html.contains("$5"), "{html}");

    let html = transform_html("From $5.00 to $5-$10 plus US$ and a lone $.", math_on());
    assert!(!html.contains("ox-math"), "{html}");
    assert!(html.contains("$5.00"), "{html}");
    assert!(html.contains("$5-$10"), "{html}");
    assert!(html.contains("US$"), "{html}");
}

#[test]
fn unclosed_dollar_stays_literal() {
    let html = transform_html("$E=mc^2 and more text", math_on());
    assert!(!html.contains("ox-math"), "{html}");
    assert!(html.contains("$E=mc^2"), "{html}");

    let html = transform_html("$$E=mc^2 and more text", math_on());
    assert!(!html.contains("ox-math"), "{html}");
    assert!(html.contains("$$E=mc^2"), "{html}");
}

#[test]
fn escaped_dollar_is_literal() {
    let html = transform_html(r"The price is \$5 and \$ stays a dollar.", math_on());
    assert!(!html.contains("ox-math"), "{html}");
    assert!(html.contains("$5"), "{html}");
    assert!(!html.contains(r"\$5"), "{html}");
    assert!(html.contains("$ stays a dollar"), "{html}");
}

#[test]
fn hostile_tex_is_escaped() {
    let html = transform_html("$<script>alert(1)</script>$", math_on());
    assert!(html.contains("ox-math-inline"), "{html}");
    assert!(!html.contains("<script>"), "{html}");
    assert!(html.contains("&lt;script&gt;"), "{html}");
    assert!(html.contains("alert(1)"), "{html}");

    let html = transform_html("$x\" onclick=\"alert(1)$", math_on());
    assert!(html.contains("ox-math-inline"), "{html}");
    assert!(html.contains("&quot;"), "{html}");
    assert!(!html.contains(r#"onclick="alert"#), "{html}");
}

#[test]
fn paragraph_display_delimiters_stay_inline() {
    let html = transform_html("Before $$x$$ after", math_on());
    assert!(html.contains(r#"class="ox-math ox-math-inline""#), "{html}");
    assert!(!html.contains("ox-math-block"), "{html}");
    assert!(!html.contains("<div"), "{html}");
    assert!(html.contains("<p>"), "{html}");
    assert!(html.contains("<mtext>x</mtext>"), "{html}");
    assert!(html.contains("Before"), "{html}");
    assert!(html.contains("after"), "{html}");
}

#[test]
fn adjacent_inline_mathes() {
    let html = transform_html("$a$$b$", math_on());
    let inline = html.matches("ox-math-inline").count();
    assert_eq!(inline, 2, "{html}");
    assert!(html.contains("<mtext>a</mtext>"), "{html}");
    assert!(html.contains("<mtext>b</mtext>"), "{html}");
}

#[test]
fn object_enables_defaults() {
    let html = transform_html(
        "$E=mc^2$",
        TransformOptions { math: Some(MathOptions { enabled: None }), ..Default::default() },
    );
    assert!(html.contains("ox-math-inline"), "{html}");
}

#[test]
fn explicit_false_stays_literal() {
    let html = transform_html(
        "$E=mc^2$",
        TransformOptions { math: Some(MathOptions { enabled: Some(false) }), ..Default::default() },
    );
    assert!(html.contains("$E=mc^2$"), "{html}");
    assert!(!html.contains("ox-math"), "{html}");
}

#[test]
fn skips_indented_code() {
    let html = transform_html("    $E=mc^2$\n", math_on());
    assert!(!html.contains("ox-math"), "{html}");
    assert!(html.contains("$E=mc^2$"), "{html}");
}

#[test]
fn github_callouts_unchanged() {
    let html = transform_html("> [!NOTE]\n> Keep this callout.\n", math_on());
    assert!(html.contains("ox-callout"), "{html}");
    assert!(html.contains("ox-callout--note"), "{html}");
    assert!(!html.contains("ox-math"), "{html}");
}

#[test]
fn unclosed_dollar_does_not_swallow_the_file() {
    let html = transform_html("$start\n\nLater paragraph stays visible.\n", math_on());
    assert!(!html.contains("ox-math"), "{html}");
    assert!(html.contains("$start"), "{html}");
    assert!(html.contains("Later paragraph stays visible."), "{html}");
}
