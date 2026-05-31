use serde_json::{Map as JsonMap, Value as JsonValue};

use crate::{extract_slide_comments, ParsedSlideDeck, SlideSource};

/// Parses a Markdown deck with optional frontmatter and slide separators.
pub fn parse_markdown_slide_deck(source: &str, separator: &str) -> ParsedSlideDeck {
    let (content, frontmatter) = parse_frontmatter(source);
    let slides = split_slides(&content, separator)
        .into_iter()
        .map(|slide| {
            let extracted = extract_slide_comments(&slide);
            SlideSource { content: extracted.content.trim().to_string(), notes: extracted.notes }
        })
        .filter(|slide| !slide.content.is_empty())
        .collect();

    ParsedSlideDeck { frontmatter, slides }
}

fn parse_frontmatter(source: &str) -> (String, JsonValue) {
    if !source.starts_with("---") {
        return (source.to_string(), JsonValue::Object(JsonMap::new()));
    }

    let rest = &source[3..];
    let Some(end_index) = rest.find("\n---") else {
        return (source.to_string(), JsonValue::Object(JsonMap::new()));
    };

    let frontmatter_source = rest[..end_index].trim();
    let content = rest[end_index + 4..].trim_start().to_string();
    let frontmatter = serde_yaml::from_str::<serde_yaml::Value>(frontmatter_source)
        .ok()
        .and_then(|value| serde_json::to_value(value).ok())
        .unwrap_or_else(|| JsonValue::Object(JsonMap::new()));

    (content, frontmatter)
}

fn split_slides(content: &str, separator: &str) -> Vec<String> {
    let mut slides = Vec::new();
    let mut buffer = String::new();

    for line in content.lines() {
        if line.trim() == separator {
            slides.push(std::mem::take(&mut buffer));
            continue;
        }

        if !buffer.is_empty() {
            buffer.push('\n');
        }
        buffer.push_str(line);
    }

    if !buffer.is_empty() || slides.is_empty() {
        slides.push(buffer);
    }

    slides
}

#[cfg(test)]
mod tests {
    use serde_json::Value as JsonValue;

    use super::parse_markdown_slide_deck;

    #[test]
    fn parses_markdown_deck_frontmatter_and_splits_slides() {
        let source = r"---
title: Deck Title
description: Demo
---

# One

<!-- notes: first note -->

---

# Two
";

        let result = parse_markdown_slide_deck(source, "---");
        assert_eq!(result.frontmatter["title"], JsonValue::String("Deck Title".to_string()));
        assert_eq!(result.slides.len(), 2);
        assert_eq!(result.slides[0].notes, vec!["first note".to_string()]);
        assert!(result.slides[0].content.contains("# One"));
    }
}
