mod html;
mod render;
#[cfg(test)]
mod tests;

use crate::{html_scan::find_ci, MediaEmbedsOptions};

use html::{find_component, ComponentElement};
use render::{
    render_bluesky, render_spotify, render_stackblitz, render_tweet, render_webcontainer,
};

pub fn transform_media_embeds(html: &str, options: Option<&MediaEmbedsOptions>) -> String {
    let Some(options) = options else {
        return html.to_string();
    };
    if !has_enabled_embed(options) || !html.contains('<') {
        return html.to_string();
    }

    let mut current = html.to_string();
    if options.spotify.unwrap_or(false) && contains_ci(&current, "<spotify") {
        current = transform_component(&current, "spotify", render_spotify);
    }
    if options.stack_blitz.unwrap_or(false) && contains_ci(&current, "<stackblitz") {
        current = transform_component(&current, "stackblitz", render_stackblitz);
    }
    if options.twitter.unwrap_or(false)
        && (contains_ci(&current, "<tweet") || contains_ci(&current, "<xpost"))
    {
        current = transform_component(&current, "tweet", render_tweet);
        current = transform_component(&current, "xpost", render_tweet);
    }
    if options.bluesky.unwrap_or(false) && contains_ci(&current, "<bluesky") {
        current = transform_component(&current, "bluesky", render_bluesky);
    }
    if options.web_container.unwrap_or(false) && contains_ci(&current, "<webcontainer") {
        current = transform_component(&current, "webcontainer", render_webcontainer);
    }
    current
}

fn has_enabled_embed(options: &MediaEmbedsOptions) -> bool {
    options.spotify.unwrap_or(false)
        || options.stack_blitz.unwrap_or(false)
        || options.twitter.unwrap_or(false)
        || options.bluesky.unwrap_or(false)
        || options.web_container.unwrap_or(false)
}

fn contains_ci(html: &str, needle: &str) -> bool {
    find_ci(html, 0, needle).is_some()
}

fn transform_component(
    html: &str,
    name: &str,
    render: fn(&ComponentElement<'_>) -> Option<String>,
) -> String {
    let mut out = String::with_capacity(html.len());
    let mut cursor = 0usize;
    let open = format!("<{name}");

    while let Some(element) = find_component(html, cursor, &open, name) {
        out.push_str(&html[cursor..element.span.0]);
        if let Some(rendered) = render(&element) {
            out.push_str(&rendered);
        } else {
            out.push_str(&html[element.span.0..element.span.1]);
        }
        cursor = element.span.1;
    }

    out.push_str(&html[cursor..]);
    out
}
