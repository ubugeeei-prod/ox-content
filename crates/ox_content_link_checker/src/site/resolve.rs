use std::path::{Component, Path, PathBuf};

use crate::target::percent_decode;

#[derive(Debug)]
pub enum ResolveError {
    OutsideBase,
    EscapesRoot,
    Missing(PathBuf),
}

pub fn resolve_site_target(
    site_dir: &Path,
    source_file: &Path,
    base: &str,
    raw_path: &str,
) -> Result<PathBuf, ResolveError> {
    let decoded = percent_decode(raw_path);
    let normalized_url = decoded.replace('\\', "/");
    let relative = if normalized_url.starts_with('/') {
        strip_base(&normalized_url, base).ok_or(ResolveError::OutsideBase)?.to_string()
    } else {
        let source_relative = source_file
            .parent()
            .and_then(|parent| parent.strip_prefix(site_dir).ok())
            .unwrap_or_else(|| Path::new(""));
        source_relative.join(normalized_url).to_string_lossy().into_owned()
    };

    let normalized = normalize_under_root(site_dir, Path::new(&relative))?;
    let fallback = route_fallback(&normalized, raw_path);
    if normalized.is_file() {
        return Ok(normalized);
    }
    if normalized.is_dir() {
        let index = normalized.join("index.html");
        if index.is_file() {
            return Ok(index);
        }
    }
    for candidate in route_candidates(&normalized, raw_path) {
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err(ResolveError::Missing(fallback))
}

fn strip_base<'a>(path: &'a str, base: &str) -> Option<&'a str> {
    let normalized_base = format!("/{}/", base.trim_matches('/'));
    if normalized_base == "//" {
        return Some(path.trim_start_matches('/'));
    }
    if path == normalized_base.trim_end_matches('/') {
        return Some("");
    }
    path.strip_prefix(&normalized_base)
}

fn normalize_under_root(root: &Path, relative: &Path) -> Result<PathBuf, ResolveError> {
    let mut output = root.to_path_buf();
    let floor = output.components().count();
    for component in relative.components() {
        match component {
            Component::CurDir => {}
            Component::Normal(part) => output.push(part),
            Component::ParentDir => {
                if output.components().count() == floor {
                    return Err(ResolveError::EscapesRoot);
                }
                output.pop();
            }
            Component::RootDir | Component::Prefix(_) => return Err(ResolveError::EscapesRoot),
        }
    }
    Ok(output)
}

fn route_candidates(path: &Path, raw_path: &str) -> Vec<PathBuf> {
    if raw_path.ends_with('/') {
        return vec![path.join("index.html")];
    }
    if path.extension().is_none() {
        return vec![path.join("index.html"), path.with_extension("html")];
    }
    Vec::new()
}

fn route_fallback(path: &Path, raw_path: &str) -> PathBuf {
    route_candidates(path, raw_path).into_iter().next().unwrap_or_else(|| path.to_path_buf())
}
