use std::path::PathBuf;

use super::{escape_html_attr, escape_html_text, EditThisPageOptions};

pub(super) fn append_edit_this_page(html: &str, options: &EditThisPageOptions) -> String {
    let href = edit_this_page_href(options);
    let mut out = String::with_capacity(html.len() + href.len() + options.label.len() + 96);
    out.push_str(html);
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str("<p class=\"ox-edit-this-page\"><a href=\"");
    escape_html_attr(&href, &mut out);
    out.push_str("\" target=\"_blank\" rel=\"noopener noreferrer\">");
    escape_html_text(&options.label, &mut out);
    out.push_str("</a></p>\n");
    out
}

fn edit_this_page_href(options: &EditThisPageOptions) -> String {
    let source = PathBuf::from(&options.source_path);
    let absolute = if source.is_absolute() { source } else { options.root_dir.join(source) };
    let relative = absolute
        .strip_prefix(&options.root_dir)
        .ok()
        .unwrap_or(absolute.as_path())
        .to_string_lossy()
        .replace('\\', "/");
    format!("{}/edit/{}/{}", options.repo_url, options.branch, percent_encode_path(&relative))
}

fn percent_encode_path(path: &str) -> String {
    let mut out = String::with_capacity(path.len());
    for byte in path.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'-' | b'_' | b'.') {
            out.push(byte as char);
        } else {
            use std::fmt::Write as _;
            let _ = write!(out, "%{byte:02X}");
        }
    }
    out
}
