//! # ox_content_i18n
//!
//! Core i18n library for Ox Content providing ICU MessageFormat 2 (MF2) parsing,
//! multi-locale dictionary management, and static analysis for translation keys.
//!
//! ## Modules
//!
//! - [`mf2`] — Hand-written MF2 lexer, recursive-descent parser, AST, and semantic validator
//! - [`dictionary`] — JSON/YAML dictionary loaders with nested key flattening and multi-locale [`DictionarySet`]
//! - [`checker`] — Static analysis rules: missing keys, unused keys, type mismatch, syntax errors
//! - [`locale`] — BCP 47 [`Locale`] type with validation
//! - [`key`] — Dot-separated [`KeyPath`] utilities (namespace, segments, depth)
//! - [`error`] — Unified error types

pub mod checker;
pub mod dictionary;
pub mod error;
pub mod key;
pub mod locale;
pub mod mf2;
pub mod runtime;

pub use dictionary::{Dictionary, DictionarySet};
pub use error::{I18nError, I18nResult};
pub use key::KeyPath;
pub use locale::Locale;
