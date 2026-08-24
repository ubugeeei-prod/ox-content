//! Opt-in team / members page cards.

use serde::{Deserialize, Serialize};

use super::utils::escape_html;

/// One outbound or site-relative link on a member card.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeamLink {
    /// Visible label. Escaped in HTML.
    pub label: String,
    /// Destination. Only `https:` or a site-relative `/` path is emitted.
    pub href: String,
}

/// One person on the team page.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeamMember {
    /// Display name. Escaped in HTML.
    pub name: String,
    /// Optional role or title. Escaped in HTML.
    pub role: Option<String>,
    /// Avatar URL. Only `https:` or a site-relative `/` path is emitted.
    pub avatar: Option<String>,
    /// Optional profile or social links.
    pub links: Option<Vec<TeamLink>>,
}

/// Site-wide team page option. Off unless `enabled` is true.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeamOptions {
    /// When false, `layout: team` is ignored.
    #[serde(default)]
    pub enabled: bool,
    /// Members rendered as static cards.
    #[serde(default)]
    pub members: Vec<TeamMember>,
}

impl TeamOptions {
    /// Omitted / `false` in JS maps here.
    pub fn disabled() -> Self {
        Self::default()
    }
}

pub(super) const TEAM_CSS: &str = include_str!("team.css");

/// Renders team cards when the feature is on and the page asks for `layout: team`.
///
/// Otherwise the markdown body is returned unchanged so the page stays ordinary.
pub fn render_team_page(team: &TeamOptions, layout: &str, body: &str) -> String {
    if !team.enabled || layout != "team" {
        return body.to_string();
    }

    let cards = render_team_cards(&team.members);
    if body.trim().is_empty() { cards } else { format!("{cards}\n{body}") }
}

/// `https:` or a same-origin path starting with `/` but not `//`.
pub fn is_safe_team_url(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || trimmed.bytes().any(|byte| matches!(byte, b'\n' | b'\r' | b'\0' | b'\t'))
    {
        return false;
    }
    if trimmed.starts_with("//") {
        return false;
    }
    if trimmed.starts_with('/') {
        return true;
    }
    trimmed.to_ascii_lowercase().starts_with("https:")
}

fn render_team_cards(members: &[TeamMember]) -> String {
    let mut out = String::from("<div class=\"ox-team\">\n<ul class=\"ox-team__list\">\n");
    for member in members {
        out.push_str("<li class=\"ox-team__card\">\n");
        if let Some(avatar) =
            member.avatar.as_deref().map(str::trim).filter(|url| is_safe_team_url(url))
        {
            out.push_str("<img class=\"ox-team__avatar\" src=\"");
            out.push_str(&escape_html(avatar));
            out.push_str("\" alt=\"");
            out.push_str(&escape_html(&member.name));
            out.push_str("\">\n");
        }
        out.push_str("<p class=\"ox-team__name\">");
        out.push_str(&escape_html(&member.name));
        out.push_str("</p>\n");
        if let Some(role) = member.role.as_deref().filter(|role| !role.is_empty()) {
            out.push_str("<p class=\"ox-team__role\">");
            out.push_str(&escape_html(role));
            out.push_str("</p>\n");
        }
        append_member_links(&mut out, member.links.as_deref().unwrap_or(&[]));
        out.push_str("</li>\n");
    }
    out.push_str("</ul>\n</div>");
    out
}

fn append_member_links(out: &mut String, links: &[TeamLink]) {
    let safe: Vec<&TeamLink> = links.iter().filter(|link| is_safe_team_url(&link.href)).collect();
    if safe.is_empty() {
        return;
    }
    out.push_str("<ul class=\"ox-team__links\">\n");
    for link in safe {
        out.push_str("<li><a class=\"ox-team__link\" href=\"");
        out.push_str(&escape_html(link.href.trim()));
        out.push_str("\">");
        out.push_str(&escape_html(&link.label));
        out.push_str("</a></li>\n");
    }
    out.push_str("</ul>\n");
}

#[cfg(test)]
mod tests;
