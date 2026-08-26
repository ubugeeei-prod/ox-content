use super::{Diagnostic, check};

/// Check a Markdown/MDC document, skipping YAML frontmatter so CLI and LSP
/// report the same codes, ranges, and messages.
#[must_use]
pub fn check_document(source: &str) -> Vec<Diagnostic> {
    let (body, line_offset) = markdown_body_after_frontmatter(source);
    check(body)
        .into_iter()
        .map(|mut diagnostic| {
            diagnostic.line += line_offset;
            diagnostic.end_line += line_offset;
            diagnostic
        })
        .collect()
}

/// Returns the Markdown body and how many lines precede it.
///
/// An opening `---` line starts a frontmatter block. The body begins after the
/// closing `---` / `...` line. An unterminated block has an empty body, matching
/// the language server.
#[must_use]
pub fn markdown_body_after_frontmatter(source: &str) -> (&str, u32) {
    let Some((first, rest)) = source.split_once('\n') else {
        return (source, 0);
    };
    if first.trim_end_matches('\r') != "---" {
        return (source, 0);
    }

    let mut consumed = first.len() + 1;
    let mut line_offset = 1u32;
    for line in rest.split_inclusive('\n') {
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed == "---" || trimmed == "..." {
            consumed += line.len();
            return (source.get(consumed..).unwrap_or(""), line_offset + 1);
        }
        consumed += line.len();
        line_offset += 1;
    }

    ("", line_offset)
}

#[cfg(test)]
mod tests {
    use super::{check_document, markdown_body_after_frontmatter};
    use crate::{CODE_UNQUOTED_PROP, check};

    #[test]
    fn documents_without_frontmatter_match_body_checks() {
        let source = "<Alert tone=info></Alert>\n";
        assert_eq!(check_document(source), check(source));
        assert_eq!(check_document(source)[0].code, CODE_UNQUOTED_PROP);
    }

    #[test]
    fn frontmatter_is_skipped_and_body_lines_stay_document_absolute() {
        let source = "---\ntitle: Doc\n---\n\n<Alert tone=info></Alert>\n";
        let diagnostics = check_document(source);
        assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
        assert_eq!(diagnostics[0].code, CODE_UNQUOTED_PROP);
        assert_eq!(diagnostics[0].line, 5);
        assert_eq!(diagnostics[0].end_line, 5);
    }

    #[test]
    fn unterminated_frontmatter_has_no_mdc_body() {
        let source = "---\n<Alert tone=info></Alert>\n";
        assert!(check_document(source).is_empty(), "{:?}", check_document(source));
        assert_eq!(markdown_body_after_frontmatter(source).0, "");
    }
}
