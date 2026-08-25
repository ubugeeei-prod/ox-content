use super::ResolvedMagicLinks;

const GITHUB_RESERVED: &[&str] = &[
    "about",
    "codespaces",
    "copilot",
    "discussions",
    "enterprise",
    "explore",
    "features",
    "issues",
    "login",
    "marketplace",
    "new",
    "notifications",
    "organizations",
    "orgs",
    "pricing",
    "pulls",
    "search",
    "security",
    "settings",
    "signup",
    "sponsor",
    "sponsors",
    "topics",
];

#[derive(Clone, Copy)]
pub(super) enum Kind {
    Github,
    Alias,
    Url,
}

impl Kind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Github => "github",
            Self::Alias => "alias",
            Self::Url => "url",
        }
    }
}

pub(super) fn render_body(body: &str, options: &ResolvedMagicLinks) -> Option<String> {
    let body = body.trim();
    if body.is_empty() || body.bytes().any(|byte| byte == b'\n' || byte == b'\r') {
        return None;
    }
    let parts: Vec<&str> = body.split('|').map(str::trim).collect();
    if parts.is_empty() || parts.len() > 3 {
        return None;
    }
    if let Some(username) = parts[0].strip_prefix('@') {
        return render_github(
            username,
            optional_part(parts.get(1)),
            optional_part(parts.get(2)),
            options,
        );
    }
    if parts.len() > 2 {
        return None;
    }
    render_named(parts[0], optional_part(parts.get(1)), options)
}

fn optional_part<'a>(part: Option<&&'a str>) -> Option<&'a str> {
    part.copied().filter(|value| !value.is_empty())
}

fn render_github(
    username: &str,
    label: Option<&str>,
    href: Option<&str>,
    options: &ResolvedMagicLinks,
) -> Option<String> {
    if !is_github_login(username) {
        return None;
    }
    let profile = format_github_profile(username);
    let href = href.unwrap_or(profile.as_str());
    if !is_safe_href(href) {
        return None;
    }
    let label = label.unwrap_or(username);
    let image = resolve_image(href, None, Some(username), options);
    Some(write_link(href, label, image.as_deref(), Kind::Github))
}

fn format_github_profile(username: &str) -> String {
    let mut href = String::from("https://github.com/");
    href.push_str(username);
    href
}

fn render_named(
    key: &str,
    explicit_href: Option<&str>,
    options: &ResolvedMagicLinks,
) -> Option<String> {
    let alias = options.aliases.get(key);
    let href = match (explicit_href, alias) {
        (Some(href), _) => href,
        (None, Some(alias)) => alias.href.as_str(),
        (None, None) if is_safe_href(key) => key,
        (None, None) => return None,
    };
    if !is_safe_href(href) {
        return None;
    }
    let label = if explicit_href.is_some() {
        key
    } else {
        alias
            .and_then(|alias| alias.label.as_deref())
            .filter(|value| !value.is_empty())
            .unwrap_or(key)
    };
    let label = if is_safe_href(label) { host_label(href).unwrap_or(label) } else { label };
    let configured_image = alias.and_then(|alias| alias.image.as_deref());
    let image = resolve_image(href, configured_image, None, options);
    let kind = if alias.is_some() { Kind::Alias } else { Kind::Url };
    Some(write_link(href, label, image.as_deref(), kind))
}

fn resolve_image(
    href: &str,
    configured: Option<&str>,
    github_user: Option<&str>,
    options: &ResolvedMagicLinks,
) -> Option<String> {
    let mut image = configured
        .filter(|value| is_safe_href(value))
        .map(ToOwned::to_owned)
        .or_else(|| github_avatar(href, github_user))
        .or_else(|| favicon_for(href, options));
    for override_rule in &options.image_overrides {
        if override_matches(override_rule, href) && is_safe_href(&override_rule.image) {
            image = Some(override_rule.image.clone());
            break;
        }
    }
    image.filter(|value| is_safe_href(value))
}

fn override_matches(rule: &crate::MagicLinkImageOverride, href: &str) -> bool {
    if let Some(exact) = rule.href.as_deref()
        && !exact.is_empty()
    {
        return href == exact;
    }
    rule.prefix.as_deref().is_some_and(|prefix| !prefix.is_empty() && href.starts_with(prefix))
}

fn github_avatar(href: &str, github_user: Option<&str>) -> Option<String> {
    let login = github_user.or_else(|| github_scope(href))?;
    if !is_github_login(login) || is_reserved_github_route(login) {
        return None;
    }
    let mut image = String::from("https://github.com/");
    image.push_str(login);
    image.push_str(".png");
    Some(image)
}

fn github_scope(href: &str) -> Option<&str> {
    let rest = href
        .strip_prefix("https://github.com/")
        .or_else(|| href.strip_prefix("http://github.com/"))?;
    let login = rest.split(['/', '?', '#']).next()?;
    (!login.is_empty()).then_some(login)
}

fn favicon_for(href: &str, options: &ResolvedMagicLinks) -> Option<String> {
    let template = options.favicon_template.as_deref()?;
    let host = host_label(href)?;
    let image = template.replace(concat!("{", "host}"), host);
    is_safe_href(&image).then_some(image)
}

fn host_label(href: &str) -> Option<&str> {
    let rest = href.split_once("://")?.1;
    let host = rest.split(['/', '?', '#']).next()?;
    let host = host.rsplit_once('@').map_or(host, |(_, host)| host);
    let host = strip_decimal_port(host);
    (!host.is_empty()).then_some(host)
}

fn strip_decimal_port(host: &str) -> &str {
    match host.rsplit_once(':') {
        Some((name, port))
            if !name.is_empty() && port.bytes().all(|byte| byte.is_ascii_digit()) =>
        {
            name
        }
        _ => host,
    }
}

fn is_github_login(name: &str) -> bool {
    let bytes = name.as_bytes();
    (1..=39).contains(&bytes.len())
        && bytes[0].is_ascii_alphanumeric()
        && bytes[bytes.len() - 1] != b'-'
        && bytes.iter().all(|byte| byte.is_ascii_alphanumeric() || *byte == b'-')
}

fn is_reserved_github_route(name: &str) -> bool {
    GITHUB_RESERVED.iter().any(|route| name.eq_ignore_ascii_case(route))
}

pub(super) fn is_safe_href(href: &str) -> bool {
    if href.is_empty()
        || href.bytes().any(|byte| {
            byte.is_ascii_control()
                || byte.is_ascii_whitespace()
                || matches!(byte, b'"' | b'\'' | b'<' | b'>' | b'`' | b'\\')
        })
    {
        return false;
    }
    let Some((scheme, rest)) = href.split_once("://") else {
        return false;
    };
    if !scheme.eq_ignore_ascii_case("http") && !scheme.eq_ignore_ascii_case("https") {
        return false;
    }
    let host = rest.split(['/', '?', '#']).next().unwrap_or("");
    let host = strip_decimal_port(host.rsplit_once('@').map_or(host, |(_, host)| host));
    !host.is_empty() && (host.contains('.') || host.eq_ignore_ascii_case("localhost"))
}

fn write_link(href: &str, label: &str, image: Option<&str>, kind: Kind) -> String {
    let mut out = String::new();
    out.push_str("<a href=\"");
    super::super::escape_html_attr(href, &mut out);
    out.push_str("\" class=\"ox-magic-link ox-magic-link--");
    out.push_str(kind.as_str());
    out.push_str("\">");
    if let Some(image) = image {
        out.push_str("<img class=\"ox-magic-link__image\" src=\"");
        super::super::escape_html_attr(image, &mut out);
        out.push_str("\" alt=\"\" width=\"16\" height=\"16\" decoding=\"async\">");
    }
    out.push_str("<span class=\"ox-magic-link__label\">");
    super::super::escape_html_text(label, &mut out);
    out.push_str("</span></a>");
    out
}
