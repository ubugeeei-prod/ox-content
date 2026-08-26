use super::escape_html_attr;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ParsedAttrs {
    id: Option<String>,
    classes: Vec<String>,
    attrs: Vec<(String, String)>,
}

impl ParsedAttrs {
    pub(super) fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    pub(super) fn attr_value(&self, name: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(candidate, _)| candidate.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }

    pub(super) fn parse(value: &str) -> Option<Self> {
        if !value.contains('#') && !value.contains('.') && !value.contains('=') {
            return None;
        }
        let mut parsed = Self::default();
        for token in split_attr_tokens(value) {
            if let Some(id) = token.strip_prefix('#') {
                if id.is_empty() {
                    return None;
                }
                parsed.id = Some(id.to_string());
            } else if let Some(class) = token.strip_prefix('.') {
                if class.is_empty() {
                    return None;
                }
                parsed.classes.push(class.to_string());
            } else if let Some((name, raw_value)) = token.split_once('=') {
                let name = name.trim();
                let value = raw_value.trim_matches(|ch| ch == '"' || ch == '\'');
                if !is_safe_attr_name(name) || !is_safe_attr_value(name, value) {
                    return None;
                }
                parsed.attrs.push((name.to_string(), value.to_string()));
            } else {
                return None;
            }
        }
        if parsed.id.is_none() && parsed.classes.is_empty() && parsed.attrs.is_empty() {
            None
        } else {
            Some(parsed)
        }
    }
}

fn split_attr_tokens(value: &str) -> Vec<&str> {
    let bytes = value.as_bytes();
    let mut tokens = Vec::new();
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor >= bytes.len() {
            break;
        }
        let start = cursor;
        let mut quote = None;
        while cursor < bytes.len() {
            match quote {
                Some(q) if bytes[cursor] == q => quote = None,
                Some(_) => {}
                None if bytes[cursor] == b'"' || bytes[cursor] == b'\'' => {
                    quote = Some(bytes[cursor]);
                }
                None if bytes[cursor].is_ascii_whitespace() => break,
                None => {}
            }
            cursor += 1;
        }
        tokens.push(&value[start..cursor]);
    }
    tokens
}

/// Writes `open_without_gt` plus parsed attrs, replacing an existing `id`
/// instead of emitting a second `id` attribute.
pub(super) fn write_open_tag_attrs(out: &mut String, open_without_gt: &str, attrs: &ParsedAttrs) {
    if attrs.id.is_some() {
        out.push_str(&strip_quoted_attr(open_without_gt, "id"));
    } else {
        out.push_str(open_without_gt);
    }
    write_attrs(out, attrs);
}

pub(super) fn strip_quoted_attr(open: &str, name: &str) -> String {
    for quote in ['"', '\''] {
        let mut needle = String::with_capacity(name.len() + 3);
        needle.push(' ');
        needle.push_str(name);
        needle.push('=');
        needle.push(quote);
        if let Some(start) = open.find(&needle) {
            let value_start = start + needle.len();
            if let Some(end_rel) = open[value_start..].find(quote) {
                let end = value_start + end_rel + 1;
                let mut stripped = String::with_capacity(open.len() - (end - start));
                stripped.push_str(&open[..start]);
                stripped.push_str(&open[end..]);
                return stripped;
            }
        }
    }
    open.to_string()
}

pub(super) fn write_attrs(out: &mut String, attrs: &ParsedAttrs) {
    write_attrs_except(out, attrs, &[]);
}

pub(super) fn write_attrs_except(out: &mut String, attrs: &ParsedAttrs, excluded: &[&str]) {
    if let Some(id) = &attrs.id
        && !is_excluded_attr("id", excluded)
    {
        out.push_str(" id=\"");
        escape_html_attr(id, out);
        out.push('"');
    }
    if !attrs.classes.is_empty() && !is_excluded_attr("class", excluded) {
        out.push_str(" class=\"");
        for (index, class) in attrs.classes.iter().enumerate() {
            if index > 0 {
                out.push(' ');
            }
            escape_html_attr(class, out);
        }
        out.push('"');
    }
    for (name, value) in &attrs.attrs {
        if is_excluded_attr(name, excluded) {
            continue;
        }
        out.push(' ');
        out.push_str(name);
        out.push_str("=\"");
        escape_html_attr(value, out);
        out.push('"');
    }
}

fn is_excluded_attr(name: &str, excluded: &[&str]) -> bool {
    excluded.iter().any(|excluded| name.eq_ignore_ascii_case(excluded))
}

fn is_safe_attr_name(name: &str) -> bool {
    !name.is_empty()
        && !name.to_ascii_lowercase().starts_with("on")
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b':' | b'_'))
}

fn is_safe_attr_value(name: &str, value: &str) -> bool {
    !is_url_attr_name(name) || !is_dangerous_url(value)
}

fn is_url_attr_name(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "href" | "src" | "poster" | "action" | "formaction" | "xlink:href"
    )
}

fn is_dangerous_url(value: &str) -> bool {
    let compact: String = value
        .chars()
        .filter(|ch| !ch.is_ascii_whitespace())
        .map(|ch| ch.to_ascii_lowercase())
        .collect();
    compact.starts_with("//")
        || compact.starts_with("javascript:")
        || compact.starts_with("data:")
        || compact.starts_with("vbscript:")
}
