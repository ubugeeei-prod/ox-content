mod apple_music;
mod html;
mod native;
mod package_cards;
mod playground_cards;
mod provider_cards;
mod render;
mod speaker_deck;
#[cfg(test)]
mod tests;

use crate::{MediaEmbedsOptions, html_scan::find_ci};

use apple_music::render_apple_music;
use html::{ComponentElement, find_component, find_pascal_component};
use native::{render_audio, render_video};
use package_cards::{render_crates_io, render_docker_hub, render_npm_package, render_pypi};
use playground_cards::{render_codepen, render_jsfiddle, render_observable};
use provider_cards::{
    render_discord, render_facebook, render_fediverse, render_google_maps, render_instagram,
    render_mastodon, render_misskey, render_mixi2, render_qiita, render_threads, render_zenn,
};
use render::{
    render_bluesky, render_spotify, render_stackblitz, render_tweet, render_webcontainer,
};
use speaker_deck::render_speaker_deck;

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
    if options.apple_music.unwrap_or(false) && contains_ci(&current, "<applemusic") {
        current = transform_component(&current, "applemusic", render_apple_music);
    }
    if options.speaker_deck.unwrap_or(false) && contains_ci(&current, "<speakerdeck") {
        current = transform_component(&current, "speakerdeck", render_speaker_deck);
    }
    if options.audio.unwrap_or(false) && current.contains("<Audio") {
        current = transform_pascal_component(&current, "Audio", render_audio);
    }
    if options.video.unwrap_or(false) && current.contains("<Video") {
        current = transform_pascal_component(&current, "Video", render_video);
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
    if options.google_maps.unwrap_or(false) && contains_ci(&current, "<googlemaps") {
        current = transform_component(&current, "googlemaps", render_google_maps);
    }
    if options.qiita.unwrap_or(false) && contains_ci(&current, "<qiita") {
        current = transform_component(&current, "qiita", render_qiita);
    }
    if options.zenn.unwrap_or(false) && contains_ci(&current, "<zenn") {
        current = transform_component(&current, "zenn", render_zenn);
    }
    if options.package_registry.unwrap_or(false) {
        if contains_ci(&current, "<npmpackage") {
            current = transform_component(&current, "npmpackage", render_npm_package);
        }
        if contains_ci(&current, "<cratesio") {
            current = transform_component(&current, "cratesio", render_crates_io);
        }
        if contains_ci(&current, "<pypi") {
            current = transform_component(&current, "pypi", render_pypi);
        }
        if contains_ci(&current, "<dockerhub") {
            current = transform_component(&current, "dockerhub", render_docker_hub);
        }
    }
    if options.playgrounds.unwrap_or(false) {
        if contains_ci(&current, "<codepen") {
            current = transform_component(&current, "codepen", render_codepen);
        }
        if contains_ci(&current, "<jsfiddle") {
            current = transform_component(&current, "jsfiddle", render_jsfiddle);
        }
        if contains_ci(&current, "<observable") {
            current = transform_component(&current, "observable", render_observable);
        }
    }
    if options.discord.unwrap_or(false) && contains_ci(&current, "<discord") {
        current = transform_component(&current, "discord", render_discord);
    }
    if options.fediverse.unwrap_or(false) {
        if contains_ci(&current, "<fediverse") {
            current = transform_component(&current, "fediverse", render_fediverse);
        }
        if contains_ci(&current, "<mastodon") {
            current = transform_component(&current, "mastodon", render_mastodon);
        }
        if contains_ci(&current, "<misskey") {
            current = transform_component(&current, "misskey", render_misskey);
        }
        if contains_ci(&current, "<mixi2") {
            current = transform_component(&current, "mixi2", render_mixi2);
        }
    }
    if options.facebook.unwrap_or(false) && contains_ci(&current, "<facebook") {
        current = transform_component(&current, "facebook", render_facebook);
    }
    if options.threads.unwrap_or(false) && contains_ci(&current, "<threads") {
        current = transform_component(&current, "threads", render_threads);
    }
    if options.instagram.unwrap_or(false) && contains_ci(&current, "<instagram") {
        current = transform_component(&current, "instagram", render_instagram);
    }
    if options.web_container.unwrap_or(false) && contains_ci(&current, "<webcontainer") {
        current = transform_component(&current, "webcontainer", render_webcontainer);
    }
    current
}

fn has_enabled_embed(options: &MediaEmbedsOptions) -> bool {
    options.spotify.unwrap_or(false)
        || options.apple_music.unwrap_or(false)
        || options.speaker_deck.unwrap_or(false)
        || options.audio.unwrap_or(false)
        || options.video.unwrap_or(false)
        || options.stack_blitz.unwrap_or(false)
        || options.twitter.unwrap_or(false)
        || options.bluesky.unwrap_or(false)
        || options.google_maps.unwrap_or(false)
        || options.qiita.unwrap_or(false)
        || options.zenn.unwrap_or(false)
        || options.package_registry.unwrap_or(false)
        || options.playgrounds.unwrap_or(false)
        || options.discord.unwrap_or(false)
        || options.fediverse.unwrap_or(false)
        || options.facebook.unwrap_or(false)
        || options.threads.unwrap_or(false)
        || options.instagram.unwrap_or(false)
        || options.web_container.unwrap_or(false)
}

fn contains_ci(html: &str, needle: &str) -> bool {
    find_ci(html, 0, needle).is_some()
}

fn transform_pascal_component(
    html: &str,
    name: &str,
    render: fn(&ComponentElement<'_>) -> Option<String>,
) -> String {
    rewrite_components(html, name, render, find_pascal_component)
}

fn transform_component(
    html: &str,
    name: &str,
    render: fn(&ComponentElement<'_>) -> Option<String>,
) -> String {
    rewrite_components(html, name, render, find_component)
}

fn rewrite_components(
    html: &str,
    name: &str,
    render: fn(&ComponentElement<'_>) -> Option<String>,
    find: for<'a> fn(&'a str, usize, &str, &str) -> Option<ComponentElement<'a>>,
) -> String {
    let mut out = String::with_capacity(html.len());
    let mut cursor = 0usize;
    let open = format!("<{name}");

    while let Some(element) = find(html, cursor, &open, name) {
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
