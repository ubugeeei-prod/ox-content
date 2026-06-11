//! Pure Markdown rendering (no raw HTML) for generated API reference docs.
//!
//! Selected when `MarkdownDocsOptions::render_style` is
//! `MarkdownRenderStyle::Markdown`. This is a child module of `markdown`, so it
//! reuses the parent's extraction/formatting helpers via `super::` and emits the
//! same per-entry information as the HTML renderer — but as Markdown headings,
//! tables and fenced code blocks (no `<details>`, no theme-specific HTML).

mod entry_body;
mod format;
mod index_signatures;
mod lifecycle;
mod member_bits;
mod member_details;
mod member_groups;
mod member_sections;
mod member_tables;
mod parameters;
mod return_members;
mod sections;
mod stats;
mod type_parameters;

pub(super) use entry_body::{render_entry_body_pure, render_overload_body_pure};
pub(super) use lifecycle::push_lifecycle_alerts;
pub(super) use stats::render_stats_markdown;
