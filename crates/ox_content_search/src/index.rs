//! Search index data structures.

use rustc_hash::FxHashMap;

use serde::{Deserialize, Serialize};

use crate::tokenizer::tokenize;

/// A searchable document in the index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchDocument {
    /// Unique document identifier (usually the URL path).
    pub id: String,
    /// Document title.
    pub title: String,
    /// Document URL/path.
    pub url: String,
    /// Main content text.
    pub body: String,
    /// Headings in the document.
    pub headings: Vec<String>,
    /// Code snippets (optional).
    #[serde(default)]
    pub code: Vec<String>,
}

/// Posting list entry for inverted index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Posting {
    /// Document index in the documents array.
    pub doc_idx: usize,
    /// Term frequency in this document.
    pub tf: u32,
    /// Field where term was found (for boosting).
    pub field: Field,
}

/// Document fields with different boost weights.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Field {
    /// Title field (highest weight).
    Title,
    /// Headings (high weight).
    Heading,
    /// Body text (normal weight).
    Body,
    /// Code blocks (lower weight).
    Code,
}

impl Field {
    /// Returns the boost factor for this field.
    #[must_use]
    pub fn boost(self) -> f64 {
        match self {
            Self::Title => 10.0,
            Self::Heading => 5.0,
            Self::Body => 1.0,
            Self::Code => 0.5,
        }
    }
}

/// The main search index structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchIndex {
    /// All indexed documents.
    pub documents: Vec<SearchDocument>,
    /// Inverted index: term -> list of postings.
    pub index: FxHashMap<String, Vec<Posting>>,
    /// Document frequency: term -> number of documents containing term.
    pub df: FxHashMap<String, usize>,
    /// Average document length (for BM25).
    pub avg_dl: f64,
    /// Total number of documents.
    pub doc_count: usize,
}

impl SearchIndex {
    /// Serializes the index to JSON.
    #[must_use]
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    /// Serializes the index to compact JSON.
    #[must_use]
    pub fn to_json_compact(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    /// Deserializes an index from JSON.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Returns the number of documents in the index.
    #[must_use]
    pub fn len(&self) -> usize {
        self.documents.len()
    }

    /// Returns true if the index is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.documents.is_empty()
    }
}

/// Builder for constructing a search index.
#[derive(Debug, Default)]
pub struct SearchIndexBuilder {
    documents: Vec<SearchDocument>,
}

impl SearchIndexBuilder {
    /// Creates a new index builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a document to the index.
    pub fn add_document(&mut self, doc: SearchDocument) -> &mut Self {
        self.documents.push(doc);
        self
    }

    /// Adds a simple document with just id, title, and body.
    pub fn add_simple(&mut self, id: &str, title: &str, url: &str, body: &str) -> &mut Self {
        self.documents.push(SearchDocument {
            id: id.to_string(),
            title: title.to_string(),
            url: url.to_string(),
            body: body.to_string(),
            headings: Vec::new(),
            code: Vec::new(),
        });
        self
    }

    /// Builds the search index.
    #[must_use]
    pub fn build(self) -> SearchIndex {
        let mut index: FxHashMap<String, Vec<Posting>> = FxHashMap::default();
        let mut df: FxHashMap<String, usize> = FxHashMap::default();
        let mut total_length = 0usize;

        for (doc_idx, doc) in self.documents.iter().enumerate() {
            let mut doc_terms: FxHashMap<String, (u32, Field)> = FxHashMap::default();

            // Index title
            for token in tokenize(&doc.title) {
                doc_terms
                    .entry(token)
                    .and_modify(|(count, _)| *count += 1)
                    .or_insert((1, Field::Title));
            }

            // Index headings
            for heading in &doc.headings {
                for token in tokenize(heading) {
                    doc_terms
                        .entry(token)
                        .and_modify(|(count, _)| *count += 1)
                        .or_insert((1, Field::Heading));
                }
            }

            // Index body
            let body_tokens = tokenize(&doc.body);
            total_length += body_tokens.len();
            for token in body_tokens {
                doc_terms
                    .entry(token)
                    .and_modify(|(count, _)| *count += 1)
                    .or_insert((1, Field::Body));
            }

            // Index code
            for code in &doc.code {
                for token in tokenize(code) {
                    doc_terms
                        .entry(token)
                        .and_modify(|(count, _)| *count += 1)
                        .or_insert((1, Field::Code));
                }
            }

            // Update document frequency and inverted index
            for (term, (tf, field)) in doc_terms {
                *df.entry(term.clone()).or_insert(0) += 1;
                index.entry(term).or_default().push(Posting { doc_idx, tf, field });
            }
        }

        let doc_count = self.documents.len();
        #[allow(clippy::cast_precision_loss)]
        let avg_dl = if doc_count > 0 { total_length as f64 / doc_count as f64 } else { 0.0 };

        SearchIndex { documents: self.documents, index, df, avg_dl, doc_count }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_index() {
        let mut builder = SearchIndexBuilder::new();
        builder.add_simple(
            "1",
            "Getting Started",
            "/getting-started",
            "Welcome to the documentation",
        );
        builder.add_simple("2", "Installation", "/installation", "How to install the package");

        let index = builder.build();

        assert_eq!(index.len(), 2);
        assert!(index.index.contains_key("getting"));
        assert!(index.index.contains_key("started"));
        assert!(index.index.contains_key("install"));
    }

    #[test]
    fn test_serialize_deserialize() {
        let mut builder = SearchIndexBuilder::new();
        builder.add_simple("1", "Test", "/test", "Test content");

        let index = builder.build();
        let json = index.to_json();
        let restored = SearchIndex::from_json(&json).unwrap();

        assert_eq!(restored.len(), 1);
        assert_eq!(restored.documents[0].title, "Test");
    }
}
