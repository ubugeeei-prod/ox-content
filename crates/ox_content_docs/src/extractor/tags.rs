use oxc_ast::ast::BindingPattern;
use oxc_span::GetSpan;

use super::model::{DocTag, ParsedParamTag};
use super::DocVisitor;

impl<'a> DocVisitor<'a> {
    fn has_private_tag(tags: &[DocTag]) -> bool {
        tags.iter().any(|tag| tag.tag == "private")
    }

    fn has_internal_tag(tags: &[DocTag]) -> bool {
        tags.iter().any(|tag| tag.tag == "internal")
    }

    pub(super) fn should_skip_by_visibility(&self, tags: &[DocTag]) -> bool {
        (!self.include_private && Self::has_private_tag(tags))
            || (!self.include_internal && Self::has_internal_tag(tags))
    }

    /// Folds a TypeScript `private` accessibility modifier into the tag list,
    /// so class members declared `private` flow through the same visibility
    /// filtering (and `private` output flag) as members tagged `@private`.
    pub(super) fn apply_ts_private_accessibility(
        accessibility: Option<oxc_ast::ast::TSAccessibility>,
        tags: &mut Vec<DocTag>,
    ) {
        if matches!(accessibility, Some(oxc_ast::ast::TSAccessibility::Private))
            && !Self::has_private_tag(tags)
        {
            tags.push(DocTag::new("private".to_string(), String::new()));
        }
    }

    fn split_leading_jsdoc_type(value: &str) -> (Option<String>, &str) {
        let value = value.trim_start();
        let Some(rest) = value.strip_prefix('{') else {
            return (None, value);
        };

        let mut depth = 1_u32;
        for (index, ch) in rest.char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        let type_annotation = rest[..index].trim();
                        let remaining = rest[index + ch.len_utf8()..].trim_start();
                        return (
                            (!type_annotation.is_empty()).then(|| type_annotation.to_string()),
                            remaining,
                        );
                    }
                }
                _ => {}
            }
        }

        (None, value)
    }

    pub(super) fn clean_tag_description(value: &str) -> Option<String> {
        let value = value.trim();
        let value = value.strip_prefix('-').map_or(value, str::trim_start).trim();
        (!value.is_empty()).then(|| value.to_string())
    }

    fn split_name_and_description(value: &str) -> (&str, &str) {
        let value = value.trim_start();
        if let Some(rest) = value.strip_prefix('[') {
            if let Some(close_index) = rest.find(']') {
                let close_index = close_index + 2;
                return (&value[..close_index], value[close_index..].trim_start());
            }
        }

        value
            .char_indices()
            .find_map(|(index, ch)| {
                ch.is_whitespace()
                    .then_some((&value[..index], value[index + ch.len_utf8()..].trim_start()))
            })
            .unwrap_or((value, ""))
    }

    fn parse_param_tag_value(value: &str) -> Option<ParsedParamTag> {
        let (type_annotation, rest) = Self::split_leading_jsdoc_type(value);
        let (name, description) = Self::split_name_and_description(rest);
        let mut name = name.trim().to_string();
        if name.is_empty() {
            return None;
        }

        let mut optional = false;
        let mut default_value = None;
        if name.starts_with('[') && name.ends_with(']') {
            optional = true;
            let inner = name[1..name.len() - 1].to_string();
            if let Some((inner_name, inner_default)) = inner.split_once('=') {
                name = inner_name.trim().to_string();
                let inner_default = inner_default.trim();
                if !inner_default.is_empty() {
                    default_value = Some(inner_default.to_string());
                }
            } else {
                name = inner.trim().to_string();
            }
        }

        (!name.is_empty()).then(|| ParsedParamTag {
            name,
            type_annotation,
            optional,
            default_value,
            description: Self::clean_tag_description(description),
        })
    }

    pub(super) fn parse_param_tag(tag: &DocTag) -> Option<ParsedParamTag> {
        if tag.name.is_none()
            && tag.type_annotation.is_none()
            && tag.default_value.is_none()
            && tag.description.is_none()
        {
            return Self::parse_param_tag_value(&tag.value);
        }

        let name = tag.name.as_ref()?.trim().to_string();
        (!name.is_empty()).then(|| ParsedParamTag {
            name,
            type_annotation: tag.type_annotation.clone(),
            optional: tag.optional.unwrap_or(false),
            default_value: tag.default_value.clone(),
            description: tag.description.clone(),
        })
    }

    /// Find the first pre-parsed `@param` tag matching `name`, using the same
    /// predicate as before (strip a leading `...`, then exact-name or
    /// dotted-prefix match). Operating on already-parsed tags avoids re-parsing
    /// every `@param` for each formal parameter.
    pub(super) fn find_parsed_param_tag_index(
        parsed: &[ParsedParamTag],
        name: &str,
    ) -> Option<usize> {
        parsed.iter().position(|tag| {
            let tag_name = tag.name.trim_start_matches("...");
            tag_name == name || tag_name.split('.').next() == Some(name)
        })
    }

    pub(super) fn find_exact_parsed_param_tag<'t>(
        parsed: &'t [ParsedParamTag],
        name: &str,
    ) -> Option<&'t ParsedParamTag> {
        parsed.iter().find(|tag| tag.name.trim_start_matches("...").trim_end_matches('?') == name)
    }

    pub(super) fn parse_return_tag(tag: &DocTag) -> (Option<String>, Option<String>) {
        if tag.type_annotation.is_some() || tag.description.is_some() {
            return (tag.type_annotation.clone(), tag.description.clone());
        }

        let (type_annotation, rest) = Self::split_leading_jsdoc_type(&tag.value);
        (type_annotation, Self::clean_tag_description(rest))
    }

    pub(super) fn binding_pattern_name(pattern: &BindingPattern<'a>) -> String {
        match pattern {
            BindingPattern::BindingIdentifier(id) => id.name.to_string(),
            BindingPattern::AssignmentPattern(assign) => Self::binding_pattern_name(&assign.left),
            BindingPattern::ObjectPattern(_) => "param".to_string(),
            BindingPattern::ArrayPattern(_) => "param".to_string(),
        }
    }

    pub(super) fn binding_pattern_identifier_name(pattern: &BindingPattern<'a>) -> Option<String> {
        match pattern {
            BindingPattern::BindingIdentifier(id) => Some(id.name.to_string()),
            BindingPattern::AssignmentPattern(assign) => {
                Self::binding_pattern_identifier_name(&assign.left)
            }
            BindingPattern::ObjectPattern(_) | BindingPattern::ArrayPattern(_) => None,
        }
    }

    pub(super) fn binding_pattern_is_destructured(pattern: &BindingPattern<'a>) -> bool {
        match pattern {
            BindingPattern::AssignmentPattern(assign) => {
                Self::binding_pattern_is_destructured(&assign.left)
            }
            BindingPattern::ObjectPattern(_) | BindingPattern::ArrayPattern(_) => true,
            BindingPattern::BindingIdentifier(_) => false,
        }
    }

    pub(super) fn top_level_param_tag_name(tag: &ParsedParamTag) -> Option<&str> {
        let raw_name = tag.name.trim();
        if raw_name.starts_with("...") {
            return None;
        }
        let name = raw_name.trim_end_matches('?');
        (!name.is_empty() && !name.contains('.')).then_some(name)
    }

    pub(super) fn find_destructured_param_tag_index(
        parsed: &[ParsedParamTag],
        used_indices: &[usize],
        reserved_names: &[String],
    ) -> Option<usize> {
        parsed.iter().enumerate().find_map(|(index, tag)| {
            if used_indices.contains(&index) {
                return None;
            }
            let name = Self::top_level_param_tag_name(tag)?;
            if reserved_names.iter().any(|reserved| reserved == name) {
                return None;
            }
            Some(index)
        })
    }

    pub(super) fn binding_pattern_default_value(
        &self,
        pattern: &BindingPattern<'a>,
    ) -> Option<String> {
        match pattern {
            BindingPattern::AssignmentPattern(assign) => {
                Some(self.slice(assign.right.span().start, assign.right.span().end))
            }
            _ => None,
        }
    }
}
