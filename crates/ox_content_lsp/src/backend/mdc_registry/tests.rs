use super::*;

#[test]
fn indexes_component_and_attribute_name_ranges() {
    let source = r#"{
  "components": [
    {
      "name": "Alert",
      "attributes": [
        { "name": "tone", "type": "info | warn | error" }
      ]
    }
  ]
}
"#;
    let locations = locations_for_source(source);

    let component = locations.component("Alert").expect("component location");
    assert_eq!(component.start.line, 3);
    assert_eq!(component.start.character, 15);
    assert_eq!(component.end.character, 20);

    let attribute = locations.attribute("Alert", "tone").expect("attribute location");
    assert_eq!(attribute.start.line, 5);
    assert_eq!(attribute.start.character, 19);
    assert_eq!(attribute.end.character, 23);
}

#[test]
fn ignores_same_attribute_name_on_other_components() {
    let source = r#"{
  "components": [
    { "name": "Alert", "attributes": [{ "name": "tone" }] },
    { "name": "Badge", "attributes": [{ "name": "tone" }] }
  ]
}
"#;
    let locations = locations_for_source(source);

    let alert = locations.attribute("Alert", "tone").expect("alert tone");
    let badge = locations.attribute("Badge", "tone").expect("badge tone");
    assert_ne!(alert.start.line, badge.start.line);
}
