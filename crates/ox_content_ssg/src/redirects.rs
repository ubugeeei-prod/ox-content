//! Opt-in static redirect HTML and optional host `_redirects` bodies.

use rustc_hash::FxHashMap;

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
}

/// One static HTML redirect to write at `from`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedirectEntry {
    /// Normalized source path (`/old`).
    pub from: String,
    /// Normalized destination path (`/guide`).
    pub to: String,
    /// Escaped static HTML body.
    pub html: String,
}

/// Generated redirect pages and an optional host rewrite file.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RedirectsOutput {
    /// Static HTML redirects, first-seen source order, last-wins dest.
    pub pages: Vec<RedirectEntry>,
    /// Netlify `_redirects` body when requested.
    pub netlify: Option<String>,
}

/// Builds redirect HTML (and optional `_redirects`) without writing files.
pub fn generate_redirects(options: &RedirectsOptions, pages: &[RedirectPage]) -> RedirectsOutput {
    if !options.enabled {
        return RedirectsOutput::default();
    }

    let mut entries = Vec::new();
    let mut index: FxHashMap<String, usize> = FxHashMap::default();

    for page in pages {
        let Some(to) = normalize_path(&page.dest) else {
            continue;
        };
        for alias in &page.aliases {
            upsert(&mut entries, &mut index, alias, &to);
        }
        if let Some(redirect) = page.redirect.as_deref() {
            upsert(&mut entries, &mut index, redirect, &to);
        }
    }
    for (from, to) in &options.map {
        let Some(to) = normalize_path(to) else {
            continue;
        };
        upsert(&mut entries, &mut index, from, &to);
    }

    let netlify = options.netlify.then(|| netlify_body(&entries));
    RedirectsOutput { pages: entries, netlify }
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
    let trimmed = value.trim();
    if !trimmed.starts_with('/') || trimmed.starts_with("//") {
        return false;
    }
    let lower = trimmed.to_ascii_lowercase();
    !lower.contains("javascript:") && !lower.contains("data:") && !lower.contains("://")
}

/// Strips a trailing slash except for `/`. Unsafe values become `None`.
pub fn normalize_path(value: &str) -> Option<String> {
    if !is_safe_dest(value) {
        return None;
    }
    let trimmed = value.trim();
    if trimmed == "/" {
        return Some("/".to_string());
    }
    Some(trimmed.trim_end_matches('/').to_string())
}

fn upsert(
    entries: &mut Vec<RedirectEntry>,
    index: &mut FxHashMap<String, usize>,
    from: &str,
    to: &str,
) {
    let Some(from) = normalize_path(from) else {
        return;
    };
    if from == to {
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

#[cfg(test)]
mod tests;
