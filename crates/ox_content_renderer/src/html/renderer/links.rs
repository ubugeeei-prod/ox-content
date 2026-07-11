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
        let base = self.options.base_url.trim_end_matches('/');

        if base.is_empty() {
            None
        } else if path == "/" {
            Some(join3(base, "/", suffix))
        } else {
            Some(join3(base, path, suffix))
        }
    }

    pub(in crate::html::renderer) fn rewrite_html_root_urls(&self, html: &str) -> String {
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
                        let rewritten = self
                            .convert_md_url(value)
                            .or_else(|| self.apply_base_to_root_absolute_url(value));
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
        // Split URL into path and fragment
        let (path, fragment) = match url.split_once('#') {
            Some((p, f)) => (p, Some(f)),
            None => (url, None),
        };

        let markdown_extension =
            std::path::Path::new(path).extension().and_then(|ext| ext.to_str()).filter(|ext| {
                ext.eq_ignore_ascii_case("md")
                    || ext.eq_ignore_ascii_case("mdx")
                    || ext.eq_ignore_ascii_case("markdown")
            });

        let markdown_extension = markdown_extension?;

        if !self.options.convert_md_links {
            return None;
        }

        // Remove the Markdown extension, including the leading dot.
        let path_without_ext = &path[..path.len() - markdown_extension.len() - 1];

        // Check if the source file is an index file
        // index.md stays at the directory level, so relative paths work differently
        let source_is_index = self.is_source_index();

        // Convert path
        let converted = if path.starts_with('/') {
            // Absolute path: /getting-started.md -> {base}getting-started/index.html
            let path_without_slash = &path_without_ext[1..];
            let base = &self.options.base_url;
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
                if rest == "index" || rest.ends_with("/index") {
                    let dir = rest.trim_end_matches("/index").trim_end_matches("index");
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
                if rest == "index" || rest.ends_with("/index") {
                    let dir = rest.trim_end_matches("/index").trim_end_matches("index");
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
            if path_without_ext == "index" || path_without_ext.ends_with("/index") {
                let dir = path_without_ext.trim_end_matches("/index").trim_end_matches("index");
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

        // Reattach fragment if present
        Some(match fragment {
            Some(f) => append_fragment(converted, f),
            None => converted,
        })
    }

    /// Checks if the source file is an index file (index.md).
    pub(in crate::html::renderer) fn is_source_index(&self) -> bool {
        if self.options.source_path.is_empty() {
            return false;
        }
        let source = std::path::Path::new(&self.options.source_path);
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

fn append_fragment(mut converted: String, fragment: &str) -> String {
    converted.reserve(1 + fragment.len());
    converted.push('#');
    converted.push_str(fragment);
    converted
}
