//! Code block annotation parsing and render state.
//!
//! The HTML renderer accepts two annotation grammars: a native `annotate="..."`
//! attribute grammar and a VitePress-compatible grammar. The submodules keep those
//! parsers isolated while exporting a single set of semantic line states for rendering.

mod attribute;
mod meta;
mod state;
mod vitepress;

pub(super) use attribute::{parse_code_annotations, parse_line_numbers};
pub(super) use meta::{
    apply_annotation_numbers, apply_btree_annotations, normalize_code_block_info,
    normalize_code_block_language, split_code_block_meta,
};
pub(super) use state::{
    CodeAnnotationKind, CodeBlockRenderState, CodeLineRenderState, MetaTokenKind,
};
pub(super) use vitepress::parse_vitepress_inline_annotations;
