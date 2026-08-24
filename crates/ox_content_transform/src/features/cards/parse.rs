#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CardKind {
    Card,
    LinkCard,
    CardGrid,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ParsedCardOpener {
    pub(super) kind: CardKind,
    pub(super) title: Option<String>,
    pub(super) href: Option<String>,
    pub(super) colon_count: usize,
}

pub(super) fn parse_opener(line: &str) -> Option<ParsedCardOpener> {
    let (name, title, href, colon_count) = parse_opener_parts(line)?;
    let kind = match name.as_str() {
        "card" => CardKind::Card,
        "link-card" => CardKind::LinkCard,
        "card-grid" => CardKind::CardGrid,
        _ => return None,
    };
    Some(ParsedCardOpener { kind, title, href, colon_count })
}

pub(super) fn parse_any_opener(line: &str) -> Option<usize> {
    parse_opener_parts(line).map(|parts| parts.3)
}

fn parse_opener_parts(line: &str) -> Option<(String, Option<String>, Option<String>, usize)> {
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
    let mut href = None;
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
        let value = inner[..end].trim();
        if !value.is_empty() {
            href = Some(value.to_string());
        }
        cursor = inner[end + 1..].trim_start();
    }

    if title.is_none() && !cursor.is_empty() && !cursor.starts_with('{') {
        title = Some(cursor.trim().to_string());
    }

    Some((name, title, href, colon_count))
}

fn split_name(rest: &str) -> Option<(&str, &str)> {
    let end = rest
        .find(|ch: char| ch.is_ascii_whitespace() || ch == '[' || ch == '{')
        .unwrap_or(rest.len());
    let name = &rest[..end];
    (!name.is_empty()).then_some((name, &rest[end..]))
}

fn normalize_type_name(name: &str) -> Option<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return None;
    }
    if !trimmed.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_') {
        return None;
    }
    Some(trimmed.to_ascii_lowercase())
}

pub(super) fn parse_closer(line: &str) -> Option<usize> {
    let colon_count = line.bytes().take_while(|byte| *byte == b':').count();
    if colon_count < 3 {
        return None;
    }
    line[colon_count..].bytes().all(|byte| byte.is_ascii_whitespace()).then_some(colon_count)
}

pub(super) fn is_safe_href(href: &str) -> bool {
    let trimmed = href.trim();
    if trimmed.is_empty() || trimmed.starts_with("//") {
        return false;
    }
    let lower = trimmed.to_ascii_lowercase();
    let scheme = lower.split(':').next().unwrap_or("");
    !matches!(scheme, "javascript" | "data" | "vbscript")
}

#[cfg(test)]
mod tests {
    use super::{CardKind, is_safe_href, parse_closer, parse_opener};

    #[test]
    fn parse_opener_reads_card_bracket_title() {
        let parsed = parse_opener("::: card[Install]").unwrap();
        assert_eq!(parsed.kind, CardKind::Card);
        assert_eq!(parsed.title.as_deref(), Some("Install"));
        assert_eq!(parsed.colon_count, 3);
    }

    #[test]
    fn parse_opener_reads_link_card_title_and_href() {
        let parsed = parse_opener("::: link-card[Guide]{/getting-started}").unwrap();
        assert_eq!(parsed.kind, CardKind::LinkCard);
        assert_eq!(parsed.title.as_deref(), Some("Guide"));
        assert_eq!(parsed.href.as_deref(), Some("/getting-started"));
    }

    #[test]
    fn parse_opener_reads_card_grid() {
        let parsed = parse_opener("::: card-grid").unwrap();
        assert_eq!(parsed.kind, CardKind::CardGrid);
    }

    #[test]
    fn parse_opener_rejects_unknown_types() {
        assert!(parse_opener("::: tip").is_none());
        assert!(parse_opener(":::").is_none());
    }

    #[test]
    fn parse_closer_requires_only_colons() {
        assert_eq!(parse_closer(":::"), Some(3));
        assert!(parse_closer("::: card").is_none());
    }

    #[test]
    fn is_safe_href_rejects_dangerous_schemes() {
        assert!(!is_safe_href("javascript:alert(1)"));
        assert!(!is_safe_href("JAVASCRIPT:alert(1)"));
        assert!(!is_safe_href(" data:text/html,x "));
        assert!(!is_safe_href("vbscript:msgbox(1)"));
        assert!(!is_safe_href("//evil.example"));
        assert!(is_safe_href("/getting-started"));
        assert!(is_safe_href("https://example.com"));
        assert!(is_safe_href("./guide"));
    }
}
