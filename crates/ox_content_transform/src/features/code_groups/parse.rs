use super::super::segments::{is_closing_fence, parse_opening_fence};

const TYPE_NAME: &str = "code-group";

pub(super) struct GroupedFence {
    pub(super) label: String,
    pub(super) source: String,
    pub(super) warning: Option<String>,
}

pub(super) enum GroupInner {
    Fences(Vec<GroupedFence>),
    Degrade(String),
}

pub(super) fn parse_opener(line: &str) -> Option<usize> {
    let (colon_count, name) = split_container_name(line)?;
    name.eq_ignore_ascii_case(TYPE_NAME).then_some(colon_count)
}

pub(super) fn parse_any_opener(line: &str) -> Option<usize> {
    split_container_name(line).map(|(colon_count, _)| colon_count)
}

pub(super) fn parse_closer(line: &str) -> Option<usize> {
    let colon_count = line.bytes().take_while(|byte| *byte == b':').count();
    if colon_count < 3 {
        return None;
    }
    line[colon_count..].bytes().all(|byte| byte.is_ascii_whitespace()).then_some(colon_count)
}

pub(super) fn analyze_inner(inner: &str) -> GroupInner {
    let mut fences = Vec::new();
    let mut extra_content = false;
    let mut in_fence = false;
    let mut fence_char = b'\0';
    let mut fence_len = 0usize;
    let mut current = String::new();
    let mut current_info = String::new();
    let mut opened = false;

    for line_with_end in inner.split_inclusive('\n') {
        let (line, ending) = split_ending(line_with_end);

        if in_fence {
            current.push_str(line);
            current.push_str(ending);
            if is_closing_fence(line, fence_char, fence_len) {
                let index = fences.len();
                let (label, warning) = fence_label(&current_info, index);
                fences.push(GroupedFence { label, source: std::mem::take(&mut current), warning });
                in_fence = false;
                opened = false;
            }
            continue;
        }

        if let Some(open) = parse_opening_fence(line) {
            in_fence = true;
            opened = true;
            fence_char = open.fence_char;
            fence_len = open.fence_len;
            current_info = info_string(line, open.fence_len);
            current.push_str(line);
            current.push_str(ending);
            continue;
        }

        if !line.trim().is_empty() {
            extra_content = true;
        }
    }

    if opened {
        return GroupInner::Degrade(
            "code-group: unclosed fence; rendering inner content as ordinary Markdown".to_string(),
        );
    }
    if extra_content {
        return GroupInner::Degrade(
            "code-group: non-fence content; rendering inner fences as ordinary code".to_string(),
        );
    }
    if fences.is_empty() {
        return GroupInner::Degrade(
            "code-group: no fenced blocks; rendering the container as ordinary Markdown"
                .to_string(),
        );
    }
    GroupInner::Fences(fences)
}

pub(super) fn split_ending(line_with_end: &str) -> (&str, &str) {
    if let Some(line) = line_with_end.strip_suffix("\r\n") {
        (line, "\n")
    } else if let Some(line) = line_with_end.strip_suffix('\n') {
        (line, "\n")
    } else {
        (line_with_end.strip_suffix('\r').unwrap_or(line_with_end), "")
    }
}

pub(super) fn trim_container_indent(line: &str) -> &str {
    let bytes = line.as_bytes();
    let mut indent = 0usize;
    while indent < bytes.len() && indent < 3 && bytes[indent] == b' ' {
        indent += 1;
    }
    &line[indent..]
}

fn split_container_name(line: &str) -> Option<(usize, &str)> {
    let colon_count = line.bytes().take_while(|byte| *byte == b':').count();
    if colon_count < 3 {
        return None;
    }
    let rest = line[colon_count..].trim_start();
    if rest.is_empty() {
        return None;
    }
    let end = rest
        .find(|ch: char| ch.is_ascii_whitespace() || ch == '[' || ch == '{')
        .unwrap_or(rest.len());
    let name = &rest[..end];
    (!name.is_empty()).then_some((colon_count, name))
}

fn info_string(line: &str, fence_len: usize) -> String {
    line.trim_start().get(fence_len..).unwrap_or("").trim().to_string()
}

fn fence_label(info: &str, index: usize) -> (String, Option<String>) {
    match parse_bracket_title(info) {
        Ok(Some(label)) if !label.is_empty() => return (label, None),
        Ok(_) => {}
        Err(()) => {
            return fallback_label(
                info,
                index,
                Some("code-group: malformed fence title; using a language or Tab N label"),
            );
        }
    }

    match parse_meta_title(info) {
        Ok(Some(label)) if !label.is_empty() => return (label, None),
        Ok(_) => {}
        Err(()) => {
            return fallback_label(
                info,
                index,
                Some("code-group: malformed fence title; using a language or Tab N label"),
            );
        }
    }

    fallback_label(info, index, None)
}

fn fallback_label(info: &str, index: usize, warning: Option<&str>) -> (String, Option<String>) {
    let language = language_from_info(info);
    if language.is_empty() {
        (format!("Tab {}", index + 1), warning.map(str::to_string))
    } else {
        (language, warning.map(str::to_string))
    }
}

fn parse_bracket_title(info: &str) -> Result<Option<String>, ()> {
    let bytes = info.as_bytes();
    let mut last_open = None;
    let mut last_pair = None;
    for (index, byte) in bytes.iter().enumerate() {
        if *byte == b'[' {
            last_open = Some(index);
        } else if *byte == b']'
            && let Some(open) = last_open.take()
        {
            last_pair = Some((open, index));
        }
    }
    if last_open.is_some() {
        return Err(());
    }
    Ok(last_pair.map(|(start, end)| info[start + 1..end].trim().to_string()))
}

fn parse_meta_title(info: &str) -> Result<Option<String>, ()> {
    let lower = info.to_ascii_lowercase();
    let mut search = 0usize;
    while let Some(relative) = lower[search..].find("title") {
        let at = search + relative;
        let before_ok = at == 0
            || info.as_bytes().get(at.wrapping_sub(1)).is_some_and(|byte| {
                !byte.is_ascii_alphanumeric() && *byte != b'_' && *byte != b'-'
            });
        let after = at + 5;
        if before_ok {
            let rest = info.get(after..).unwrap_or("").trim_start();
            if let Some(value) = rest.strip_prefix('=') {
                return parse_title_value(value.trim_start());
            }
        }
        search = after;
    }
    Ok(None)
}

fn parse_title_value(value: &str) -> Result<Option<String>, ()> {
    let bytes = value.as_bytes();
    let first = *bytes.first().ok_or(())?;
    if first == b'"' || first == b'\'' {
        let inner = &value[1..];
        let end = inner.as_bytes().iter().position(|byte| *byte == first).ok_or(())?;
        return Ok(Some(inner[..end].to_string()));
    }
    Ok(value.split_whitespace().next().filter(|token| !token.is_empty()).map(str::to_string))
}

fn language_from_info(info: &str) -> String {
    let token = info.split_whitespace().next().unwrap_or("");
    if token.starts_with('[') || token.to_ascii_lowercase().starts_with("title=") {
        return String::new();
    }
    token.split('{').next().unwrap_or("").to_string()
}

#[cfg(test)]
mod title_tests {
    use super::{fence_label, parse_bracket_title, parse_meta_title};

    #[test]
    fn bracket_title_wins() {
        let (label, warning) = fence_label("js [config.js]", 0);
        assert_eq!(label, "config.js");
        assert!(warning.is_none());
    }

    #[test]
    fn meta_title_is_used_without_brackets() {
        let (label, warning) = fence_label(r#"ts title="config.ts""#, 0);
        assert_eq!(label, "config.ts");
        assert!(warning.is_none());
        assert_eq!(parse_meta_title("js title=bare").unwrap().as_deref(), Some("bare"));
    }

    #[test]
    fn language_is_the_fallback() {
        let (label, warning) = fence_label("js{1,2}", 0);
        assert_eq!(label, "js");
        assert!(warning.is_none());
    }

    #[test]
    fn unclosed_bracket_is_malformed() {
        assert!(parse_bracket_title("js [config.js").is_err());
        let (label, warning) = fence_label("js [config.js", 0);
        assert_eq!(label, "js");
        assert!(warning.is_some());
    }
}
