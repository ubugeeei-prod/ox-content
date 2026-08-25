//! Default file-tree icons and trusted site-config replacements.

use super::super::escape_html_attr;
use super::ResolvedFileTreeOptions;

const FOLDER: &str = concat!(
    r#"<svg class="ox-file-tree__glyph" viewBox="0 0 16 16" width="14" height="14" aria-hidden="true">"#,
    r#"<path fill="currentColor" d="M1.75 2.5h4.17l.83 1.1H14c.69 0 1.25.56 1.25 1.25v7.4c0 .69-.56 1.25-1.25 1.25H2c-.69 0-1.25-.56-1.25-1.25v-8.5C.75 3.06 1.2 2.5 1.75 2.5Z"/>"#,
    "</svg>",
);

const FOLDER_OPEN: &str = concat!(
    r#"<svg class="ox-file-tree__glyph" viewBox="0 0 16 16" width="14" height="14" aria-hidden="true">"#,
    r#"<path fill="currentColor" d="M1.5 3.75A1.25 1.25 0 0 1 2.75 2.5h3.2c.28 0 .54.12.72.33l.86 1.02h5.72c.69 0 1.25.56 1.25 1.25v.4H3.28c-.55 0-1.04.36-1.2.89L1.5 6.7z"/>"#,
    r#"<path fill="currentColor" d="M1.62 7.85 2.95 12c.16.5.63.85 1.15.85h8.3c.52 0 .99-.35 1.15-.85l1.4-4.15a.6.6 0 0 0-.57-.8H2.2a.6.6 0 0 0-.58.8z"/>"#,
    "</svg>",
);

const FILE: &str = concat!(
    r#"<svg class="ox-file-tree__glyph" viewBox="0 0 16 16" width="14" height="14" aria-hidden="true">"#,
    r#"<path fill="currentColor" d="M3.5 1.5A1.5 1.5 0 0 0 2 3v10a1.5 1.5 0 0 0 1.5 1.5h9A1.5 1.5 0 0 0 14 13V5.7L9.8 1.5H3.5zm6 .95L12.55 5.7H9.5z"/>"#,
    "</svg>",
);

pub(super) fn emit_dir_icons(out: &mut String, options: &ResolvedFileTreeOptions, toggle: bool) {
    if !options.icons {
        return;
    }
    emit_icon(out, "ox-file-tree__icon--folder", options.icon_folder.as_deref(), FOLDER);
    if toggle {
        emit_icon(
            out,
            "ox-file-tree__icon--folder-open",
            options.icon_folder_open.as_deref(),
            FOLDER_OPEN,
        );
    }
}

pub(super) fn emit_file_icon(out: &mut String, options: &ResolvedFileTreeOptions, name: &str) {
    if !options.icons {
        return;
    }
    let custom = file_ext(name)
        .and_then(|ext| options.icon_files.get(&ext).map(String::as_str))
        .or(options.icon_file.as_deref());
    emit_icon(out, "ox-file-tree__icon--file", custom, FILE);
}

fn emit_icon(out: &mut String, modifier: &str, custom: Option<&str>, fallback: &str) {
    out.push_str("<span class=\"ox-file-tree__icon ");
    out.push_str(modifier);
    out.push_str("\" aria-hidden=\"true\">");
    match custom.map(str::trim).filter(|value| !value.is_empty()) {
        Some(custom) => emit_custom(out, custom),
        None => out.push_str(fallback),
    }
    out.push_str("</span>");
}

fn emit_custom(out: &mut String, custom: &str) {
    if custom.starts_with('<') {
        out.push_str(custom);
        return;
    }
    out.push_str("<span class=\"");
    let mut first = true;
    for token in custom.split_whitespace() {
        if !is_safe_class(token) {
            continue;
        }
        if !first {
            out.push(' ');
        }
        first = false;
        escape_html_attr(token, out);
    }
    out.push_str("\"></span>");
}

fn file_ext(name: &str) -> Option<String> {
    let name = name.trim_end_matches('/');
    let (_, ext) = name.rsplit_once('.')?;
    if ext.is_empty() {
        return None;
    }
    Some(ext.to_ascii_lowercase())
}

fn is_safe_class(token: &str) -> bool {
    !token.is_empty()
        && token.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
}
