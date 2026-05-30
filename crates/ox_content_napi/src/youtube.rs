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

use std::sync::LazyLock;

use regex::Regex;

use crate::html_scan::find_ci;

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

/// HTML-escape an attribute value the way `hast-util-to-html` does for the
/// characters that can occur in titles/urls here.
fn escape_attribute(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&#x26;"),
            '"' => out.push_str("&#x22;"),
            '<' => out.push_str("&#x3C;"),
            '>' => out.push_str("&#x3E;"),
            _ => out.push(ch),
        }
    }
    out
}

fn build_embed_url(video_id: &str, options: &YouTubeEmbedOptions) -> String {
    let domain =
        if options.privacy_enhanced { "www.youtube-nocookie.com" } else { "www.youtube.com" };
    format!("https://{domain}/embed/{video_id}")
}

fn render_embed(video_id: &str, options: &YouTubeEmbedOptions, title: Option<&str>) -> String {
    let embed_url = build_embed_url(video_id, options);
    // Escape the borrowed title directly instead of copying it into an owned
    // String first (`escape_attribute` already returns an owned String).
    let escaped_title = match title {
        Some(title) => escape_attribute(title),
        None => escape_attribute(&format!("YouTube video {video_id}")),
    };
    let mut html = String::new();
    html.push_str("<div class=\"ox-youtube\" style=\"aspect-ratio: ");
    html.push_str(&options.aspect_ratio);
    html.push_str(";\"><iframe src=\"");
    html.push_str(&escape_attribute(&embed_url));
    html.push_str("\" title=\"");
    html.push_str(&escaped_title);
    html.push_str("\" allow=\"accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share\" referrerpolicy=\"strict-origin-when-cross-origin\"");
    if options.allow_fullscreen {
        html.push_str(" allowfullscreen");
    }
    if options.lazy_load {
        html.push_str(" loading=\"lazy\"");
    }
    html.push_str("></iframe></div>");
    html
}

/// A `<youtube …>` element located in the source HTML.
///
/// Note: the `start` attribute is intentionally not read. The TS
/// implementation this ports parsed HTML via hast, which coerces `start`
/// (a known numeric attribute on `<ol>`) to a number and then dropped it in
/// its string-only attribute reader — so `start` never reached the embed
/// URL. Reproducing that exactly keeps this a behaviour-preserving port;
/// honouring `start` is left to a separate change.
struct YouTubeElement {
    /// Byte range of the whole element (open tag through close tag or `/>`).
    span: (usize, usize),
    id: Option<String>,
    url: Option<String>,
    title: Option<String>,
}

/// Find the next `<youtube …>` element at or after `from`. Recognises both
/// `<youtube …></youtube>` and self-closing `<youtube … />` forms.
fn find_youtube_element(html: &str, from: usize) -> Option<YouTubeElement> {
    let bytes = html.as_bytes();
    let mut search = from;
    loop {
        let rel = find_ci(html, search, "<youtube")?;
        let tag_start = rel;
        let after_name = tag_start + "<youtube".len();
        // Require an element boundary after the name so `<youtuber>` etc. never
        // matches.
        let boundary = bytes.get(after_name).copied();
        let is_boundary =
            matches!(boundary, Some(b) if b == b'>' || b == b'/' || b.is_ascii_whitespace());
        if !is_boundary {
            search = after_name;
            continue;
        }

        // Scan to the end of the start tag, respecting quoted attribute values
        // so a `>` inside a value doesn't terminate the tag early.
        let mut i = after_name;
        let mut quote: Option<u8> = None;
        let mut tag_end = None;
        while i < bytes.len() {
            let b = bytes[i];
            match quote {
                Some(q) => {
                    if b == q {
                        quote = None;
                    }
                }
                None => {
                    if b == b'"' || b == b'\'' {
                        quote = Some(b);
                    } else if b == b'>' {
                        tag_end = Some(i);
                        break;
                    }
                }
            }
            i += 1;
        }
        let tag_end = tag_end?;
        let self_closing = tag_end > after_name && bytes[tag_end - 1] == b'/';

        let inner_end = if self_closing { tag_end - 1 } else { tag_end };

        // Pull out the three attributes we care about in a single pass, keeping
        // the first occurrence of each (matching hast's first-wins semantics)
        // and moving the values out instead of cloning them.
        let (mut id, mut url, mut title) = (None, None, None);
        for (name, value) in parse_attributes(&html[after_name..inner_end]) {
            match name.as_str() {
                "id" if id.is_none() => id = Some(value),
                "url" if url.is_none() => url = Some(value),
                "title" if title.is_none() => title = Some(value),
                _ => {}
            }
        }

        let span_end = if self_closing {
            tag_end + 1
        } else {
            // Find the matching close tag.
            match find_ci(html, tag_end + 1, "</youtube>") {
                Some(close_start) => close_start + "</youtube>".len(),
                None => tag_end + 1,
            }
        };

        return Some(YouTubeElement { span: (tag_start, span_end), id, url, title });
    }
}

/// Parse `name="value"` / `name='value'` / `name=value` / bare `name`
/// attributes from the inside of a start tag. Names are lower-cased.
fn parse_attributes(inner: &str) -> Vec<(String, String)> {
    let bytes = inner.as_bytes();
    let mut attrs = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() || bytes[i] == b'/' {
            break;
        }
        let name_start = i;
        while i < bytes.len()
            && !bytes[i].is_ascii_whitespace()
            && bytes[i] != b'='
            && bytes[i] != b'/'
        {
            i += 1;
        }
        if name_start == i {
            break;
        }
        let name = inner[name_start..i].to_ascii_lowercase();
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        let value = if i < bytes.len() && bytes[i] == b'=' {
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if i >= bytes.len() {
                String::new()
            } else if bytes[i] == b'"' || bytes[i] == b'\'' {
                let q = bytes[i];
                i += 1;
                let vs = i;
                while i < bytes.len() && bytes[i] != q {
                    i += 1;
                }
                let v = inner[vs..i].to_string();
                if i < bytes.len() {
                    i += 1;
                }
                v
            } else {
                let vs = i;
                while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'/' {
                    i += 1;
                }
                inner[vs..i].to_string()
            }
        } else {
            String::new()
        };
        attrs.push((name, value));
    }
    attrs
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

#[cfg(test)]
mod tests {
    use super::*;

    fn opts() -> YouTubeEmbedOptions {
        YouTubeEmbedOptions::default()
    }

    #[test]
    fn wraps_bare_id_matching_characterization() {
        let html = transform_youtube(r#"<p><youtube id="dQw4w9WgXcQ"></youtube></p>"#, &opts());
        assert_eq!(
            html,
            r#"<p><div class="ox-youtube" style="aspect-ratio: 16/9;"><iframe src="https://www.youtube-nocookie.com/embed/dQw4w9WgXcQ" title="YouTube video dQw4w9WgXcQ" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen loading="lazy"></iframe></div></p>"#
        );
    }

    #[test]
    fn extracts_from_url_and_title_ignoring_start() {
        // `start` is intentionally not applied — see `YouTubeElement` — so the
        // embed URL has no query string even though the element carries one.
        let html = transform_youtube(
            r#"<youtube url="https://youtu.be/dQw4w9WgXcQ" title="Demo" start="30"></youtube>"#,
            &opts(),
        );
        assert_eq!(
            html,
            r#"<div class="ox-youtube" style="aspect-ratio: 16/9;"><iframe src="https://www.youtube-nocookie.com/embed/dQw4w9WgXcQ" title="Demo" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen loading="lazy"></iframe></div>"#
        );
    }

    #[test]
    fn passes_through_when_no_element() {
        let html = r#"<p>Plain prose with a <a href="/x">link</a> and no embeds.</p>"#;
        assert_eq!(transform_youtube(html, &opts()), html);
    }

    #[test]
    fn leaves_element_untouched_when_id_invalid() {
        let html = r#"<youtube id="not-a-valid-id"></youtube>"#;
        assert_eq!(transform_youtube(html, &opts()), html);
    }

    #[test]
    fn does_not_match_youtuber() {
        let html = r#"<youtuber id="dQw4w9WgXcQ"></youtuber>"#;
        assert_eq!(transform_youtube(html, &opts()), html);
    }

    #[test]
    fn handles_self_closing() {
        let html = transform_youtube(r#"<youtube id="dQw4w9WgXcQ" />"#, &opts());
        assert!(html.starts_with(r#"<div class="ox-youtube""#));
        assert!(html.ends_with("></iframe></div>"));
    }

    #[test]
    fn non_privacy_and_no_fullscreen_no_lazy() {
        let options = YouTubeEmbedOptions {
            privacy_enhanced: false,
            aspect_ratio: "4/3".to_string(),
            allow_fullscreen: false,
            lazy_load: false,
        };
        let html = transform_youtube(r#"<youtube id="dQw4w9WgXcQ"></youtube>"#, &options);
        assert_eq!(
            html,
            r#"<div class="ox-youtube" style="aspect-ratio: 4/3;"><iframe src="https://www.youtube.com/embed/dQw4w9WgXcQ" title="YouTube video dQw4w9WgXcQ" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin"></iframe></div>"#
        );
    }
}
