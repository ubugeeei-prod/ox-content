use super::html::{attr, ComponentElement};

pub(super) fn render_spotify(element: &ComponentElement<'_>) -> Option<String> {
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

pub(super) fn render_stackblitz(element: &ComponentElement<'_>) -> Option<String> {
    let url = attr(element, "url").or_else(|| attr(element, "href"))?;
    let embed = stackblitz_embed_url(url)?;
    Some(render_iframe("ox-stackblitz", &embed, "StackBlitz project", "100%", "480"))
}

pub(super) fn render_tweet(element: &ComponentElement<'_>) -> Option<String> {
    let url =
        attr(element, "url").or_else(|| attr(element, "href")).or_else(|| attr(element, "id"))?;
    let href = tweet_url(url)?;
    Some(render_static_card("ox-tweet", "X", &href, element.body.trim()))
}

pub(super) fn render_bluesky(element: &ComponentElement<'_>) -> Option<String> {
    let url = attr(element, "url").or_else(|| attr(element, "href"))?;
    if !url.starts_with("https://bsky.app/profile/") {
        return None;
    }
    Some(render_static_card("ox-bluesky", "Bluesky", url, element.body.trim()))
}

pub(super) fn render_webcontainer(element: &ComponentElement<'_>) -> Option<String> {
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

fn stackblitz_embed_url(input: &str) -> Option<String> {
    if input.starts_with("https://stackblitz.com/edit/")
        || input.starts_with("https://stackblitz.com/github/")
    {
        let separator = if input.contains('?') { '&' } else { '?' };
        return Some(format!("{input}{separator}embed=1"));
    }
    None
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
