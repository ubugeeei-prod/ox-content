use super::html::{ComponentElement, attr};
use super::render::{escape_attr, escape_text};

const SHARE_HOST: &str = "speakerdeck.com";
const PLAYER_ORIGIN: &str = "https://speakerdeck.com/player/";

pub(super) fn render_speaker_deck(element: &ComponentElement<'_>) -> Option<String> {
    let url = attr(element, "url")
        .or_else(|| attr(element, "href"))
        .or_else(|| attr(element, "src"))
        .or_else(|| (!element.body.trim().is_empty()).then(|| element.body.trim()))?;
    if is_dangerous_url(url) {
        return None;
    }
    let parsed = parse_speaker_deck_url(url)?;
    let player =
        attr(element, "player").or_else(|| attr(element, "id")).or(parsed.player.as_deref());
    let title = attr(element, "title");
    let author = attr(element, "author").or_else(|| attr(element, "authorName"));
    let preview = attr(element, "preview")
        .or_else(|| attr(element, "thumbnail"))
        .or_else(|| attr(element, "image"))
        .filter(|value| is_safe_https_url(value));

    if let Some(player) = player.filter(|id| is_player_id(id)) {
        return Some(render_resolved(&parsed.href, player, title, author));
    }
    Some(render_fallback(&parsed.href, title, author, preview, parsed.user, parsed.slug))
}

struct ParsedSpeakerDeck {
    href: String,
    player: Option<String>,
    user: Option<String>,
    slug: Option<String>,
}

fn parse_speaker_deck_url(input: &str) -> Option<ParsedSpeakerDeck> {
    let parsed = parse_https_url(input.trim())?;
    if parsed.host != SHARE_HOST {
        return None;
    }
    let segments: Vec<&str> =
        parsed.path.split('/').filter(|segment| !segment.is_empty()).collect();
    match segments.as_slice() {
        ["player", id] if is_player_id(id) => Some(ParsedSpeakerDeck {
            href: format!("{PLAYER_ORIGIN}{id}"),
            player: Some((*id).to_string()),
            user: None,
            slug: None,
        }),
        [user, slug] if *user != "player" && is_path_segment(user) && is_path_segment(slug) => {
            Some(ParsedSpeakerDeck {
                href: format!("https://speakerdeck.com/{user}/{slug}"),
                player: None,
                user: Some((*user).to_string()),
                slug: Some((*slug).to_string()),
            })
        }
        [user] if *user != "player" && is_path_segment(user) => Some(ParsedSpeakerDeck {
            href: format!("https://speakerdeck.com/{user}"),
            player: None,
            user: Some((*user).to_string()),
            slug: None,
        }),
        _ => None,
    }
}

struct ParsedHttpsUrl<'a> {
    host: String,
    path: &'a str,
}

fn parse_https_url(input: &str) -> Option<ParsedHttpsUrl<'_>> {
    let rest = input.strip_prefix("https://").or_else(|| input.strip_prefix("HTTPS://"))?;
    let authority_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let authority = &rest[..authority_end];
    if authority.is_empty() || authority.contains('@') || authority.contains(':') {
        return None;
    }
    let host = authority.to_ascii_lowercase();
    if host.ends_with('.') {
        return None;
    }

    let after_authority = &rest[authority_end..];
    let (path_and_query, fragment) = after_authority
        .split_once('#')
        .map_or((after_authority, None), |(path, fragment)| (path, Some(fragment)));
    if fragment.is_some_and(|value| !value.is_empty()) {
        return None;
    }

    let path = path_and_query.split_once('?').map_or(path_and_query, |(path, _)| path);
    if path.contains('\\') {
        return None;
    }
    Some(ParsedHttpsUrl { host, path: if path.is_empty() { "/" } else { path } })
}

fn render_resolved(href: &str, player: &str, title: Option<&str>, author: Option<&str>) -> String {
    let mut html = String::new();
    html.push_str("<figure class=\"ox-speaker-deck\">");
    html.push_str("<iframe class=\"ox-speaker-deck\" src=\"");
    escape_attr(&format!("{PLAYER_ORIGIN}{player}"), &mut html);
    html.push_str("\" title=\"");
    escape_attr(title.unwrap_or("Speaker Deck"), &mut html);
    html.push_str("\" width=\"100%\" height=\"480\" loading=\"lazy\" allowfullscreen ");
    html.push_str("sandbox=\"allow-scripts allow-same-origin allow-popups allow-popups-to-escape-sandbox allow-presentation\" ");
    html.push_str("referrerpolicy=\"strict-origin-when-cross-origin\"></iframe>");
    if title.is_some() || author.is_some() {
        html.push_str("<figcaption class=\"ox-speaker-deck__meta\"><a href=\"");
        escape_attr(href, &mut html);
        html.push_str("\" target=\"_blank\" rel=\"noopener noreferrer\">");
        if let Some(title) = title {
            html.push_str("<strong class=\"ox-speaker-deck__title\">");
            escape_text(title, &mut html);
            html.push_str("</strong>");
        }
        if let Some(author) = author {
            html.push_str("<span class=\"ox-speaker-deck__author\">");
            escape_text(author, &mut html);
            html.push_str("</span>");
        }
        html.push_str("</a></figcaption>");
    }
    html.push_str("</figure>");
    html
}

fn render_fallback(
    href: &str,
    title: Option<&str>,
    author: Option<&str>,
    preview: Option<&str>,
    user: Option<String>,
    slug: Option<String>,
) -> String {
    let title = title.map(str::to_string).or_else(|| slug.as_deref().map(humanize_slug));
    let author = author.map(str::to_string).or(user);
    let mut html = String::new();
    html.push_str("<a class=\"ox-speaker-deck ox-speaker-deck--fallback\" href=\"");
    escape_attr(href, &mut html);
    html.push_str("\" target=\"_blank\" rel=\"noopener noreferrer\">");
    if let Some(preview) = preview {
        html.push_str("<img class=\"ox-speaker-deck__preview\" src=\"");
        escape_attr(preview, &mut html);
        html.push_str("\" alt=\"\" loading=\"lazy\" decoding=\"async\">");
    }
    html.push_str("<span class=\"ox-speaker-deck__title\">");
    escape_text(title.as_deref().unwrap_or("Speaker Deck"), &mut html);
    html.push_str("</span>");
    if let Some(author) = author {
        html.push_str("<span class=\"ox-speaker-deck__author\">");
        escape_text(&author, &mut html);
        html.push_str("</span>");
    }
    html.push_str("</a>");
    html
}

fn humanize_slug(slug: &str) -> String {
    let mut out = String::with_capacity(slug.len());
    let mut chars = slug.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '-' || ch == '_' {
            if out.ends_with(' ') || out.is_empty() {
                continue;
            }
            if chars.peek().is_some() {
                out.push(' ');
            }
            continue;
        }
        if out.is_empty() || out.ends_with(' ') {
            out.extend(ch.to_uppercase());
        } else {
            out.push(ch);
        }
    }
    if out.is_empty() { "Speaker Deck".into() } else { out }
}

fn is_dangerous_url(input: &str) -> bool {
    let compact: String = input
        .chars()
        .filter(|ch| !ch.is_ascii_whitespace())
        .map(|ch| ch.to_ascii_lowercase())
        .collect();
    compact.starts_with("javascript:")
        || compact.starts_with("data:")
        || compact.starts_with("vbscript:")
        || compact.starts_with("blob:")
}

fn is_safe_https_url(input: &str) -> bool {
    let parsed = parse_https_url(input.trim());
    parsed.is_some_and(|parsed| {
        !parsed.host.is_empty()
            && !input.chars().any(|ch| ch.is_control() || matches!(ch, '<' | '>' | '"' | '\''))
    })
}

fn is_player_id(value: &str) -> bool {
    (8..=64).contains(&value.len()) && value.bytes().all(|byte| byte.is_ascii_alphanumeric())
}

fn is_path_segment(value: &str) -> bool {
    let bytes = value.as_bytes();
    (1..=64).contains(&bytes.len())
        && bytes[0].is_ascii_alphanumeric()
        && bytes.iter().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        && value != "."
        && value != ".."
}

#[cfg(test)]
mod url_tests {
    use super::parse_speaker_deck_url;

    #[test]
    fn parses_player_and_share_urls() {
        let player =
            parse_speaker_deck_url("https://speakerdeck.com/player/abcdef1234567890").unwrap();
        assert_eq!(player.player.as_deref(), Some("abcdef1234567890"));
        let share = parse_speaker_deck_url("https://speakerdeck.com/jane/my-cool-talk").unwrap();
        assert_eq!(share.user.as_deref(), Some("jane"));
        assert_eq!(share.slug.as_deref(), Some("my-cool-talk"));
        assert!(share.player.is_none());
    }

    #[test]
    fn rejects_dangerous_and_lookalike_urls() {
        for input in [
            "javascript:alert(1)",
            "data:text/html,hi",
            "http://speakerdeck.com/jane/talk",
            "https://speakerdeck.com.evil.com/jane/talk",
            "https://user:pass@speakerdeck.com/jane/talk",
            "https://speakerdeck.com/jane/talk#oops",
            "https://speakerdeck.com/player/short",
        ] {
            assert!(parse_speaker_deck_url(input).is_none(), "{input}");
        }
    }
}
