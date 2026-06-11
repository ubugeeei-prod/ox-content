use std::path::{Path, PathBuf};

use super::options::GraphOptions;
use crate::string_builder::join4;

pub(super) fn graph_root(options: &GraphOptions) -> PathBuf {
    options.root.as_ref().map_or_else(
        || std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        PathBuf::from,
    )
}

pub(super) fn module_name_from_path(path: &Path) -> String {
    path.file_stem().and_then(|stem| stem.to_str()).unwrap_or("module").to_string()
}

pub(super) fn is_local_specifier(specifier: &str) -> bool {
    specifier.starts_with('.') || specifier.starts_with('/')
}

pub(super) fn external_package_name(specifier: &str) -> String {
    if let Some(rest) = specifier.strip_prefix('@') {
        let mut segments = rest.split('/');
        let scope = segments.next().unwrap_or_default();
        let package = segments.next().unwrap_or_default();
        if !scope.is_empty() && !package.is_empty() {
            return join4("@", scope, "/", package);
        }
    }

    specifier.split('/').next().unwrap_or(specifier).to_string()
}

pub(super) fn absolutize(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

pub(super) fn normalize_existing_path(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}
