//! Static YouTube embed transform (Rust port of the TS `transformYouTube`).
//!
//! Rewrites `<youtube …>` elements in already-rendered HTML into a responsive,
//! privacy-enhanced iframe embed. This replaces a `rehype-parse` +
//! `rehype-stringify` round-trip on the JS side: the Rust renderer's HTML is a
//! rehype fixed-point, so rewriting only the `<youtube>` spans and leaving the
//! surrounding bytes untouched reproduces the previous output byte-for-byte.
//!
//! The exact output is pinned by the `embed-transform` characterization tests
//! in `@ox-content/vite-plugin`.

mod parser;
mod render;
#[cfg(test)]
mod tests;

use crate::html_scan::find_ci;

use parser::find_youtube_element;
use render::render_embed;

const VIDEO_ID_LEN: usize = 11;
const URL_ID_PREFIXES: &[&str] =
    &["youtube.com/watch?v=", "youtu.be/", "youtube.com/embed/", "youtube.com/v/"];
const SHORTS_PREFIX: &str = "youtube.com/shorts/";

/// Options mirroring the TS `YouTubeOptions`, with the same defaults.
#[derive(Debug, Clone)]
pub struct YouTubeEmbedOptions {
    pub privacy_enhanced: bool,
    pub aspect_ratio: String,
    pub allow_fullscreen: bool,
    pub lazy_load: bool,
}

impl Default for YouTubeEmbedOptions {
    fn default() -> Self {
        Self {
            privacy_enhanced: true,
            aspect_ratio: "16/9".to_string(),
            allow_fullscreen: true,
            lazy_load: true,
        }
    }
}

fn is_video_id(value: &str) -> bool {
    value.len() == VIDEO_ID_LEN
        && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
}

fn video_id_after(input: &str, start: usize) -> Option<&str> {
    let id = input.get(start..start + VIDEO_ID_LEN)?;
    is_video_id(id).then_some(id)
}

/// Leftmost `prefix + 11-char id` match, continuing past prefixes that are not
/// followed by a valid id. Mirrors the previous unanchored regex search.
fn first_prefixed_id<'a>(input: &'a str, prefixes: &[&str]) -> Option<&'a str> {
    let mut best: Option<(usize, &'a str)> = None;
    for prefix in prefixes {
        let mut search = 0;
        while let Some(relative) = input.get(search..).and_then(|rest| rest.find(prefix)) {
            let id_start = search + relative + prefix.len();
            if let Some(id) = video_id_after(input, id_start) {
                if best.is_none_or(|(position, _)| id_start < position) {
                    best = Some((id_start, id));
                }
                break;
            }
            search += relative + 1;
        }
    }
    best.map(|(_, id)| id)
}

/// Extract a YouTube video id from a bare id or a watch/share/embed/shorts URL.
/// Mirrors the TS `extractVideoId` without compiling regexes at startup.
pub fn extract_video_id(input: &str) -> Option<String> {
    if is_video_id(input) {
        return Some(input.to_string());
    }
    first_prefixed_id(input, URL_ID_PREFIXES)
        .or_else(|| first_prefixed_id(input, &[SHORTS_PREFIX]))
        .map(str::to_string)
}

/// Transform every `<youtube …>` element in `html` into an iframe embed.
/// Elements whose `id`/`url` yield no valid video id are left untouched.
pub fn transform_youtube(html: &str, options: &YouTubeEmbedOptions) -> String {
    // Fast path: nothing to do when the marker is absent.
    if find_ci(html, 0, "<youtube").is_none() {
        return html.to_string();
    }

    let mut out = String::with_capacity(html.len());
    let mut cursor = 0;
    while let Some(element) = find_youtube_element(html, cursor) {
        let (start, end) = element.span;
        out.push_str(&html[cursor..start]);

        let video_id = match &element.id {
            Some(id) => extract_video_id(id),
            None => element.url.as_deref().and_then(extract_video_id),
        };

        match video_id {
            Some(video_id) => {
                out.push_str(&render_embed(
                    &video_id,
                    options,
                    element.title.as_deref(),
                    element.start,
                ));
            }
            // No usable id: leave the original element bytes in place.
            None => out.push_str(&html[start..end]),
        }
        cursor = end;
    }
    out.push_str(&html[cursor..]);
    out
}
