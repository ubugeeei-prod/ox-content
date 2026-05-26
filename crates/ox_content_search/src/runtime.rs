//! Client-side search runtime module generation.

use serde::Serialize;

const SEARCH_RUNTIME_MODULE: &str = include_str!("search-runtime.js");

/// Resolved options embedded in the client-side search runtime.
#[derive(Clone, Debug, Serialize)]
pub struct SearchRuntimeOptions {
    /// Whether search is enabled.
    pub enabled: bool,
    /// Maximum number of results.
    pub limit: u32,
    /// Enable prefix matching.
    pub prefix: bool,
    /// Search input placeholder.
    pub placeholder: String,
    /// Keyboard shortcut to focus search.
    pub hotkey: String,
}

/// Generates the client-side search runtime module.
pub fn generate_search_module(options_json: &str, index_path: &str) -> String {
    let index_path_json =
        serde_json::to_string(index_path).unwrap_or_else(|_| "\"/search-index.json\"".to_string());

    SEARCH_RUNTIME_MODULE
        .replace("__OX_CONTENT_SEARCH_OPTIONS__", options_json)
        .replace("__OX_CONTENT_SEARCH_INDEX_PATH__", &index_path_json)
}

/// Generates the client-side search runtime module from typed options.
pub fn generate_search_module_with_options(
    options: &SearchRuntimeOptions,
    index_path: &str,
) -> String {
    let options_json = serde_json::to_string(options).unwrap_or_else(|_| "{}".to_string());
    generate_search_module(&options_json, index_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn injects_options_and_json_escaped_index_path() {
        let module = generate_search_module(r#"{"limit":5,"prefix":true}"#, "/docs/search's.json");

        assert!(module.contains("const searchOptions = {\"limit\":5,\"prefix\":true};"));
        assert!(module.contains("fetch(\"/docs/search's.json\")"));
        assert!(module.contains("export async function search"));
    }

    #[test]
    fn serializes_typed_options() {
        let module = generate_search_module_with_options(
            &SearchRuntimeOptions {
                enabled: true,
                limit: 12,
                prefix: false,
                placeholder: "Find docs".to_string(),
                hotkey: "k".to_string(),
            },
            "/docs/search-index.json",
        );

        assert!(module.contains(
            r#"const searchOptions = {"enabled":true,"limit":12,"prefix":false,"placeholder":"Find docs","hotkey":"k"};"#
        ));
        assert!(module.contains(r#"fetch("/docs/search-index.json")"#));
    }
}
