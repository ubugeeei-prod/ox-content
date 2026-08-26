use std::path::{Path, PathBuf};

use super::ResolvedPartials;

pub(super) fn resolve_partial_path(
    value: &str,
    options: &ResolvedPartials,
    current_source: Option<&Path>,
) -> Result<PathBuf, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err("Partial directive is missing a path.".to_string());
    }

    let candidate = if let Some(rest) = value.strip_prefix("@/") {
        options.root_dir.join(rest)
    } else if let Some(rest) = value.strip_prefix('/') {
        options.root_dir.join(rest)
    } else if is_explicit_relative(value) {
        if let Some(source_path) = current_source {
            source_path.parent().unwrap_or_else(|| Path::new(".")).join(value)
        } else {
            options.root_dir.join(value)
        }
    } else {
        options.root_dir.join(&options.root).join(value)
    };

    let canonical_root =
        options.root_dir.canonicalize().unwrap_or_else(|_| options.root_dir.clone());
    let canonical_candidate = candidate.canonicalize().map_err(|error| {
        format!("Partial path {} could not be resolved: {error}", candidate.display())
    })?;

    if !canonical_candidate.starts_with(&canonical_root) {
        return Err(format!(
            "Partial path {} is outside root {}.",
            canonical_candidate.display(),
            canonical_root.display()
        ));
    }

    Ok(canonical_candidate)
}

fn is_explicit_relative(value: &str) -> bool {
    value.starts_with("./") || value.starts_with("../") || Path::new(value).is_absolute()
}
