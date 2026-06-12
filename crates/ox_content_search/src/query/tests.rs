use super::*;
use crate::index::SearchIndexBuilder;

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
    assert!(results[0].snippet.contains("test"));
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
    assert!(results[0].snippet.contains("検索"));
}

#[test]
fn test_generate_snippet_keeps_word_boundary_character() {
    let index = SearchIndexBuilder::new().build();

    let snippet =
        index.generate_snippet("alpha beta gamma delta epsilon", &[String::from("delta")], 12);

    assert!(snippet.contains("gamma"));
    assert!(snippet.contains("delta"));
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
