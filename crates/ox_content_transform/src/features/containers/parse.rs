#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ParsedOpener {
    pub(super) name: String,
    pub(super) title: Option<String>,
    pub(super) attrs: Vec<(String, Option<String>)>,
    pub(super) colon_count: usize,
}

pub(super) fn normalize_type_name(name: &str) -> Option<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return None;
    }
    if !trimmed.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_') {
        return None;
    }
    Some(trimmed.to_ascii_lowercase())
}

pub(super) fn parse_opener(line: &str) -> Option<ParsedOpener> {
    let colon_count = line.bytes().take_while(|byte| *byte == b':').count();
    if colon_count < 3 {
        return None;
    }
    let rest = line[colon_count..].trim_start();
    if rest.is_empty() {
        return None;
    }

    let (name_part, after_name) = split_name(rest)?;
    let name = normalize_type_name(name_part)?;

    let mut title = None;
    let mut attrs = Vec::new();
    let mut cursor = after_name.trim_start();

    if let Some(inner) = cursor.strip_prefix('[') {
        let end = inner.find(']')?;
        let value = inner[..end].trim();
        if !value.is_empty() {
            title = Some(value.to_string());
        }
        cursor = inner[end + 1..].trim_start();
    }

    if let Some(inner) = cursor.strip_prefix('{') {
        let end = inner.find('}')?;
        attrs = parse_attrs(&inner[..end]);
        cursor = inner[end + 1..].trim_start();
    }

    if title.is_none() && !cursor.is_empty() && !cursor.starts_with('{') {
        title = Some(cursor.trim().to_string());
    }

    Some(ParsedOpener { name, title, attrs, colon_count })
}

fn split_name(rest: &str) -> Option<(&str, &str)> {
    let end = rest
        .find(|ch: char| ch.is_ascii_whitespace() || ch == '[' || ch == '{')
        .unwrap_or(rest.len());
    let name = &rest[..end];
    (!name.is_empty()).then_some((name, &rest[end..]))
}

fn parse_attrs(raw: &str) -> Vec<(String, Option<String>)> {
    let mut attrs = Vec::new();
    for token in raw.split_whitespace() {
        if let Some(class) = token.strip_prefix('.') {
            if is_safe_ident(class) {
                attrs.push(("class".to_string(), Some(class.to_string())));
            }
        } else if let Some(id) = token.strip_prefix('#') {
            if is_safe_ident(id) {
                attrs.push(("id".to_string(), Some(id.to_string())));
            }
        } else if let Some((key, value)) = token.split_once('=') {
            if is_safe_ident(key) && is_safe_attr_value(value.trim_matches(['"', '\''])) {
                attrs.push((key.to_string(), Some(value.trim_matches(['"', '\'']).to_string())));
            }
        } else if is_safe_ident(token) {
            attrs.push((token.to_string(), None));
        }
    }
    attrs
}

fn is_safe_ident(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
}

fn is_safe_attr_value(value: &str) -> bool {
    !value.bytes().any(|byte| matches!(byte, b'<' | b'>' | b'"' | b'\'' | b'`' | b'=' | b'\n'))
}

pub(super) fn parse_closer(line: &str) -> Option<usize> {
    let colon_count = line.bytes().take_while(|byte| *byte == b':').count();
    if colon_count < 3 {
        return None;
    }
    line[colon_count..].bytes().all(|byte| byte.is_ascii_whitespace()).then_some(colon_count)
}

#[cfg(test)]
mod tests {
    use super::{ParsedOpener, parse_closer, parse_opener};

    #[test]
    fn parse_opener_reads_type_bracket_title_and_attrs() {
        let parsed = parse_opener("::: tip[Did you know?]{.lead #install open}").unwrap();
        assert_eq!(
            parsed,
            ParsedOpener {
                name: "tip".into(),
                title: Some("Did you know?".into()),
                attrs: vec![
                    ("class".into(), Some("lead".into())),
                    ("id".into(), Some("install".into())),
                    ("open".into(), None),
                ],
                colon_count: 3,
            }
        );
    }

    #[test]
    fn parse_opener_reads_trailing_title_without_brackets() {
        let parsed = parse_opener("::: warning Watch out").unwrap();
        assert_eq!(parsed.name, "warning");
        assert_eq!(parsed.title.as_deref(), Some("Watch out"));
    }

    #[test]
    fn parse_opener_is_case_insensitive() {
        assert_eq!(parse_opener("::: TIP").unwrap().name, "tip");
        assert_eq!(parse_opener("::: Warning").unwrap().name, "warning");
    }

    #[test]
    fn parse_opener_rejects_fewer_than_three_colons() {
        assert!(parse_opener(":: tip").is_none());
        assert!(parse_opener(": tip").is_none());
        assert!(parse_opener("tip").is_none());
    }

    #[test]
    fn parse_opener_rejects_empty_or_hostile_type_names() {
        assert!(parse_opener(":::").is_none());
        assert!(parse_opener(":::   ").is_none());
        assert!(parse_opener(r#"::: tip"onclick=alert(1)"#).is_none());
        assert!(parse_opener("::: tip<script>").is_none());
    }

    #[test]
    fn parse_opener_allows_four_colons_for_nesting() {
        let parsed = parse_opener(":::: note").unwrap();
        assert_eq!(parsed.colon_count, 4);
        assert_eq!(parsed.name, "note");
    }

    #[test]
    fn parse_closer_requires_only_colons() {
        assert_eq!(parse_closer(":::"), Some(3));
        assert_eq!(parse_closer("::::"), Some(4));
        assert_eq!(parse_closer(":::   "), Some(3));
        assert!(parse_closer("::: tip").is_none());
        assert!(parse_closer("::").is_none());
    }
}
