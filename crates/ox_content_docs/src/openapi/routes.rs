use crate::string_builder::{join2, join3};

use super::{OperationDoc, SPEC_SLUG_PLACEHOLDER, SpecDoc};

pub(super) fn operation_file(operation: &OperationDoc, spec: &SpecDoc) -> String {
    operation.file_name.replace(SPEC_SLUG_PLACEHOLDER, &spec.slug)
}

pub(super) fn spec_file(spec: &SpecDoc, file_name: &str) -> String {
    join3("openapi/", &spec.slug, &join2("/", file_name))
}

pub(super) fn route_path(base_path: Option<&str>, file_name: &str) -> String {
    let normalized_base = normalize_base_path(base_path.unwrap_or("/api"));
    let mut route = file_name.strip_suffix(".md").unwrap_or(file_name).to_string();
    if let Some(stripped) = route.strip_suffix("/index") {
        route = stripped.to_string();
    }
    if normalized_base.is_empty() {
        join2("/", &route)
    } else {
        join3(&normalized_base, "/", &route)
    }
}

fn normalize_base_path(base_path: &str) -> String {
    let value = base_path.trim().trim_end_matches('/');
    if value.is_empty() || value == "/" {
        return String::new();
    }
    if value.starts_with('/') { value.to_string() } else { join2("/", value) }
}
