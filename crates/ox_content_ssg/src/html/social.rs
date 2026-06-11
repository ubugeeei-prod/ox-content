use askama::Template;

use super::utils::escape_html;
use super::{MobileSocialLinksTemplate, SocialLink, SocialLinks, SocialLinksTemplate};

pub(super) fn generate_social_links_html(links: &SocialLinks) -> String {
    let template = SocialLinksTemplate {
        github: links.github.as_deref(),
        twitter: links.twitter.as_deref(),
        discord: links.discord.as_deref(),
    };
    let mut html = template.render().unwrap_or_default();
    if let Some(custom_links) = &links.links {
        for link in custom_links {
            html.push_str(&render_custom_social_link(link, false));
        }
    }
    html
}

pub(super) fn generate_mobile_social_links_html(links: &SocialLinks) -> String {
    let template = MobileSocialLinksTemplate {
        github: links.github.as_deref(),
        twitter: links.twitter.as_deref(),
        discord: links.discord.as_deref(),
    };
    let mut html = template.render().unwrap_or_default();
    if let Some(custom_links) = &links.links {
        for link in custom_links {
            html.push_str(&render_custom_social_link(link, true));
        }
    }
    html
}

fn render_custom_social_link(link: &SocialLink, mobile: bool) -> String {
    let label = link.aria_label.as_deref().or(link.icon.as_deref()).unwrap_or("Social link");
    let href = escape_html(&link.link);
    let label = escape_html(label);
    let icon_html = render_social_icon(link);

    if mobile {
        format!(
            "<a href=\"{href}\" class=\"mobile-footer-btn\" aria-label=\"{label}\" target=\"_blank\" rel=\"noopener\">{icon_html}<span class=\"mobile-footer-label\">{label}</span></a>\n"
        )
    } else {
        format!(
            "<a href=\"{href}\" class=\"social-link\" aria-label=\"{label}\" target=\"_blank\" rel=\"noopener\">{icon_html}</a>\n"
        )
    }
}

fn render_social_icon(link: &SocialLink) -> String {
    if let Some(svg) = link.icon_svg.as_deref().and_then(validate_social_svg) {
        return svg.to_string();
    }

    link.icon
        .as_deref()
        .map(|icon| format!("<span class=\"social-link-icon\">{}</span>", escape_html(icon)))
        .unwrap_or_default()
}

fn validate_social_svg(svg: &str) -> Option<&str> {
    let trimmed = svg.trim();
    let lower = trimmed.to_ascii_lowercase();
    if trimmed.starts_with("<svg") && !lower.contains("<script") {
        Some(trimmed)
    } else {
        None
    }
}
