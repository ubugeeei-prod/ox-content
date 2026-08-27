use napi_derive::napi;

/// One page considered for the crawl manifests.
#[napi(object)]
pub struct JsSiteMapPage {
    pub loc: String,
    pub title: String,
    pub description: Option<String>,
    /// Source-file git commit time in milliseconds. `None` omits `<lastmod>`.
    pub last_updated: Option<i64>,
    pub draft: Option<bool>,
    pub unlisted: Option<bool>,
}

/// Switches and site metadata for crawl-manifest generation.
#[napi(object)]
pub struct JsSiteMapsOptions {
    pub enabled: bool,
    pub site_url: Option<String>,
    pub sitemap_loc: String,
    pub site_name: String,
    pub site_description: Option<String>,
    pub robots: bool,
    pub llms: bool,
}

/// Generated crawl-manifest bodies, or a warning when generation is skipped.
#[napi(object)]
pub struct JsSiteMapsOutput {
    pub sitemap_xml: Option<String>,
    pub robots_txt: Option<String>,
    pub llms_txt: Option<String>,
    pub warning: Option<String>,
}

/// Builds `sitemap.xml`, `robots.txt`, and `llms.txt` bodies without writing files.
#[napi(js_name = "generateSiteMapBodies")]
pub fn generate_site_map_bodies(
    options: JsSiteMapsOptions,
    pages: Vec<JsSiteMapPage>,
) -> JsSiteMapsOutput {
    let resolved = ox_content_ssg::SiteMapsOptions {
        enabled: options.enabled,
        site_url: options.site_url,
        sitemap_loc: options.sitemap_loc,
        site_name: options.site_name,
        site_description: options.site_description,
        robots: options.robots,
        llms: options.llms,
    };
    let pages: Vec<_> = pages.into_iter().map(convert_page).collect();
    let output = ox_content_ssg::generate_site_maps(&resolved, &pages);

    JsSiteMapsOutput {
        sitemap_xml: output.sitemap_xml,
        robots_txt: output.robots_txt,
        llms_txt: output.llms_txt,
        warning: output.warning,
    }
}

fn convert_page(page: JsSiteMapPage) -> ox_content_ssg::SiteMapPage {
    ox_content_ssg::SiteMapPage {
        loc: page.loc,
        title: page.title,
        description: page.description,
        last_updated: page.last_updated,
        draft: page.draft.unwrap_or(false),
        unlisted: page.unlisted.unwrap_or(false),
    }
}
