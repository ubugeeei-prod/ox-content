use std::path::{Path, PathBuf};

use super::{ResolvedEditThisPageOptions, escape_html_attr, escape_html_text};

pub(super) mod provider;

use provider::render_pattern;

pub(super) fn append_edit_this_page(html: &str, options: &ResolvedEditThisPageOptions) -> String {
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

fn edit_this_page_href(options: &ResolvedEditThisPageOptions) -> String {
    render_pattern(
        &options.url_pattern,
        &options.repo_url,
        &options.branch,
        &percent_encode_path(&page_path(options)),
    )
}

/// The page's path inside the repository.
///
/// `rootDir` has two shapes, and the source path tells them apart. An
/// absolute path that contains the page is a directory on disk to measure
/// from — the long-standing way to point at the repository root from a
/// build that runs below it. Anything else names a location *inside the
/// repository*: it goes in front of the page's path within `srcDir`.
///
/// Neither shape may put a path from the build machine into the link, so
/// there is no branch here that falls back to the absolute source path.
fn page_path(options: &ResolvedEditThisPageOptions) -> String {
    let source = PathBuf::from(&options.source_path);
    let source = if source.is_absolute() {
        source
    } else {
        options.src_dir.as_ref().unwrap_or(&options.working_dir).join(source)
    };

    let Some(root_dir) = options.root_dir.as_deref() else {
        // The working directory is the repository root for the usual build.
        // When the page sits outside it, the source root is the next best
        // thing to measure from.
        return source.strip_prefix(&options.working_dir).map_or_else(
            |_| relative_to(&source, options.src_dir.as_ref().unwrap_or(&options.working_dir)),
            to_url_path,
        );
    };

    let root_path = Path::new(root_dir);
    if root_path.is_absolute()
        && let Ok(relative) = source.strip_prefix(root_path)
    {
        return to_url_path(relative);
    }

    let within_source_root =
        relative_to(&source, options.src_dir.as_ref().unwrap_or(&options.working_dir));
    let prefix = root_dir.trim_matches('/');
    if within_source_root.is_empty() {
        prefix.to_string()
    } else {
        format!("{prefix}/{within_source_root}")
    }
}

/// `path` seen from `base`, or its file name when it lies outside `base`.
///
/// The fallback keeps a mismatch from publishing the checkout's path: a
/// file name is wrong but harmless, an absolute path leaks the build
/// machine's directory layout to every visitor.
fn relative_to(path: &Path, base: &Path) -> String {
    path.strip_prefix(base).map_or_else(
        |_| path.file_name().map(Path::new).map(to_url_path).unwrap_or_default(),
        to_url_path,
    )
}

fn to_url_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
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

#[cfg(test)]
mod tests;
