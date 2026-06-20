use napi::bindgen_prelude::*;
use napi_derive::napi;
use ox_content_parser::ParserOptions;
use ox_content_search::{SearchDocument, SearchIndex, SearchOptions, SearchRuntimeOptions};

use crate::JsParserOptions;

/// Search document for JavaScript.
#[napi(object)]
#[derive(Clone)]
pub struct JsSearchDocument {
    /// Unique document identifier.
    pub id: String,
    /// Document title.
    pub title: String,
    /// Document URL.
    pub url: String,
    /// Document body text.
    pub body: String,
    /// Document headings.
    pub headings: Vec<String>,
    /// Code snippets.
    pub code: Vec<String>,
}

fn map_search_document(doc: SearchDocument) -> JsSearchDocument {
    JsSearchDocument {
        id: doc.id,
        title: doc.title,
        url: doc.url,
        body: doc.body,
        headings: doc.headings,
        code: doc.code,
    }
}

/// Search result for JavaScript.
#[napi(object)]
pub struct JsSearchResult {
    /// Document ID.
    pub id: String,
    /// Document title.
    pub title: String,
    /// Document URL.
    pub url: String,
    /// Relevance score.
    pub score: f64,
    /// Matched terms.
    pub matches: Vec<String>,
    /// Content snippet.
    pub snippet: String,
}

/// Search query split into free text and scope prefixes.
#[napi(object)]
pub struct JsScopedSearchQuery {
    /// Free-text terms after removing scope prefixes.
    pub text: String,
    /// Deduplicated lowercase scopes.
    pub scopes: Vec<String>,
}

/// Search options for JavaScript.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsSearchOptions {
    /// Maximum number of results.
    pub limit: Option<u32>,
    /// Enable prefix matching.
    pub prefix: Option<bool>,
    /// Enable fuzzy matching.
    pub fuzzy: Option<bool>,
    /// Minimum score threshold.
    pub threshold: Option<f64>,
}

impl From<JsSearchOptions> for SearchOptions {
    fn from(opts: JsSearchOptions) -> Self {
        Self {
            limit: opts.limit.unwrap_or(10) as usize,
            prefix: opts.prefix.unwrap_or(true),
            fuzzy: opts.fuzzy.unwrap_or(false),
            threshold: opts.threshold.unwrap_or(0.0),
        }
    }
}

/// Resolved search runtime options for JavaScript.
#[napi(object)]
pub struct JsSearchRuntimeOptions {
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

impl From<JsSearchRuntimeOptions> for SearchRuntimeOptions {
    fn from(options: JsSearchRuntimeOptions) -> Self {
        Self {
            enabled: options.enabled,
            limit: options.limit,
            prefix: options.prefix,
            placeholder: options.placeholder,
            hotkey: options.hotkey,
        }
    }
}

/// Builds a search index from documents.
///
/// Takes an array of documents and returns a serialized search index as JSON.
#[napi]
pub fn build_search_index(documents: Vec<JsSearchDocument>) -> String {
    ox_content_search::build_search_index_json(documents.into_iter().map(|doc| SearchDocument {
        id: doc.id,
        title: doc.title,
        url: doc.url,
        body: doc.body,
        headings: doc.headings,
        code: doc.code,
    }))
}

/// Builds a search index directly from Markdown files under a source directory.
///
/// File discovery, Markdown parsing, search document extraction, and index
/// construction all run on the Rust side.
#[napi(js_name = "buildSearchIndexFromDirectory")]
pub fn build_search_index_from_directory(
    src_dir: String,
    base: String,
    extensions: Vec<String>,
) -> String {
    ox_content_search::build_search_index_from_directory(&src_dir, &base, &extensions)
}

/// Searches a serialized index.
///
/// Takes a JSON-serialized index, query string, and options.
/// Returns an array of search results.
#[napi]
pub fn search_index(
    index_json: String,
    query: String,
    options: Option<JsSearchOptions>,
) -> Vec<JsSearchResult> {
    let Ok(index) = SearchIndex::from_json(&index_json) else {
        return Vec::new();
    };

    let opts = options.map(SearchOptions::from).unwrap_or_default();
    let results = index.search(&query, &opts);

    results
        .into_iter()
        .map(|r| JsSearchResult {
            id: r.id,
            title: r.title,
            url: r.url,
            score: r.score,
            matches: r.matches,
            snippet: r.snippet,
        })
        .collect()
}

/// Splits a search query into free-text terms and `@scope` prefixes.
#[napi(js_name = "parseScopedSearchQuery")]
pub fn parse_scoped_search_query(query: String) -> JsScopedSearchQuery {
    let parsed = ox_content_search::parse_scoped_search_query(&query);
    JsScopedSearchQuery { text: parsed.text, scopes: parsed.scopes }
}

/// Derives hierarchical search scopes from a document id or URL.
#[napi(js_name = "getSearchDocumentScopes")]
pub fn get_search_document_scopes(id: String, url: String) -> Vec<String> {
    ox_content_search::get_search_document_scopes(&id, &url)
}

/// Returns true when a document belongs to at least one requested search scope.
#[napi(js_name = "matchesSearchScopes")]
pub fn matches_search_scopes(id: String, url: String, scopes: Vec<String>) -> bool {
    ox_content_search::matches_search_scopes(&id, &url, &scopes)
}

/// Generates the client-side search runtime module.
#[napi(js_name = "generateSearchModule")]
pub fn generate_search_module(options_json: String, index_path: String) -> String {
    ox_content_search::generate_search_module(&options_json, &index_path)
}

/// Generates the client-side search runtime module from typed options.
#[napi(js_name = "generateSearchModuleFromOptions")]
pub fn generate_search_module_from_options(
    options: JsSearchRuntimeOptions,
    index_path: String,
) -> String {
    ox_content_search::generate_search_module_with_options(&options.into(), &index_path)
}

/// Collects Markdown files for search indexing from a source directory.
#[napi(js_name = "collectSearchMarkdownFiles")]
pub fn collect_search_markdown_files(src_dir: String, extensions: Vec<String>) -> Vec<String> {
    ox_content_search::collect_markdown_files(&src_dir, &extensions)
}

/// Writes a serialized search index to `search-index.json` under an output directory.
#[napi(js_name = "writeSearchIndex")]
pub fn write_search_index(index_json: String, out_dir: String) -> Result<()> {
    ox_content_search::write_search_index(&index_json, &out_dir)
        .map_err(|err| Error::from_reason(format!("failed to write search index: {err}")))?;
    Ok(())
}

// =============================================================================
// SSG HTML Generation API
// =============================================================================

/// Extracts searchable content from Markdown source.
///
/// Parses the Markdown and extracts title, body text, headings, and code.
#[napi]
pub fn extract_search_content(
    source: String,
    id: String,
    url: String,
    options: Option<JsParserOptions>,
) -> JsSearchDocument {
    let parser_options = options.map(ParserOptions::from).unwrap_or_default();
    map_search_document(ox_content_search::extract_search_document_from_source(
        &source,
        id,
        url,
        parser_options,
    ))
}
