//! Search query engine with BM25 scoring.

use std::cmp::Ordering;

use rustc_hash::FxHashMap;

use serde::{Deserialize, Serialize};

use crate::fuzzy::fuzzy_match_weight;
use crate::index::SearchIndex;
use crate::scope::parse_search_query;

mod rich;
mod snippet;

pub use rich::{SearchResultMetadata, SearchResultRanking};

use rich::{
    MatchKind, PHRASE_MATCH_BOOST, SearchCandidate, SearchConstraints, phrase_fields,
    phrase_tokens, result_aria_label, result_metadata, result_ranking, term_tokens,
};

/// Search options.
///
/// Defaults match the client-side search runtime defaults.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOptions {
    /// Maximum number of results to return.
    ///
    /// Default: `10`.
    #[serde(default = "default_limit")]
    pub limit: usize,

    /// Enable prefix matching for the last token.
    ///
    /// Default: `true`.
    #[serde(default = "default_prefix")]
    pub prefix: bool,

    /// Enable fuzzy matching (edit distance).
    ///
    /// Default: `false`.
    #[serde(default)]
    pub fuzzy: bool,

    /// Minimum score threshold (0.0 - 1.0).
    ///
    /// Default: `0.0`.
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

    /// Hierarchical result metadata for richer cards.
    pub metadata: SearchResultMetadata,

    /// Stable explanation of the ranking inputs.
    pub ranking: SearchResultRanking,

    /// Concise accessible label for result cards.
    #[serde(rename = "ariaLabel")]
    pub aria_label: String,
}

/// BM25 parameters.
const K1: f64 = 1.2;
const B: f64 = 0.75;
const MIN_PREFIX_MATCH_LEN: usize = 2;
const SNIPPET_MAX_CHARS: usize = 150;

impl SearchIndex {
    /// Searches the index with the given query.
    #[must_use]
    pub fn search(&self, query: &str, options: &SearchOptions) -> Vec<SearchResult> {
        if query.is_empty() || self.is_empty() {
            return Vec::new();
        }

        let parsed_query = parse_search_query(query);
        if parsed_query.is_empty() {
            return Vec::new();
        }
        let constraints = SearchConstraints::from_query(&parsed_query);

        // Calculate scores for each document
        let mut doc_scores: FxHashMap<usize, SearchCandidate> = FxHashMap::default();

        let term_tokens = term_tokens(&parsed_query);
        let phrase_tokens = phrase_tokens(&parsed_query);
        let has_scoring_query = !term_tokens.is_empty()
            || !phrase_tokens.is_empty()
            || !parsed_query.prefixes.is_empty();

        if !has_scoring_query && constraints.has_refinements() {
            for (doc_idx, doc) in self.documents.iter().enumerate() {
                if constraints.matches(doc) {
                    doc_scores.entry(doc_idx).or_default();
                }
            }
        }

        for (i, token) in term_tokens.iter().enumerate() {
            let is_last = i == term_tokens.len() - 1;

            if is_last && options.prefix && token.len() >= MIN_PREFIX_MATCH_LEN {
                // Prefix expansion is limited to the final token so a query
                // like "render mar" can reuse exact postings for "render" and
                // only scan vocabulary terms for the active completion token.
                for term in self.index.keys().filter(|term| term.starts_with(token)) {
                    self.score_matching_term(
                        term,
                        1.0,
                        MatchKind::Prefix,
                        constraints,
                        &mut doc_scores,
                    );
                }
            } else {
                self.score_matching_term(token, 1.0, MatchKind::Term, constraints, &mut doc_scores);
            }

            if options.fuzzy {
                for (term, weight) in self.fuzzy_terms(token, is_last && options.prefix) {
                    self.score_matching_term(
                        term,
                        weight,
                        MatchKind::Fuzzy,
                        constraints,
                        &mut doc_scores,
                    );
                }
            }
        }

        for token in &phrase_tokens {
            self.score_matching_term(token, 1.0, MatchKind::Term, constraints, &mut doc_scores);
            if options.fuzzy {
                for (term, weight) in self.fuzzy_terms(token, false) {
                    self.score_matching_term(
                        term,
                        weight,
                        MatchKind::Fuzzy,
                        constraints,
                        &mut doc_scores,
                    );
                }
            }
        }

        for prefix in &parsed_query.prefixes {
            if prefix.len() >= MIN_PREFIX_MATCH_LEN {
                for term in self.index.keys().filter(|term| term.starts_with(prefix)) {
                    self.score_matching_term(
                        term,
                        1.0,
                        MatchKind::Prefix,
                        constraints,
                        &mut doc_scores,
                    );
                }
            }
        }

        for phrase in &parsed_query.phrases {
            self.score_matching_phrase(phrase, constraints, &mut doc_scores);
        }

        // Sort and limit candidates before constructing result payloads.
        // Snippet generation scans and lowercases document bodies, so doing it
        // only for returned results avoids wasted work on documents that lose
        // the ranking step.
        let mut ranked_docs: Vec<_> = doc_scores
            .into_iter()
            .filter(|(_, candidate)| candidate.score >= options.threshold)
            .collect();

        ranked_docs.sort_by(|(left_idx, left), (right_idx, right)| {
            right
                .score
                .partial_cmp(&left.score)
                .unwrap_or(Ordering::Equal)
                .then_with(|| left_idx.cmp(right_idx))
        });
        ranked_docs.truncate(options.limit);

        ranked_docs
            .into_iter()
            .map(|(doc_idx, candidate)| {
                let doc = &self.documents[doc_idx];
                let score = candidate.score;
                let matches = candidate.matches.clone();
                let snippet = self.generate_snippet(&doc.body, &matches, SNIPPET_MAX_CHARS);
                let metadata =
                    result_metadata(doc, &matches, constraints, parsed_query.filters.clone());
                let ranking = result_ranking(score, candidate, &parsed_query);
                let aria_label = result_aria_label(doc, &metadata, &ranking);
                SearchResult {
                    id: doc.id.clone(),
                    title: doc.title.clone(),
                    url: doc.url.clone(),
                    score,
                    matches,
                    snippet,
                    metadata,
                    ranking,
                    aria_label,
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

    fn score_matching_term(
        &self,
        term: &str,
        weight: f64,
        kind: MatchKind,
        constraints: SearchConstraints<'_>,
        doc_scores: &mut FxHashMap<usize, SearchCandidate>,
    ) {
        let Some(postings) = self.index.get(term) else {
            return;
        };

        let df = self.df.get(term).copied().unwrap_or(1);
        let idf = self.compute_idf(df);

        for posting in postings {
            let doc = &self.documents[posting.doc_idx];
            if !constraints.matches(doc) {
                continue;
            }

            #[allow(clippy::cast_precision_loss)]
            let doc_len = doc.body.len() as f64;
            let tf = f64::from(posting.tf);

            // BM25 score with field boost
            let score = idf
                * ((tf * (K1 + 1.0)) / K1.mul_add(1.0 - B + B * doc_len / self.avg_dl, tf))
                * posting.field.boost()
                * weight;

            // Accumulate in one per-document entry so repeated matches across
            // query tokens do not allocate intermediate result rows. The small
            // `matches` Vec is de-duplicated in place because it is surfaced in
            // the final result payload.
            let entry = doc_scores.entry(posting.doc_idx).or_default();
            entry.score += score;
            entry.add_term_match(term, posting.field, kind);
        }
    }

    fn score_matching_phrase(
        &self,
        phrase: &str,
        constraints: SearchConstraints<'_>,
        doc_scores: &mut FxHashMap<usize, SearchCandidate>,
    ) {
        for (doc_idx, doc) in self.documents.iter().enumerate() {
            if !constraints.matches(doc) {
                continue;
            }

            let fields = phrase_fields(doc, phrase);
            if fields.is_empty() {
                continue;
            }

            let entry = doc_scores.entry(doc_idx).or_default();
            for field in fields {
                entry.score = field.boost().mul_add(PHRASE_MATCH_BOOST, entry.score);
                entry.add_phrase_match(phrase, field);
            }
        }
    }

    fn fuzzy_terms<'a>(
        &'a self,
        token: &'a str,
        skip_prefix_matches: bool,
    ) -> impl Iterator<Item = (&'a str, f64)> {
        self.index.keys().filter_map(move |term| {
            if term == token || (skip_prefix_matches && term.starts_with(token)) {
                return None;
            }
            fuzzy_match_weight(token, term).map(|weight| (term.as_str(), weight))
        })
    }

    /// Generates a snippet of text around matched terms.
    #[allow(clippy::unused_self)]
    fn generate_snippet(&self, body: &str, matches: &[String], max_len: usize) -> String {
        snippet::generate_snippet(body, matches, max_len)
    }
}

#[cfg(test)]
mod tests;
