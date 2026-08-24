//! Opt-in static redirect HTML and optional host rewrite bodies.

use rustc_hash::{FxHashMap, FxHashSet};

/// One published page that may declare aliases or a single `redirect` source.
#[derive(Debug, Clone)]
pub struct RedirectPage {
    /// Current page path. Aliases and `redirect` point here.
    pub dest: String,
    /// Frontmatter `aliases` — each value is an old path.
    pub aliases: Vec<String>,
    /// Frontmatter `redirect` — a single extra old path.
    pub redirect: Option<String>,
}

/// Switches and the config rewrite map.
#[derive(Debug, Clone, Default)]
pub struct RedirectsOptions {
    /// When false, no files or host rules are generated.
    pub enabled: bool,
    /// Config map from old path to new path, in author order.
    pub map: Vec<(String, String)>,
    /// When true, also build a Netlify-style `_redirects` body.
    pub netlify: bool,
    /// When true, also build a `_headers` Location map.
    pub headers: bool,
    /// When true, also build a JSON rewrite map.
    pub json: bool,
    /// When true, `http://` and `https://` destinations are allowed.
    pub allow_external: bool,
}

/// One static HTML redirect to write at `from`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedirectEntry {
    /// Normalized source path (`/old`).
    pub from: String,
    /// Normalized destination path or allowed absolute URL.
    pub to: String,
    /// Escaped static HTML body.
    pub html: String,
}

/// Generated redirect pages and optional machine-readable host files.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RedirectsOutput {
    /// Static HTML redirects, first-seen source order, last-wins dest.
    pub pages: Vec<RedirectEntry>,
    /// Netlify `_redirects` body when requested.
    pub netlify: Option<String>,
    /// Host `_headers` body when requested.
    pub headers: Option<String>,
    /// JSON rewrite map when requested.
    pub json: Option<String>,
}

/// Builds redirect HTML (and optional host files) without writing files.
pub fn generate_redirects(options: &RedirectsOptions, pages: &[RedirectPage]) -> RedirectsOutput {
    if !options.enabled {
        return RedirectsOutput::default();
    }

    let occupied: FxHashSet<String> =
        pages.iter().filter_map(|page| normalize_path(&page.dest)).collect();
    let mut entries = Vec::new();
    let mut index: FxHashMap<String, usize> = FxHashMap::default();

    for page in pages {
        let Some(to) = normalize_dest(&page.dest, options.allow_external) else {
            continue;
        };
        for alias in &page.aliases {
            upsert(&mut entries, &mut index, &occupied, alias, &to);
        }
        if let Some(redirect) = page.redirect.as_deref() {
            upsert(&mut entries, &mut index, &occupied, redirect, &to);
        }
    }
    for (from, to) in &options.map {
        let Some(to) = normalize_dest(to, options.allow_external) else {
            continue;
        };
        upsert(&mut entries, &mut index, &occupied, from, &to);
    }

    if entries.is_empty() {
        return RedirectsOutput::default();
    }

    let netlify = options.netlify.then(|| netlify_body(&entries));
    let headers = options.headers.then(|| headers_body(&entries));
    let json = options.json.then(|| json_body(&entries));
    RedirectsOutput { pages: entries, netlify, headers, json }
}

/// Static HTML redirect body. `dest` is escaped; callers still validate it.
pub fn generate_redirect_html(dest: &str) -> String {
    let escaped = escape_html(dest);
    format!(
        "<!DOCTYPE html>\n\
         <html lang=\"en\">\n\
         <head>\n\
         <meta charset=\"utf-8\">\n\
         <meta http-equiv=\"refresh\" content=\"0;url={escaped}\">\n\
         <link rel=\"canonical\" href=\"{escaped}\">\n\
         <title>Redirecting</title>\n\
         </head>\n\
         <body>\n\
         <p>Redirecting to <a href=\"{escaped}\">{escaped}</a>.</p>\n\
         </body>\n\
         </html>\n"
    )
}

/// Same-origin path: leading `/`, not `//`, and no scheme.
pub fn is_safe_dest(value: &str) -> bool {
    is_allowed_dest(value, false)
}

/// Strips a trailing slash except for `/`. Unsafe values become `None`.
pub fn normalize_path(value: &str) -> Option<String> {
    normalize_dest(value, false)
}

fn normalize_dest(value: &str, allow_external: bool) -> Option<String> {
    if !is_allowed_dest(value, allow_external) {
        return None;
    }
    let trimmed = value.trim();
    if is_http_url(trimmed) {
        return Some(trimmed.to_string());
    }
    if trimmed == "/" {
        return Some("/".to_string());
    }
    Some(trimmed.trim_end_matches('/').to_string())
}

fn is_allowed_dest(value: &str, allow_external: bool) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.bytes().any(|byte| byte.is_ascii_control() || byte == b';') {
        return false;
    }
    if is_http_url(trimmed) {
        return allow_external;
    }
    if !trimmed.starts_with('/') || trimmed.starts_with("//") || has_unsafe_path_segments(trimmed) {
        return false;
    }
    let lower = trimmed.to_ascii_lowercase();
    !lower.contains("javascript:") && !lower.contains("data:") && !lower.contains("://")
}

fn is_http_url(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.starts_with("https://") || lower.starts_with("http://")
}

fn has_unsafe_path_segments(value: &str) -> bool {
    value.contains('\\') || value.split('/').any(|segment| matches!(segment, "." | ".."))
}

fn upsert(
    entries: &mut Vec<RedirectEntry>,
    index: &mut FxHashMap<String, usize>,
    occupied: &FxHashSet<String>,
    from: &str,
    to: &str,
) {
    let Some(from) = normalize_path(from) else {
        return;
    };
    if from == to || occupied.contains(&from) {
        return;
    }
    if let Some(slot) = index.get(&from).copied() {
        entries[slot].to = to.to_string();
        entries[slot].html = generate_redirect_html(to);
        return;
    }
    index.insert(from.clone(), entries.len());
    entries.push(RedirectEntry { from, to: to.to_string(), html: generate_redirect_html(to) });
}

fn netlify_body(entries: &[RedirectEntry]) -> String {
    let mut body = String::new();
    for entry in entries {
        body.push_str(&entry.from);
        body.push(' ');
        body.push_str(&entry.to);
        body.push_str(" 301\n");
    }
    body
}

fn headers_body(entries: &[RedirectEntry]) -> String {
    let mut body = String::new();
    for entry in entries {
        body.push_str(&entry.from);
        body.push_str("\n  Location: ");
        body.push_str(&entry.to);
        body.push('\n');
    }
    body
}

fn json_body(entries: &[RedirectEntry]) -> String {
    let mut body = String::from("[");
    for (index, entry) in entries.iter().enumerate() {
        if index > 0 {
            body.push(',');
        }
        body.push_str("{\"from\":\"");
        escape_json(&entry.from, &mut body);
        body.push_str("\",\"to\":\"");
        escape_json(&entry.to, &mut body);
        body.push_str("\"}");
    }
    body.push(']');
    body
}

fn escape_html(value: &str) -> String {
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

fn escape_json(value: &str, output: &mut String) {
    for ch in value.chars() {
        match ch {
            '\\' => output.push_str("\\\\"),
            '"' => output.push_str("\\\""),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            _ => output.push(ch),
        }
    }
}

#[cfg(test)]
mod tests;
