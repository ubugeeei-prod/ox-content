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

/// Splits a raw query into free-text terms and `@scope` prefixes.
pub fn parse_scoped_search_query(query: &str) -> ScopedSearchQuery {
    let mut scopes = Vec::new();
    let mut terms = Vec::new();

    for part in query.split_whitespace() {
        if let Some(scope) = part.strip_prefix('@').filter(|scope| !scope.is_empty()) {
            let scope = scope.to_lowercase();
            if !scopes.contains(&scope) {
                scopes.push(scope);
            }
        } else {
            terms.push(part);
        }
    }

    ScopedSearchQuery { text: terms.join(" ").trim().to_string(), scopes }
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
