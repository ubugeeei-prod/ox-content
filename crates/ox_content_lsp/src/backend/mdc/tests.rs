use ox_content_mdc_checker::Registry;

use super::*;

fn registry() -> Registry {
    Registry::from_json(
        r#"{
        "components": [
          {
            "name": "Alert",
            "description": "Inline alert callout.",
            "attributes": [
              { "name": "tone", "type": "info | warn | error" },
              { "name": "icon", "description": "Icon name" }
            ]
          },
          {
            "name": "Card",
            "attributes": [{ "name": "title" }]
          }
        ]
      }"#,
    )
    .expect("parse registry")
}

mod detect_site {
    use super::*;

    #[test]
    fn typing_component_name_returns_component_site() {
        let site = detect_site("paragraph <Ale").expect("site");
        assert_eq!(site, CompletionSite::ComponentName { prefix: "Ale" });
    }

    #[test]
    fn empty_component_prefix_after_lt_returns_component_site() {
        // Detection only fires after at least one uppercase character —
        // a lone `<` is ambiguous with typography and HTML comments.
        assert!(detect_site("text <").is_none());
    }

    #[test]
    fn lowercase_tag_is_left_to_other_completions() {
        assert!(detect_site("<sect").is_none());
    }

    #[test]
    fn closing_tag_is_not_a_completion_site() {
        assert!(detect_site("</Ale").is_none());
    }

    #[test]
    fn past_the_close_angle_we_drop_out() {
        assert!(detect_site("<Alert tone=\"x\"> body ").is_none());
    }

    #[test]
    fn whitespace_after_name_means_attribute_site_with_empty_prefix() {
        let site = detect_site("<Alert ").expect("site");
        assert_eq!(site, CompletionSite::AttributeName { component: "Alert", prefix: "" });
    }

    #[test]
    fn typing_attribute_name_after_existing_attribute() {
        let site = detect_site("<Alert tone=\"info\" ic").expect("site");
        assert_eq!(site, CompletionSite::AttributeName { component: "Alert", prefix: "ic" });
    }

    #[test]
    fn inside_attribute_value_is_not_a_site() {
        assert!(detect_site("<Alert tone=\"in").is_none());
    }

    #[test]
    fn right_after_equals_is_not_a_site() {
        assert!(detect_site("<Alert tone=").is_none());
    }
}

mod completion_items {
    use super::*;

    fn labels(items: &[CompletionItem]) -> Vec<&str> {
        items.iter().map(|item| item.label.as_str()).collect()
    }

    #[test]
    fn component_completion_returns_alphabetical_names_with_class_kind() {
        let registry = registry();
        let items = completion_items(&CompletionSite::ComponentName { prefix: "" }, &registry);
        assert_eq!(labels(&items), vec!["Alert", "Card"]);
        assert!(items.iter().all(|item| item.kind == Some(CompletionItemKind::CLASS)));
        let alert = &items[0];
        assert!(alert.documentation.is_some());
    }

    #[test]
    fn component_completion_filters_by_prefix() {
        let registry = registry();
        let items = completion_items(&CompletionSite::ComponentName { prefix: "A" }, &registry);
        assert_eq!(labels(&items), vec!["Alert"]);
    }

    #[test]
    fn attribute_completion_returns_props_with_property_kind() {
        let registry = registry();
        let items = completion_items(
            &CompletionSite::AttributeName { component: "Alert", prefix: "" },
            &registry,
        );
        assert_eq!(labels(&items), vec!["icon", "tone"]);
        let tone = items.iter().find(|item| item.label == "tone").unwrap();
        assert_eq!(tone.kind, Some(CompletionItemKind::PROPERTY));
        assert_eq!(tone.detail.as_deref(), Some("MDC prop: info | warn | error"));
        assert!(tone.insert_text.as_deref().unwrap().ends_with(r#"="$0""#));
    }

    #[test]
    fn attribute_completion_for_unknown_component_is_empty() {
        let registry = registry();
        let items = completion_items(
            &CompletionSite::AttributeName { component: "Unknown", prefix: "" },
            &registry,
        );
        assert!(items.is_empty());
    }
}
