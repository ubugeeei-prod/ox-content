use super::html::{ComponentElement, attr, parse_attrs};
use super::render::{escape_attr, escape_text};
use crate::html_scan::find_ci;

pub(super) fn render_audio(element: &ComponentElement<'_>) -> Option<String> {
    render_player(element, Kind::Audio)
}

pub(super) fn render_video(element: &ComponentElement<'_>) -> Option<String> {
    render_player(element, Kind::Video)
}

#[derive(Clone, Copy)]
enum Kind {
    Audio,
    Video,
}

impl Kind {
    fn tag(self) -> &'static str {
        match self {
            Self::Audio => "audio",
            Self::Video => "video",
        }
    }

    fn class_name(self) -> &'static str {
        match self {
            Self::Audio => "ox-audio",
            Self::Video => "ox-video",
        }
    }

    fn fallback_label(self) -> &'static str {
        match self {
            Self::Audio => "Audio",
            Self::Video => "Video",
        }
    }
}

fn render_player(element: &ComponentElement<'_>, kind: Kind) -> Option<String> {
    let src = media_src(element)?;
    let title = attr(element, "title");
    let poster = (matches!(kind, Kind::Video))
        .then(|| attr(element, "poster"))
        .flatten()
        .filter(|value| is_safe_media_url(value));
    let transcript = attr(element, "transcript").filter(|value| is_safe_media_url(value));
    let download = attr(element, "download").filter(|value| is_safe_media_url(value));
    let tracks = collect_tracks(element);
    let (width, height) = video_size(element, kind);

    let caption = title.is_some() || transcript.is_some() || download.is_some();
    let mut html = String::new();
    if caption {
        html.push_str("<figure>");
    }
    html.push('<');
    html.push_str(kind.tag());
    html.push_str(" class=\"");
    html.push_str(kind.class_name());
    html.push_str("\" controls preload=\"metadata\"");
    if matches!(kind, Kind::Video) {
        html.push_str(" playsinline");
    }
    html.push_str(" src=\"");
    escape_attr(src, &mut html);
    html.push_str("\" aria-label=\"");
    escape_attr(title.unwrap_or_else(|| kind.fallback_label()), &mut html);
    html.push('"');
    if let Some(poster) = poster {
        html.push_str(" poster=\"");
        escape_attr(poster, &mut html);
        html.push('"');
    }
    if let Some((width, height)) = width.zip(height) {
        html.push_str(" width=\"");
        html.push_str(&width.to_string());
        html.push_str("\" height=\"");
        html.push_str(&height.to_string());
        html.push('"');
    }
    html.push('>');
    for track in tracks {
        push_track(&mut html, &track);
    }
    html.push_str("</");
    html.push_str(kind.tag());
    html.push('>');
    push_caption(&mut html, title, transcript, download);
    if caption {
        html.push_str("</figure>");
    }
    Some(html)
}

fn media_src<'a>(element: &'a ComponentElement<'_>) -> Option<&'a str> {
    attr(element, "src")
        .or_else(|| attr(element, "url"))
        .or_else(|| attr(element, "href"))
        .filter(|value| is_safe_media_url(value))
}

fn video_size(element: &ComponentElement<'_>, kind: Kind) -> (Option<u32>, Option<u32>) {
    if !matches!(kind, Kind::Video) {
        return (None, None);
    }
    match (positive_int(attr(element, "width")), positive_int(attr(element, "height"))) {
        (Some(width), Some(height)) => (Some(width), Some(height)),
        _ => (Some(16), Some(9)),
    }
}

fn positive_int(value: Option<&str>) -> Option<u32> {
    let parsed = value?.parse::<u32>().ok()?;
    (parsed > 0).then_some(parsed)
}

struct Track<'a> {
    src: &'a str,
    kind: &'a str,
    srclang: Option<&'a str>,
    label: Option<&'a str>,
    default: bool,
}

fn collect_tracks<'a>(element: &'a ComponentElement<'_>) -> Vec<Track<'a>> {
    let mut tracks = Vec::new();
    if let Some(src) = attr(element, "captions")
        .or_else(|| attr(element, "subtitles"))
        .filter(|value| is_safe_media_url(value))
    {
        tracks.push(Track {
            src,
            kind: attr(element, "kind").filter(|value| is_track_kind(value)).unwrap_or("captions"),
            srclang: attr(element, "srclang").filter(|value| is_srclang(value)),
            label: attr(element, "label").or_else(|| attr(element, "captionLabel")),
            default: true,
        });
    }
    tracks.extend(parse_body_tracks(element.body));
    tracks
}

fn parse_body_tracks(body: &str) -> Vec<Track<'_>> {
    let mut tracks = Vec::new();
    let mut search = 0;
    while let Some(start) = find_ci(body, search, "<track") {
        let after_name = start + "<track".len();
        let Some(rel_end) = body[after_name..].find('>') else {
            break;
        };
        let end = after_name + rel_end;
        search = end + 1;
        let attrs = parse_attrs(&body[after_name..end]);
        let Some(src) =
            attrs.iter().find(|(key, _)| key.eq_ignore_ascii_case("src")).map(|(_, v)| *v)
        else {
            continue;
        };
        if !is_safe_media_url(src) {
            continue;
        }
        let kind = attrs
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case("kind"))
            .map(|(_, v)| *v)
            .filter(|value| is_track_kind(value))
            .unwrap_or("captions");
        tracks.push(Track {
            src,
            kind,
            srclang: attrs
                .iter()
                .find(|(key, _)| key.eq_ignore_ascii_case("srclang"))
                .map(|(_, v)| *v)
                .filter(|value| is_srclang(value)),
            label: attrs.iter().find(|(key, _)| key.eq_ignore_ascii_case("label")).map(|(_, v)| *v),
            default: attrs.iter().any(|(key, _)| key.eq_ignore_ascii_case("default")),
        });
    }
    tracks
}

fn push_track(html: &mut String, track: &Track<'_>) {
    html.push_str("<track kind=\"");
    escape_attr(track.kind, html);
    html.push_str("\" src=\"");
    escape_attr(track.src, html);
    html.push('"');
    if let Some(srclang) = track.srclang {
        html.push_str(" srclang=\"");
        escape_attr(srclang, html);
        html.push('"');
    }
    if let Some(label) = track.label {
        html.push_str(" label=\"");
        escape_attr(label, html);
        html.push('"');
    }
    if track.default {
        html.push_str(" default");
    }
    html.push('>');
}

fn push_caption(
    html: &mut String,
    title: Option<&str>,
    transcript: Option<&str>,
    download: Option<&str>,
) {
    if title.is_none() && transcript.is_none() && download.is_none() {
        return;
    }
    html.push_str("<figcaption>");
    if let Some(title) = title {
        html.push_str("<span class=\"ox-av-title\">");
        escape_text(title, html);
        html.push_str("</span>");
    }
    if let Some(href) = transcript {
        push_link(html, href, "ox-av-transcript", "Transcript", false);
    }
    if let Some(href) = download {
        push_link(html, href, "ox-av-download", "Download", true);
    }
    html.push_str("</figcaption>");
}

fn push_link(html: &mut String, href: &str, class_name: &str, label: &str, download: bool) {
    html.push_str("<a class=\"");
    html.push_str(class_name);
    html.push_str("\" href=\"");
    escape_attr(href, html);
    html.push('"');
    if download {
        html.push_str(" download");
    }
    html.push('>');
    html.push_str(label);
    html.push_str("</a>");
}

fn is_track_kind(value: &str) -> bool {
    matches!(value, "captions" | "subtitles" | "descriptions" | "chapters" | "metadata")
}

fn is_srclang(value: &str) -> bool {
    let len = value.len();
    (2..=16).contains(&len)
        && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}

pub(super) fn is_safe_media_url(src: &str) -> bool {
    let trimmed = src.trim();
    if trimmed.is_empty() {
        return false;
    }
    let compact: String = trimmed
        .chars()
        .filter(|ch| !ch.is_ascii_whitespace())
        .map(|ch| ch.to_ascii_lowercase())
        .collect();
    if compact.starts_with("//")
        || compact.starts_with("javascript:")
        || compact.starts_with("data:")
        || compact.starts_with("vbscript:")
        || compact.starts_with("blob:")
        || compact.starts_with("file:")
    {
        return false;
    }
    if compact.starts_with("https://") {
        return is_safe_https_url(trimmed);
    }
    if has_scheme(&compact) {
        return false;
    }
    is_safe_relative_path(trimmed)
}

fn has_scheme(compact: &str) -> bool {
    let Some(colon) = compact.find(':') else {
        return false;
    };
    let scheme = &compact[..colon];
    let bytes = scheme.as_bytes();
    !bytes.is_empty()
        && bytes[0].is_ascii_alphabetic()
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'-' | b'.'))
}

fn is_safe_https_url(input: &str) -> bool {
    let trimmed = input.trim();
    let Some(rest) = trimmed
        .get(8..)
        .filter(|_| trimmed.get(..8).is_some_and(|s| s.eq_ignore_ascii_case("https://")))
    else {
        return false;
    };
    let authority_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let authority = &rest[..authority_end];
    if authority.is_empty() || authority.contains('@') || authority.contains('\\') {
        return false;
    }
    !trimmed.chars().any(|ch| ch.is_control() || matches!(ch, '<' | '>' | '"' | '\''))
}

fn is_safe_relative_path(input: &str) -> bool {
    let trimmed = input.trim();
    if trimmed.starts_with("//") || trimmed.contains('\\') {
        return false;
    }
    !trimmed.chars().any(|ch| ch.is_control() || matches!(ch, '<' | '>' | '"' | '\''))
}
