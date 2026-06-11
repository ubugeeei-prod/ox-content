use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Resolved output and public paths for an SSG page.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RoutePaths {
    /// HTML output file path.
    pub output_path: String,
    /// Route path without extension.
    pub url_path: String,
    /// Public HTML href.
    pub href: String,
    /// OG image output file path.
    pub og_image_path: String,
    /// OG image public URL.
    pub og_image_url: String,
}

/// Resolves all path variants needed to generate a page.
pub fn resolve_route_paths(
    input_path: &str,
    src_dir: &str,
    out_dir: &str,
    base: &str,
    extension: &str,
    site_url: Option<&str>,
) -> RoutePaths {
    RoutePaths {
        output_path: get_output_path(input_path, src_dir, out_dir, extension),
        url_path: get_url_path(input_path, src_dir),
        href: get_href(input_path, src_dir, base, extension),
        og_image_path: get_og_image_path(input_path, src_dir, out_dir),
        og_image_url: get_og_image_url(input_path, src_dir, base, site_url),
    }
}

/// Converts a markdown file path to its corresponding HTML output path.
pub fn get_output_path(input_path: &str, src_dir: &str, out_dir: &str, extension: &str) -> String {
    let relative_path = relative_path(input_path, src_dir);
    let base_name = replace_markdown_extension(&relative_path, extension);

    if base_name.ends_with(&format!("index{extension}")) {
        return join_path(out_dir, &base_name);
    }

    let dir_name = trim_suffix(&base_name, extension);
    join_path(out_dir, &join_path(&dir_name, &format!("index{extension}")))
}

/// Converts a markdown file path to a relative URL path.
pub fn get_url_path(input_path: &str, src_dir: &str) -> String {
    let relative_path = normalize_separators(&relative_path(input_path, src_dir));
    let base_name = strip_markdown_extension(&relative_path);

    if base_name == "index" || base_name.ends_with("/index") {
        let trimmed = trim_trailing_index(&base_name);
        if trimmed.is_empty() {
            "/".to_string()
        } else {
            trimmed
        }
    } else {
        base_name
    }
}

/// Converts a markdown file path to an HTML href.
pub fn get_href(input_path: &str, src_dir: &str, base: &str, extension: &str) -> String {
    let url_path = get_url_path(input_path, src_dir);
    if url_path == "/" || url_path.is_empty() {
        format!("{base}index{extension}")
    } else {
        format!("{base}{url_path}/index{extension}")
    }
}

/// Gets the OG image output path for a markdown file.
pub fn get_og_image_path(input_path: &str, src_dir: &str, out_dir: &str) -> String {
    let relative_path = normalize_separators(&relative_path(input_path, src_dir));
    let base_name = strip_markdown_extension(&relative_path);

    if base_name == "index" || base_name.ends_with("/index") {
        let dir_path = trim_trailing_index(&base_name);
        return join_path(out_dir, &join_path(&dir_path, "og-image.png"));
    }

    join_path(out_dir, &join_path(&base_name, "og-image.png"))
}

/// Gets the OG image URL for use in meta tags.
pub fn get_og_image_url(
    input_path: &str,
    src_dir: &str,
    base: &str,
    site_url: Option<&str>,
) -> String {
    let url_path = get_url_path(input_path, src_dir);
    let relative_path = if url_path == "/" || url_path.is_empty() {
        format!("{base}og-image.png")
    } else {
        format!("{base}{url_path}/og-image.png")
    };

    match site_url {
        Some(site_url) => format!("{}{}", site_url.trim_end_matches('/'), relative_path),
        None => relative_path,
    }
}

/// Returns the page locale for a localized route.
pub fn get_page_locale(
    url_path: &str,
    default_locale: &str,
    locale_codes: &[String],
) -> Option<String> {
    let first_segment = url_path.split('/').find(|segment| !segment.is_empty());
    match first_segment {
        Some(segment) if locale_codes.iter().any(|locale| locale == segment) => {
            Some(segment.to_string())
        }
        _ => Some(default_locale.to_string()),
    }
}

pub(super) fn relative_path(input_path: &str, src_dir: &str) -> String {
    let input = Path::new(input_path);
    let src = Path::new(src_dir);
    input.strip_prefix(src).unwrap_or(input).to_string_lossy().into_owned()
}

pub(super) fn normalize_separators(value: &str) -> String {
    value.replace('\\', "/")
}

fn join_path(left: &str, right: &str) -> String {
    if left.is_empty() {
        return right.to_string();
    }
    if right.is_empty() {
        return left.to_string();
    }
    PathBuf::from(left).join(right).to_string_lossy().into_owned()
}

pub(super) fn strip_markdown_extension(path: &str) -> String {
    if path.len() >= 3 && path[path.len() - 3..].eq_ignore_ascii_case(".md") {
        return path[..path.len() - 3].to_string();
    }
    if path.len() >= 4 && path[path.len() - 4..].eq_ignore_ascii_case(".mdx") {
        return path[..path.len() - 4].to_string();
    }
    if path.len() >= 9 && path[path.len() - 9..].eq_ignore_ascii_case(".markdown") {
        return path[..path.len() - 9].to_string();
    }
    path.to_string()
}

fn replace_markdown_extension(path: &str, extension: &str) -> String {
    format!("{}{}", strip_markdown_extension(path), extension)
}

fn trim_suffix(value: &str, suffix: &str) -> String {
    value.strip_suffix(suffix).unwrap_or(value).to_string()
}

fn trim_trailing_index(value: &str) -> String {
    value
        .strip_suffix("/index")
        .or_else(|| value.strip_suffix("index"))
        .unwrap_or(value)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_index_and_nested_routes() {
        let root = "/repo/docs";

        assert_eq!(get_url_path("/repo/docs/index.md", root), "/");
        assert_eq!(get_href("/repo/docs/index.md", root, "/base/", ".html"), "/base/index.html");
        assert_eq!(
            get_output_path("/repo/docs/guide/intro.md", root, "/repo/dist", ".html"),
            join_path("/repo/dist", "guide/intro/index.html")
        );
        assert_eq!(get_url_path("/repo/docs/reference.mdx", root), "reference");
        assert_eq!(
            get_og_image_url(
                "/repo/docs/guide/index.markdown",
                root,
                "/base/",
                Some("https://example.com/")
            ),
            "https://example.com/base/guide/og-image.png"
        );
    }
}
