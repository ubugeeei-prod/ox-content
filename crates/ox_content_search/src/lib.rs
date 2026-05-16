//! Full-text search engine for Ox Content.
//!
//! This crate provides a lightweight, high-performance full-text search engine
//! for Markdown documents processed by Ox Content.
//!
//! # Features
//!
//! - TF-IDF based scoring for relevance ranking
//! - Multi-field search (title, body, headings, code)
//! - Prefix matching for autocomplete
//! - Serializable index for build-time generation
//!
//! # Example
//!
//! ```ignore
//! use ox_content_search::{SearchIndex, SearchIndexBuilder, SearchOptions};
//!
//! // Build index at build time
//! let mut builder = SearchIndexBuilder::new();
//! builder.add_document("getting-started", "Getting Started", "Welcome to the docs...");
//! let index = builder.build();
//!
//! // Serialize for client-side use
//! let json = index.to_json();
//!
//! // Search at runtime
//! let results = index.search("getting started", &SearchOptions::default());
//! ```

mod index;
mod indexer;
mod query;
mod runtime;
mod scope;
mod tokenizer;

pub use index::{Field, Posting, SearchDocument, SearchIndex, SearchIndexBuilder};
pub use indexer::DocumentIndexer;
pub use query::{SearchOptions, SearchResult};
pub use runtime::generate_search_module;
pub use scope::{
    get_search_document_scopes, matches_search_scopes, parse_scoped_search_query, ScopedSearchQuery,
};
