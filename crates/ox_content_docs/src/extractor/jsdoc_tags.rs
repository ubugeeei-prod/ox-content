use ox_jsdoc::decoder::nodes::comment_ast::{LazyJsdocTag, LazyJsdocTagBody};

use crate::string_builder::{join2, join3, join5};

use super::{model::DocTag, DocVisitor};

impl<'a> DocVisitor<'a> {
    pub(super) fn convert_jsdoc_tag(tag: LazyJsdocTag<'_>, capture_value: bool) -> DocTag {
        let tag_name = tag.tag().value().to_string();
        // `value` is the formatted tag body that the raw `extract_file_docs`
        // path surfaces. Normalize ignores it for param-family tags (it reads
        // the structured name/type/description instead), so skip the costly
        // `format_jsdoc_tag_value` join when the caller will not read it.
        let value = if capture_value || !matches!(tag_name.as_str(), "param" | "arg" | "argument") {
            Self::format_jsdoc_tag_value(&tag_name, &tag)
        } else {
            String::new()
        };
        let type_annotation = tag
            .raw_type()
            .map(|raw_type| raw_type.raw().trim().to_string())
            .filter(|value| !value.is_empty());
        let name =
            tag.name().map(|name| name.raw().trim().to_string()).filter(|value| !value.is_empty());
        let optional = name.as_ref().map(|_| tag.optional());
        let default_value = tag.default_value().map(str::trim).filter(|value| !value.is_empty());
        let description =
            Self::format_structured_tag_description(&tag_name, &tag, type_annotation.as_deref());

        DocTag {
            tag: tag_name,
            value,
            type_annotation,
            name,
            optional,
            default_value: default_value.map(str::to_string),
            description,
        }
    }

    fn format_structured_tag_description(
        tag_name: &str,
        tag: &LazyJsdocTag<'_>,
        type_annotation: Option<&str>,
    ) -> Option<String> {
        if matches!(tag_name, "returns" | "return") {
            let raw_body = tag.raw_body().map(str::trim).filter(|value| !value.is_empty())?;
            let without_type = type_annotation
                .and_then(|type_annotation| {
                    let type_prefix = join3("{", type_annotation, "}");
                    raw_body.strip_prefix(&type_prefix).map(str::trim_start)
                })
                .unwrap_or(raw_body);
            return Self::clean_tag_description(without_type);
        }

        tag.description().and_then(Self::clean_tag_description)
    }

    fn format_jsdoc_tag_value(tag_name: &str, tag: &LazyJsdocTag<'_>) -> String {
        if !matches!(tag_name, "param" | "arg" | "argument") {
            if let Some(raw_body) = tag.raw_body().map(str::trim).filter(|value| !value.is_empty())
            {
                return raw_body.to_string();
            }
        }

        let mut parts = Vec::new();

        if let Some(raw_type) = tag.raw_type() {
            let raw_type = raw_type.raw().trim();
            if !raw_type.is_empty() {
                parts.push(join3("{", raw_type, "}"));
            }
        }

        if let Some(name) = tag.name() {
            let name = name.raw().trim();
            if !name.is_empty() {
                let name = if tag.optional() {
                    tag.default_value().map_or_else(
                        || join3("[", name, "]"),
                        |default_value| join5("[", name, "=", default_value, "]"),
                    )
                } else {
                    name.to_string()
                };
                parts.push(name);
            }
        }

        if let Some(description) =
            tag.description().map(str::trim).filter(|value| !value.is_empty())
        {
            if parts.is_empty() {
                parts.push(description.to_string());
            } else {
                parts.push(join2("- ", description));
            }
        }

        if !parts.is_empty() {
            return parts.join(" ");
        }

        if let Some(raw_body) = tag.raw_body().map(str::trim).filter(|value| !value.is_empty()) {
            return raw_body.to_string();
        }

        tag.body().map_or_else(String::new, |body| match body {
            LazyJsdocTagBody::Generic(body) => body.description().unwrap_or_default().to_string(),
            LazyJsdocTagBody::Raw(body) => body.raw().to_string(),
            LazyJsdocTagBody::Borrows(_) => String::new(),
        })
    }

    /// Fallback parser used when the external JSDoc parser cannot produce a root.
    pub(super) fn parse_jsdoc_fallback(comment: &str) -> (String, Vec<DocTag>) {
        let mut description_lines = Vec::new();
        let mut tags = Vec::new();
        let mut current_tag: Option<(String, Vec<String>)> = None;

        let lines: Vec<String> = comment
            .lines()
            .map(|line| {
                let trimmed = line.trim_start();
                let trimmed = trimmed.strip_prefix('*').unwrap_or(trimmed);
                trimmed.strip_prefix(' ').unwrap_or(trimmed).trim_end().to_string()
            })
            .collect();

        for line in lines {
            let trimmed = line.trim_start();
            if let Some(without_at) = trimmed.strip_prefix('@') {
                if let Some((tag, value_lines)) = current_tag.take() {
                    tags.push(DocTag::new(tag, value_lines.join("\n").trim().to_string()));
                }

                let split_at = without_at
                    .char_indices()
                    .find_map(|(index, ch)| ch.is_whitespace().then_some(index))
                    .unwrap_or(without_at.len());
                let tag_name = without_at[..split_at].to_string();
                let tag_value = without_at[split_at..].trim_start().to_string();
                current_tag = Some((tag_name, Vec::from([tag_value])));
            } else if let Some((_, ref mut value_lines)) = current_tag {
                value_lines.push(line);
            } else {
                description_lines.push(line);
            }
        }

        if let Some((tag, value_lines)) = current_tag {
            tags.push(DocTag::new(tag, value_lines.join("\n").trim().to_string()));
        }

        (description_lines.join("\n").trim().to_string(), tags)
    }
}
