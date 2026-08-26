use super::html::{ComponentElement, attr};
use super::render::escape_attr;

const EMBED_ORIGIN: &str = "https://embed.music.apple.com";
const SHARE_HOST: &str = "music.apple.com";
const EMBED_HOST: &str = "embed.music.apple.com";

pub(super) fn render_apple_music(element: &ComponentElement<'_>) -> Option<String> {
    let url = attr(element, "url")
        .or_else(|| attr(element, "href"))
        .or_else(|| attr(element, "src"))
        .or_else(|| (!element.body.trim().is_empty()).then(|| element.body.trim()))?;
    let embed = apple_music_embed_url(url)?;
    let height = if embed.contains("/song/") || embed.contains("?i=") { "175" } else { "450" };

    let mut html = String::new();
    html.push_str("<iframe class=\"ox-apple-music\" src=\"");
    escape_attr(&embed, &mut html);
    html.push_str("\" title=\"Apple Music\" width=\"100%\" height=\"");
    html.push_str(height);
    html.push_str("\" loading=\"lazy\" allow=\"autoplay *; encrypted-media *; fullscreen *; clipboard-write\" ");
    html.push_str("sandbox=\"allow-forms allow-popups allow-same-origin allow-scripts allow-storage-access-by-user-activation allow-top-navigation-by-user-activation\" ");
    html.push_str("referrerpolicy=\"strict-origin-when-cross-origin\"></iframe>");
    Some(html)
}

fn apple_music_embed_url(input: &str) -> Option<String> {
    let parsed = parse_https_url(input.trim())?;
    if parsed.host != SHARE_HOST && parsed.host != EMBED_HOST {
        return None;
    }
    let path = normalize_path(parsed.path)?;
    let query = song_selection_query(parsed.query)?;
    if query.is_empty() {
        Some(format!("{EMBED_ORIGIN}{path}"))
    } else {
        Some(format!("{EMBED_ORIGIN}{path}?{query}"))
    }
}

struct ParsedHttpsUrl<'a> {
    host: String,
    path: &'a str,
    query: Option<&'a str>,
}

fn parse_https_url(input: &str) -> Option<ParsedHttpsUrl<'_>> {
    let rest = input.strip_prefix("https://")?;
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

    let (path, query) = if let Some(path) = path_and_query.strip_prefix('?') {
        ("/", Some(path))
    } else {
        match path_and_query.split_once('?') {
            Some((path, query)) => (path, Some(query)),
            None => (path_and_query, None),
        }
    };
    if path.is_empty() {
        return None;
    }
    Some(ParsedHttpsUrl { host, path, query })
}

fn normalize_path(path: &str) -> Option<String> {
    if !path.starts_with('/') || path.contains('\\') {
        return None;
    }
    let segments: Vec<&str> = path.split('/').skip(1).collect();
    if segments.len() != 3 && segments.len() != 4 {
        return None;
    }
    if segments.iter().any(|segment| segment.is_empty()) {
        return None;
    }

    let storefront = segments[0].to_ascii_lowercase();
    let kind = segments[1].to_ascii_lowercase();
    if !is_storefront(&storefront) || !is_kind(&kind) {
        return None;
    }

    let (slug, id) =
        if segments.len() == 4 { (Some(segments[2]), segments[3]) } else { (None, segments[2]) };
    if slug.is_some_and(|value| !is_safe_slug(value)) || !is_safe_id(&kind, id) {
        return None;
    }

    Some(match slug {
        Some(slug) => format!("/{storefront}/{kind}/{slug}/{id}"),
        None => format!("/{storefront}/{kind}/{id}"),
    })
}

fn is_storefront(value: &str) -> bool {
    value.len() == 2 && value.bytes().all(|byte| byte.is_ascii_alphabetic())
}

fn is_kind(value: &str) -> bool {
    matches!(value, "album" | "playlist" | "song" | "artist" | "music-video")
}

fn is_safe_id(kind: &str, id: &str) -> bool {
    if kind == "playlist" {
        let Some(rest) = id.strip_prefix("pl.") else {
            return false;
        };
        !rest.is_empty()
            && rest
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    } else {
        !id.is_empty() && id.bytes().all(|byte| byte.is_ascii_digit())
    }
}

fn is_safe_slug(value: &str) -> bool {
    if value.is_empty() || value == "." || value == ".." || decoded_is_dot_segment(value) {
        return false;
    }
    if value.chars().any(|ch| {
        ch.is_control() || matches!(ch, '/' | '\\' | '?' | '#' | '@' | ':' | '"' | '\'' | '<' | '>')
    }) {
        return false;
    }
    has_valid_percent_encoding(value)
}

fn decoded_is_dot_segment(value: &str) -> bool {
    let decoded = value.to_ascii_lowercase().replace("%2e", ".");
    decoded == "." || decoded == ".."
}

fn has_valid_percent_encoding(value: &str) -> bool {
    let bytes = value.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] != b'%' {
            index += 1;
            continue;
        }
        if index + 2 >= bytes.len()
            || !bytes[index + 1].is_ascii_hexdigit()
            || !bytes[index + 2].is_ascii_hexdigit()
        {
            return false;
        }
        let hex = &value[index + 1..index + 3];
        if hex.eq_ignore_ascii_case("2f")
            || hex.eq_ignore_ascii_case("5c")
            || hex.eq_ignore_ascii_case("00")
        {
            return false;
        }
        index += 3;
    }
    true
}

fn song_selection_query(query: Option<&str>) -> Option<String> {
    let Some(query) = query.filter(|value| !value.is_empty()) else {
        return Some(String::new());
    };

    let mut song_id = None;
    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        if !is_safe_query_token(key) || !is_safe_query_token(value) {
            return None;
        }
        if key == "i" {
            if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
                return None;
            }
            if song_id.is_some() {
                return None;
            }
            song_id = Some(value);
        }
    }
    Some(song_id.map_or_else(String::new, |id| format!("i={id}")))
}

fn is_safe_query_token(value: &str) -> bool {
    value.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

#[cfg(test)]
mod url_tests {
    use super::apple_music_embed_url;

    #[test]
    fn normalizes_localized_album_share_url() {
        assert_eq!(
            apple_music_embed_url(
                "https://music.apple.com/gb/album/1989-taylors-version/1708308989"
            ),
            Some("https://embed.music.apple.com/gb/album/1989-taylors-version/1708308989".into())
        );
    }

    #[test]
    fn keeps_playlist_path_and_id() {
        assert_eq!(
            apple_music_embed_url("https://music.apple.com/us/playlist/todays-hits/pl.u-abc123"),
            Some("https://embed.music.apple.com/us/playlist/todays-hits/pl.u-abc123".into())
        );
    }

    #[test]
    fn keeps_song_selection_query() {
        assert_eq!(
            apple_music_embed_url(
                "https://music.apple.com/gb/album/1989-taylors-version/1708308989?i=1708309399"
            ),
            Some(
                "https://embed.music.apple.com/gb/album/1989-taylors-version/1708308989?i=1708309399"
                    .into()
            )
        );
    }

    #[test]
    fn accepts_already_embedded_url() {
        assert_eq!(
            apple_music_embed_url(
                "https://embed.music.apple.com/us/album/folklore/1524801260?i=1524801265"
            ),
            Some("https://embed.music.apple.com/us/album/folklore/1524801260?i=1524801265".into())
        );
    }

    #[test]
    fn rejects_non_https_lookalikes_credentials_and_malformed_paths() {
        for input in [
            "http://music.apple.com/us/album/folklore/1524801260",
            "https://music.apple.com.evil.com/us/album/folklore/1524801260",
            "https://embed.music.apple.com.evil.com/us/album/folklore/1524801260",
            "https://not-music.apple.com/us/album/folklore/1524801260",
            "https://user:pass@music.apple.com/us/album/folklore/1524801260",
            "https://music.apple.com/us/album",
            "https://music.apple.com/us/not-a-kind/folklore/1524801260",
            "https://music.apple.com/us/album/../1524801260",
            "https://music.apple.com/us/album/folklore/1524801260#oops",
            "https://music.apple.com/us/album/folklore/1524801260?i=\"><script>",
        ] {
            assert_eq!(apple_music_embed_url(input), None, "{input}");
        }
    }
}
