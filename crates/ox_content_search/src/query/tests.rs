use super::*;
use crate::index::SearchIndexBuilder;
use crate::{SearchDocument, parse_search_query};

#[test]
fn test_search_basic() {
    let mut builder = SearchIndexBuilder::new();
    builder.add_simple(
        "1",
        "Getting Started",
        "/getting-started",
        "Welcome to the documentation. This guide will help you get started quickly.",
    );
    builder.add_simple(
        "2",
        "Installation Guide",
        "/installation",
        "Learn how to install the package on your system.",
    );
    builder.add_simple("3", "API Reference", "/api", "Complete API documentation for developers.");

    let index = builder.build();
    let options = SearchOptions::default();

    let results = index.search("getting started", &options);
    assert!(!results.is_empty());
    assert_eq!(results[0].id, "1");

    let results = index.search("install", &options);
    assert!(!results.is_empty());
    assert_eq!(results[0].id, "2");
}

#[test]
fn test_search_prefix() {
    let mut builder = SearchIndexBuilder::new();
    builder.add_simple("1", "Documentation", "/docs", "Complete documentation.");

    let index = builder.build();
    let options = SearchOptions { prefix: true, ..Default::default() };

    let results = index.search("doc", &options);
    assert!(!results.is_empty());
}

#[test]
fn test_search_fuzzy_typo() {
    let mut builder = SearchIndexBuilder::new();
    builder.add_simple("1", "Installation", "/installation", "Install the package.");
    builder.add_simple("2", "Reference", "/reference", "Render Markdown quickly.");

    let index = builder.build();
    let options = SearchOptions { fuzzy: true, prefix: false, ..Default::default() };

    let results = index.search("isntall", &options);
    assert!(!results.is_empty());
    assert_eq!(results[0].id, "1");
    assert!(results[0].score > 0.0);
}

#[test]
fn test_search_fuzzy_stays_opt_in() {
    let mut builder = SearchIndexBuilder::new();
    builder.add_simple("1", "Installation", "/installation", "Install the package.");

    let index = builder.build();
    let options = SearchOptions { prefix: false, ..Default::default() };

    assert!(index.search("isntall", &options).is_empty());
}

#[test]
fn test_parse_rich_search_query() {
    let parsed = parse_search_query(r#"@api lang:ja version:2.90 "static index" render* cli"#);

    assert_eq!(parsed.text, "static index render cli");
    assert_eq!(parsed.terms, vec!["cli"]);
    assert_eq!(parsed.phrases, vec!["static index"]);
    assert_eq!(parsed.prefixes, vec!["render"]);
    assert_eq!(parsed.scopes, vec!["api"]);
    assert_eq!(parsed.first_filter_value(&["locale"]), Some("ja"));
    assert_eq!(parsed.first_filter_value(&["version"]), Some("2.90"));
}

#[test]
fn test_search_rich_query_ranking_metadata() {
    let mut builder = SearchIndexBuilder::new();
    builder.add_document(SearchDocument {
        id: "2.90/ja/api/search".to_string(),
        title: "Query Grammar".to_string(),
        url: "/2.90/ja/api/search".to_string(),
        body: "The static index includes render pipelines for the CLI.".to_string(),
        headings: vec!["Search API".to_string()],
        code: vec!["render_page".to_string()],
    });
    builder.add_document(SearchDocument {
        id: "2.90/ja/api/legacy".to_string(),
        title: "Legacy Search".to_string(),
        url: "/2.90/ja/api/legacy".to_string(),
        body: "A static page may mention an index and renderer separately.".to_string(),
        headings: vec!["Search API".to_string()],
        code: Vec::new(),
    });
    builder.add_document(SearchDocument {
        id: "guide/search".to_string(),
        title: "English Search".to_string(),
        url: "/guide/search".to_string(),
        body: "The static index includes render pipelines.".to_string(),
        headings: vec!["Guide".to_string()],
        code: Vec::new(),
    });

    let index = builder.build();
    let results = index.search(
        r#"@2.90/ja/api lang:ja version:2.90 "static index" render*"#,
        &SearchOptions { prefix: false, ..Default::default() },
    );

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].id, "2.90/ja/api/search");
    assert_eq!(results[0].metadata.scopes, vec!["2.90", "2.90/ja", "2.90/ja/api"]);
    assert_eq!(results[0].metadata.section.as_deref(), Some("Search API"));
    assert_eq!(results[0].metadata.locale.as_deref(), Some("ja"));
    assert_eq!(results[0].metadata.version.as_deref(), Some("2.90"));
    assert!(results[0].matches.iter().any(|value| value == "static index"));
    assert!(results[0].ranking.fields.iter().any(|field| field == "body"));
    assert!(
        results[0].ranking.reasons.iter().any(|reason| reason == "body phrase match: static index")
    );
    assert!(results[0].ranking.reasons.iter().any(|reason| reason == "scope filter: 2.90/ja/api"));
    assert!(results[0].aria_label.contains("Query Grammar"));
    assert!(results[0].aria_label.contains("language ja"));

    let serialized = serde_json::to_value(&results[0]).unwrap();
    assert!(serialized.get("ariaLabel").is_some());
    assert!(serialized.get("aria_label").is_none());
}

#[test]
fn test_search_filter_only_queries_return_matching_documents() {
    let mut builder = SearchIndexBuilder::new();
    builder.add_simple("2.90/api/search", "Versioned", "/2.90/api/search", "Search content.");
    builder.add_simple("api/search", "Current", "/api/search", "Search content.");

    let index = builder.build();
    let results = index.search("version:2.90", &SearchOptions::default());

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "2.90/api/search");
    assert_eq!(results[0].metadata.version.as_deref(), Some("2.90"));
    assert!(results[0].ranking.reasons.iter().any(|reason| reason == "version filter: 2.90"));
}

#[test]
fn test_search_empty() {
    let index = SearchIndexBuilder::new().build();
    let options = SearchOptions::default();

    let results = index.search("test", &options);
    assert!(results.is_empty());
}

#[test]
fn test_search_limit() {
    let mut builder = SearchIndexBuilder::new();
    for i in 0..20 {
        builder.add_simple(
            &format!("{i}"),
            &format!("Test {i}"),
            &format!("/test-{i}"),
            "test content",
        );
    }

    let index = builder.build();
    let options = SearchOptions { limit: 5, ..Default::default() };

    let results = index.search("test", &options);
    assert_eq!(results.len(), 5);
}

#[test]
fn test_search_limit_generates_returned_snippets_only() {
    let mut builder = SearchIndexBuilder::new();
    builder.add_simple("1", "Best match", "/best", "test ".repeat(160).trim_end());
    builder.add_simple("2", "Lower match", "/lower", "test content");

    let index = builder.build();
    let options = SearchOptions { limit: 1, ..Default::default() };

    let results = index.search("test", &options);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "1");
    insta::assert_snapshot!(results[0].snippet);
}

#[test]
fn test_search_snippet_handles_multibyte_boundaries() {
    let mut builder = SearchIndexBuilder::new();
    builder.add_simple(
        "jp",
        "日本語検索",
        "/jp",
        "前置きの文章です。Rustで検索エンジンを作ります。追加の説明です。",
    );

    let index = builder.build();
    let options = SearchOptions { limit: 1, ..Default::default() };

    let results = index.search("検索", &options);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "jp");
    insta::assert_snapshot!(results[0].snippet);
}

#[test]
fn test_generate_snippet_keeps_word_boundary_character() {
    let index = SearchIndexBuilder::new().build();

    let snippet =
        index.generate_snippet("alpha beta gamma delta epsilon", &[String::from("delta")], 12);

    insta::assert_snapshot!(snippet);
}

#[test]
fn test_generate_snippet_skips_boundary_whitespace() {
    let index = SearchIndexBuilder::new().build();

    let snippet =
        index.generate_snippet("alpha beta gamma delta epsilon", &[String::from("gamma")], 18);

    assert!(snippet.starts_with("...beta"));
}

#[test]
fn test_generate_snippet_respects_zero_length() {
    let index = SearchIndexBuilder::new().build();

    assert_eq!(index.generate_snippet("content", &[String::from("content")], 0), "");
}
