mod apple_music;
#[cfg(test)]
mod catalog_tests;
mod document_cards;
mod fallback;
#[cfg(test)]
mod fallback_tests;
mod html;
mod native;
mod package_cards;
mod playground_cards;
mod provider_cards;
mod registry;
#[cfg(test)]
mod registry_tests;
mod render;
mod speaker_deck;
#[cfg(test)]
mod tests;
mod video_cards;

use crate::{MediaEmbedsOptions, html_scan::find_ci};

use fallback::{fallback_url, render_fallback};
use html::{ComponentElement, find_component, find_pascal_component};
use registry::{PROVIDERS, Provider, Tag};

/// Why an enabled provider did not render the tag it was given.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmbedFallback {
    /// The tag carried a link-safe URL, so it became a plain link.
    Linked,
    /// Nothing safe to link to, so the authored markup was left alone.
    Kept,
}

/// One tag an enabled provider refused, and what happened to it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbedDiagnostic {
    /// Tag name as the provider registers it, for example `speakerdeck`.
    pub provider: String,
    /// The URL the tag carried, when it carried one worth naming.
    pub url: Option<String>,
    /// 1-based line of the opening tag in the transformed HTML.
    pub line: u32,
    /// What the transform did instead.
    pub fallback: EmbedFallback,
}

/// One provider tag as authors write it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbedTag {
    /// Tag name as the provider registers it, for example `speakerdeck`.
    pub name: String,
    /// True when only the PascalCase spelling is an embed, because the
    /// lowercase one is a real HTML element — `<audio>`, `<video>`.
    pub pascal_only: bool,
}

/// Every tag the transform can rewrite.
///
/// Callers that pre-scan a document to decide whether to run the transform at
/// all need this list. Keeping their own copy is how `<CodeSandbox>` came to be
/// skipped on a page that contained nothing else.
pub fn embed_tags() -> Vec<EmbedTag> {
    PROVIDERS
        .iter()
        .map(|provider| EmbedTag {
            name: provider.name.to_string(),
            pascal_only: matches!(provider.tag, Tag::Pascal),
        })
        .collect()
}

pub fn transform_media_embeds(html: &str, options: Option<&MediaEmbedsOptions>) -> String {
    transform_media_embeds_with_diagnostics(html, options).0
}

/// The transform plus every tag an enabled provider refused.
///
/// A refusal is invisible in the output — the tag becomes a link, or stays as
/// written — so without this a typo in a URL ships silently.
pub fn transform_media_embeds_with_diagnostics(
    html: &str,
    options: Option<&MediaEmbedsOptions>,
) -> (String, Vec<EmbedDiagnostic>) {
    let Some(options) = options else {
        return (html.to_string(), Vec::new());
    };
    if !has_enabled_embed(options) || !html.contains('<') {
        return (html.to_string(), Vec::new());
    }

    let mut current = html.to_string();
    let mut diagnostics = Vec::new();
    for provider in PROVIDERS {
        if (provider.enabled)(options) && provider.appears_in(&current) {
            current = rewrite_components(&current, provider, &mut diagnostics);
        }
    }
    (current, diagnostics)
}

/// 1-based line of `offset` within `html`.
fn line_of(html: &str, offset: usize) -> u32 {
    let counted = html[..offset.min(html.len())].bytes().filter(|byte| *byte == b'\n').count();
    u32::try_from(counted + 1).unwrap_or(u32::MAX)
}

fn has_enabled_embed(options: &MediaEmbedsOptions) -> bool {
    PROVIDERS.iter().any(|provider| (provider.enabled)(options))
}

impl Provider {
    /// Cheap pre-scan so a document without the tag never pays for a rewrite.
    fn appears_in(&self, html: &str) -> bool {
        match self.tag {
            Tag::AnyCase => find_ci(html, 0, &format!("<{}", self.name)).is_some(),
            Tag::Pascal => html.contains(&format!("<{}", self.name)),
        }
    }
}

fn rewrite_components(
    html: &str,
    provider: &Provider,
    diagnostics: &mut Vec<EmbedDiagnostic>,
) -> String {
    let find: for<'a> fn(&'a str, usize, &str, &str) -> Option<ComponentElement<'a>> =
        match provider.tag {
            Tag::AnyCase => find_component,
            Tag::Pascal => find_pascal_component,
        };

    let mut out = String::with_capacity(html.len());
    let mut cursor = 0usize;
    let open = format!("<{}", provider.name);

    while let Some(element) = find(html, cursor, &open, provider.name) {
        out.push_str(&html[cursor..element.span.0]);
        let original = &html[element.span.0..element.span.1];
        // An enabled provider that cannot resolve its input still owes the
        // reader the link. Only a tag with no safe URL keeps its markup.
        if let Some(rendered) = (provider.render)(&element) {
            out.push_str(&rendered);
        } else {
            let fallback = render_fallback(&element);
            diagnostics.push(EmbedDiagnostic {
                provider: provider.name.to_string(),
                url: fallback_url(&element).map(str::to_string),
                line: line_of(html, element.span.0),
                fallback: if fallback.is_some() {
                    EmbedFallback::Linked
                } else {
                    EmbedFallback::Kept
                },
            });
            out.push_str(fallback.as_deref().unwrap_or(original));
        }
        cursor = element.span.1;
    }

    out.push_str(&html[cursor..]);
    out
}
