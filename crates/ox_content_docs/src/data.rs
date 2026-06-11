use serde_json::json;

mod serialize;
mod summary;

use serialize::module_to_json;
use summary::build_docs_summary;

use crate::model::ApiDocModule;
#[allow(unused_imports)]
use crate::profile_span;

/// Generates the machine-readable docs data JSON payload.
///
/// The returned JSON is pretty-formatted and preserves the shape consumed by
/// TypeScript docs tooling.
pub fn generate_docs_data_json(
    docs: &[ApiDocModule],
    generated_at: &str,
) -> serde_json::Result<String> {
    profile_span!("docs::generate_json");
    serde_json::to_string_pretty(&json!({
        "version": 1,
        "generatedAt": generated_at,
        "summary": build_docs_summary(docs),
        "modules": docs.iter().map(module_to_json).collect::<Vec<_>>(),
    }))
}

#[cfg(test)]
mod tests;
