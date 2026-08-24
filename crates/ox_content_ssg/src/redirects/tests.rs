use super::{
    RedirectPage, RedirectsOptions, generate_redirect_html, generate_redirects, is_safe_dest,
};

fn page(dest: &str, aliases: &[&str], redirect: Option<&str>) -> RedirectPage {
    RedirectPage {
        dest: dest.to_string(),
        aliases: aliases.iter().map(|alias| (*alias).to_string()).collect(),
        redirect: redirect.map(str::to_string),
    }
}

fn enabled(map: &[(&str, &str)]) -> RedirectsOptions {
    RedirectsOptions {
        enabled: true,
        map: map.iter().map(|(from, to)| ((*from).to_string(), (*to).to_string())).collect(),
        netlify: false,
    }
}

fn html_for(dest: &str) -> String {
    generate_redirects(&enabled(&[]), &[page(dest, &["/old"], None)])
        .pages
        .into_iter()
        .find(|entry| entry.from == "/old")
        .map(|entry| entry.html)
        .unwrap_or_default()
}

#[test]
fn disabled_by_default() {
    let output =
        generate_redirects(&RedirectsOptions::default(), &[page("/guide", &["/old"], None)]);

    assert!(output.pages.is_empty(), "{output:?}");
    assert_eq!(output.netlify, None);
}

#[test]
fn frontmatter_alias_emits_redirect_html() {
    let output = generate_redirects(&enabled(&[]), &[page("/guide", &["/old", "/legacy"], None)]);

    assert_eq!(output.pages.len(), 2, "{output:?}");
    assert_eq!(output.pages[0].from, "/old");
    assert_eq!(output.pages[0].to, "/guide");
    assert!(output.pages[0].html.contains("url=/guide"), "{}", output.pages[0].html);
    assert!(
        output.pages[0].html.contains(r#"<link rel="canonical" href="/guide">"#),
        "{}",
        output.pages[0].html
    );
    assert_eq!(output.pages[1].from, "/legacy");
    assert_eq!(output.pages[1].to, "/guide");
}

#[test]
fn config_map_emits_redirect() {
    let output = generate_redirects(&enabled(&[("/old-guide", "/guide")]), &[]);

    assert_eq!(output.pages.len(), 1, "{output:?}");
    assert_eq!(output.pages[0].from, "/old-guide");
    assert_eq!(output.pages[0].to, "/guide");
    assert!(output.pages[0].html.contains(r#"href="/guide""#), "{}", output.pages[0].html);
}

#[test]
fn javascript_destination_rejected() {
    let output = generate_redirects(
        &enabled(&[("/old", "javascript:alert(1)")]),
        &[page("javascript:alert(1)", &["/legacy"], None)],
    );

    assert!(output.pages.is_empty(), "{output:?}");
    assert!(!is_safe_dest("javascript:alert(1)"));
    assert!(!is_safe_dest("JAVASCRIPT:alert(1)"));
}

#[test]
fn protocol_relative_destination_rejected() {
    let output = generate_redirects(
        &enabled(&[("/old", "//evil.example"), ("/also", "https://evil.example")]),
        &[page("//evil.example", &["/legacy"], None)],
    );

    assert!(output.pages.is_empty(), "{output:?}");
    assert!(!is_safe_dest("//evil.example"));
    assert!(!is_safe_dest("https://evil.example"));
    assert!(!is_safe_dest("data:text/html,hi"));
}

#[test]
fn trailing_slash_normalization() {
    let output = generate_redirects(
        &enabled(&[("/old/", "/guide/"), ("/old", "/other")]),
        &[page("/page/", &["/alias/"], None)],
    );

    let from_old = output.pages.iter().find(|entry| entry.from == "/old").expect("normalized /old");
    assert_eq!(from_old.to, "/other", "last-wins after slash folding: {output:?}");
    assert!(output.pages.iter().all(|entry| entry.from != "/old/"), "{output:?}");

    let from_alias =
        output.pages.iter().find(|entry| entry.from == "/alias").expect("normalized /alias/");
    assert_eq!(from_alias.to, "/page");
}

#[test]
fn hostile_dest_escaped_in_html() {
    let html = html_for("/foo<bar>");

    assert!(!html.contains("url=/foo<bar>"), "{html}");
    assert!(!html.contains(r#"href="/foo<bar>""#), "{html}");
    assert!(html.contains("url=/foo&lt;bar&gt;"), "{html}");
    assert!(html.contains(r#"href="/foo&lt;bar&gt;""#), "{html}");
    assert_eq!(generate_redirect_html("/foo<bar>"), html);
}

#[test]
fn overlapping_rules_last_wins() {
    let output = generate_redirects(
        &enabled(&[("/old", "/first"), ("/old", "/second")]),
        &[page("/frontmatter", &["/old"], None)],
    );

    assert_eq!(output.pages.len(), 1, "{output:?}");
    assert_eq!(output.pages[0].to, "/second");
}

#[test]
fn frontmatter_redirect_is_an_alias() {
    let output = generate_redirects(&enabled(&[]), &[page("/guide", &[], Some("/legacy"))]);

    assert_eq!(output.pages.len(), 1, "{output:?}");
    assert_eq!(output.pages[0].from, "/legacy");
    assert_eq!(output.pages[0].to, "/guide");
}

#[test]
fn netlify_body_is_opt_in() {
    let off = generate_redirects(&enabled(&[("/old", "/guide")]), &[]);
    assert_eq!(off.netlify, None);

    let mut on = enabled(&[("/old", "/guide")]);
    on.netlify = true;
    let output = generate_redirects(&on, &[]);
    assert_eq!(output.netlify.as_deref(), Some("/old /guide 301\n"));
}
