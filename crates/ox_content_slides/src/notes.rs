use crate::ExtractedSlideComments;

/// Extracts HTML comment notes from a slide source.
pub fn extract_slide_comments(source: &str) -> ExtractedSlideComments {
    let mut notes = Vec::new();
    let mut content = String::with_capacity(source.len());
    let mut cursor = 0;

    while let Some(start_rel) = source[cursor..].find("<!--") {
        let start = cursor + start_rel;
        content.push_str(&source[cursor..start]);

        let comment_start = start + 4;
        if let Some(end_rel) = source[comment_start..].find("-->") {
            let end = comment_start + end_rel;
            let note = normalize_comment(&source[comment_start..end]);
            if !note.is_empty() {
                notes.push(note);
            }
            cursor = end + 3;
        } else {
            content.push_str(&source[start..]);
            cursor = source.len();
            break;
        }
    }

    if cursor < source.len() {
        content.push_str(&source[cursor..]);
    }

    ExtractedSlideComments { content, notes }
}

fn normalize_comment(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    if let Some(stripped) = trimmed
        .strip_prefix("note:")
        .or_else(|| trimmed.strip_prefix("notes:"))
        .or_else(|| trimmed.strip_prefix("Note:"))
        .or_else(|| trimmed.strip_prefix("Notes:"))
    {
        return stripped.trim().to_string();
    }

    trimmed.to_string()
}

#[cfg(test)]
mod tests {
    use super::extract_slide_comments;

    #[test]
    fn extracts_html_comment_notes() {
        let source = "<h1>Hello</h1><!-- notes: talk track --><p>World</p>";
        let extracted = extract_slide_comments(source);
        assert_eq!(extracted.notes, vec!["talk track".to_string()]);
        assert!(!extracted.content.contains("notes:"));
    }
}
