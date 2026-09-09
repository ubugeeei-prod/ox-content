//! Preserve the existing URL policy while removing temporary scheme strings.

use super::HtmlRenderer;

fn previous_url_policy(url: &str) -> bool {
    if url.bytes().any(|byte| byte.is_ascii_control()) {
        return false;
    }
    let Some(colon_index) = url.find(':') else {
        return true;
    };
    let first_path_marker = url.find(&['/', '?', '#'][..]).unwrap_or(usize::MAX);
    if first_path_marker < colon_index {
        return true;
    }
    let scheme = url[..colon_index]
        .chars()
        .filter(|ch| !ch.is_ascii_whitespace())
        .map(|ch| ch.to_ascii_lowercase())
        .collect::<String>();
    matches!(scheme.as_str(), "http" | "https" | "mailto" | "tel")
}

fn assert_previous_policy(url: &str) {
    assert_eq!(HtmlRenderer::is_safe_url(url), previous_url_policy(url), "{url:?}");
}

#[test]
fn allowed_schemes_preserve_all_case_and_space_combinations() {
    for scheme in ["http", "https", "mailto", "tel"] {
        for case_mask in 0..(1 << scheme.len()) {
            for space_mask in 0..(1 << (scheme.len() + 1)) {
                let mut candidate = String::new();
                for (index, byte) in scheme.bytes().enumerate() {
                    if space_mask & (1 << index) != 0 {
                        candidate.push(' ');
                    }
                    let byte = if case_mask & (1 << index) != 0 {
                        byte.to_ascii_uppercase()
                    } else {
                        byte
                    };
                    candidate.push(char::from(byte));
                }
                if space_mask & (1 << scheme.len()) != 0 {
                    candidate.push(' ');
                }
                candidate.push_str(":example");
                assert!(HtmlRenderer::is_safe_url(&candidate), "{candidate:?}");
            }
        }
    }
}

#[test]
fn scheme_mutations_match_the_previous_policy() {
    for scheme in
        ["http", "https", "mailto", "tel", "javascript", "data", "vbscript", "file", "ftp", ""]
    {
        for index in 0..=scheme.len() {
            for inserted in (0u8..=255).map(char::from).chain(['日', '🙂', '\u{200b}', '\u{3000}'])
            {
                let mut candidate = scheme.to_string();
                candidate.insert(index, inserted);
                candidate.push_str(":example");
                assert_previous_policy(&candidate);
            }
        }
        for prefix in ["", "./", "/", "//", "?", "#"] {
            for suffix in
                ["", ":payload", "://example.com", "/path:part", "?query:value", "#part:value"]
            {
                assert_previous_policy(&format!("{prefix}{scheme}{suffix}"));
            }
        }
    }
    assert_previous_policy(&format!("{}:payload", "x".repeat(8192)));
    assert_previous_policy(&format!("{}https{}:example", " ".repeat(128), " ".repeat(128)));
}

#[test]
fn controls_remain_rejected_anywhere_in_the_url() {
    for byte in (0u8..=31).chain([127]) {
        for url in ["https://example.com/api", "./relative/path", "mailto:hi@example.com"] {
            for index in 0..=url.len() {
                let mut candidate = url.to_string();
                candidate.insert(index, char::from(byte));
                assert!(!HtmlRenderer::is_safe_url(&candidate), "{candidate:?}");
            }
        }
    }
}
