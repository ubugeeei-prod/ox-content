use crate::{html_scan::find_ci, JsMediaEmbedsOptions};

pub fn transform_media_embeds(html: &str, options: Option<&JsMediaEmbedsOptions>) -> String {
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
    if options.web_container.unwrap_or(false) && contains_ci(&current, "<webcontainer") {
        current = transform_component(&current, "webcontainer", render_webcontainer);
    }
    current
}

fn has_enabled_embed(options: &JsMediaEmbedsOptions) -> bool {
    options.spotify.unwrap_or(false)
        || options.stack_blitz.unwrap_or(false)
        || options.twitter.unwrap_or(false)
        || options.bluesky.unwrap_or(false)
        || options.web_container.unwrap_or(false)
}

fn contains_ci(html: &str, needle: &str) -> bool {
    find_ci(html, 0, needle).is_some()
}

fn transform_component(
    html: &str,
    name: &str,
    render: fn(&ComponentElement<'_>) -> Option<String>,
) -> String {
    let mut out = String::with_capacity(html.len());
    let mut cursor = 0usize;
    let open = format!("<{name}");

    while let Some(element) = find_component(html, cursor, &open, name) {
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

struct ComponentElement<'a> {
    span: (usize, usize),
    attrs: Vec<(&'a str, &'a str)>,
    body: &'a str,
}

fn find_component<'a>(
    html: &'a str,
    from: usize,
    open: &str,
    name: &str,
) -> Option<ComponentElement<'a>> {
    let bytes = html.as_bytes();
    let mut search = from;
    loop {
        let tag_start = find_ci(html, search, open)?;
        let after_name = tag_start + open.len();
        let boundary = bytes.get(after_name).copied();
        if !matches!(boundary, Some(b) if b == b'>' || b == b'/' || b.is_ascii_whitespace()) {
            search = after_name;
            continue;
        }

        let start_tag = scan_start_tag(html, tag_start)?;
        let attr_end = if start_tag.self_closing { start_tag.inner_end } else { start_tag.tag_end };
        let attrs = parse_attrs(&html[after_name..attr_end]);
        if start_tag.self_closing {
            return Some(ComponentElement { span: (tag_start, start_tag.end), attrs, body: "" });
        }
        let close = format!("</{name}>");
        let close_start = find_ci(html, start_tag.end, &close).unwrap_or(start_tag.end);
        let span_end =
            if close_start == start_tag.end { start_tag.end } else { close_start + close.len() };
        return Some(ComponentElement {
            span: (tag_start, span_end),
            attrs,
            body: &html[start_tag.end..close_start],
        });
    }
}

struct StartTag {
    inner_end: usize,
    tag_end: usize,
    end: usize,
    self_closing: bool,
}

fn scan_start_tag(html: &str, start: usize) -> Option<StartTag> {
    let bytes = html.as_bytes();
    let mut cursor = start;
    let mut quote = None;
    while cursor < bytes.len() {
        match quote {
            Some(q) if bytes[cursor] == q => quote = None,
            Some(_) => {}
            None if bytes[cursor] == b'"' || bytes[cursor] == b'\'' => quote = Some(bytes[cursor]),
            None if bytes[cursor] == b'>' => {
                let self_closing = cursor > start && bytes[cursor - 1] == b'/';
                return Some(StartTag {
                    inner_end: if self_closing { cursor - 1 } else { cursor },
                    tag_end: cursor,
                    end: cursor + 1,
                    self_closing,
                });
            }
            None => {}
        }
        cursor += 1;
    }
    None
}

fn parse_attrs(inner: &str) -> Vec<(&str, &str)> {
    let bytes = inner.as_bytes();
    let mut attrs = Vec::new();
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor >= bytes.len() || bytes[cursor] == b'/' {
            break;
        }
        let name_start = cursor;
        while cursor < bytes.len()
            && !bytes[cursor].is_ascii_whitespace()
            && bytes[cursor] != b'='
            && bytes[cursor] != b'/'
        {
            cursor += 1;
        }
        let name = &inner[name_start..cursor];
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        let value = if cursor < bytes.len() && bytes[cursor] == b'=' {
            cursor += 1;
            while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
                cursor += 1;
            }
            if cursor < bytes.len() && (bytes[cursor] == b'"' || bytes[cursor] == b'\'') {
                let quote = bytes[cursor];
                cursor += 1;
                let value_start = cursor;
                while cursor < bytes.len() && bytes[cursor] != quote {
                    cursor += 1;
                }
                let value = &inner[value_start..cursor];
                if cursor < bytes.len() {
                    cursor += 1;
                }
                value
            } else {
                let value_start = cursor;
                while cursor < bytes.len() && !bytes[cursor].is_ascii_whitespace() {
                    cursor += 1;
                }
                &inner[value_start..cursor]
            }
        } else {
            ""
        };
        attrs.push((name, value));
    }
    attrs
}

fn attr<'a>(element: &'a ComponentElement<'_>, name: &str) -> Option<&'a str> {
    element
        .attrs
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(name))
        .map(|(_, value)| *value)
        .filter(|value| !value.is_empty())
}

fn render_spotify(element: &ComponentElement<'_>) -> Option<String> {
    let url = attr(element, "url")
        .or_else(|| attr(element, "href"))
        .or_else(|| attr(element, "src"))
        .or_else(|| (!element.body.trim().is_empty()).then(|| element.body.trim()))?;
    let embed = spotify_embed_url(url)?;
    let mut html = String::new();
    html.push_str("<iframe class=\"ox-spotify\" src=\"");
    escape_attr(&embed, &mut html);
    html.push_str("\" width=\"100%\" height=\"352\" loading=\"lazy\" allow=\"autoplay; clipboard-write; encrypted-media; fullscreen; picture-in-picture\" referrerpolicy=\"strict-origin-when-cross-origin\"></iframe>");
    Some(html)
}

fn spotify_embed_url(input: &str) -> Option<String> {
    if input.starts_with("https://open.spotify.com/embed/") {
        return Some(input.to_string());
    }
    let path = input.strip_prefix("https://open.spotify.com/")?;
    let (kind, rest) = path.split_once('/')?;
    if !matches!(kind, "track" | "album" | "playlist" | "episode" | "show" | "artist") {
        return None;
    }
    let id = rest.split(&['?', '#'][..]).next()?.trim();
    if id.is_empty() || !id.bytes().all(|b| b.is_ascii_alphanumeric()) {
        return None;
    }
    Some(format!("https://open.spotify.com/embed/{kind}/{id}"))
}

fn render_stackblitz(element: &ComponentElement<'_>) -> Option<String> {
    let url = attr(element, "url").or_else(|| attr(element, "href"))?;
    let embed = stackblitz_embed_url(url)?;
    Some(render_iframe("ox-stackblitz", &embed, "StackBlitz project", "100%", "480"))
}

fn stackblitz_embed_url(input: &str) -> Option<String> {
    if input.starts_with("https://stackblitz.com/edit/")
        || input.starts_with("https://stackblitz.com/github/")
    {
        let separator = if input.contains('?') { '&' } else { '?' };
        return Some(format!("{input}{separator}embed=1"));
    }
    None
}

fn render_tweet(element: &ComponentElement<'_>) -> Option<String> {
    let url =
        attr(element, "url").or_else(|| attr(element, "href")).or_else(|| attr(element, "id"))?;
    let href = tweet_url(url)?;
    Some(render_static_card("ox-tweet", "X", &href, element.body.trim()))
}

fn tweet_url(input: &str) -> Option<String> {
    if input.starts_with("https://twitter.com/") || input.starts_with("https://x.com/") {
        return Some(input.to_string());
    }
    if input.bytes().all(|byte| byte.is_ascii_digit()) {
        return Some(format!("https://x.com/i/web/status/{input}"));
    }
    None
}

fn render_bluesky(element: &ComponentElement<'_>) -> Option<String> {
    let url = attr(element, "url").or_else(|| attr(element, "href"))?;
    if !url.starts_with("https://bsky.app/profile/") {
        return None;
    }
    Some(render_static_card("ox-bluesky", "Bluesky", url, element.body.trim()))
}

fn render_webcontainer(element: &ComponentElement<'_>) -> Option<String> {
    let entry = attr(element, "entry").unwrap_or("index.html");
    let title = attr(element, "title").unwrap_or("WebContainer");
    let mut html = String::new();
    html.push_str("<div class=\"ox-webcontainer\" data-entry=\"");
    escape_attr(entry, &mut html);
    html.push_str("\" data-cross-origin-isolation=\"required\"><div class=\"ox-webcontainer__header\"><strong>");
    escape_text(title, &mut html);
    html.push_str("</strong><span>Requires cross-origin isolation</span></div><pre><code>");
    escape_text(element.body.trim(), &mut html);
    html.push_str("</code></pre></div>");
    Some(html)
}

fn render_iframe(class_name: &str, src: &str, title: &str, width: &str, height: &str) -> String {
    let mut html = String::new();
    html.push_str("<iframe class=\"");
    html.push_str(class_name);
    html.push_str("\" src=\"");
    escape_attr(src, &mut html);
    html.push_str("\" title=\"");
    escape_attr(title, &mut html);
    html.push_str("\" width=\"");
    escape_attr(width, &mut html);
    html.push_str("\" height=\"");
    escape_attr(height, &mut html);
    html.push_str("\" loading=\"lazy\" allow=\"accelerometer; ambient-light-sensor; camera; encrypted-media; geolocation; gyroscope; hid; microphone; midi; payment; serial; usb; vr; xr-spatial-tracking; clipboard-read; clipboard-write; fullscreen\" sandbox=\"allow-forms allow-modals allow-popups allow-presentation allow-same-origin allow-scripts\"></iframe>");
    html
}

fn render_static_card(class_name: &str, label: &str, href: &str, body: &str) -> String {
    let mut html = String::new();
    html.push_str("<article class=\"");
    html.push_str(class_name);
    html.push_str("\"><a href=\"");
    escape_attr(href, &mut html);
    html.push_str("\" target=\"_blank\" rel=\"noopener noreferrer\"><span>");
    escape_text(label, &mut html);
    html.push_str("</span><strong>");
    escape_text(href, &mut html);
    html.push_str("</strong>");
    if !body.is_empty() {
        html.push_str("<p>");
        escape_text(body, &mut html);
        html.push_str("</p>");
    }
    html.push_str("</a></article>");
    html
}

fn escape_text(value: &str, out: &mut String) {
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(ch),
        }
    }
}

fn escape_attr(value: &str, out: &mut String) {
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(ch),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_spotify_iframe() {
        let html = transform_media_embeds(
            r#"<Spotify url="https://open.spotify.com/track/abc123"></Spotify>"#,
            Some(&JsMediaEmbedsOptions {
                spotify: Some(true),
                stack_blitz: None,
                twitter: None,
                bluesky: None,
                web_container: None,
            }),
        );
        assert!(html.contains("https://open.spotify.com/embed/track/abc123"));
    }

    #[test]
    fn renders_static_tweet_card() {
        let html = transform_media_embeds(
            r#"<Tweet id="123">hello</Tweet>"#,
            Some(&JsMediaEmbedsOptions {
                spotify: None,
                stack_blitz: None,
                twitter: Some(true),
                bluesky: None,
                web_container: None,
            }),
        );
        assert!(html.contains("https://x.com/i/web/status/123"));
        assert!(html.contains("hello"));
    }
}
