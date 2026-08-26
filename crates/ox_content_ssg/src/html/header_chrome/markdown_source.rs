use super::super::utils::escape_html;

pub fn markdown_source_chrome_enabled(href: Option<&str>) -> bool {
    href.is_some_and(is_safe_markdown_source_href)
}

pub fn insert_markdown_source_chrome(html: &str, href: Option<&str>) -> String {
    let Some(href) = href.filter(|value| is_safe_markdown_source_href(value)) else {
        return html.to_string();
    };
    let chrome = render_markdown_source_chrome(href);
    insert_after_first_h1(html, &chrome)
}

fn render_markdown_source_chrome(href: &str) -> String {
    let escaped = escape_html(href);
    format!(
        "<p class=\"ox-markdown-source\">\
<button type=\"button\" class=\"ox-copy-markdown\" data-ox-copy-markdown aria-label=\"Copy as Markdown\">Copy as Markdown</button>\
<a class=\"ox-view-markdown\" href=\"{escaped}\">View Markdown</a>\
<span class=\"ox-copy-markdown-status\" data-ox-copy-markdown-status role=\"status\" aria-live=\"polite\"></span>\
</p>\n"
    )
}

fn insert_after_first_h1(html: &str, chrome: &str) -> String {
    if let Some(end) = first_h1_end(html) {
        return format!("{}{}{}", &html[..end], chrome, &html[end..]);
    }
    format!("{chrome}{html}")
}

fn first_h1_end(html: &str) -> Option<usize> {
    let lower = html.to_ascii_lowercase();
    let mut from = 0;
    while let Some(rel) = lower[from..].find("<h1") {
        let start = from + rel;
        let after = html.get(start + 3..)?;
        let next = after.chars().next()?;
        if next != '>' && !next.is_ascii_whitespace() {
            from = start + 3;
            continue;
        }
        let close = lower[start..].find("</h1>")?;
        return Some(start + close + 5);
    }
    None
}

fn is_safe_markdown_source_href(href: &str) -> bool {
    let trimmed = href.trim();
    if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.contains('\\') {
        return false;
    }
    let compact: String = trimmed.chars().filter(|ch| !ch.is_ascii_whitespace()).collect();
    let lower = compact.to_ascii_lowercase();
    if lower.starts_with("javascript:")
        || lower.starts_with("data:")
        || lower.starts_with("vbscript:")
        || lower.contains(':')
    {
        return false;
    }
    if lower.contains("/../") || lower.starts_with("../") || lower.ends_with("/..") {
        return false;
    }
    (lower.starts_with('/') || lower.starts_with("./")) && lower.ends_with(".md")
}
