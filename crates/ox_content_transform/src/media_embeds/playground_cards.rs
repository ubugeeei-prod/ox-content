use super::html::{ComponentElement, attr};
use super::provider_cards::{
    Card, body_text, first_attr, host_in, is_safe_https_url, parse_https_url, provider_url,
    render_card,
};

pub(super) fn render_codepen(element: &ComponentElement<'_>) -> Option<String> {
    render_playground_card(element, Provider::CodePen)
}

pub(super) fn render_jsfiddle(element: &ComponentElement<'_>) -> Option<String> {
    render_playground_card(element, Provider::JsFiddle)
}

pub(super) fn render_observable(element: &ComponentElement<'_>) -> Option<String> {
    render_playground_card(element, Provider::Observable)
}

enum Provider {
    CodePen,
    JsFiddle,
    Observable,
}

struct PlaygroundReference {
    modifier: &'static str,
    network: &'static str,
    title: String,
    author: Option<String>,
}

fn render_playground_card(element: &ComponentElement<'_>, provider: Provider) -> Option<String> {
    let href = provider_url(element)?;
    let reference = playground_reference(href, provider)?;
    let title = first_attr(element, &["title", "name"]).unwrap_or(reference.title.as_str());
    let author =
        first_attr(element, &["author", "authorName", "user"]).or(reference.author.as_deref());
    let iframe = first_attr(element, &["embed", "embedUrl", "iframe", "iframeSrc"])
        .filter(|value| is_playground_embed(value, reference.network));

    Some(render_card(Card {
        modifier: reference.modifier,
        network: reference.network,
        href,
        title,
        body: body_text(element)
            .or_else(|| attr(element, "description"))
            .or_else(|| attr(element, "excerpt")),
        source_label: "Open playground",
        image: first_attr(element, &["image", "thumbnail", "preview"])
            .filter(|value| is_safe_https_url(value)),
        avatar: None,
        author,
        date: first_attr(element, &["dateTime", "updatedAt", "createdAt", "date"]),
        date_label: first_attr(element, &["dateLabel", "updatedLabel", "createdLabel"]),
        meta: vec![("Language", attr(element, "language")), ("Runtime", attr(element, "runtime"))],
        iframe,
    }))
}

fn playground_reference(input: &str, provider: Provider) -> Option<PlaygroundReference> {
    let parsed = parse_https_url(input)?;
    let segments = path_segments(parsed.path);
    match provider {
        Provider::CodePen => codepen_reference(&parsed.host, &segments),
        Provider::JsFiddle => jsfiddle_reference(&parsed.host, &segments),
        Provider::Observable => observable_reference(&parsed.host, &segments),
    }
}

fn codepen_reference(host: &str, segments: &[&str]) -> Option<PlaygroundReference> {
    if !host_in(host, &["codepen.io"]) || segments.get(1) != Some(&"pen") {
        return None;
    }
    let author = safe_segment(segments.first()?)?.to_string();
    let slug = safe_segment(segments.get(2)?)?;
    Some(PlaygroundReference {
        modifier: "codepen",
        network: "CodePen",
        title: titleize(slug),
        author: Some(author),
    })
}

fn jsfiddle_reference(host: &str, segments: &[&str]) -> Option<PlaygroundReference> {
    if !host_in(host, &["jsfiddle.net", "www.jsfiddle.net"]) || segments.is_empty() {
        return None;
    }
    let segments: Vec<&str> =
        segments.iter().copied().take(3).map(safe_segment).collect::<Option<Vec<_>>>()?;
    let slug = segments.get(1).or_else(|| segments.first())?;
    Some(PlaygroundReference {
        modifier: "jsfiddle",
        network: "JSFiddle",
        title: titleize(slug),
        author: (segments.len() > 1).then(|| segments[0].to_string()),
    })
}

fn observable_reference(host: &str, segments: &[&str]) -> Option<PlaygroundReference> {
    if !host_in(host, &["observablehq.com"]) || segments.len() < 2 {
        return None;
    }
    let first = safe_observable_segment(segments.first()?)?;
    let second = safe_observable_segment(segments.get(1)?)?;
    let (title, author) = if first.starts_with('@') {
        (titleize(second), Some(first.to_string()))
    } else if first == "d" {
        (format!("Notebook {second}"), None)
    } else {
        return None;
    };
    Some(PlaygroundReference { modifier: "observable", network: "Observable", title, author })
}

fn is_playground_embed(input: &str, network: &str) -> bool {
    let Some(parsed) = parse_https_url(input) else {
        return false;
    };
    match network {
        "CodePen" => host_in(&parsed.host, &["codepen.io"]) && parsed.path.contains("/embed/"),
        "JSFiddle" => {
            host_in(&parsed.host, &["jsfiddle.net", "www.jsfiddle.net"])
                && parsed.path.contains("/embedded/")
        }
        "Observable" => {
            host_in(&parsed.host, &["observablehq.com"]) && parsed.path.starts_with("/embed/")
        }
        _ => false,
    }
}

fn path_segments(path: &str) -> Vec<&str> {
    path.split('/').filter(|segment| !segment.is_empty()).collect()
}

fn safe_segment(value: &str) -> Option<&str> {
    (value.len() <= 128
        && value.chars().all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.')))
    .then_some(value)
}

fn safe_observable_segment(value: &str) -> Option<&str> {
    (value.len() <= 128
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '@' | '-' | '_' | '.')))
    .then_some(value)
}

fn titleize(value: &str) -> String {
    value.replace(['-', '_'], " ")
}
