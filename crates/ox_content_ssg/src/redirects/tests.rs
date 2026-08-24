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
        ..RedirectsOptions::default()
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
fn disabled_by_default_writes_nothing() {
    let output =
        generate_redirects(&RedirectsOptions::default(), &[page("/guide", &["/old"], None)]);

    assert!(output.pages.is_empty(), "{output:?}");
    assert_eq!(output.netlify, None);
    assert_eq!(output.headers, None);
    assert_eq!(output.json, None);
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
}

#[test]
fn frontmatter_redirect_is_an_old_path() {
    let output = generate_redirects(&enabled(&[]), &[page("/guide", &[], Some("/retired"))]);

    assert_eq!(output.pages.len(), 1, "{output:?}");
    assert_eq!(output.pages[0].from, "/retired");
    assert_eq!(output.pages[0].to, "/guide");
}

#[test]
fn config_map_emits_redirect() {
    let output = generate_redirects(&enabled(&[("/old-guide", "/guide")]), &[]);

    assert_eq!(output.pages.len(), 1, "{output:?}");
    assert_eq!(output.pages[0].from, "/old-guide");
    assert_eq!(output.pages[0].to, "/guide");
}

#[test]
fn control_characters_and_traversal_are_rejected() {
    assert!(!is_safe_dest("/guide\tv2"));
    assert!(!is_safe_dest("/../outside"));
    assert!(!is_safe_dest("/..\\outside"));
    assert!(!is_safe_dest("/./inside"));

    let output = generate_redirects(
        &enabled(&[("/../outside", "/guide"), ("/..\\outside", "/guide"), ("/old", "/guide\tv2")]),
        &[],
    );

    assert!(output.pages.is_empty(), "{output:?}");
    assert_eq!(output.json, None);
}

#[test]
fn javascript_and_data_destinations_are_rejected() {
    let output = generate_redirects(
        &enabled(&[("/old", "javascript:alert(1)"), ("/data", "data:text/html,hi")]),
        &[page("javascript:alert(1)", &["/legacy"], None)],
    );

    assert!(output.pages.is_empty(), "{output:?}");
    assert!(!is_safe_dest("javascript:alert(1)"));
    assert!(!is_safe_dest("JAVASCRIPT:alert(1)"));
    assert!(!is_safe_dest("data:text/html,hi"));
}

#[test]
fn protocol_relative_and_offsite_destinations_are_rejected() {
    let output = generate_redirects(
        &enabled(&[("/old", "//evil.example"), ("/abs", "https://evil.example")]),
        &[page("//evil.example", &["/legacy"], None)],
    );

    assert!(output.pages.is_empty(), "{output:?}");
    assert!(!is_safe_dest("//evil.example"));
    assert!(!is_safe_dest("https://evil.example"));
}

#[test]
fn allow_external_accepts_http_urls_but_still_rejects_open_redirects() {
    let mut options = enabled(&[
        ("/ok", "https://example.com/new"),
        ("/js", "javascript:alert(1)"),
        ("/proto", "//evil.example"),
    ]);
    options.allow_external = true;
    let output = generate_redirects(&options, &[]);

    assert_eq!(output.pages.len(), 1, "{output:?}");
    assert_eq!(output.pages[0].to, "https://example.com/new");
}

#[test]
fn trailing_slash_variants_fold_and_last_wins() {
    let output = generate_redirects(
        &enabled(&[("/old/", "/guide/"), ("/old", "/other")]),
        &[page("/page/", &["/alias/"], None)],
    );

    let from_old = output.pages.iter().find(|entry| entry.from == "/old").expect("normalized /old");
    assert_eq!(from_old.to, "/other", "{output:?}");
    assert!(output.pages.iter().all(|entry| entry.from != "/old/"), "{output:?}");
    let from_alias = output.pages.iter().find(|entry| entry.from == "/alias").expect("/alias/");
    assert_eq!(from_alias.to, "/page");
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
fn occupied_page_source_is_not_overwritten() {
    let output = generate_redirects(
        &enabled(&[("/guide", "/elsewhere")]),
        &[page("/guide", &["/guide"], None), page("/other", &[], None)],
    );

    assert!(output.pages.iter().all(|entry| entry.from != "/guide"), "{output:?}");
}

#[test]
fn self_redirect_is_skipped() {
    let output = generate_redirects(&enabled(&[("/same", "/same/")]), &[]);

    assert!(output.pages.is_empty(), "{output:?}");
}

#[test]
fn hostile_dest_is_escaped_in_html() {
    let html = html_for("/foo<bar>&\"'");

    assert!(!html.contains("url=/foo<bar>"), "{html}");
    assert!(!html.contains(r#"href="/foo<bar>""#), "{html}");
    assert!(html.contains("url=/foo&lt;bar&gt;&amp;&quot;&#39;"), "{html}");
    assert!(!html.contains("<script>"), "{html}");
    assert_eq!(generate_redirect_html("/foo<bar>&\"'"), html);
}

#[test]
fn refresh_parameter_injection_is_rejected() {
    let output = generate_redirects(&enabled(&[("/old", "/foo;url=https://evil.example")]), &[]);

    assert!(output.pages.is_empty(), "{output:?}");
}

#[test]
fn host_files_and_json_are_opt_in() {
    let off = generate_redirects(&enabled(&[("/old", "/guide")]), &[]);
    assert_eq!(off.netlify, None);
    assert_eq!(off.headers, None);
    assert_eq!(off.json, None);

    let mut on = enabled(&[("/old", "/guide")]);
    on.netlify = true;
    on.headers = true;
    on.json = true;
    let output = generate_redirects(&on, &[]);
    assert_eq!(output.netlify.as_deref(), Some("/old /guide 301\n"));
    assert_eq!(output.headers.as_deref(), Some("/old\n  Location: /guide\n"));
    assert_eq!(output.json.as_deref(), Some(r#"[{"from":"/old","to":"/guide"}]"#));
}
