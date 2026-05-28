//! Search query engine with BM25 scoring.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::index::SearchIndex;
use crate::tokenizer::tokenize_query;

/// Search options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOptions {
    /// Maximum number of results to return.
    #[serde(default = "default_limit")]
    pub limit: usize,
    /// Enable prefix matching for the last token.
    #[serde(default = "default_prefix")]
    pub prefix: bool,
    /// Enable fuzzy matching (edit distance).
    #[serde(default)]
    pub fuzzy: bool,
    /// Minimum score threshold (0.0 - 1.0).
    #[serde(default)]
    pub threshold: f64,
}

fn default_limit() -> usize {
    10
}

fn default_prefix() -> bool {
    true
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self { limit: 10, prefix: true, fuzzy: false, threshold: 0.0 }
    }
}

/// A search result with relevance score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
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
    /// Content snippet with highlights.
    pub snippet: String,
}

/// BM25 parameters.
const K1: f64 = 1.2;
const B: f64 = 0.75;
const MIN_PREFIX_MATCH_LEN: usize = 2;
const SNIPPET_CONTEXT_DIVISOR: usize = 3;
const SNIPPET_MAX_CHARS: usize = 150;
const SNIPPET_ELLIPSIS: &str = "...";

impl SearchIndex {
    /// Searches the index with the given query.
    #[must_use]
    pub fn search(&self, query: &str, options: &SearchOptions) -> Vec<SearchResult> {
        if query.is_empty() || self.is_empty() {
            return Vec::new();
        }

        let tokens = tokenize_query(query);
        if tokens.is_empty() {
            return Vec::new();
        }

        // Calculate scores for each document
        let mut doc_scores: HashMap<usize, (f64, Vec<String>)> = HashMap::new();

        for (i, token) in tokens.iter().enumerate() {
            let is_last = i == tokens.len() - 1;

            if is_last && options.prefix && token.len() >= MIN_PREFIX_MATCH_LEN {
                for term in self.index.keys().filter(|term| term.starts_with(token)) {
                    self.score_matching_term(term, &mut doc_scores);
                }
            } else {
                self.score_matching_term(token, &mut doc_scores);
            }
        }

        // Sort and limit candidates before constructing result payloads. Snippet generation
        // scans document bodies, so doing it only for returned results avoids wasted work.
        let mut ranked_docs: Vec<_> = doc_scores
            .into_iter()
            .filter(|(_, (score, _))| *score >= options.threshold)
            .map(|(doc_idx, (score, matches))| (doc_idx, score, matches))
            .collect();

        ranked_docs.sort_by(|(left_idx, left_score, _), (right_idx, right_score, _)| {
            right_score
                .partial_cmp(left_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| left_idx.cmp(right_idx))
        });
        ranked_docs.truncate(options.limit);

        ranked_docs
            .into_iter()
            .map(|(doc_idx, score, matches)| {
                let doc = &self.documents[doc_idx];
                let snippet = self.generate_snippet(&doc.body, &matches, SNIPPET_MAX_CHARS);
                SearchResult {
                    id: doc.id.clone(),
                    title: doc.title.clone(),
                    url: doc.url.clone(),
                    score,
                    matches,
                    snippet,
                }
            })
            .collect()
    }

    /// Computes IDF (Inverse Document Frequency).
    #[allow(clippy::cast_precision_loss)]
    fn compute_idf(&self, df: usize) -> f64 {
        let n = self.doc_count as f64;
        let df = df as f64;
        ((n - df + 0.5) / (df + 0.5)).ln_1p()
    }

    fn score_matching_term(&self, term: &str, doc_scores: &mut HashMap<usize, (f64, Vec<String>)>) {
        let Some(postings) = self.index.get(term) else {
            return;
        };

        let df = self.df.get(term).copied().unwrap_or(1);
        let idf = self.compute_idf(df);

        for posting in postings {
            let doc = &self.documents[posting.doc_idx];
            #[allow(clippy::cast_precision_loss)]
            let doc_len = doc.body.len() as f64;
            let tf = f64::from(posting.tf);

            // BM25 score with field boost
            let score = idf
                * ((tf * (K1 + 1.0)) / K1.mul_add(1.0 - B + B * doc_len / self.avg_dl, tf))
                * posting.field.boost();

            let entry = doc_scores.entry(posting.doc_idx).or_insert((0.0, Vec::new()));
            entry.0 += score;
            if !entry.1.iter().any(|matched| matched == term) {
                entry.1.push(term.to_owned());
            }
        }
    }

    /// Generates a snippet of text around matched terms.
    #[allow(clippy::unused_self)]
    fn generate_snippet(&self, body: &str, matches: &[String], max_len: usize) -> String {
        if body.is_empty() || max_len == 0 {
            return String::new();
        }

        let body_lower = body.to_lowercase();

        // Find the first match position
        let mut first_match_pos: Option<usize> = None;
        for term in matches {
            if let Some(pos) = body_lower.find(term) {
                first_match_pos = Some(first_match_pos.map_or(pos, |current| current.min(pos)));
            }
        }

        let start_pos = previous_char_boundary(body, first_match_pos.unwrap_or(0));
        let start_char = body[..start_pos.min(body.len())].chars().count();

        // Find a good start position (at word boundary, before match)
        let context_before = max_len / SNIPPET_CONTEXT_DIVISOR;
        let mut snippet_start =
            byte_index_for_char(body, start_char.saturating_sub(context_before));
        snippet_start = previous_word_start_byte(body, snippet_start);

        // Calculate end position
        let snippet_end = byte_index_after_chars(body, snippet_start, max_len);

        // Build snippet
        let needs_prefix = snippet_start > 0;
        let needs_suffix = snippet_end < body.len();
        let mut snippet = String::with_capacity(
            snippet_end.saturating_sub(snippet_start)
                + usize::from(needs_prefix) * SNIPPET_ELLIPSIS.len()
                + usize::from(needs_suffix) * SNIPPET_ELLIPSIS.len(),
        );

        // Add ellipsis if needed
        if needs_prefix {
            snippet.push_str(SNIPPET_ELLIPSIS);
        }
        snippet.push_str(&body[snippet_start..snippet_end]);
        if needs_suffix {
            snippet.push_str(SNIPPET_ELLIPSIS);
        }

        snippet
    }
}

fn byte_index_for_char(body: &str, target_char: usize) -> usize {
    if target_char == 0 {
        return 0;
    }

    body.char_indices().nth(target_char).map_or(body.len(), |(byte, _)| byte)
}

fn previous_char_boundary(body: &str, byte_index: usize) -> usize {
    let mut byte_index = byte_index.min(body.len());
    while !body.is_char_boundary(byte_index) {
        byte_index -= 1;
    }
    byte_index
}

fn previous_word_start_byte(body: &str, start_byte: usize) -> usize {
    let mut start_byte = previous_char_boundary(body, start_byte);

    while start_byte < body.len() {
        let Some(current_char) = body[start_byte..].chars().next() else {
            return start_byte;
        };

        if current_char.is_whitespace() {
            return start_byte + current_char.len_utf8();
        }

        if start_byte == 0 {
            return 0;
        }

        let Some((prev_byte, _)) = body[..start_byte].char_indices().next_back() else {
            return 0;
        };
        start_byte = prev_byte;
    }

    body.len()
}

fn byte_index_after_chars(body: &str, start_byte: usize, char_count: usize) -> usize {
    if char_count == 0 {
        return start_byte;
    }

    body[start_byte..]
        .char_indices()
        .nth(char_count)
        .map_or(body.len(), |(byte, _)| start_byte + byte)
}

#[cfg(test)]
mod tests {
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
        builder.add_simple(
            "3",
            "API Reference",
            "/api",
            "Complete API documentation for developers.",
        );

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
}
