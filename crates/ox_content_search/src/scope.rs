//! Scoped search query helpers.

use compact_str::CompactString;
use serde::{Deserialize, Serialize};

/// Parsed free-text query and requested search scopes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopedSearchQuery {
    /// Free-text terms after removing `@scope` prefixes.
    pub text: String,
    /// Deduplicated lowercase scopes.
    pub scopes: Vec<String>,
}

/// A normalized `name:value` query filter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchQueryFilter {
    /// Filter name, normalized to lowercase.
    pub name: String,
    /// Filter value, normalized to lowercase.
    pub value: String,
}

/// Parsed rich search query model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchQuery {
    /// Original query string.
    pub raw: String,
    /// Free text with scopes, filters, and explicit `*` suffixes removed.
    pub text: String,
    /// Normalized free-text terms.
    pub terms: Vec<String>,
    /// Normalized quoted phrases.
    pub phrases: Vec<String>,
    /// Normalized explicit prefix roots from tokens like `render*`.
    pub prefixes: Vec<String>,
    /// Normalized query filters such as `lang:ja` and `version:2.90`.
    pub filters: Vec<SearchQueryFilter>,
    /// Deduplicated lowercase scopes from `@scope` or `scope:value`.
    pub scopes: Vec<String>,
}

impl SearchQuery {
    /// Returns true when the query has no searchable terms or refinements.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
            && self.terms.is_empty()
            && self.phrases.is_empty()
            && self.prefixes.is_empty()
            && self.filters.is_empty()
            && self.scopes.is_empty()
    }

    /// Returns the first value for any of the supplied filter names.
    #[must_use]
    pub fn first_filter_value(&self, names: &[&str]) -> Option<&str> {
        self.filters
            .iter()
            .find(|filter| names.iter().any(|name| filter.name == *name))
            .map(|filter| filter.value.as_str())
    }
}

struct QueryPart {
    value: String,
    quoted: bool,
}

/// Splits a raw query into terms, quoted phrases, prefixes, filters, and scopes.
pub fn parse_search_query(query: &str) -> SearchQuery {
    let mut scopes = Vec::new();
    let mut filters = Vec::new();
    let mut terms = Vec::new();
    let mut phrases = Vec::new();
    let mut prefixes = Vec::new();
    let mut text_parts = Vec::new();

    for part in parse_query_parts(query) {
        if part.quoted {
            let phrase = normalize_search_value(&part.value);
            if !phrase.is_empty() {
                push_unique(&mut phrases, phrase.clone());
                text_parts.push(part.value);
            }
            continue;
        }

        if let Some(scope) = part.value.strip_prefix('@').filter(|scope| !scope.is_empty()) {
            let scope = normalize_filter_value(scope);
            if !scope.is_empty() {
                push_unique(&mut scopes, scope);
            }
            continue;
        }

        if let Some(filter) = parse_query_filter(&part.value) {
            if filter.name == "scope" {
                push_unique(&mut scopes, filter.value.clone());
            }
            if !filters.contains(&filter) {
                filters.push(filter);
            }
            continue;
        }

        if let Some(prefix) = part.value.strip_suffix('*') {
            let prefix = normalize_search_value(prefix);
            if !prefix.is_empty() {
                push_unique(&mut prefixes, prefix);
                text_parts.push(part.value.trim_end_matches('*').to_string());
            }
            continue;
        }

        let term = normalize_search_value(&part.value);
        if !term.is_empty() {
            push_unique(&mut terms, term);
            text_parts.push(part.value);
        }
    }

    SearchQuery {
        raw: query.to_string(),
        text: text_parts.join(" ").trim().to_string(),
        terms,
        phrases,
        prefixes,
        filters,
        scopes,
    }
}

/// Splits a raw query into free-text terms and `@scope` prefixes.
pub fn parse_scoped_search_query(query: &str) -> ScopedSearchQuery {
    let parsed = parse_search_query(query);
    ScopedSearchQuery { text: parsed.text, scopes: parsed.scopes }
}

fn parse_query_parts(query: &str) -> Vec<QueryPart> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    let mut escaping = false;

    for c in query.chars() {
        if quoted {
            if escaping {
                current.push(c);
                escaping = false;
            } else if c == '\\' {
                escaping = true;
            } else if c == '"' {
                push_query_part(&mut parts, &mut current, true);
                quoted = false;
            } else {
                current.push(c);
            }
            continue;
        }

        if c == '"' {
            push_query_part(&mut parts, &mut current, false);
            quoted = true;
        } else if c.is_whitespace() {
            push_query_part(&mut parts, &mut current, false);
        } else {
            current.push(c);
        }
    }

    if escaping {
        current.push('\\');
    }
    push_query_part(&mut parts, &mut current, quoted);

    parts
}

fn push_query_part(parts: &mut Vec<QueryPart>, current: &mut String, quoted: bool) {
    let trimmed = current.trim();
    if !trimmed.is_empty() {
        parts.push(QueryPart { value: trimmed.to_string(), quoted });
    }
    current.clear();
}

fn parse_query_filter(value: &str) -> Option<SearchQueryFilter> {
    let (name, filter_value) = value.split_once(':')?;
    let name = normalize_filter_name(name)?;
    let filter_value = normalize_filter_value(filter_value);
    if filter_value.is_empty() {
        return None;
    }
    Some(SearchQueryFilter { name, value: filter_value })
}

fn normalize_filter_name(name: &str) -> Option<String> {
    let normalized = name.trim().to_lowercase().replace('_', "-");
    if normalized.is_empty() || !normalized.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return None;
    }
    Some(match normalized.as_str() {
        "lang" | "language" | "locale" => "locale".to_string(),
        "v" | "version" => "version".to_string(),
        "section" | "scope" => "scope".to_string(),
        _ => normalized,
    })
}

fn normalize_search_value(value: &str) -> String {
    value.trim().to_lowercase()
}

fn normalize_filter_value(value: &str) -> String {
    value.trim().trim_matches('/').to_lowercase()
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.contains(&value) {
        values.push(value);
    }
}

/// Derives hierarchical search scopes from a document id or URL.
pub fn get_search_document_scopes(id: &str, url: &str) -> Vec<String> {
    let source = if id.is_empty() { url } else { id };
    let source = source.trim_start_matches('/').to_lowercase();
    let segments: Vec<&str> = source.split('/').filter(|segment| !segment.is_empty()).collect();

    if segments.len() <= 1 {
        return Vec::new();
    }

    let mut scopes = Vec::new();
    let mut current = CompactString::default();

    for segment in &segments[..segments.len() - 1] {
        if !current.is_empty() {
            current.push('/');
        }
        current.push_str(segment);
        scopes.push(current.clone().into_string());
    }

    scopes
}

/// Returns true when a search document belongs to at least one requested scope.
pub fn matches_search_scopes(id: &str, url: &str, scopes: &[String]) -> bool {
    if scopes.is_empty() {
        return true;
    }

    let doc_scopes = get_search_document_scopes(id, url);
    scopes.iter().map(|scope| scope.to_lowercase()).any(|scope| doc_scopes.contains(&scope))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_scoped_query() {
        assert_eq!(
            parse_scoped_search_query("@api @api clamp util"),
            ScopedSearchQuery { text: "clamp util".to_string(), scopes: vec!["api".to_string()] }
        );
    }

    #[test]
    fn parses_rich_query_model() {
        assert_eq!(
            parse_search_query(r#"@api lang:ja v:2.90 "Static Index" render* cli"#),
            SearchQuery {
                raw: r#"@api lang:ja v:2.90 "Static Index" render* cli"#.to_string(),
                text: "Static Index render cli".to_string(),
                terms: vec!["cli".to_string()],
                phrases: vec!["static index".to_string()],
                prefixes: vec!["render".to_string()],
                filters: vec![
                    SearchQueryFilter { name: "locale".to_string(), value: "ja".to_string() },
                    SearchQueryFilter { name: "version".to_string(), value: "2.90".to_string() },
                ],
                scopes: vec!["api".to_string()],
            }
        );
    }

    #[test]
    fn parses_scope_filter_as_scope_refinement() {
        let parsed = parse_search_query("scope:guide section:api setup");

        assert_eq!(parsed.terms, vec!["setup".to_string()]);
        assert_eq!(parsed.scopes, vec!["guide".to_string(), "api".to_string()]);
        assert_eq!(
            parsed.filters,
            vec![
                SearchQueryFilter { name: "scope".to_string(), value: "guide".to_string() },
                SearchQueryFilter { name: "scope".to_string(), value: "api".to_string() },
            ]
        );
    }

    #[test]
    fn derives_cumulative_document_scopes() {
        assert_eq!(
            get_search_document_scopes("api/math/index", "/api/math/index"),
            vec!["api".to_string(), "api/math".to_string()]
        );
    }

    #[test]
    fn matches_requested_scopes() {
        assert!(matches_search_scopes("api/utils", "/api/utils", &["api".to_string()]));
        assert!(!matches_search_scopes("api/utils", "/api/utils", &["api/utils".to_string()]));
        assert!(!matches_search_scopes("api/utils", "/api/utils", &["guides".to_string()]));
    }
}
