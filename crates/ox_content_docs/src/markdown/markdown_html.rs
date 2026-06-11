//! HTML rendering (raw-HTML-laced Markdown) for generated API reference docs.
//!
//! Selected when `MarkdownDocsOptions::render_style` is `MarkdownRenderStyle::Html`
//! (the default). Child module of `markdown`; reuses the parent's
//! extraction/formatting/link helpers via `super::` and emits the ox-content theme
//! HTML structures (`<details>`, stats, member tables, prose blocks, …).

use rustc_hash::FxHashSet;

use super::{
    clean_summary_text, effective_members_format, effective_parameters_format, entry_anchor,
    format_kind_label, generate_source_href, is_structured_tag, member_anchor,
    member_table_includes_kind, normalize_signature, process_doc_text, rendered_throws,
    MarkdownDisplayFormat, MarkdownDocsOptions, MarkdownLinkContext, MarkdownPathStrategy,
    SINCE_TAGS,
};
use crate::model::{
    ApiDocEntry, ApiDocMember, ApiDocTag, ApiParamDoc, ApiReturnDoc, ApiThrowsDoc, ApiTypeParamDoc,
};
use crate::string_builder::{join3, StringBuilder};

mod blocks;
mod entry_body;
mod examples;
mod index_signatures;
mod inline;
mod member_bits;
mod member_details;
mod member_groups;
mod member_tables;
mod modules;
mod nested_members;
mod overview;
mod parameters;
mod return_members;
mod sections;
mod stats;
mod tags;
mod throws;
mod type_parameters;

use blocks::render_markdown_blocks_html;
pub(super) use entry_body::{render_entry_html, render_entry_page_html, render_overload_body_html};
use examples::push_examples_html;
pub(super) use examples::render_module_examples_html;
use index_signatures::{render_index_signature_code_block_html, render_index_signature_group_html};
use inline::{
    escape_html, render_code_block_html, render_doc_inline_html,
    render_highlighted_inline_code_html, render_inline_html, render_type_inner_html,
};
use member_bits::{render_member_description_html, render_member_flags, render_member_type_html};
use member_details::{is_callable_member, render_callable_member_group_html};
use member_groups::render_members_html;
use member_tables::{render_member_list_html, render_member_table_html};
pub(super) use modules::{
    render_module_index_html, render_module_lifecycle_badges_html, render_module_section_html,
};
use nested_members::{
    render_nested_members_list_html, render_nested_members_table_html, render_property_members_html,
};
use overview::render_entry_badges_html;
use parameters::{render_member_params_html, render_params_list_html, render_params_table_html};
use return_members::render_return_members_html;
use sections::{
    push_heritage_sections_html, push_params_html, push_returns_html, push_tag_list_html,
    push_throws_html, push_type_parameters_html,
};
pub(super) use stats::{render_details_controls_html, render_stats_html};
use tags::render_tag_list_html;
use throws::{render_throws_inline_html, render_throws_list_html};
use type_parameters::{
    render_member_type_parameters_html, render_type_parameters_list_html,
    render_type_parameters_table_html,
};
