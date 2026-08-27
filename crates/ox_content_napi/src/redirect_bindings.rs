use napi_derive::napi;

/// One page that other paths can redirect to.
#[napi(object)]
pub struct JsRedirectPage {
    pub dest: String,
    pub aliases: Option<Vec<String>>,
    pub redirect: Option<String>,
}

/// One entry of the config rewrite map, in author order.
#[napi(object)]
pub struct JsRedirectMapEntry {
    pub from: String,
    pub to: String,
}

/// Switches and the config rewrite map.
#[napi(object)]
pub struct JsRedirectsOptions {
    pub enabled: bool,
    pub map: Vec<JsRedirectMapEntry>,
    pub netlify: bool,
    pub headers: bool,
    pub json: bool,
    pub allow_external: bool,
    pub html: bool,
    pub base: Option<String>,
}

/// One planned static HTML redirect.
#[napi(object)]
pub struct JsRedirectEntry {
    pub from: String,
    pub to: String,
    pub html: String,
    pub relative_path: String,
}

/// Planned redirect pages and optional host files.
#[napi(object)]
pub struct JsRedirectsOutput {
    pub pages: Vec<JsRedirectEntry>,
    pub netlify: Option<String>,
    pub headers: Option<String>,
    pub json: Option<String>,
}

/// Plans redirect pages and host files without writing anything.
#[napi(js_name = "planRedirects")]
pub fn plan_redirects(
    options: JsRedirectsOptions,
    pages: Vec<JsRedirectPage>,
) -> JsRedirectsOutput {
    let resolved = ox_content_ssg::RedirectsOptions {
        enabled: options.enabled,
        map: options.map.into_iter().map(|entry| (entry.from, entry.to)).collect(),
        netlify: options.netlify,
        headers: options.headers,
        json: options.json,
        allow_external: options.allow_external,
        html: options.html,
        base: options.base,
    };
    let pages: Vec<_> = pages
        .into_iter()
        .map(|page| ox_content_ssg::RedirectPage {
            dest: page.dest,
            aliases: page.aliases.unwrap_or_default(),
            redirect: page.redirect,
        })
        .collect();
    let output = ox_content_ssg::generate_redirects(&resolved, &pages);

    JsRedirectsOutput {
        pages: output
            .pages
            .into_iter()
            .map(|entry| JsRedirectEntry {
                from: entry.from,
                to: entry.to,
                html: entry.html,
                relative_path: entry.relative_path,
            })
            .collect(),
        netlify: output.netlify,
        headers: output.headers,
        json: output.json,
    }
}

/// Static HTML redirect body. `dest` is escaped; callers still validate it.
#[napi(js_name = "renderRedirectHtml")]
pub fn render_redirect_html(dest: String) -> String {
    ox_content_ssg::generate_redirect_html(&dest)
}

/// Same-origin path check: leading `/`, not `//`, and no scheme.
#[napi(js_name = "isSafeRedirectDest")]
pub fn is_safe_redirect_dest(value: String) -> bool {
    ox_content_ssg::is_safe_dest(&value)
}

/// Strips a trailing slash except for `/`. Unsafe values return `None`.
#[napi(js_name = "normalizeRedirectPath")]
pub fn normalize_redirect_path(value: String) -> Option<String> {
    ox_content_ssg::normalize_path(&value)
}
