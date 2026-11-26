use super::transform_attribute_syntax;

fn apply(input: &str) -> String {
    transform_attribute_syntax(input).unwrap_or_else(|| input.to_string())
}

#[test]
fn adjacent_link_attrs_target_anchor_inside_paragraph() {
    let html =
        apply(r#"<p><a href="https://example.com">slides</a>{#deck .text-xl data-kind=deck}</p>"#);

    assert_eq!(
        html,
        r#"<p><a href="https://example.com" id="deck" class="text-xl" data-kind="deck">slides</a></p>"#
    );
}

#[test]
fn adjacent_image_attrs_target_img_inside_paragraph() {
    let html = apply(r#"<p><img src="./image.png" alt="alt">{#hero .w-1/2 width=480}</p>"#);

    assert_eq!(
        html,
        r#"<p><img src="./image.png" alt="alt" id="hero" class="w-1/2" width="480"></p>"#
    );
}

#[test]
fn adjacent_code_and_span_attrs_target_inline_element() {
    let html = apply(r"<p><code>codeBlockTypecheck</code>{.token} and <span>beta</span>{#b}</p>");

    assert_eq!(
        html,
        r#"<p><code class="token">codeBlockTypecheck</code> and <span id="b">beta</span></p>"#
    );
}

#[test]
fn link_attrs_do_not_jump_across_text_or_comments() {
    let text = apply(r#"<p><a href="/slides">slides</a> later {.lead}</p>"#);
    let comment = apply(r#"<p><a href="/slides">slides</a><!-- stop -->{.lead}</p>"#);

    assert_eq!(text, r#"<p class="lead"><a href="/slides">slides</a> later</p>"#);
    assert_eq!(comment, r#"<p class="lead"><a href="/slides">slides</a><!-- stop --></p>"#);
}

#[test]
fn heading_attrs_still_target_heading_and_rewrite_permalink() {
    let html = apply(
        r##"<h2 id="old">Install {#new .section}<a class="header-anchor" href="#old" aria-label="Permalink to &quot;Install&quot;">#</a></h2>"##,
    );

    assert!(html.starts_with(r#"<h2 id="new" class="section">Install"#), "{html}");
    assert!(html.contains(r##"href="#new""##), "{html}");
    assert!(!html.contains(r##"href="#old""##), "{html}");
}

#[test]
fn unsafe_or_malformed_attr_blocks_stay_literal() {
    let html = apply(r#"<p><a href="/slides">slides</a>{.ok onclick=alert(1)}</p>"#);
    let href = apply(r#"<p><a href="/slides">slides</a>{href=javascript:alert(1)}</p>"#);

    assert_eq!(html, r#"<p><a href="/slides">slides</a>{.ok onclick=alert(1)}</p>"#);
    assert_eq!(href, r#"<p><a href="/slides">slides</a>{href=javascript:alert(1)}</p>"#);
}

/// HTML pieces whose mix puts attribute blocks after, inside, and away from
/// every kind of element they can apply to, with stray `<`, `>` and quotes.
const PIECES: [&str; 34] = [
    "<p>",
    "</p>",
    "<h2 id=\"x\">",
    "</h2>",
    "<a class=\"header-anchor\" href=\"#x\">#</a>",
    "<a href=\"u\">",
    "</a>",
    "<img src=\"i\">",
    "<br>",
    "<br />",
    "<wbr>",
    "<span>",
    "</span>",
    "<code>",
    "</code>",
    "<em>",
    "</em>",
    "<div>",
    "</div>",
    "<li>",
    "</li>",
    "{.c}",
    "{#i}",
    "{.c #i k=v}",
    "{",
    "}",
    "<",
    ">",
    "\"",
    "'",
    "text",
    " ",
    "\n",
    "</",
];

#[test]
fn remembered_searches_agree_with_reading_afresh() {
    let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
    for round in 0..6000 {
        let pieces = if round < 5000 { 1 + round % 24 } else { 100 + round % 200 };
        let mut html = String::new();
        for _ in 0..pieces {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            html.push_str(PIECES[(state % PIECES.len() as u64) as usize]);
        }
        let remembered = super::transform_with(&html, &mut super::TagFinder::new(true));
        let afresh = super::transform_with(&html, &mut super::TagFinder::new(false));
        assert_eq!(remembered, afresh, "for {html:?}");
    }
}
