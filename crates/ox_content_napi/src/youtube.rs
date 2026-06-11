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

use std::sync::LazyLock;

use regex::Regex;

use crate::html_scan::find_ci;

use parser::find_youtube_element;
use render::render_embed;

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

static VIDEO_ID: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9_-]{11}$").expect("valid video id regex"));
static URL_PATTERNS: LazyLock<[Regex; 2]> = LazyLock::new(|| {
    [
        Regex::new(
            r"(?:youtube\.com/watch\?v=|youtu\.be/|youtube\.com/embed/|youtube\.com/v/)([a-zA-Z0-9_-]{11})",
        )
        .expect("valid url regex"),
        Regex::new(r"youtube\.com/shorts/([a-zA-Z0-9_-]{11})").expect("valid shorts regex"),
    ]
});

/// Extract a YouTube video id from a bare id or a watch/share/embed/shorts URL.
/// Mirrors the TS `extractVideoId`.
pub fn extract_video_id(input: &str) -> Option<String> {
    if VIDEO_ID.is_match(input) {
        return Some(input.to_string());
    }
    for pattern in URL_PATTERNS.iter() {
        if let Some(captures) = pattern.captures(input) {
            if let Some(id) = captures.get(1) {
                return Some(id.as_str().to_string());
            }
        }
    }
    None
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
                out.push_str(&render_embed(&video_id, options, element.title.as_deref()));
            }
            // No usable id: leave the original element bytes in place.
            None => out.push_str(&html[start..end]),
        }
        cursor = end;
    }
    out.push_str(&html[cursor..]);
    out
}
