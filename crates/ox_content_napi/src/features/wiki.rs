use super::WikiLinkOptions;

pub(super) fn replace_wiki_links(segment: &str, options: &WikiLinkOptions, out: &mut String) {
    let mut cursor = 0usize;
    while let Some(relative) = segment[cursor..].find("[[") {
        let start = cursor + relative;
        let embed = start > 0 && segment.as_bytes()[start - 1] == b'!';
        let literal_start = if embed { start - 1 } else { start };
        out.push_str(&segment[cursor..literal_start]);
        let inner_start = start + 2;
        let Some(close_relative) = segment[inner_start..].find("]]") else {
            out.push_str(&segment[literal_start..]);
            return;
        };
        let close = inner_start + close_relative;
        let inner = segment[inner_start..close].trim();
        if inner.is_empty() {
            out.push_str(&segment[literal_start..close + 2]);
            cursor = close + 2;
            continue;
        }

        let (target, label) = inner
            .split_once('|')
            .map_or((inner, None), |(target, label)| (target.trim(), Some(label.trim())));
        let target = target.trim();
        let label =
            label.filter(|value| !value.is_empty()).unwrap_or_else(|| default_wiki_label(target));
        let url = wiki_target_to_url(target, &options.base_url);
        if embed {
            out.push_str("![");
        } else {
            out.push('[');
        }
        escape_markdown_link_text(label, out);
        out.push_str("](");
        out.push_str(&url);
        out.push(')');
        cursor = close + 2;
    }
    out.push_str(&segment[cursor..]);
}

fn wiki_target_to_url(target: &str, base_url: &str) -> String {
    if target.starts_with("http://")
        || target.starts_with("https://")
        || target.starts_with("mailto:")
        || target.starts_with('#')
    {
        return percent_encode_spaces(target);
    }

    let (path, anchor) =
        target.split_once('#').map_or((target, None), |(path, anchor)| (path, Some(anchor)));
    let mut normalized = path.trim().trim_end_matches(".md").trim_end_matches("/index").to_string();
    if normalized.is_empty() {
        normalized.push('/');
    }
    let mut url = join_base_url(base_url, &normalized);
    if let Some(anchor) = anchor {
        if !anchor.is_empty() {
            url.push('#');
            slugify_anchor(anchor, &mut url);
        }
    }
    percent_encode_spaces(&url)
}

fn join_base_url(base_url: &str, path: &str) -> String {
    if path.starts_with('/') {
        let base = base_url.trim_end_matches('/');
        if base.is_empty() || base == "/" {
            path.to_string()
        } else {
            format!("{base}{path}")
        }
    } else {
        let base = if base_url.is_empty() { "/" } else { base_url };
        format!("{}/{}", base.trim_end_matches('/'), path)
    }
}

fn default_wiki_label(target: &str) -> &str {
    let path = target.split('#').next().unwrap_or(target).trim();
    path.rsplit('/').next().filter(|value| !value.is_empty()).unwrap_or(target)
}

fn escape_markdown_link_text(value: &str, out: &mut String) {
    for ch in value.chars() {
        if matches!(ch, '[' | ']') {
            out.push('\\');
        }
        out.push(ch);
    }
}

fn slugify_anchor(value: &str, out: &mut String) {
    let mut last_dash = false;
    for ch in value.trim().chars().flat_map(char::to_lowercase) {
        if ch.is_alphanumeric() {
            out.push(ch);
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    if out.ends_with('-') {
        out.pop();
    }
}

fn percent_encode_spaces(value: &str) -> String {
    if !value.contains(' ') {
        return value.to_string();
    }
    value.replace(' ', "%20")
}
