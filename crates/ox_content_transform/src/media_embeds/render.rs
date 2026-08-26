use super::html::{ComponentElement, attr};

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
    Some(render_tweet_card(element, &href))
}

pub(super) fn render_bluesky(element: &ComponentElement<'_>) -> Option<String> {
    let url = attr(element, "url").or_else(|| attr(element, "href"))?;
    if !url.starts_with("https://bsky.app/profile/") {
        return None;
    }
    Some(render_bluesky_card(element, url))
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

fn render_tweet_card(element: &ComponentElement<'_>, href: &str) -> String {
    let handle = first_attr(element, &["handle", "screenName", "authorHandle"])
        .or_else(|| tweet_handle_from_url(href))
        .unwrap_or("x.com");
    let author =
        first_attr(element, &["displayName", "authorName", "name", "author"]).unwrap_or(handle);
    let avatar = first_attr(element, &["avatar", "avatarUrl", "authorAvatar", "profileImage"]);
    let date = first_attr(element, &["datetime", "dateTime", "createdAt", "date", "timestamp"]);
    let date_label = first_attr(element, &["dateLabel", "time", "publishedAtLabel"]).or(date);
    let body = element.body.trim();

    let mut html = String::new();
    html.push_str("<article class=\"ox-tweet ox-tweet--rich\"><a class=\"ox-tweet__card\" href=\"");
    escape_attr(href, &mut html);
    html.push_str("\" target=\"_blank\" rel=\"noopener noreferrer\"><header class=\"ox-tweet__header\"><span class=\"ox-tweet__profile\"><span class=\"ox-tweet__avatar-wrap\">");
    if let Some(avatar) = avatar {
        html.push_str("<img class=\"ox-tweet__avatar\" src=\"");
        escape_attr(avatar, &mut html);
        html.push_str("\" alt=\"\" loading=\"lazy\" decoding=\"async\">");
    } else {
        html.push_str("<span class=\"ox-tweet__avatar-fallback\" aria-hidden=\"true\">X</span>");
    }
    html.push_str(
        "</span><span class=\"ox-tweet__identity\"><strong class=\"ox-tweet__author-name\">",
    );
    escape_text(author, &mut html);
    html.push_str("</strong><span class=\"ox-tweet__author-handle\">@");
    escape_text(handle.trim_start_matches('@'), &mut html);
    html.push_str("</span></span></span><span class=\"ox-tweet__network\">X</span></header>");

    if !body.is_empty() {
        html.push_str("<p class=\"ox-tweet__body\">");
        escape_text(body, &mut html);
        html.push_str("</p>");
    }

    html.push_str("<footer class=\"ox-tweet__meta\">");
    if let Some(date_label) = date_label {
        html.push_str("<time datetime=\"");
        escape_attr(date.unwrap_or(date_label), &mut html);
        html.push_str("\">");
        escape_text(date_label, &mut html);
        html.push_str("</time>");
    }
    push_count(
        &mut html,
        first_attr(element, &["replies", "replyCount", "conversationCount"]),
        "replies",
    );
    push_count(&mut html, first_attr(element, &["reposts", "retweets", "retweetCount"]), "reposts");
    push_count(&mut html, first_attr(element, &["quotes", "quoteCount"]), "quotes");
    push_count(&mut html, first_attr(element, &["likes", "likeCount", "favoriteCount"]), "likes");
    push_count(&mut html, first_attr(element, &["views", "viewCount", "impressions"]), "views");
    html.push_str("<span class=\"ox-tweet__source\">Open post</span></footer></a></article>");
    html
}

fn tweet_handle_from_url(url: &str) -> Option<&str> {
    let rest =
        url.strip_prefix("https://x.com/").or_else(|| url.strip_prefix("https://twitter.com/"))?;
    let (handle, _) = rest.split_once("/status/")?;
    (!handle.is_empty() && handle != "i/web").then_some(handle)
}

fn render_bluesky_card(element: &ComponentElement<'_>, href: &str) -> String {
    let handle = first_attr(element, &["handle", "authorHandle"])
        .or_else(|| bluesky_handle_from_url(href))
        .unwrap_or("bsky.app");
    let author =
        first_attr(element, &["displayName", "authorName", "name", "author"]).unwrap_or(handle);
    let avatar = first_attr(element, &["avatar", "avatarUrl", "authorAvatar", "authorAvatarUrl"]);
    let date = first_attr(element, &["datetime", "dateTime", "createdAt", "date", "timestamp"]);
    let date_label = first_attr(element, &["dateLabel", "time", "publishedAtLabel"]).or(date);
    let body = element.body.trim();

    let mut html = String::new();
    html.push_str(
        "<article class=\"ox-bluesky ox-bluesky--rich\"><a class=\"ox-bluesky__card\" href=\"",
    );
    escape_attr(href, &mut html);
    html.push_str("\" target=\"_blank\" rel=\"noopener noreferrer\"><header class=\"ox-bluesky__header\"><span class=\"ox-bluesky__avatar-wrap\">");
    if let Some(avatar) = avatar {
        html.push_str("<img class=\"ox-bluesky__avatar\" src=\"");
        escape_attr(avatar, &mut html);
        html.push_str("\" alt=\"\" loading=\"lazy\">");
    } else {
        html.push_str("<span class=\"ox-bluesky__avatar-fallback\" aria-hidden=\"true\">B</span>");
    }
    html.push_str(
        "</span><span class=\"ox-bluesky__identity\"><strong class=\"ox-bluesky__author-name\">",
    );
    escape_text(author, &mut html);
    html.push_str("</strong><span class=\"ox-bluesky__handle\">@");
    escape_text(handle.trim_start_matches('@'), &mut html);
    html.push_str("</span></span><span class=\"ox-bluesky__network\">Bluesky</span></header>");

    if !body.is_empty() {
        html.push_str("<p class=\"ox-bluesky__body\">");
        escape_text(body, &mut html);
        html.push_str("</p>");
    }

    html.push_str("<footer class=\"ox-bluesky__meta\">");
    if let Some(date_label) = date_label {
        html.push_str("<time datetime=\"");
        escape_attr(date.unwrap_or(date_label), &mut html);
        html.push_str("\">");
        escape_text(date_label, &mut html);
        html.push_str("</time>");
    }
    push_count(
        &mut html,
        first_attr(element, &["replies", "replyCount", "reply_count"]),
        "replies",
    );
    push_count(
        &mut html,
        first_attr(element, &["reposts", "repostCount", "repost_count"]),
        "reposts",
    );
    push_count(&mut html, first_attr(element, &["likes", "likeCount", "like_count"]), "likes");
    push_count(&mut html, first_attr(element, &["quotes", "quoteCount", "quote_count"]), "quotes");
    html.push_str("<span class=\"ox-bluesky__source\">Open post</span></footer></a></article>");
    html
}

fn first_attr<'a>(element: &'a ComponentElement<'_>, names: &[&str]) -> Option<&'a str> {
    names.iter().find_map(|name| attr(element, name))
}

fn bluesky_handle_from_url(url: &str) -> Option<&str> {
    let rest = url.strip_prefix("https://bsky.app/profile/")?;
    let (handle, _) = rest.split_once("/post/")?;
    (!handle.is_empty()).then_some(handle)
}

fn push_count(html: &mut String, value: Option<&str>, label: &str) {
    let Some(value) = value else {
        return;
    };
    html.push_str("<span>");
    escape_text(value, html);
    html.push(' ');
    html.push_str(label);
    html.push_str("</span>");
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

pub(super) fn escape_attr(value: &str, out: &mut String) {
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
