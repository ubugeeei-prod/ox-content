//! Client-side search runtime module generation.

const SEARCH_RUNTIME_MODULE: &str = include_str!("search-runtime.js");

/// Generates the client-side search runtime module.
pub fn generate_search_module(options_json: &str, index_path: &str) -> String {
    let index_path_json =
        serde_json::to_string(index_path).unwrap_or_else(|_| "\"/search-index.json\"".to_string());

    SEARCH_RUNTIME_MODULE
        .replace("__OX_CONTENT_SEARCH_OPTIONS__", options_json)
        .replace("__OX_CONTENT_SEARCH_INDEX_PATH__", &index_path_json)
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
}
