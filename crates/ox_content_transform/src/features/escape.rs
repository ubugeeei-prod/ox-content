pub(crate) fn escape_html_text(value: &str, out: &mut String) {
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(ch),
        }
    }
}

/// Whether a URL resolves to a script, data, or protocol-relative target once
/// a browser has parsed it: leading C0 controls and spaces are dropped and
/// ASCII tab, LF, and CR are removed anywhere before the scheme is read.
pub(crate) fn has_blocked_url_scheme(value: &str) -> bool {
    let mut prefix = [0_u8; 11];
    let mut len = 0;
    for byte in value.bytes().skip_while(|byte| *byte <= b' ') {
        if matches!(byte, b'\t' | b'\n' | b'\r') {
            continue;
        }
        let Some(slot) = prefix.get_mut(len) else {
            break;
        };
        *slot = byte.to_ascii_lowercase();
        len += 1;
    }
    let prefix = &prefix[..len];
    [&b"//"[..], b"javascript:", b"data:", b"vbscript:"]
        .iter()
        .any(|blocked| prefix.starts_with(blocked))
}

pub(crate) fn escape_html_attr(value: &str, out: &mut String) {
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(ch),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::has_blocked_url_scheme;

    #[test]
    fn blocked_url_schemes_match_the_browser_reading() {
        for url in [
            "javascript:alert(1)",
            " JavaScript:alert(1)",
            "\u{1}javascript:alert(1)",
            "\u{b}javascript:alert(1)",
            "java\tscript:alert(1)",
            "java\nscr\ript:alert(1)",
            "data:text/html,x",
            "vbscript:x",
            "//evil.example",
            "\u{1}//evil.example",
        ] {
            assert!(has_blocked_url_scheme(url), "{url:?}");
        }
        for url in ["/guide", "./a:b", "https://example.com", "mailto:a@b.c", "#javascript:"] {
            assert!(!has_blocked_url_scheme(url), "{url:?}");
        }
    }
}
