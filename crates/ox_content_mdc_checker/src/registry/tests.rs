use super::*;

const SAMPLE: &str = r#"{
  "components": [
    {
      "name": "Alert",
      "description": "Inline alert callout.",
      "attributes": [
        { "name": "tone", "description": "info | warn | error" },
        { "name": "icon" }
      ]
    },
    {
      "name": "Card",
      "attributes": [
        { "name": "title" }
      ]
    }
  ]
}"#;

#[test]
fn parses_components_and_attributes() {
    let registry = Registry::from_json(SAMPLE).expect("parse");
    let alert = registry.get("Alert").expect("Alert");
    assert_eq!(alert.description.as_deref(), Some("Inline alert callout."));
    assert_eq!(alert.attributes.len(), 2);
    let tone = alert.attributes.get("tone").expect("tone");
    assert_eq!(tone.description.as_deref(), Some("info | warn | error"));
}

#[test]
fn complete_components_filters_by_prefix() {
    let registry = Registry::from_json(SAMPLE).expect("parse");
    let names: Vec<_> = registry.complete_components("A").map(|(n, _)| n).collect();
    assert_eq!(names, vec!["Alert"]);
    let all: Vec<_> = registry.complete_components("").map(|(n, _)| n).collect();
    assert_eq!(all, vec!["Alert", "Card"]);
}

#[test]
fn complete_attributes_returns_empty_for_unknown_component() {
    let registry = Registry::from_json(SAMPLE).expect("parse");
    assert!(registry.complete_attributes("Nope", "").next().is_none());
}

#[test]
fn complete_attributes_filters_by_prefix() {
    let registry = Registry::from_json(SAMPLE).expect("parse");
    let names: Vec<_> = registry.complete_attributes("Alert", "ic").map(|(name, _)| name).collect();
    assert_eq!(names, vec!["icon"]);
}

#[test]
fn unknown_fields_are_tolerated() {
    let json = r#"{
      "components": [
        { "name": "X", "future": true, "attributes": [{ "name": "y", "extra": 1 }] }
      ],
      "$schema": "https://example.com/schema.json"
    }"#;
    let registry = Registry::from_json(json).expect("parse");
    assert!(registry.get("X").is_some());
}

#[test]
fn empty_registry_round_trips() {
    let registry = Registry::from_json("{}").expect("parse");
    assert!(registry.components.is_empty());
}

#[test]
fn missing_file_returns_ok_none() {
    let result =
        Registry::from_path(std::path::Path::new("/this/path/should/not/exist/registry.json"));
    assert!(matches!(result, Ok(None)));
}

#[test]
fn parse_error_surfaces_as_error_variant() {
    let dir = std::env::temp_dir().join("ox-content-mdc-registry-error");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("bad.json");
    std::fs::write(&path, "{ not json").unwrap();
    let result = Registry::from_path(&path);
    assert!(matches!(result, Err(RegistryError::Parse(_))));
}
