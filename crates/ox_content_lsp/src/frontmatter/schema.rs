use std::fs;
use std::path::Path;
use std::sync::OnceLock;

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
    static SCHEMA: OnceLock<FrontmatterSchema> = OnceLock::new();
    SCHEMA.get_or_init(parse_builtin_schema).clone()
}

fn parse_builtin_schema() -> FrontmatterSchema {
    // Compile-time object literal. A parse failure would mean the checked-in
    // schema is invalid; fall back to an empty object schema so the language
    // server still starts instead of aborting when a Markdown file is opened.
    serde_json::from_value(builtin_schema_value()).unwrap_or_else(|_| FrontmatterSchema {
        type_name: Some("object".to_string()),
        ..FrontmatterSchema::default()
    })
}

fn builtin_schema_value() -> serde_json::Value {
    serde_json::json!({
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
            "unlisted": {
                "type": "boolean",
                "description": "Builds the page but omits it from nav, search, and sitemaps."
            },
            "scheduled": {
                "type": "string",
                "description": "ISO-8601 instant. The page stays unpublished until this time."
            },
            "date": {
                "type": "string",
                "description": "ISO-8601 publish instant when scheduled is omitted."
            },
            "expiry": {
                "type": "string",
                "description": "ISO-8601 instant after which the page is unpublished."
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
    })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    fn temp_schema(name: &str) -> PathBuf {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("ox-content-lsp-schema-{nanos}-{seq}"));
        fs::create_dir_all(&dir).unwrap();
        dir.join(name)
    }

    #[test]
    fn builtin_schema_exposes_core_frontmatter_fields() {
        let schema = builtin_schema();
        assert_eq!(schema.type_name.as_deref(), Some("object"));
        for name in ["title", "description", "layout", "draft", "unlisted", "tags", "meta"] {
            assert!(schema.property(name).is_some(), "missing {name}");
        }
        let layout = schema.property("layout").unwrap();
        assert!(layout.enum_values.iter().any(|value| value.as_str() == Some("doc")));
        let meta = schema.property("meta").unwrap();
        assert_eq!(meta.additional_properties, Some(false));
        assert!(meta.property("ogImage").is_some());
    }

    #[test]
    fn load_schema_rejects_missing_and_hostile_inputs() {
        let missing = PathBuf::from("/definitely/does-not-exist-853.json");
        assert!(load_schema(&missing).unwrap_err().contains("Failed to read schema"));

        let broken_json = temp_schema("broken.json");
        fs::write(&broken_json, r#"{"type": "object", "properties": {"#).unwrap();
        assert!(load_schema(&broken_json).unwrap_err().contains("Failed to parse schema"));

        let broken_yaml = temp_schema("broken.yaml");
        fs::write(&broken_yaml, "type: [unterminated\n").unwrap();
        assert!(load_schema(&broken_yaml).unwrap_err().contains("Failed to parse schema"));

        let not_object = temp_schema("array.json");
        fs::write(&not_object, "[1, 2, 3]").unwrap();
        assert!(load_schema(&not_object).is_err());
    }

    #[test]
    fn load_schema_accepts_json_and_yaml_objects() {
        let json = temp_schema("ok.json");
        fs::write(&json, r#"{"type":"object","properties":{"title":{"type":"string"}}}"#).unwrap();
        let schema = load_schema(&json).unwrap();
        assert_eq!(schema.type_name.as_deref(), Some("object"));
        assert!(schema.property("title").is_some());

        let yaml = temp_schema("ok.yml");
        fs::write(&yaml, "type: object\nproperties:\n  draft:\n    type: boolean\n").unwrap();
        let schema = load_schema(&yaml).unwrap();
        assert!(schema.property("draft").is_some());
    }
}
