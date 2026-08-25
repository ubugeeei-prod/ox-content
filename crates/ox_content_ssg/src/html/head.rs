//! Build-time page-head resolver shared by themed, bare, and custom SSG hosts.

mod from_page;
mod input;
mod resolve;
mod serialize;

#[cfg(test)]
mod tests;

pub use input::{HeadAlternate, HeadInput, HeadJsonLd, HeadLink, HeadMeta, SiteHead};
pub use resolve::{HeadDiagnostic, ResolvedHead, ResolvedTag, resolve_head};
pub use serialize::serialize_head;

use from_page::{bare_head_input, themed_head_input};

use crate::html::BarePageData;
use crate::html::page::{PageData, SsgConfig};

/// HTML plus diagnostics from [`render_head`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RenderedHead {
    pub html: String,
    pub diagnostics: Vec<HeadDiagnostic>,
}

/// Resolve and serialize descriptors. Invalid values are dropped.
pub fn render_head(input: &HeadInput) -> RenderedHead {
    let resolved = resolve_head(input);
    RenderedHead { html: serialize_head(&resolved.tags), diagnostics: resolved.diagnostics }
}

pub(super) fn render_themed_head(
    page: &PageData,
    config: &SsgConfig,
    json_ld: Option<&str>,
) -> RenderedHead {
    render_head(&themed_head_input(page, config, json_ld))
}

pub(super) fn render_bare_head(data: &BarePageData<'_>) -> RenderedHead {
    render_head(&bare_head_input(data))
}
