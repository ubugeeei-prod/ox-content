use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use rustc_hash::FxHashMap;

use crate::line_index::LineIndex;
use crate::target::{classify, percent_decode, split_anchor};
use crate::{Diagnostic, LinkKind, Severity, SiteCheckOptions, SiteReport};

use self::html::{HtmlDocument, HtmlLink, parse_html};
use self::resolve::{ResolveError, resolve_site_target};

mod html;
mod resolve;

struct Page {
    source: String,
    document: HtmlDocument,
}

pub fn check_site(options: &SiteCheckOptions) -> io::Result<Vec<SiteReport>> {
    let site_dir = fs::canonicalize(&options.site_dir)?;
    let mut html_files = Vec::new();
    collect_html_files(&site_dir, &mut html_files)?;
    html_files.sort();

    let mut ordered_pages = Vec::with_capacity(html_files.len());
    for file in html_files {
        let source = fs::read_to_string(&file)?;
        let document = parse_html(&source);
        ordered_pages.push((file, Page { source, document }));
    }

    // Walk the same owned pages we just parsed. Cross-file lookups go through
    // this map; the current page never needs a fallible get-after-insert.
    let pages: FxHashMap<&Path, &Page> =
        ordered_pages.iter().map(|(file, page)| (file.as_path(), page)).collect();

    let mut reports = Vec::new();
    for (file, page) in &ordered_pages {
        let line_index = LineIndex::new(&page.source);
        let mut diagnostics = Vec::new();
        for link in &page.document.links {
            if let Some(diagnostic) =
                check_link(&site_dir, file, &options.base, link, &pages, &line_index)
            {
                diagnostics.push(diagnostic);
            }
        }
        if !diagnostics.is_empty() {
            reports.push(SiteReport { file_path: file.clone(), diagnostics });
        }
    }
    Ok(reports)
}

fn check_link(
    site_dir: &Path,
    source_file: &Path,
    base: &str,
    link: &HtmlLink,
    pages: &FxHashMap<&Path, &Page>,
    line_index: &LineIndex,
) -> Option<Diagnostic> {
    let target = link.target.trim();
    if target.starts_with("//") {
        return None;
    }
    match classify(target) {
        LinkKind::External | LinkKind::Scheme => return None,
        _ => {}
    }

    let (path_with_query, anchor) = split_anchor(target);
    let path = path_with_query.split_once('?').map_or(path_with_query, |(path, _)| path);
    let target_file = if path.is_empty() {
        source_file.to_path_buf()
    } else {
        match resolve_site_target(site_dir, source_file, base, path) {
            Ok(path) => path,
            Err(error) => {
                return Some(diagnostic(
                    line_index,
                    link,
                    target,
                    if anchor.is_some() { LinkKind::FileAnchor } else { LinkKind::File },
                    Severity::Error,
                    resolve_error_message(path, error),
                ));
            }
        }
    };

    if let Some(fragment) = anchor.filter(|fragment| !fragment.is_empty() && *fragment != "top") {
        let decoded = percent_decode(fragment);
        let Some(target_page) = pages.get(target_file.as_path()) else {
            return Some(diagnostic(
                line_index,
                link,
                target,
                LinkKind::FileAnchor,
                Severity::Error,
                format!(
                    "Fragment `#{fragment}` targets a non-HTML file (resolved to {}).",
                    target_file.display()
                ),
            ));
        };
        if !target_page.document.anchors.contains(decoded.as_ref()) {
            return Some(diagnostic(
                line_index,
                link,
                target,
                if path.is_empty() { LinkKind::Anchor } else { LinkKind::FileAnchor },
                Severity::Error,
                format!("Anchor `#{fragment}` is missing from {}.", target_file.display()),
            ));
        }
    }

    if !link.is_redirect_destination
        && pages.get(target_file.as_path()).is_some_and(|page| page.document.is_redirect)
    {
        return Some(diagnostic(
            line_index,
            link,
            target,
            LinkKind::File,
            Severity::Warning,
            format!("Link resolves to redirect page {}.", target_file.display()),
        ));
    }
    None
}

fn diagnostic(
    line_index: &LineIndex,
    link: &HtmlLink,
    target: &str,
    kind: LinkKind,
    severity: Severity,
    message: String,
) -> Diagnostic {
    let (line, column) = line_index.position(link.offset);
    let (end_line, end_column) = line_index.position(link.offset + link.target.len());
    Diagnostic {
        severity,
        message,
        line,
        column,
        end_line,
        end_column,
        kind,
        target: target.to_string(),
    }
}

fn resolve_error_message(target: &str, error: ResolveError) -> String {
    match error {
        ResolveError::OutsideBase => {
            format!("Internal target `{target}` is outside the configured site base.")
        }
        ResolveError::EscapesRoot => {
            format!("Internal target `{target}` escapes the generated site root.")
        }
        ResolveError::Missing(resolved) => format!(
            "Broken generated target: `{target}` does not exist (resolved to {}).",
            resolved.display()
        ),
    }
}

fn collect_html_files(directory: &Path, output: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            continue;
        }
        let path = entry.path();
        if file_type.is_dir() {
            collect_html_files(&path, output)?;
        } else if path.extension().is_some_and(|extension| extension == "html") {
            output.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
