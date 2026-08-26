//! Independent SSG route/output mount prefix (`routePrefix`).

use std::path::{Path, PathBuf};

use super::path::RoutePaths;

/// `blog`, `/blog`, and `/blog/` all become `blog`. Empty or unsafe values stay off.
pub fn normalize_route_prefix(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || trimmed.contains('\\')
        || trimmed.contains('\0')
        || trimmed.contains("://")
        || trimmed.starts_with("//")
        || is_windows_abs(trimmed)
    {
        return String::new();
    }
    let segments: Vec<&str> =
        trimmed.trim_matches('/').split('/').filter(|segment| !segment.is_empty()).collect();
    if segments.is_empty() || segments.iter().any(|segment| *segment == "." || *segment == "..") {
        return String::new();
    }
    segments.join("/")
}

/// Mounts file-tree route paths under `prefix` without changing `base` or `out_dir`.
pub fn apply_route_prefix(
    paths: RoutePaths,
    prefix: &str,
    out_dir: &str,
    base: &str,
) -> RoutePaths {
    let prefix = normalize_route_prefix(prefix);
    if prefix.is_empty() {
        return paths;
    }
    RoutePaths {
        output_path: prefix_fs_path(&paths.output_path, out_dir, &prefix),
        url_path: prefix_url_path(&paths.url_path, &prefix),
        href: prefix_public_path(&paths.href, base, &prefix),
        og_image_path: prefix_fs_path(&paths.og_image_path, out_dir, &prefix),
        og_image_url: prefix_og_url(&paths.og_image_url, base, &prefix),
    }
}

fn prefix_url_path(url_path: &str, prefix: &str) -> String {
    if url_path.is_empty() || url_path == "/" {
        prefix.to_string()
    } else {
        format!("{prefix}/{}", url_path.trim_start_matches('/'))
    }
}

fn prefix_fs_path(output_path: &str, out_dir: &str, prefix: &str) -> String {
    let output = Path::new(output_path);
    let root = Path::new(out_dir);
    let Ok(rel) = output.strip_prefix(root) else {
        return output_path.to_string();
    };
    PathBuf::from(out_dir).join(prefix).join(rel).to_string_lossy().into_owned()
}

fn prefix_public_path(href: &str, base: &str, prefix: &str) -> String {
    let root = normalize_base(base);
    if let Some(rest) = href.strip_prefix(&root) {
        format!("{root}{prefix}/{rest}")
    } else if href.starts_with('/') {
        format!("/{prefix}{href}")
    } else {
        format!("{root}{prefix}/{href}")
    }
}

fn prefix_og_url(url: &str, base: &str, prefix: &str) -> String {
    if let Some((origin, path)) = split_origin(url) {
        format!("{origin}{}", prefix_public_path(path, base, prefix))
    } else {
        prefix_public_path(url, base, prefix)
    }
}

fn normalize_base(base: &str) -> String {
    if base.is_empty() || base == "/" {
        "/".to_string()
    } else if base.ends_with('/') {
        base.to_string()
    } else {
        format!("{base}/")
    }
}

fn split_origin(url: &str) -> Option<(&str, &str)> {
    let scheme = url.find("://")?;
    let rest = &url[scheme + 3..];
    let slash = rest.find('/')?;
    let origin_end = scheme + 3 + slash;
    Some((&url[..origin_end], &url[origin_end..]))
}

fn is_windows_abs(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_slash_variants_and_rejects_escapes() {
        assert_eq!(normalize_route_prefix("blog"), "blog");
        assert_eq!(normalize_route_prefix("/blog"), "blog");
        assert_eq!(normalize_route_prefix("/blog/"), "blog");
        assert_eq!(normalize_route_prefix(""), "");
        assert_eq!(normalize_route_prefix("../secret"), "");
        assert_eq!(normalize_route_prefix("//evil.example"), "");
    }

    #[test]
    fn prefixes_page_paths_and_leaves_empty_prefix_unchanged() {
        let raw = RoutePaths {
            output_path: "/repo/dist/first-post/index.html".into(),
            url_path: "first-post".into(),
            href: "/first-post/index.html".into(),
            og_image_path: "/repo/dist/first-post/og-image.png".into(),
            og_image_url: "https://example.com/first-post/og-image.png".into(),
        };
        assert_eq!(apply_route_prefix(raw.clone(), "", "/repo/dist", "/"), raw);

        let prefixed = apply_route_prefix(raw, "/blog/", "/repo/dist", "/");
        assert_eq!(
            prefixed.output_path,
            PathBuf::from("/repo/dist/blog/first-post/index.html").to_string_lossy()
        );
        assert_eq!(prefixed.url_path, "blog/first-post");
        assert_eq!(prefixed.href, "/blog/first-post/index.html");
        assert_eq!(
            prefixed.og_image_path,
            PathBuf::from("/repo/dist/blog/first-post/og-image.png").to_string_lossy()
        );
        assert_eq!(prefixed.og_image_url, "https://example.com/blog/first-post/og-image.png");
    }
}
