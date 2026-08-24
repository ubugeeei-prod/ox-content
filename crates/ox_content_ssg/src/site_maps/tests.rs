use super::{SiteMapPage, SiteMapsOptions, generate_site_maps};

fn page(loc: &str, title: &str, description: Option<&str>) -> SiteMapPage {
    SiteMapPage {
        loc: loc.to_string(),
        title: title.to_string(),
        description: description.map(str::to_string),
        draft: false,
        unlisted: false,
    }
}

fn draft_page(loc: &str, title: &str) -> SiteMapPage {
    SiteMapPage { draft: true, ..page(loc, title, None) }
}

fn enabled(site_url: Option<&str>) -> SiteMapsOptions {
    SiteMapsOptions {
        enabled: true,
        site_url: site_url.map(str::to_string),
        sitemap_loc: "https://example.com/sitemap.xml".to_string(),
        site_name: "Docs".to_string(),
        site_description: Some("Example docs".to_string()),
        robots: true,
        llms: true,
    }
}

fn sample_pages() -> Vec<SiteMapPage> {
    vec![
        page("https://example.com/guide/", "Getting Started", Some("How to install")),
        page("https://example.com/api/", "API", None),
    ]
}

#[test]
fn disabled_by_default_writes_nothing() {
    let output = generate_site_maps(&SiteMapsOptions::default(), &sample_pages());

    assert_eq!(output, super::SiteMapsOutput::default());
}

#[test]
fn enabled_without_site_url_warns_and_writes_nothing() {
    let output = generate_site_maps(&enabled(None), &sample_pages());

    assert_eq!(output.sitemap_xml, None);
    assert_eq!(output.robots_txt, None);
    assert_eq!(output.llms_txt, None);
    assert_eq!(
        output.warning.as_deref(),
        Some(
            "[ox-content] siteMaps is enabled but ssg.siteUrl is not set; sitemap.xml, robots.txt, and llms.txt were not written"
        )
    );
}

#[test]
fn blank_site_url_is_treated_as_missing() {
    let output = generate_site_maps(&enabled(Some("   ")), &sample_pages());

    assert_eq!(output.sitemap_xml, None);
    assert!(output.warning.is_some(), "blank siteUrl must warn: {output:?}");
}

#[test]
fn happy_path_writes_sorted_sitemap_robots_and_llms() {
    let output = generate_site_maps(&enabled(Some("https://example.com")), &sample_pages());

    assert_eq!(
        output.sitemap_xml.as_deref(),
        Some(
            "\
<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">
  <url>
    <loc>https://example.com/api/</loc>
  </url>
  <url>
    <loc>https://example.com/guide/</loc>
  </url>
</urlset>
"
        )
    );
    assert_eq!(
        output.robots_txt.as_deref(),
        Some("User-agent: *\nAllow: /\n\nSitemap: https://example.com/sitemap.xml\n")
    );
    assert_eq!(
        output.llms_txt.as_deref(),
        Some(
            "\
# Docs

> Example docs

## Pages

- [API](https://example.com/api/)
- [Getting Started](https://example.com/guide/): How to install
"
        )
    );
    assert_eq!(output.warning, None);
}

#[test]
fn object_overrides_can_disable_robots_and_llms() {
    let options =
        SiteMapsOptions { robots: false, llms: false, ..enabled(Some("https://example.com")) };
    let output = generate_site_maps(&options, &sample_pages());

    assert!(output.sitemap_xml.is_some(), "sitemap stays on: {output:?}");
    assert_eq!(output.robots_txt, None);
    assert_eq!(output.llms_txt, None);
    assert_eq!(output.warning, None);
}

#[test]
fn unlisted_pages_are_omitted() {
    let pages = vec![
        SiteMapPage { unlisted: true, ..page("https://example.com/secret/", "Secret", None) },
        page("https://example.com/public/", "Public", None),
    ];
    let output = generate_site_maps(&enabled(Some("https://example.com")), &pages);
    let sitemap = output.sitemap_xml.expect("published pages should still emit a sitemap");

    assert!(sitemap.contains("https://example.com/public/"), "{sitemap}");
    assert!(!sitemap.contains("secret"), "{sitemap}");
}

#[test]
fn draft_pages_are_omitted() {
    let pages = vec![
        draft_page("https://example.com/secret/", "Secret"),
        page("https://example.com/public/", "Public", None),
    ];
    let output = generate_site_maps(&enabled(Some("https://example.com")), &pages);
    let sitemap = output.sitemap_xml.expect("published pages should still emit a sitemap");
    let llms = output.llms_txt.expect("published pages should still emit llms.txt");

    assert!(sitemap.contains("https://example.com/public/"), "{sitemap}");
    assert!(!sitemap.contains("secret"), "{sitemap}");
    assert!(llms.contains("Public"), "{llms}");
    assert!(!llms.contains("Secret"), "{llms}");
}

#[test]
fn hostile_title_and_description_cannot_break_out() {
    let pages = vec![page(
        "https://example.com/x?a=1&b=2<>\"'",
        "</loc></urlset><script>alert(1)</script>\n- [Injected](https://evil.example/)",
        Some("\">\n<img src=x onerror=alert(1)>"),
    )];
    let output = generate_site_maps(&enabled(Some("https://example.com")), &pages);
    let sitemap = output.sitemap_xml.expect("hostile input must still emit a sitemap");
    let llms = output.llms_txt.expect("hostile input must still emit llms.txt");

    assert!(!sitemap.contains("<script>"), "{sitemap}");
    assert!(!sitemap.contains("</urlset><script>"), "{sitemap}");
    assert!(sitemap.contains("&amp;"), "{sitemap}");
    assert!(sitemap.contains("&lt;"), "{sitemap}");
    assert!(sitemap.contains("&gt;"), "{sitemap}");
    assert!(sitemap.contains("&quot;"), "{sitemap}");
    assert!(sitemap.contains("&#39;"), "{sitemap}");
    assert!(!llms.contains("<script>"), "{llms}");
    assert!(!llms.contains("\n- [Injected](https://evil.example/)"), "{llms}");
    assert!(!llms.contains("<img"), "{llms}");
    assert!(llms.contains("\\[Injected\\]"), "{llms}");
}

#[test]
fn sitemap_order_is_deterministic() {
    let reversed = vec![
        page("https://example.com/z/", "Z", None),
        page("https://example.com/a/", "A", None),
        page("https://example.com/m/", "M", None),
    ];
    let output = generate_site_maps(&enabled(Some("https://example.com")), &reversed);
    let sitemap = output.sitemap_xml.expect("sorted pages should emit a sitemap");
    let a = sitemap.find("https://example.com/a/").expect("a");
    let m = sitemap.find("https://example.com/m/").expect("m");
    let z = sitemap.find("https://example.com/z/").expect("z");
    assert!(a < m && m < z, "{sitemap}");
}
