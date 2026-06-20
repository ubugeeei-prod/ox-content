use super::escape_html_attr;

#[derive(Default)]
pub(super) struct ParsedAttrs {
    id: Option<String>,
    classes: Vec<String>,
    attrs: Vec<(String, String)>,
}

impl ParsedAttrs {
    pub(super) fn parse(value: &str) -> Option<Self> {
        if !value.contains('#') && !value.contains('.') && !value.contains('=') {
            return None;
        }
        let mut parsed = Self::default();
        for token in split_attr_tokens(value) {
            if let Some(id) = token.strip_prefix('#') {
                if !id.is_empty() {
                    parsed.id = Some(id.to_string());
                }
            } else if let Some(class) = token.strip_prefix('.') {
                if !class.is_empty() {
                    parsed.classes.push(class.to_string());
                }
            } else if let Some((name, raw_value)) = token.split_once('=') {
                let name = name.trim();
                if is_safe_attr_name(name) {
                    parsed.attrs.push((
                        name.to_string(),
                        raw_value.trim_matches(|ch| ch == '"' || ch == '\'').to_string(),
                    ));
                }
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

pub(super) fn write_attrs(out: &mut String, attrs: &ParsedAttrs) {
    if let Some(id) = &attrs.id {
        out.push_str(" id=\"");
        escape_html_attr(id, out);
        out.push('"');
    }
    if !attrs.classes.is_empty() {
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
        out.push(' ');
        out.push_str(name);
        out.push_str("=\"");
        escape_html_attr(value, out);
        out.push('"');
    }
}

fn is_safe_attr_name(name: &str) -> bool {
    !name.is_empty()
        && !name.to_ascii_lowercase().starts_with("on")
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b':' | b'_'))
}
