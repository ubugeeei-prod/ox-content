use std::fs;
use std::path::Path;

use crate::frontmatter::FrontmatterSchema;

pub fn load_schema(path: &Path) -> Result<FrontmatterSchema, String> {
    let content = fs::read_to_string(path)
        .map_err(|error| format!("Failed to read schema {}: {error}", path.display()))?;
    let extension = path.extension().and_then(|value| value.to_str()).unwrap_or_default();

    if matches!(extension, "yaml" | "yml") {
        serde_yaml::from_str::<FrontmatterSchema>(&content)
            .map_err(|error| format!("Failed to parse schema {}: {error}", path.display()))
    } else {
        serde_json::from_str::<FrontmatterSchema>(&content)
            .map_err(|error| format!("Failed to parse schema {}: {error}", path.display()))
    }
}

pub fn builtin_schema() -> FrontmatterSchema {
    serde_json::from_value(serde_json::json!({
        "type": "object",
        "properties": {
            "title": {
                "type": "string",
                "description": "Page title used by previews and generated pages."
            },
            "description": {
                "type": "string",
                "description": "Short page summary used for metadata."
            },
            "layout": {
                "type": "string",
                "enum": ["doc", "home", "page"],
                "description": "Page layout."
            },
            "draft": {
                "type": "boolean",
                "description": "Marks the page as draft content."
            },
            "tags": {
                "type": "array",
                "items": { "type": "string" },
                "description": "Tags associated with the page."
            },
            "meta": {
                "type": "object",
                "description": "HTML metadata and social preview fields.",
                "additionalProperties": false,
                "properties": {
                    "title": {
                        "type": "string",
                        "description": "Metadata title. Falls back to the page title."
                    },
                    "description": {
                        "type": "string",
                        "description": "Metadata description. Falls back to the page description."
                    },
                    "image": {
                        "type": "string",
                        "description": "Social preview image URL or project-relative path."
                    },
                    "ogImage": {
                        "type": "string",
                        "description": "Open Graph image URL or project-relative path."
                    },
                    "canonical": {
                        "type": "string",
                        "description": "Canonical page URL."
                    },
                    "robots": {
                        "type": "string",
                        "enum": [
                            "index,follow",
                            "noindex,nofollow",
                            "noindex,follow",
                            "index,nofollow"
                        ],
                        "description": "Robots indexing directive."
                    },
                    "keywords": {
                        "type": "string",
                        "description": "Comma-separated metadata keywords."
                    }
                }
            }
        }
    }))
    .expect("builtin frontmatter schema is valid")
}
