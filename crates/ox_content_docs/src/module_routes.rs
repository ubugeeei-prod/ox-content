use std::collections::{BTreeMap, HashSet};
use std::path::Path;

use rustc_hash::FxHashMap;

use crate::model::ApiDocModule;
use crate::string_builder::StringBuilder;

/// Assigns stable, collision-free routes to flat file-grouped API modules.
///
/// The shallowest module keeps the historical basename route. Colliding nested
/// modules gain the minimum parent context needed to disambiguate them.
pub fn build_flat_module_routes(docs: &[ApiDocModule]) -> FxHashMap<String, String> {
    let mut groups = BTreeMap::<String, Vec<&ApiDocModule>>::new();
    for doc in docs {
        groups.entry(module_file_name(&doc.file)).or_default().push(doc);
    }

    // Reserve every historical basename before choosing qualified routes. This
    // prevents a colliding module from taking a route owned by an unrelated file.
    let mut used = groups.keys().cloned().collect::<HashSet<_>>();
    let mut routes = FxHashMap::default();

    for (base_route, mut group) in groups {
        group.sort_by(|left, right| {
            route_parent_parts(&left.file)
                .len()
                .cmp(&route_parent_parts(&right.file).len())
                .then_with(|| left.file.cmp(&right.file))
        });

        if let Some(primary) = group.first() {
            routes.insert(primary.file.clone(), base_route.clone());
        }

        for doc in group.into_iter().skip(1) {
            let route = qualified_route(&doc.file, &base_route, &mut used);
            routes.insert(doc.file.clone(), route);
        }
    }

    routes
}

pub fn module_file_name(file_path: &str) -> String {
    let mut file_name = file_stem(file_path);
    if file_name == "index" {
        file_name = "index-module".to_string();
    }
    sanitize_doc_path_segment(&file_name)
}

fn qualified_route(file_path: &str, fallback: &str, used: &mut HashSet<String>) -> String {
    let stem = sanitize_doc_path_segment(&file_stem(file_path));
    let parents = route_parent_parts(file_path);

    for count in 1..=parents.len() {
        let first = parents.len() - count;
        let mut route = parents[first..].join("-");
        if !route.is_empty() {
            route.push('-');
        }
        route.push_str(&stem);
        let route = sanitize_doc_path_segment(&route);
        if used.insert(route.clone()) {
            return route;
        }
    }

    let mut suffix = 2;
    loop {
        let mut route = StringBuilder::with_capacity(fallback.len() + 4);
        route.push_str(fallback);
        route.push_char('-');
        route.push_usize(suffix);
        let route = route.into_string();
        if used.insert(route.clone()) {
            return route;
        }
        suffix += 1;
    }
}

fn route_parent_parts(file_path: &str) -> Vec<String> {
    let normalized = file_path.replace('\\', "/");
    let mut parts = normalized
        .split('/')
        .filter(|part| !part.is_empty())
        .map(sanitize_doc_path_segment)
        .collect::<Vec<_>>();
    parts.pop();

    // Source-relative routes are stable across checkout locations and package
    // managers. The closest `src` marker is the useful namespace boundary.
    if let Some(index) = parts.iter().rposition(|part| part == "src") {
        parts.drain(..=index);
    }
    parts
}

fn file_stem(file_path: &str) -> String {
    Path::new(file_path)
        .file_stem()
        .and_then(|value| value.to_str())
        .map_or_else(|| file_path.to_string(), ToString::to_string)
}

fn sanitize_doc_path_segment(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|ch| match ch {
            '/' | '\\' | '?' | '#' | '[' | ']' | '<' | '>' | ':' | '"' | '|' | '*' => '-',
            _ => ch,
        })
        .collect::<String>();
    if sanitized.is_empty() { "symbol".to_string() } else { sanitized }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_routes_preserve_primary_and_qualify_collisions() {
        let docs = [
            module("/repo/src/index.ts"),
            module("/repo/src/island/index.ts"),
            module("/repo/src/plugins/index.ts"),
            module("/repo/src/transform.ts"),
            module("/repo/src/plugins/github/transform.ts"),
        ];

        let routes = build_flat_module_routes(&docs);

        assert_eq!(routes["/repo/src/index.ts"], "index-module");
        assert_eq!(routes["/repo/src/island/index.ts"], "island-index");
        assert_eq!(routes["/repo/src/plugins/index.ts"], "plugins-index");
        assert_eq!(routes["/repo/src/transform.ts"], "transform");
        assert_eq!(routes["/repo/src/plugins/github/transform.ts"], "github-transform");
    }

    fn module(file: &str) -> ApiDocModule {
        ApiDocModule { file: file.to_string(), ..ApiDocModule::default() }
    }
}
