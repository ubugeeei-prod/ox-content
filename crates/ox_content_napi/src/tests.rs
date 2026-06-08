use ox_content_docs::{
    NormalizedDocEntry, NormalizedDocKind, NormalizedMember, NormalizedMemberKind,
    NormalizedReturnDoc, NormalizedTypeParam,
};
use serde_json::json;
use std::collections::BTreeMap;
use std::fs;
use std::process::Command;

use super::docs_bindings::{convert_markdown_entry, map_normalized_doc_entry};
use super::transformer::parse_frontmatter;
use super::{
    extract_docs_from_entry_points_napi, extract_file_doc_entries, generate_docs_markdown,
    generate_docs_nav_metadata_from_docs_napi, get_git_last_updated, JsDocMember, JsDocParam,
    JsDocReturn, JsDocsMarkdownEntry, JsDocsMarkdownModule, JsDocsMarkdownOptions,
    JsDocsMarkdownTag, JsDocsNavOptions, JsEntryPointDocsOptions, JsEntryPointSpec, JsTypeParam,
};

mod doc_conversion;
mod doc_mapping;
mod docs_markdown_index_options;
mod docs_markdown_links;
mod docs_markdown_reexports_and_generics;
mod docs_markdown_returns;
mod docs_markdown_typedoc;
mod docs_markdown_types_and_modules;
mod docs_nav_output;
mod entry_points;
mod frontmatter;
mod runtime_features;
