use askama::Template;

use super::{FooterTemplate, ThemeConfig};

/// Footer CSS styles (added when footer is used).
pub(super) const FOOTER_CSS: &str = r"
.site-footer {
  margin-top: 3rem;
  padding: 2rem 1.5rem;
  border-top: 1px solid var(--octc-color-border);
  text-align: center;
  color: var(--octc-color-text-muted);
  font-size: 0.875rem;
}
.site-footer p {
  margin: 0.25rem 0;
}
.site-footer a {
  color: var(--octc-color-primary);
}
.site-footer a:hover {
  color: var(--octc-color-primary-hover);
}
";

/// Generates footer HTML from theme configuration.
pub(super) fn generate_footer_html(theme: &ThemeConfig) -> String {
    let footer = match &theme.footer {
        Some(f) if f.message.is_some() || f.copyright.is_some() => f,
        _ => return String::new(),
    };

    let template = FooterTemplate {
        message: footer.message.as_deref(),
        copyright: footer.copyright.as_deref(),
    };
    template.render().unwrap_or_default()
}
