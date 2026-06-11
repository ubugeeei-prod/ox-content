use std::fmt::Write as _;

use super::{escape_html, NavGroup, NavItem};

pub(super) fn generate_nav_html(nav_groups: &[NavGroup], current_path: &str) -> String {
    let mut html = String::new();
    for (group_index, group) in nav_groups.iter().enumerate() {
        if group.collapsed.is_some() {
            let open = if group.collapsed == Some(true) { "" } else { " open" };
            let state_key = format!("group:{group_index}:{}", group.title);
            let state_attr = nav_state_attr(group.sticky_collapsed, &state_key);
            push_fmt(&mut html, format_args!(
                "<details class=\"nav-section nav-section--collapsible\"{open}{state_attr}>\n  <summary class=\"nav-title nav-title--summary\">{}</summary>\n",
                escape_html(&group.title)
            ));
            render_nav_list(
                &mut html,
                &group.items,
                current_path,
                false,
                &format!("group:{group_index}"),
            );
            html.push_str("</details>\n");
        } else {
            push_fmt(
                &mut html,
                format_args!(
                    "<div class=\"nav-section\">\n  <div class=\"nav-title\">{}</div>\n",
                    escape_html(&group.title)
                ),
            );
            render_nav_list(
                &mut html,
                &group.items,
                current_path,
                false,
                &format!("group:{group_index}"),
            );
            html.push_str("</div>\n");
        }
    }
    html
}

fn render_nav_list(
    html: &mut String,
    items: &[NavItem],
    current_path: &str,
    nested: bool,
    key_prefix: &str,
) {
    let class_name = if nested { "nav-list nav-list--nested" } else { "nav-list" };
    push_fmt(html, format_args!("  <ul class=\"{class_name}\">\n"));
    for (index, item) in items.iter().enumerate() {
        render_nav_item(html, item, current_path, &format!("{key_prefix}.{index}"));
    }
    html.push_str("  </ul>\n");
}

fn render_nav_item(html: &mut String, item: &NavItem, current_path: &str, key_path: &str) {
    let href = safe_nav_href(&item.href);
    let title = escape_html(&item.title);
    let active_class = if item.path == current_path { " active" } else { "" };
    if item.children.is_empty() {
        push_fmt(html, format_args!(
            "    <li class=\"nav-item\"><a href=\"{href}\" class=\"nav-link{active_class}\">{title}</a></li>\n"
        ));
        return;
    }

    let open = if item.collapsed == Some(true) { "" } else { " open" };
    let state_key = format!("item:{key_path}:{}:{}", item.path, item.title);
    let state_attr = nav_state_attr(item.sticky_collapsed, &state_key);
    push_fmt(html, format_args!(
        "    <li class=\"nav-item nav-item--group\"><details class=\"nav-details\"{open}{state_attr}><summary class=\"nav-summary\"><a href=\"{href}\" class=\"nav-link nav-link--summary{active_class}\">{title}</a></summary>\n"
    ));
    render_nav_list(html, &item.children, current_path, true, key_path);
    html.push_str("    </details></li>\n");
}

fn nav_state_attr(sticky_collapsed: Option<bool>, key: &str) -> String {
    if sticky_collapsed == Some(true) {
        format!(" data-ox-nav-state-key=\"{}\"", escape_html(key))
    } else {
        String::new()
    }
}

fn safe_nav_href(href: &str) -> String {
    let trimmed = href.trim();
    let lower = trimmed.to_ascii_lowercase();
    let safe_scheme = lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("mailto:");
    if trimmed.starts_with("//") || (trimmed.contains(':') && !safe_scheme) {
        return "#".to_string();
    }
    escape_html(trimmed)
}

fn push_fmt(output: &mut String, args: std::fmt::Arguments<'_>) {
    if output.write_fmt(args).is_err() {
        output.push_str("[formatting failed]");
    }
}
