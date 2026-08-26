use napi_derive::napi;
use serde_json::{Map, Value};

/// One page considered for cascade and permalink resolution.
#[napi(object)]
pub struct JsSsgRoutePageInput {
    /// Source path relative to the content root.
    pub source: String,
    /// File-tree URL before permalink or slug rewriting.
    pub file_url: String,
    /// Parsed frontmatter object.
    #[napi(ts_type = "Record<string, unknown>")]
    pub frontmatter: Value,
}

/// One page after cascade and optional permalink / slug rewriting.
#[napi(object)]
pub struct JsResolvedSsgRoutePage {
    /// Source path relative to the content root.
    pub source: String,
    /// Resolved URL path.
    pub url_path: String,
    /// Frontmatter after cascade fills.
    #[napi(ts_type = "Record<string, unknown>")]
    pub frontmatter: Value,
}

/// Resolved page routes plus collision and rejection diagnostics.
#[napi(object)]
pub struct JsSsgPageRouteOutput {
    /// Pages that still have a unique URL.
    pub pages: Vec<JsResolvedSsgRoutePage>,
    /// Human-readable collision and rejection messages.
    pub errors: Vec<String>,
}

/// Resolves permalink / slug routes and `_index` frontmatter cascade.
#[napi(js_name = "resolveSsgPageRoutes")]
pub fn resolve_ssg_page_routes(
    pages: Vec<JsSsgRoutePageInput>,
    permalinks_enabled: bool,
    cascade_enabled: bool,
) -> JsSsgPageRouteOutput {
    let pages = pages.into_iter().map(convert_page).collect::<Vec<_>>();
    let output = ox_content_ssg::resolve_page_routes(
        &pages,
        &ox_content_ssg::PermalinksOptions { enabled: permalinks_enabled },
        &ox_content_ssg::CascadeOptions { enabled: cascade_enabled },
    );

    JsSsgPageRouteOutput {
        pages: output
            .pages
            .into_iter()
            .map(|page| JsResolvedSsgRoutePage {
                source: page.source,
                url_path: page.url_path,
                frontmatter: Value::Object(page.frontmatter),
            })
            .collect(),
        errors: output.errors,
    }
}

fn convert_page(page: JsSsgRoutePageInput) -> ox_content_ssg::RoutePage {
    ox_content_ssg::RoutePage {
        source: page.source,
        file_url: page.file_url,
        frontmatter: object_frontmatter(page.frontmatter),
    }
}

fn object_frontmatter(value: Value) -> Map<String, Value> {
    match value {
        Value::Object(map) => map,
        _ => Map::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_routes_through_native_ssg() {
        let output = resolve_ssg_page_routes(
            vec![
                page("guide/_index.md", "guide/_index", serde_json::json!({ "section": "Guide" })),
                page("guide/intro.md", "guide/intro", serde_json::json!({ "permalink": "/start" })),
                page("guide/child.md", "guide/child", serde_json::json!({ "slug": "hello" })),
            ],
            true,
            true,
        );

        assert_eq!(output.pages.len(), 3);
        assert_eq!(output.pages[1].url_path, "start");
        assert_eq!(output.pages[2].url_path, "guide/hello");
        assert_eq!(output.pages[2].frontmatter["section"], "Guide");
        assert!(output.errors.is_empty());
    }

    fn page(source: &str, file_url: &str, frontmatter: Value) -> JsSsgRoutePageInput {
        JsSsgRoutePageInput {
            source: source.to_string(),
            file_url: file_url.to_string(),
            frontmatter,
        }
    }
}
