use askama::Template;

use super::BarePageTemplate;
use super::head::render_bare_head;

/// Everything the bare template can put around a rendered page body.
///
/// Bare mode leaves the shell to the consumer, but the metadata below is
/// already computed for the themed page and is not something a consumer can
/// recover afterwards — the generated OG image in particular is only
/// discoverable by guessing at the output directory. Every field is optional,
/// and a `BarePageData` carrying none of them renders exactly the document
/// bare mode emitted before: no `<meta>` beyond charset and viewport, which
/// keeps the no-JS size baseline honest.
#[derive(Default)]
pub struct BarePageData<'a> {
    /// Page title, used for `<title>` and the OG/Twitter title.
    pub title: &'a str,
    /// Rendered page body.
    pub content: &'a str,
    /// `lang` attribute. Defaults to `en` when empty.
    pub lang: &'a str,
    /// `dir` attribute. Omitted entirely when empty.
    pub dir: &'a str,
    /// Page description, used for `description` and the OG/Twitter variants.
    pub description: Option<&'a str>,
    /// Absolute page URL, used for `<link rel="canonical">` and `og:url`.
    pub canonical_url: Option<&'a str>,
    /// Site name for `og:site_name`.
    pub site_name: Option<&'a str>,
    /// Image URL for `og:image` and `twitter:image`.
    pub og_image: Option<&'a str>,
    /// Raw markup appended to `<head>`.
    pub head: &'a str,
    /// Raw markup inserted directly after `<body>`.
    pub body_start: &'a str,
    /// Raw markup inserted directly before `</body>`.
    pub body_end: &'a str,
}

/// Generates a bare HTML page for SSG.
///
/// This page intentionally omits navigation, styles, and scripts.
pub fn generate_bare_html(content: &str, title: &str) -> String {
    generate_bare_page(&BarePageData { title, content, ..BarePageData::default() })
}

/// Generates a bare HTML page with whatever head metadata and injected markup
/// the caller has.
pub fn generate_bare_page(data: &BarePageData<'_>) -> String {
    let page_head = render_bare_head(data);
    BarePageTemplate {
        lang: if data.lang.is_empty() { "en" } else { data.lang },
        dir: data.dir,
        page_head: &page_head.html,
        content: data.content,
        head: data.head,
        body_start: data.body_start,
        body_end: data.body_end,
    }
    .render()
    .unwrap_or_default()
}
