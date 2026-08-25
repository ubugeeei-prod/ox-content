//! Opt-in `sitemap.xml`, `robots.txt`, and `llms.txt` bodies.

const MISSING_SITE_URL: &str = "[ox-content] siteMaps is enabled but ssg.siteUrl is not set; sitemap.xml, robots.txt, and llms.txt were not written";

/// One published or draft page considered for crawl manifests.
#[derive(Debug, Clone)]
pub struct SiteMapPage {
    /// Absolute page URL.
    pub loc: String,
    /// Page title used by `llms.txt`.
    pub title: String,
    /// Optional page summary used by `llms.txt`.
    pub description: Option<String>,
    /// Source-file git commit time in milliseconds. `None` omits `<lastmod>`.
    pub last_updated: Option<i64>,
    /// When true, the page is omitted from every generated file.
    pub draft: bool,
    /// When true, the page is omitted from every generated file.
    pub unlisted: bool,
}

/// Switches and site metadata for crawl-manifest generation.
#[derive(Debug, Clone)]
pub struct SiteMapsOptions {
    /// When false, no files are generated.
    pub enabled: bool,
    /// Absolute site origin, required when the feature is on.
    pub site_url: Option<String>,
    /// Absolute URL of the generated `sitemap.xml`.
    pub sitemap_loc: String,
    /// Site title written to `llms.txt`.
    pub site_name: String,
    /// Optional site summary written to `llms.txt`.
    pub site_description: Option<String>,
    /// Write `robots.txt` when the feature is on.
    pub robots: bool,
    /// Write `llms.txt` when the feature is on.
    pub llms: bool,
}

/// Generated crawl-manifest bodies, or a warning when generation is skipped.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SiteMapsOutput {
    /// `sitemap.xml` body.
    pub sitemap_xml: Option<String>,
    /// `robots.txt` body.
    pub robots_txt: Option<String>,
    /// `llms.txt` body.
    pub llms_txt: Option<String>,
    /// Non-fatal skip reason. Never used as a panic.
    pub warning: Option<String>,
}

impl Default for SiteMapsOptions {
    fn default() -> Self {
        Self {
            enabled: false,
            site_url: None,
            sitemap_loc: String::new(),
            site_name: String::new(),
            site_description: None,
            robots: true,
            llms: true,
        }
    }
}

/// Builds sitemap / robots / llms bodies without writing files.
pub fn generate_site_maps(options: &SiteMapsOptions, pages: &[SiteMapPage]) -> SiteMapsOutput {
    if !options.enabled {
        return SiteMapsOutput::default();
    }
    if !has_site_url(options.site_url.as_deref()) {
        return SiteMapsOutput {
            warning: Some(MISSING_SITE_URL.to_string()),
            ..SiteMapsOutput::default()
        };
    }

    let mut published: Vec<&SiteMapPage> =
        pages.iter().filter(|page| !page.draft && !page.unlisted && !page.loc.is_empty()).collect();
    published.sort_unstable_by(|left, right| left.loc.cmp(&right.loc));

    SiteMapsOutput {
        sitemap_xml: Some(generate_sitemap_xml(&published)),
        robots_txt: options.robots.then(|| generate_robots_txt(&options.sitemap_loc)),
        llms_txt: options.llms.then(|| generate_llms_txt(options, &published)),
        warning: None,
    }
}

/// UTC `YYYY-MM-DD` for W3C lastmod. Negative timestamps are dropped.
fn format_lastmod(timestamp_ms: Option<i64>) -> Option<String> {
    let timestamp_ms = timestamp_ms.filter(|value| *value >= 0)?;
    let days = (timestamp_ms / 1_000).div_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    if !(0..=9999).contains(&year) {
        return None;
    }
    Some(format!("{year:04}-{month:02}-{day:02}"))
}

fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = year + i64::from(month <= 2);
    (year, month, day)
}

fn has_site_url(site_url: Option<&str>) -> bool {
    site_url.is_some_and(|value| !value.trim().is_empty())
}

fn generate_sitemap_xml(pages: &[&SiteMapPage]) -> String {
    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
    );
    for page in pages {
        xml.push_str("  <url>\n    <loc>");
        escape_xml(&page.loc, &mut xml);
        xml.push_str("</loc>\n");
        if let Some(lastmod) = format_lastmod(page.last_updated) {
            xml.push_str("    <lastmod>");
            xml.push_str(&lastmod);
            xml.push_str("</lastmod>\n");
        }
        xml.push_str("  </url>\n");
    }
    xml.push_str("</urlset>\n");
    xml
}

fn generate_robots_txt(sitemap_loc: &str) -> String {
    let mut text = String::from("User-agent: *\nAllow: /\n\nSitemap: ");
    for ch in sitemap_loc.chars() {
        if ch != '\n' && ch != '\r' {
            text.push(ch);
        }
    }
    text.push('\n');
    text
}

fn generate_llms_txt(options: &SiteMapsOptions, pages: &[&SiteMapPage]) -> String {
    let mut text = String::from("# ");
    escape_llms_text(&options.site_name, &mut text);
    text.push_str("\n\n");
    if let Some(description) =
        options.site_description.as_deref().filter(|value| !value.trim().is_empty())
    {
        text.push_str("> ");
        escape_llms_text(description, &mut text);
        text.push_str("\n\n");
    }
    text.push_str("## Pages\n\n");
    for page in pages {
        text.push_str("- [");
        escape_llms_text(&page.title, &mut text);
        text.push_str("](");
        escape_llms_url(&page.loc, &mut text);
        text.push(')');
        if let Some(description) =
            page.description.as_deref().filter(|value| !value.trim().is_empty())
        {
            text.push_str(": ");
            escape_llms_text(description, &mut text);
        }
        text.push('\n');
    }
    text
}

fn escape_xml(value: &str, output: &mut String) {
    for ch in value.chars() {
        match ch {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&#39;"),
            _ => output.push(ch),
        }
    }
}

fn flatten_text(value: &str) -> String {
    let mut flattened = String::with_capacity(value.len());
    let mut pending_space = false;
    for ch in value.chars() {
        if ch.is_whitespace() {
            pending_space = !flattened.is_empty();
            continue;
        }
        if pending_space {
            flattened.push(' ');
            pending_space = false;
        }
        flattened.push(ch);
    }
    flattened
}

fn escape_llms_text(value: &str, output: &mut String) {
    for ch in flatten_text(value).chars() {
        match ch {
            '\\' => output.push_str("\\\\"),
            '[' => output.push_str("\\["),
            ']' => output.push_str("\\]"),
            '(' => output.push_str("\\("),
            ')' => output.push_str("\\)"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '&' => output.push_str("&amp;"),
            '"' => output.push_str("&quot;"),
            _ => output.push(ch),
        }
    }
}

fn escape_llms_url(value: &str, output: &mut String) {
    for ch in value.chars() {
        match ch {
            ' ' => output.push_str("%20"),
            '(' => output.push_str("%28"),
            ')' => output.push_str("%29"),
            '\n' | '\r' | '\t' => {}
            _ => output.push(ch),
        }
    }
}

#[cfg(test)]
mod tests;
