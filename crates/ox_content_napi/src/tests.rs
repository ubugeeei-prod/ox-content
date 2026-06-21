use ox_content_docs::{
    NormalizedDocEntry, NormalizedDocKind, NormalizedMember, NormalizedMemberKind,
    NormalizedReturnDoc, NormalizedThrowsDoc, NormalizedTypeParam,
};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::process::Command;

use super::docs_bindings::{convert_markdown_entry, map_normalized_doc_entry};
use super::{
    extract_docs_from_entry_points_napi, extract_file_doc_entries, generate_docs_markdown,
    generate_docs_nav_metadata_from_docs_napi, get_git_last_updated, JsDocMember, JsDocParam,
    JsDocReturn, JsDocThrows, JsDocsMarkdownEntry, JsDocsMarkdownModule, JsDocsMarkdownOptions,
    JsDocsMarkdownTag, JsDocsNavOptions, JsEntryPointDocsOptions, JsEntryPointSpec, JsTypeParam,
};
use ox_content_transform::transformer::parse_frontmatter;

fn assert_string_map_snapshot(name: &str, map: &HashMap<String, String>) {
    let mut entries = map.iter().collect::<Vec<_>>();
    entries.sort_by_key(|(path, _)| *path);

    let mut rendered = String::new();
    for (path, content) in entries {
        rendered.push_str("===== ");
        rendered.push_str(path);
        rendered.push_str(" =====\n");
        rendered.push_str(content);
        if !content.ends_with('\n') {
            rendered.push('\n');
        }
    }

    insta::with_settings!({
        snapshot_path => "tests/snapshots",
        prepend_module_to_snapshot => false,
        omit_expression => true,
    }, {
        insta::assert_snapshot!(name, rendered);
    });
}

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
