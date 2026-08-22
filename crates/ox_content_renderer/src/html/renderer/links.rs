//! Link rewriting helpers for SSG-friendly output.
//!
//! Markdown links can be converted to generated `index.html` routes, while raw HTML
//! `href` and `src` attributes can receive the configured base URL. Keeping those rules
//! together prevents the Markdown and raw-HTML paths from drifting apart.

use super::super::html_attr::{html_attr_value_range, is_html_attr_char, is_html_attr_start};
use super::HtmlRenderer;

impl HtmlRenderer {
    pub(in crate::html::renderer) fn convert_markdown_url(&self, url: &str) -> Option<String> {
        if let Some(converted) = self.convert_md_url(url) {
            return Some(converted);
        }

        self.apply_base_to_root_absolute_url(url)
    }

    pub(in crate::html::renderer) fn apply_base_to_root_absolute_url(
        &self,
        url: &str,
    ) -> Option<String> {
        if !self.options.convert_md_links || !url.starts_with('/') || url.starts_with("//") {
            return None;
        }

        let suffix_start = url.find(&['?', '#'][..]).unwrap_or(url.len());
        let (path, suffix) = url.split_at(suffix_start);
        let base = self.options.base_url().trim_end_matches('/');

        if base.is_empty() {
            None
        } else if path == "/" {
            Some(join3(base, "/", suffix))
        } else {
            Some(join3(base, path, suffix))
        }
    }

    pub(in crate::html::renderer) fn rewrite_html_root_urls(&self, html: &str) -> String {
        crate::profile_span!("renderer::rewrite_html_urls");
        let mut output = String::with_capacity(html.len());
        let bytes = html.as_bytes();
        let mut i = 0;
        let mut in_tag = false;

        while i < bytes.len() {
            match bytes[i] {
                b'<' => {
                    in_tag = true;
                    output.push('<');
                    i += 1;
                }
                b'>' => {
                    in_tag = false;
                    output.push('>');
                    i += 1;
                }
                byte if in_tag && is_html_attr_start(byte) => {
                    let name_start = i;
                    let mut name_end = i + 1;
                    while name_end < bytes.len() && is_html_attr_char(bytes[name_end]) {
                        name_end += 1;
                    }

                    let name = &html[name_start..name_end];
                    if name.eq_ignore_ascii_case("href") || name.eq_ignore_ascii_case("src") {
                        let Some((value_start, value_end)) =
                            html_attr_value_range(html, bytes, name_end)
                        else {
                            output.push_str(name);
                            i = name_end;
                            continue;
                        };
                        let value = &html[value_start..value_end];
                        // Raw anchors link pages the same way Markdown links
                        // do; convert .md targets first, then fall back to
                        // rebasing root-absolute URLs.
                        let rewritten = self.convert_markdown_url(value);
                        if let Some(rewritten) = rewritten {
                            output.push_str(&html[i..value_start]);
                            output.push_str(&rewritten);
                            i = value_end;
                            continue;
                        }
                    }

                    output.push_str(name);
                    i = name_end;
                }
                _ => {
                    if let Some(ch) = html[i..].chars().next() {
                        output.push(ch);
                        i += ch.len_utf8();
                    } else {
                        break;
                    }
                }
            }
        }

        output
    }

    /// Converts a Markdown URL to an `.html` URL for SSG output.
    pub(in crate::html::renderer) fn convert_md_url(&self, url: &str) -> Option<String> {
        crate::profile_span_detail!("renderer::convert_md_url");
        // A URL is not a filesystem path. Split the query and fragment off
        // before looking at the extension so `./guide.md?plain=1` is still
        // recognized as Markdown, and so whatever follows the path is carried
        // through the rewrite untouched.
        let suffix_start = url.find(['?', '#']).unwrap_or(url.len());
        let (path, suffix) = url.split_at(suffix_start);

        let markdown_extension_len = markdown_extension_len(path)?;

        if !self.options.convert_md_links {
            return None;
        }

        // Another origin's `.md` is not a page this build generates, so there
        // is no `index.html` route to point it at. Leaving it alone also keeps
        // it recognizable as external further down `render_link`, which is
        // what adds `target="_blank" rel="noopener noreferrer"`.
        if is_non_local_url(path) {
            return None;
        }

        // Remove the Markdown extension, including the leading dot.
        let path_without_ext = &path[..path.len() - markdown_extension_len - 1];

        // Check if the source file is an index file
        // index.md stays at the directory level, so relative paths work differently
        let source_is_index = self.is_source_index();

        // Convert path
        let converted = if path.starts_with('/') {
            // Absolute path: /getting-started.md -> {base}getting-started/index.html
            let path_without_slash = &path_without_ext[1..];
            let base = self.options.base_url();
            if path_without_slash.is_empty() || path_without_slash == "index" {
                join2(base, "index.html")
            } else if let Some(dir) = path_without_slash.strip_suffix("/index") {
                // /lib/index.md names the lib/ directory page
                join3(base, dir, "/index.html")
            } else {
                join3(base, path_without_slash, "/index.html")
            }
        } else if path.starts_with("./") {
            // Same-directory relative path
            let name = &path_without_ext[2..]; // Remove "./"
            if name == "index" {
                // ./index.md -> ./index.html (stay in same directory)
                "./index.html".to_string()
            } else if let Some(dir) = name.strip_suffix("/index") {
                // ./lib/index.md names the lib/ directory page
                if source_is_index {
                    join3("./", dir, "/index.html")
                } else {
                    join3("../", dir, "/index.html")
                }
            } else if source_is_index {
                // Source is index.md, so we're at directory level
                // ./types.md -> ./types/index.html
                join3("./", name, "/index.html")
            } else {
                // Source is not index.md (e.g., types.md -> types/index.html)
                // So we need to go up one level
                // ./types.md -> ../types/index.html
                join3("../", name, "/index.html")
            }
        } else if path.starts_with("../") {
            // Parent-relative path
            let rest = &path_without_ext[3..]; // Remove "../"
            if source_is_index {
                // Source is index.md at directory level
                // ../types.md -> ../types/index.html
                if let Some(dir) =
                    rest.strip_suffix("/index").or_else(|| (rest == "index").then_some(""))
                {
                    if dir.is_empty() {
                        "../index.html".to_string()
                    } else {
                        join3("../", dir, "/index.html")
                    }
                } else {
                    join3("../", rest, "/index.html")
                }
            } else {
                // Source is not index.md, need extra ../
                // ../types.md -> ../../types/index.html
                if let Some(dir) =
                    rest.strip_suffix("/index").or_else(|| (rest == "index").then_some(""))
                {
                    if dir.is_empty() {
                        "../../index.html".to_string()
                    } else {
                        join3("../../", dir, "/index.html")
                    }
                } else {
                    join3("../../", rest, "/index.html")
                }
            }
        } else {
            // Plain relative path: types.md
            if let Some(dir) = path_without_ext
                .strip_suffix("/index")
                .or_else(|| (path_without_ext == "index").then_some(""))
            {
                if dir.is_empty() {
                    "./index.html".to_string()
                } else if source_is_index {
                    join3("./", dir, "/index.html")
                } else {
                    join3("../", dir, "/index.html")
                }
            } else if source_is_index {
                // Source is index.md
                // types.md -> ./types/index.html
                join3("./", path_without_ext, "/index.html")
            } else {
                // Source is not index.md
                // types.md -> ../types/index.html
                join3("../", path_without_ext, "/index.html")
            }
        };

        // Reattach the query and/or fragment if there was one.
        Some(if suffix.is_empty() { converted } else { append_suffix(converted, suffix) })
    }

    /// Checks if the source file is an index file (index.md).
    pub(in crate::html::renderer) fn is_source_index(&self) -> bool {
        if self.options.source_path().is_empty() {
            return false;
        }
        let source = std::path::Path::new(self.options.source_path());
        source.file_stem().is_some_and(|stem| stem.eq_ignore_ascii_case("index"))
    }
}

fn join2(a: &str, b: &str) -> String {
    let mut out = String::with_capacity(a.len() + b.len());
    out.push_str(a);
    out.push_str(b);
    out
}

fn join3(a: &str, b: &str, c: &str) -> String {
    let mut out = String::with_capacity(a.len() + b.len() + c.len());
    out.push_str(a);
    out.push_str(b);
    out.push_str(c);
    out
}

fn append_suffix(mut converted: String, suffix: &str) -> String {
    converted.reserve(suffix.len());
    converted.push_str(suffix);
    converted
}

/// Returns the length of the trailing Markdown extension of `path`, if it has
/// one.
///
/// This deliberately does not go through `std::path::Path`: a URL only ever
/// separates segments with `/`, and on Windows `Path` would also split on `\`
/// and read a drive letter, so the same href would convert differently
/// depending on the build host.
fn markdown_extension_len(path: &str) -> Option<usize> {
    let file_name = match path.rfind('/') {
        Some(slash) => &path[slash + 1..],
        None => path,
    };
    let dot = file_name.rfind('.')?;
    // A leading dot names a hidden file rather than an extension, matching
    // what `Path::extension` reports for `.md`.
    if dot == 0 {
        return None;
    }

    let extension = &file_name[dot + 1..];
    (extension.eq_ignore_ascii_case("md")
        || extension.eq_ignore_ascii_case("mdx")
        || extension.eq_ignore_ascii_case("markdown"))
    .then_some(extension.len())
}

/// Reports whether `path` addresses another origin, either through a URI
/// scheme (`https:`, `mailto:`) or as a protocol-relative URL (`//cdn.test`).
///
/// `path` must already have had its query and fragment removed, so a `:` or
/// `//` appearing only in those parts cannot be mistaken for a scheme.
fn is_non_local_url(path: &str) -> bool {
    if path.starts_with("//") {
        return true;
    }

    let mut chars = path.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_alphabetic() {
        return false;
    }

    for ch in chars {
        if ch == ':' {
            return true;
        }
        if !(ch.is_ascii_alphanumeric() || matches!(ch, '+' | '.' | '-')) {
            return false;
        }
    }
    false
}
