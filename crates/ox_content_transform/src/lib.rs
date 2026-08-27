//! Markdown transformation pipeline for Ox Content.

#![cfg_attr(
    not(test),
    deny(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::todo,
        clippy::unimplemented
    )
)]

pub mod cross_references;
pub mod features;
pub mod highlight;
pub(crate) mod html_scan;
pub mod media_embeds;
pub mod pm;
pub mod publish_state;
pub mod sanitize;
pub mod tabs;
pub mod transformer;
pub mod youtube;

mod mdx_metadata;
mod options;

pub use mdx_metadata::{
    MdxImport, MdxImportSpecifier, MdxImportSpecifierKind, MdxMetadata, extract_mdx_metadata,
};
pub use options::*;
pub use publish_state::{PublishDecision, PublishStateOptions, classify_publish_state};
