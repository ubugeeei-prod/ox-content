/// A collected translation key from Markdown content.
#[derive(Debug, Clone)]
pub struct MdKeyUsage {
    pub key: String,
    pub file_path: String,
    pub line: u32,
    pub column: u32,
}

/// Collects translation keys from Markdown files.
///
/// Looks for patterns like `{{t('key')}}` or `{{ $t('key') }}` in Markdown content.
pub fn collect_md_keys(source: &str, file_path: &str) -> Vec<MdKeyUsage> {
    let mut usages = Vec::new();
    let mut line = 1u32;
    let mut col = 1u32;
    let bytes = source.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if bytes[i] == b'\n' {
            line += 1;
            col = 1;
            i += 1;
            continue;
        }

        // Look for `{{` start
        if i + 1 < len && bytes[i] == b'{' && bytes[i + 1] == b'{' {
            let start_col = col;
            let start_line = line;
            i += 2;
            col += 2;

            // Find matching `}}`
            let content_start = i;
            while i + 1 < len && !(bytes[i] == b'}' && bytes[i + 1] == b'}') {
                if bytes[i] == b'\n' {
                    line += 1;
                    col = 1;
                } else {
                    col += 1;
                }
                i += 1;
            }

            if i + 1 < len {
                let content = &source[content_start..i];
                let content = content.trim();

                // Match t('key') or $t('key')
                if let Some(key) = extract_key_from_expression(content) {
                    usages.push(MdKeyUsage {
                        key,
                        file_path: file_path.to_string(),
                        line: start_line,
                        column: start_col,
                    });
                }

                i += 2; // skip `}}`
                col += 2;
            }
        } else {
            col += 1;
            i += 1;
        }
    }

    usages
}

/// Extracts a key from an expression like `t('key')` or `$t('key')`.
fn extract_key_from_expression(expr: &str) -> Option<String> {
    let expr = expr.trim();

    // Match t('...') or $t('...')
    let rest = expr.strip_prefix("$t(").or_else(|| expr.strip_prefix("t("))?;

    let rest = rest.strip_suffix(')')?;
    let rest = rest.trim();

    // Extract string literal (single or double quotes)
    if (rest.starts_with('\'') && rest.ends_with('\''))
        || (rest.starts_with('"') && rest.ends_with('"'))
    {
        Some(rest[1..rest.len() - 1].to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_md_key() {
        let usages = collect_md_keys("# Title\n\n{{t('common.greeting')}}", "test.md");
        assert_eq!(usages.len(), 1);
        assert_eq!(usages[0].key, "common.greeting");
        assert_eq!(usages[0].line, 3);
    }

    #[test]
    fn dollar_t_in_md() {
        let usages = collect_md_keys("{{$t('nav.home')}}", "test.md");
        assert_eq!(usages.len(), 1);
        assert_eq!(usages[0].key, "nav.home");
    }

    #[test]
    fn spaces_in_braces() {
        let usages = collect_md_keys("{{ t('key') }}", "test.md");
        assert_eq!(usages.len(), 1);
        assert_eq!(usages[0].key, "key");
    }

    #[test]
    fn double_quotes() {
        let usages = collect_md_keys(r#"{{t("key")}}"#, "test.md");
        assert_eq!(usages.len(), 1);
        assert_eq!(usages[0].key, "key");
    }

    #[test]
    fn multiple_keys() {
        let usages = collect_md_keys("{{t('key1')}} some text {{t('key2')}}", "test.md");
        assert_eq!(usages.len(), 2);
    }

    #[test]
    fn no_keys() {
        let usages = collect_md_keys("# Just a heading\n\nNo keys here.", "test.md");
        assert!(usages.is_empty());
    }
}
