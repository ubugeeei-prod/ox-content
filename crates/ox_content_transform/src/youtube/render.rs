use super::YouTubeEmbedOptions;

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

fn build_embed_url(video_id: &str, options: &YouTubeEmbedOptions, start: Option<u32>) -> String {
    // The video id is already restricted to `[A-Za-z0-9_-]{11}`, so it is
    // safe to interpolate. `start` is digits-only `u32` from the parser.
    let domain =
        if options.privacy_enhanced { "www.youtube-nocookie.com" } else { "www.youtube.com" };
    match start {
        Some(seconds) => format!("https://{domain}/embed/{video_id}?start={seconds}"),
        None => format!("https://{domain}/embed/{video_id}"),
    }
}

pub(super) fn render_embed(
    video_id: &str,
    options: &YouTubeEmbedOptions,
    title: Option<&str>,
    start: Option<u32>,
) -> String {
    let embed_url = build_embed_url(video_id, options, start);
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
