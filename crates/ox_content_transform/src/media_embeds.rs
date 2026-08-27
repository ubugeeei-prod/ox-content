mod apple_music;
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

use html::{ComponentElement, find_component, find_pascal_component};
use registry::{PROVIDERS, Provider, Tag};

pub fn transform_media_embeds(html: &str, options: Option<&MediaEmbedsOptions>) -> String {
    let Some(options) = options else {
        return html.to_string();
    };
    if !has_enabled_embed(options) || !html.contains('<') {
        return html.to_string();
    }

    let mut current = html.to_string();
    for provider in PROVIDERS {
        if (provider.enabled)(options) && provider.appears_in(&current) {
            current = rewrite_components(&current, provider);
        }
    }
    current
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

fn rewrite_components(html: &str, provider: &Provider) -> String {
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
        if let Some(rendered) = (provider.render)(&element) {
            out.push_str(&rendered);
        } else {
            out.push_str(&html[element.span.0..element.span.1]);
        }
        cursor = element.span.1;
    }

    out.push_str(&html[cursor..]);
    out
}
