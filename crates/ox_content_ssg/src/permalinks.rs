//! Opt-in permalink / slug routing and directory frontmatter cascade.

use rustc_hash::FxHashMap;
use serde_json::{Map, Value};

const RESERVED_CASCADE_KEYS: &[&str] = &["permalink", "slug"];

/// Switches for frontmatter `permalink` / `slug` routing.
#[derive(Debug, Clone, Default)]
pub struct PermalinksOptions {
    /// When false, every page keeps its file-tree URL.
    pub enabled: bool,
}

/// Switches for `_index` directory frontmatter inheritance.
#[derive(Debug, Clone, Default)]
pub struct CascadeOptions {
    /// When false, child frontmatter is left unchanged.
    pub enabled: bool,
}

/// One page considered for cascade and permalink resolution.
#[derive(Debug, Clone)]
pub struct RoutePage {
    /// Source path relative to the content root (`guide/intro.md`).
    pub source: String,
    /// File-tree URL (`guide/intro` or `/`).
    pub file_url: String,
    /// Parsed frontmatter object.
    pub frontmatter: Map<String, Value>,
}

/// A page after cascade and optional permalink / slug rewriting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedRoutePage {
    /// Source path relative to the content root.
    pub source: String,
    /// Resolved URL path (`guide/intro` or `/`).
    pub url_path: String,
    /// Frontmatter after cascade fills.
    pub frontmatter: Map<String, Value>,
}

/// Resolved pages plus collision / rejection errors.
///
/// Collisions skip the later page and keep the first. Rejected permalinks
/// stay on the file-tree URL. Neither case panics.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RouteResolveOutput {
    /// Pages that still have a unique URL.
    pub pages: Vec<ResolvedRoutePage>,
    /// Human-readable collision and rejection messages.
    pub errors: Vec<String>,
}

/// Applies cascade (when on) then permalink / slug rewriting (when on).
pub fn resolve_page_routes(
    pages: &[RoutePage],
    permalinks: &PermalinksOptions,
    cascade: &CascadeOptions,
) -> RouteResolveOutput {
    let cascaded = apply_cascade(pages, cascade);
    if !permalinks.enabled {
        return RouteResolveOutput {
            pages: cascaded
                .into_iter()
                .map(|page| ResolvedRoutePage {
                    url_path: normalize_url_path(&page.file_url),
                    source: page.source,
                    frontmatter: page.frontmatter,
                })
                .collect(),
            errors: Vec::new(),
        };
    }

    let mut output = RouteResolveOutput::default();
    let mut claimed: FxHashMap<String, String> = FxHashMap::default();
    for page in cascaded {
        let (url_path, error) = resolve_one(&page);
        if let Some(error) = error {
            output.errors.push(error);
        }
        if let Some(owner) = claimed.get(&url_path) {
            output.errors.push(format!(
                "[ox-content] URL collision at \"{url_path}\": {owner} kept, {} skipped",
                page.source
            ));
            continue;
        }
        claimed.insert(url_path.clone(), page.source.clone());
        output.pages.push(ResolvedRoutePage {
            source: page.source,
            url_path,
            frontmatter: page.frontmatter,
        });
    }
    output
}

/// Fills missing frontmatter keys from ancestor `_index` files. Child wins.
pub fn apply_cascade(pages: &[RoutePage], options: &CascadeOptions) -> Vec<RoutePage> {
    if !options.enabled {
        return pages.to_vec();
    }

    let mut indexes: FxHashMap<String, Map<String, Value>> = FxHashMap::default();
    for page in pages {
        let source = normalize_separators(&page.source);
        if is_index_file(&source) {
            indexes.entry(directory_of(&source)).or_insert_with(|| page.frontmatter.clone());
        }
    }

    pages
        .iter()
        .map(|page| {
            let source = normalize_separators(&page.source);
            let mut frontmatter = page.frontmatter.clone();
            for dir in ancestor_dirs(&source) {
                let Some(defaults) = indexes.get(&dir) else {
                    continue;
                };
                if is_index_file(&source) && directory_of(&source) == dir {
                    continue;
                }
                merge_missing(&mut frontmatter, defaults);
            }
            RoutePage { source: page.source.clone(), file_url: page.file_url.clone(), frontmatter }
        })
        .collect()
}

/// Escapes a value for use in an HTML attribute.
pub fn escape_attribute(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
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
    output
}

/// Same-origin URL path without `..`, schemes, or filesystem roots.
pub fn is_safe_permalink(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.bytes().any(|byte| matches!(byte, b'\n' | b'\r' | b'\0')) {
        return false;
    }
    if trimmed.contains('\\') || is_windows_abs(trimmed) || trimmed.starts_with("//") {
        return false;
    }
    let lower = trimmed.to_ascii_lowercase();
    if lower.contains("javascript:")
        || lower.contains("data:")
        || lower.contains("vbscript:")
        || lower.contains("file:")
        || lower.contains("://")
    {
        return false;
    }
    !path_segments(trimmed).iter().any(|segment| *segment == ".." || *segment == ".")
}

fn resolve_one(page: &RoutePage) -> (String, Option<String>) {
    let file_url = normalize_url_path(&page.file_url);
    if let Some(permalink) = string_field(&page.frontmatter, "permalink") {
        return match rewrite_permalink(&permalink) {
            Some(url) => (url, None),
            None => (
                file_url,
                Some(format!(
                    "[ox-content] rejected permalink {permalink:?} on {} (path escape); using the file-tree URL",
                    page.source
                )),
            ),
        };
    }
    if let Some(slug) = string_field(&page.frontmatter, "slug") {
        return match rewrite_slug(&file_url, &slug) {
            Some(url) => (url, None),
            None => (
                file_url,
                Some(format!(
                    "[ox-content] rejected slug {slug:?} on {} (path escape); using the file-tree URL",
                    page.source
                )),
            ),
        };
    }
    (file_url, None)
}

fn rewrite_permalink(value: &str) -> Option<String> {
    is_safe_permalink(value).then(|| normalize_url_path(value))
}

fn rewrite_slug(file_url: &str, slug: &str) -> Option<String> {
    let trimmed = slug.trim();
    if trimmed.contains('/') || !is_safe_permalink(trimmed) {
        return None;
    }
    let slug = normalize_url_path(trimmed);
    if slug == "/" {
        return None;
    }
    if file_url == "/" {
        return Some(slug);
    }
    let mut segments: Vec<&str> = file_url.split('/').collect();
    segments.pop();
    segments.push(&slug);
    Some(segments.into_iter().filter(|segment| !segment.is_empty()).collect::<Vec<_>>().join("/"))
}

fn merge_missing(dest: &mut Map<String, Value>, src: &Map<String, Value>) {
    for (key, value) in src {
        if RESERVED_CASCADE_KEYS.contains(&key.as_str()) || dest.contains_key(key) {
            continue;
        }
        dest.insert(key.clone(), value.clone());
    }
}

fn string_field(frontmatter: &Map<String, Value>, key: &str) -> Option<String> {
    frontmatter.get(key).and_then(Value::as_str).map(str::to_string)
}

fn normalize_url_path(value: &str) -> String {
    let segments = path_segments(value);
    if segments.is_empty() { "/".to_string() } else { segments.join("/") }
}

fn path_segments(value: &str) -> Vec<&str> {
    value
        .trim()
        .trim_start_matches('/')
        .trim_end_matches('/')
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect()
}

fn normalize_separators(value: &str) -> String {
    value.replace('\\', "/")
}

fn is_index_file(source: &str) -> bool {
    let name = source.rsplit('/').next().unwrap_or(source);
    let stem = name.rsplit_once('.').map_or(name, |(stem, _)| stem);
    stem.eq_ignore_ascii_case("_index")
}

fn directory_of(source: &str) -> String {
    match source.rfind('/') {
        Some(index) => source[..index].to_string(),
        None => String::new(),
    }
}

fn ancestor_dirs(source: &str) -> Vec<String> {
    let dir = directory_of(source);
    let mut dirs = vec![String::new()];
    if dir.is_empty() {
        return dirs;
    }
    let mut acc = String::new();
    for segment in dir.split('/') {
        if !acc.is_empty() {
            acc.push('/');
        }
        acc.push_str(segment);
        dirs.push(acc.clone());
    }
    dirs
}

fn is_windows_abs(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}

#[cfg(test)]
mod tests;
