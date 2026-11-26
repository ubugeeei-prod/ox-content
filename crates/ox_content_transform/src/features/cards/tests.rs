use super::resolve;
use crate::features::{TransformFeatureOptions, preprocess_markdown};
use crate::transformer::MarkdownTransformer;
use crate::{CardOptions, ContainerOptions, TransformOptions};

fn cards_on() -> TransformOptions {
    TransformOptions {
        gfm: Some(true),
        cards: Some(CardOptions { enabled: Some(true) }),
        ..Default::default()
    }
}

fn transform_html(source: &str, options: TransformOptions) -> String {
    MarkdownTransformer::from_options(&options).transform(source).html
}

fn preprocess(source: &str, options: &TransformOptions) -> String {
    preprocess_markdown(source, &TransformFeatureOptions::from_options(options)).source.into_owned()
}

#[test]
fn disabled_by_default() {
    let source = "::: card\n### Install\nCopy the package.\n:::\n";
    let html = transform_html(source, TransformOptions::default());
    assert!(!html.contains("ox-card"), "default transform must not emit cards:\n{html}");
    assert!(
        html.contains("::: card") || html.contains("Install"),
        "source must remain visible:\n{html}"
    );

    assert!(resolve(None).is_none());
    assert!(resolve(Some(&CardOptions { enabled: Some(false) })).is_none());
    assert!(resolve(Some(&CardOptions { enabled: None })).is_some());
    assert!(resolve(Some(&CardOptions { enabled: Some(true) })).is_some());
}

#[test]
fn card_happy_path() {
    let html = transform_html(
        "::: card\n### Install\nCopy the package and run the CLI.\n:::\n",
        cards_on(),
    );
    assert!(html.contains(r#"<article class="ox-card">"#), "{html}");
    assert!(
        html.contains("<h3") && html.contains("Install"),
        "leading heading is the title:\n{html}"
    );
    assert!(html.contains("Copy the package and run the CLI."), "{html}");
    assert!(html.contains("</article>"), "{html}");
    assert!(!html.contains(":::"), "markers must be consumed:\n{html}");
}

#[test]
fn link_card_happy_path() {
    let html = transform_html(
        "::: link-card[Guide]{/getting-started}\nShort description\n:::\n",
        cards_on(),
    );
    assert!(html.contains(r#"<a class="ox-link-card" href="/getting-started">"#), "{html}");
    assert!(html.contains("Guide"), "{html}");
    assert!(html.contains("Short description"), "{html}");
    assert!(html.contains("</a>"), "{html}");
    assert!(!html.contains(":::"), "{html}");
}

#[test]
fn card_grid_wraps() {
    let html = transform_html(
        "::: card-grid\n::: card\n### Install\nCopy the package and run the CLI.\n:::\n::: link-card[Guide]{/getting-started}\nShort description\n:::\n:::\n",
        cards_on(),
    );
    assert!(html.contains(r#"<div class="ox-card-grid">"#), "{html}");
    let grid_at = html.find(r#"<div class="ox-card-grid">"#).expect("grid");
    let card_at = html.find(r#"<article class="ox-card">"#).expect("card");
    let link_at = html.find("ox-link-card").expect("link-card");
    let grid_close = html.rfind("</div>").expect("grid close");
    assert!(card_at > grid_at && link_at > grid_at, "{html}");
    assert!(html[card_at..].contains("</article>"), "{html}");
    assert!(grid_close > link_at, "{html}");
    assert!(!html.contains(":::"), "{html}");
}

#[test]
fn hostile_title_escaped() {
    let html = transform_html("::: card[<script>alert(1)</script>]\nBody\n:::\n", cards_on());
    assert!(!html.contains("<script>"), "{html}");
    assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"), "{html}");
    assert!(html.contains(r#"<article class="ox-card">"#), "{html}");
}

#[test]
fn javascript_href_rejected() {
    let html = transform_html("::: link-card[Go]{javascript:alert(1)}\nClick\n:::\n", cards_on());
    assert!(!html.contains("href=\"javascript:"), "{html}");
    assert!(!html.contains("href='javascript:"), "{html}");
    assert!(!html.to_ascii_lowercase().contains("href=\"javascript"), "{html}");
}

#[test]
fn data_href_rejected() {
    let html = transform_html(
        "::: link-card[Go]{data:text/html,<script>alert(1)</script>}\nClick\n:::\n",
        cards_on(),
    );
    assert!(!html.contains("href=\"data:"), "{html}");
    assert!(!html.contains("href='data:"), "{html}");
}

#[test]
fn unclosed_stays_literal() {
    let source = "::: card\n### Install\nCopy the package.\n# Later heading\n";
    let html = transform_html(source, cards_on());
    assert!(
        !html.contains(r#"<article class="ox-card">"#),
        "unclosed must not wrap the file:\n{html}"
    );
    assert!(html.contains("::: card") || html.contains("Install"), "{html}");
    assert!(html.contains("Later heading"), "rest of file must not be swallowed:\n{html}");
}

#[test]
fn skips_fenced_code() {
    let html = transform_html("```md\n::: card\n### Install\n:::\n```\n", cards_on());
    assert!(!html.contains("ox-card"), "{html}");
    assert!(html.contains("::: card"), "{html}");
}

#[test]
fn vbscript_and_protocol_relative_hrefs_are_rejected() {
    for href in ["vbscript:msgbox(1)", "//evil.example/x"] {
        let html =
            transform_html(&format!("::: link-card[Go]{{{href}}}\nClick\n:::\n"), cards_on());
        assert!(!html.contains(&format!("href=\"{href}\"")), "{href}: {html}");
        assert!(!html.contains(&format!("href='{href}'")), "{href}: {html}");
    }
}

#[test]
fn description_and_href_are_escaped() {
    let html = transform_html(
        "::: link-card[<em>T</em>]{/path\"onclick=alert(1)}\n<script>x</script>\n:::\n",
        cards_on(),
    );
    assert!(!html.contains("<script>"), "{html}");
    assert!(html.contains("&lt;script&gt;x&lt;/script&gt;"), "{html}");
    assert!(html.contains("&lt;em&gt;T&lt;/em&gt;"), "{html}");
    assert!(html.contains("href=\"/path&quot;onclick=alert(1)\""), "{html}");
}

#[test]
fn skips_inline_and_indented_code() {
    let inline = transform_html("Use `::: card` in docs.\n", cards_on());
    assert!(!inline.contains("ox-card"), "{inline}");
    assert!(inline.contains("::: card"), "{inline}");

    let indented = transform_html("    ::: card\n    Hello\n    :::\n", cards_on());
    assert!(!indented.contains("ox-card"), "{indented}");
}

#[test]
fn cards_run_before_generic_containers_and_reserve_type_names() {
    let source = "::: card\n### Install\n:::\n::: tip\nHello\n:::\n";
    let options = TransformOptions {
        gfm: Some(true),
        cards: Some(CardOptions { enabled: Some(true) }),
        containers: Some(ContainerOptions { enabled: Some(true), types: None }),
        ..Default::default()
    };
    let html = transform_html(source, options);
    assert!(html.contains(r#"<article class="ox-card">"#), "{html}");
    assert!(html.contains("ox-container--tip"), "{html}");
    assert!(!html.contains("ox-container--card"), "{html}");
}

#[test]
fn disabled_object_leaves_markers_literal() {
    let html = transform_html(
        "::: card\n### Install\n:::\n",
        TransformOptions {
            cards: Some(CardOptions { enabled: Some(false) }),
            ..Default::default()
        },
    );
    assert!(!html.contains("ox-card"), "{html}");
}

#[test]
fn preprocess_emits_article_so_inner_markdown_parses() {
    let source = preprocess("::: card\nHello **world**\n:::\n", &cards_on());
    assert!(
        source.contains("<article class=\"ox-card\">\n\nHello **world**\n</article>\n"),
        "{source}"
    );
}

/// The pass as it was: every opener searched for its own closer, and a grid's
/// body was rewritten as a copy.
fn reference_transform(source: &str) -> String {
    use super::parse::{CardKind, parse_any_opener, parse_closer, parse_opener};
    use super::{collect_inner, emit_block, split_ending};
    use crate::features::colon_blocks::tests::reference_closer;
    use crate::features::colon_blocks::trim_container_indent;
    use crate::features::segments::{is_closing_fence, parse_opening_fence};

    let lines: Vec<(&str, &str)> = source.split_inclusive('\n').map(split_ending).collect();
    let mut out = String::new();
    let mut index = 0usize;
    let mut fence: Option<(u8, usize)> = None;
    while index < lines.len() {
        let (line, ending) = lines[index];
        if let Some((fence_char, fence_len)) = fence {
            out.push_str(line);
            out.push_str(ending);
            if is_closing_fence(line, fence_char, fence_len) {
                fence = None;
            }
            index += 1;
            continue;
        }
        if let Some(open) = parse_opening_fence(line) {
            fence = Some((open.fence_char, open.fence_len));
            out.push_str(line);
            out.push_str(ending);
            index += 1;
            continue;
        }
        let trimmed = trim_container_indent(line);
        if let Some(opener) = parse_opener(trimmed)
            && let Some(end) =
                reference_closer(&lines, index, opener.colon_count, parse_any_opener, parse_closer)
        {
            let inner = collect_inner(&lines, index + 1, end);
            if opener.kind == CardKind::CardGrid {
                let nested = reference_transform(&inner);
                out.push_str("<div class=\"ox-card-grid\">\n\n");
                out.push_str(&nested);
                if !nested.ends_with('\n') && !nested.is_empty() {
                    out.push('\n');
                }
                out.push_str("</div>\n");
            } else {
                emit_block(&mut out, &opener, &inner);
            }
            index = end + 1;
            continue;
        }
        out.push_str(line);
        out.push_str(ending);
        index += 1;
    }
    out
}

#[test]
fn rewrites_as_the_per_opener_search_did() {
    let mut random = crate::features::colon_blocks::tests::Random(0x2545_F491_4F6C_DD1D);
    for _ in 0..4000 {
        let mut source = random.document();
        // Sometimes without a final newline, and with CRLF line endings.
        if random.below(4) == 0 {
            source.pop();
        }
        if random.below(4) == 0 {
            source = source.replace('\n', "\r\n");
        }
        assert_eq!(super::transform(&source), reference_transform(&source), "for {source:?}");
    }
}
