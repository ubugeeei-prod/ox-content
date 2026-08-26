use std::path::{Component, Path, PathBuf};

use super::{ResolvedDataTableOptions, TableError};

pub(super) fn resolve_data_path(
    value: &str,
    options: &ResolvedDataTableOptions,
) -> Result<PathBuf, TableError> {
    let value = value.trim().trim_matches(|ch| ch == '"' || ch == '\'');
    if value.is_empty() {
        return Err(TableError::Other("Data table src is missing a path.".to_string()));
    }
    if Path::new(value).components().any(|component| matches!(component, Component::ParentDir)) {
        return Err(TableError::Other(format!("Data table path {value:?} must not contain '..'.")));
    }

    let candidate = if let Some(rest) = value.strip_prefix("@/") {
        options.root_dir.join(rest)
    } else if let Some(rest) = value.strip_prefix('/') {
        options.root_dir.join(rest)
    } else if let Some(source_path) = &options.source_path {
        source_path.parent().unwrap_or_else(|| Path::new(".")).join(value)
    } else {
        options.root_dir.join(value)
    };

    let canonical_root =
        options.root_dir.canonicalize().unwrap_or_else(|_| options.root_dir.clone());
    let canonical_candidate = candidate.canonicalize().map_err(|error| {
        TableError::Missing(format!(
            "Data table path {} could not be resolved: {error}",
            candidate.display()
        ))
    })?;

    if !canonical_candidate.starts_with(&canonical_root) {
        return Err(TableError::Other(format!(
            "Data table path {} is outside root {}.",
            canonical_candidate.display(),
            canonical_root.display()
        )));
    }

    Ok(canonical_candidate)
}
