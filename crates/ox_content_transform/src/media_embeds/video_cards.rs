use super::html::{ComponentElement, attr};
use super::provider_cards::{
    Card, body_text, first_attr, host_in, is_safe_https_url, parse_https_url, path_segments,
    provider_url, render_card,
};

pub(super) fn render_vimeo(element: &ComponentElement<'_>) -> Option<String> {
    render_video_card(element, Provider::Vimeo)
}

pub(super) fn render_twitch(element: &ComponentElement<'_>) -> Option<String> {
    render_video_card(element, Provider::Twitch)
}

/// Loom demo recordings: `/share/{id}`, and `/embed/{id}` when already
/// embedded. `/share/folder/{id}` is a folder, not a recording.
pub(super) fn render_loom(element: &ComponentElement<'_>) -> Option<String> {
    render_video_card(element, Provider::Loom)
}

/// asciinema terminal recordings: `/a/{id}`, numeric on the public instance.
pub(super) fn render_asciinema(element: &ComponentElement<'_>) -> Option<String> {
    render_video_card(element, Provider::Asciinema)
}

enum Provider {
    Vimeo,
    Twitch,
    Loom,
    Asciinema,
}

struct VideoReference {
    modifier: &'static str,
    network: &'static str,
    title: String,
    author: Option<String>,
    source_label: &'static str,
}

fn render_video_card(element: &ComponentElement<'_>, provider: Provider) -> Option<String> {
    let href = provider_url(element)?;
    let reference = video_reference(href, provider)?;
    let title = first_attr(element, &["title", "name"]).unwrap_or(reference.title.as_str());
    let author = first_attr(element, &["author", "channel", "channelName", "creator"])
        .or(reference.author.as_deref());
    let iframe = first_attr(element, &["embed", "embedUrl", "iframe", "iframeSrc"])
        .filter(|value| is_video_embed(value, reference.network));

    Some(render_card(Card {
        modifier: reference.modifier,
        network: reference.network,
        href,
        title,
        body: body_text(element)
            .or_else(|| attr(element, "description"))
            .or_else(|| attr(element, "excerpt")),
        source_label: reference.source_label,
        image: first_attr(element, &["image", "thumbnail", "preview"])
            .filter(|value| is_safe_https_url(value)),
        avatar: None,
        author,
        date: first_attr(element, &["dateTime", "publishedAt", "createdAt", "date"]),
        date_label: first_attr(element, &["dateLabel", "publishedLabel", "createdLabel"]),
        meta: vec![
            ("Duration", attr(element, "duration")),
            ("Status", first_attr(element, &["status", "live", "liveState"])),
            ("Views", first_attr(element, &["views", "viewCount"])),
        ],
        iframe,
    }))
}

fn video_reference(input: &str, provider: Provider) -> Option<VideoReference> {
    let parsed = parse_https_url(input)?;
    let segments = path_segments(parsed.path);
    match provider {
        Provider::Vimeo => vimeo_reference(&parsed.host, &segments),
        Provider::Twitch => twitch_reference(&parsed.host, &segments),
        Provider::Loom => loom_reference(&parsed.host, &segments),
        Provider::Asciinema => asciinema_reference(&parsed.host, &segments),
    }
}

fn loom_reference(host: &str, segments: &[&str]) -> Option<VideoReference> {
    if !host_in(host, &["loom.com", "www.loom.com"]) {
        return None;
    }
    let id = match segments {
        // A folder shares the `/share/` prefix but is a listing, not a video.
        ["share", "folder", ..] => return None,
        ["share" | "embed", id, ..] => id,
        _ => return None,
    };
    Some(VideoReference {
        modifier: "loom",
        network: "Loom",
        title: format!("Loom {}", safe_slug(id)?),
        author: None,
        source_label: "Watch on Loom",
    })
}

/// asciinema is self-hostable, so the public host is the one that resolves
/// here; a private instance still renders through the `embed` attribute.
fn asciinema_reference(host: &str, segments: &[&str]) -> Option<VideoReference> {
    if !host_in(host, &["asciinema.org", "www.asciinema.org"]) {
        return None;
    }
    let ["a", id, ..] = segments else {
        return None;
    };
    Some(VideoReference {
        modifier: "asciinema",
        network: "asciinema",
        title: format!("Cast {}", safe_slug(id)?),
        author: None,
        source_label: "Play recording",
    })
}

fn vimeo_reference(host: &str, segments: &[&str]) -> Option<VideoReference> {
    if host_in(host, &["player.vimeo.com"]) {
        if segments.first() != Some(&"video") {
            return None;
        }
        let video_id = safe_numeric_id(segments.get(1)?)?;
        return Some(VideoReference {
            modifier: "vimeo",
            network: "Vimeo",
            title: format!("Vimeo video {video_id}"),
            author: None,
            source_label: "Open video",
        });
    }
    if !host_in(host, &["vimeo.com", "www.vimeo.com"]) {
        return None;
    }
    let video_id = segments
        .iter()
        .rev()
        .find_map(|segment| safe_numeric_id(segment))
        .filter(|_id| segments.contains(&"video") || segments.len() <= 3)?;
    Some(VideoReference {
        modifier: "vimeo",
        network: "Vimeo",
        title: format!("Vimeo video {video_id}"),
        author: None,
        source_label: "Open video",
    })
}

fn twitch_reference(host: &str, segments: &[&str]) -> Option<VideoReference> {
    if host_in(host, &["clips.twitch.tv"]) {
        let slug = safe_slug(segments.first()?)?;
        return Some(VideoReference {
            modifier: "twitch",
            network: "Twitch",
            title: titleize(slug),
            author: None,
            source_label: "Open clip",
        });
    }
    if !host_in(host, &["twitch.tv", "www.twitch.tv"]) || segments.is_empty() {
        return None;
    }
    if segments.first() == Some(&"videos") {
        let video_id = safe_numeric_id(segments.get(1)?)?;
        return Some(VideoReference {
            modifier: "twitch",
            network: "Twitch",
            title: format!("Twitch video {video_id}"),
            author: None,
            source_label: "Open video",
        });
    }
    let channel = safe_channel(segments.first()?)?;
    if is_reserved_twitch_channel(channel) {
        return None;
    }
    if segments.get(1) == Some(&"clip") {
        let slug = safe_slug(segments.get(2)?)?;
        return Some(VideoReference {
            modifier: "twitch",
            network: "Twitch",
            title: titleize(slug),
            author: Some(channel.to_string()),
            source_label: "Open clip",
        });
    }
    (segments.len() == 1).then(|| VideoReference {
        modifier: "twitch",
        network: "Twitch",
        title: format!("{channel} on Twitch"),
        author: Some(channel.to_string()),
        source_label: "Open channel",
    })
}

fn is_video_embed(input: &str, network: &str) -> bool {
    if network == "Loom" {
        return parse_https_url(input)
            .is_some_and(|parsed| host_in(&parsed.host, &["loom.com", "www.loom.com"]));
    }
    if network == "asciinema" {
        return parse_https_url(input)
            .is_some_and(|parsed| host_in(&parsed.host, &["asciinema.org", "www.asciinema.org"]));
    }
    let Some(parsed) = parse_https_url(input) else {
        return false;
    };
    match network {
        "Vimeo" => {
            host_in(&parsed.host, &["player.vimeo.com"])
                && path_segments(parsed.path).first() == Some(&"video")
        }
        "Twitch" => is_twitch_player_embed(&parsed),
        _ => false,
    }
}

fn is_twitch_player_embed(parsed: &super::provider_cards::ParsedHttpsUrl<'_>) -> bool {
    if !query_has_param(parsed.query, "parent") {
        return false;
    }
    if host_in(&parsed.host, &["player.twitch.tv"]) {
        query_has_param(parsed.query, "channel") || query_has_param(parsed.query, "video")
    } else {
        host_in(&parsed.host, &["clips.twitch.tv"])
            && parsed.path == "/embed"
            && query_has_param(parsed.query, "clip")
    }
}

fn query_has_param(query: Option<&str>, name: &str) -> bool {
    query.is_some_and(|query| {
        query.split('&').any(|part| {
            let Some((key, value)) = part.split_once('=') else {
                return false;
            };
            let key = key.strip_prefix("amp;").unwrap_or(key);
            key == name && !value.is_empty()
        })
    })
}

fn safe_numeric_id(value: &str) -> Option<&str> {
    (value.len() <= 32 && value.chars().all(|ch| ch.is_ascii_digit())).then_some(value)
}

fn safe_channel(value: &str) -> Option<&str> {
    (value.len() <= 25 && value.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '_'))
        .then_some(value)
}

fn is_reserved_twitch_channel(value: &str) -> bool {
    ["directory", "downloads", "jobs", "p", "settings", "subscriptions", "turbo", "videos"]
        .iter()
        .any(|reserved| value.eq_ignore_ascii_case(reserved))
}

fn safe_slug(value: &str) -> Option<&str> {
    (value.len() <= 128
        && value.chars().all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_')))
    .then_some(value)
}

fn titleize(value: &str) -> String {
    value.replace(['-', '_'], " ")
}
