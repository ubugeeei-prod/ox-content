mod paths;
mod typedoc_order;
mod typedoc_routes;

use crate::model::ApiDocEntry;
use crate::string_builder::join3;

fn nav_entry(name: &str, kind: &str) -> ApiDocEntry {
    ApiDocEntry {
        name: name.to_string(),
        kind: kind.to_string(),
        file: join3("/repo/src/", name, ".ts"),
        ..ApiDocEntry::default()
    }
}
