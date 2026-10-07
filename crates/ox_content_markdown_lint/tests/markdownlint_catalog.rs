//! The rule catalog and every way a configuration can address it.
//!
//! Each rule has one `MDnnn violation` case in the conformance fixture. These
//! tests replay that case through identifiers, aliases, tags, letter case,
//! severities and the default profile, so a rule that cannot be selected or
//! silenced in one of those ways fails here by name.

use ox_content_markdown_lint::*;
use serde::Deserialize;
use serde_json::{Map, Value, json};
use std::collections::BTreeSet;

/// Identifiers and aliases of markdownlint 0.41.1, in catalog order.
const NAMES: &[(&str, &[&str])] = &[
    ("MD001", &["heading-increment"]),
    ("MD003", &["heading-style"]),
    ("MD004", &["ul-style"]),
    ("MD005", &["list-indent"]),
    ("MD007", &["ul-indent"]),
    ("MD009", &["no-trailing-spaces"]),
    ("MD010", &["no-hard-tabs"]),
    ("MD011", &["no-reversed-links"]),
    ("MD012", &["no-multiple-blanks"]),
    ("MD013", &["line-length"]),
    ("MD014", &["commands-show-output"]),
    ("MD018", &["no-missing-space-atx"]),
    ("MD019", &["no-multiple-space-atx"]),
    ("MD020", &["no-missing-space-closed-atx"]),
    ("MD021", &["no-multiple-space-closed-atx"]),
    ("MD022", &["blanks-around-headings"]),
    ("MD023", &["heading-start-left"]),
    ("MD024", &["no-duplicate-heading"]),
    ("MD025", &["single-title", "single-h1"]),
    ("MD026", &["no-trailing-punctuation"]),
    ("MD027", &["no-multiple-space-blockquote"]),
    ("MD028", &["no-blanks-blockquote"]),
    ("MD029", &["ol-prefix"]),
    ("MD030", &["list-marker-space"]),
    ("MD031", &["blanks-around-fences"]),
    ("MD032", &["blanks-around-lists"]),
    ("MD033", &["no-inline-html"]),
    ("MD034", &["no-bare-urls"]),
    ("MD035", &["hr-style"]),
    ("MD036", &["no-emphasis-as-heading"]),
    ("MD037", &["no-space-in-emphasis"]),
    ("MD038", &["no-space-in-code"]),
    ("MD039", &["no-space-in-links"]),
    ("MD040", &["fenced-code-language"]),
    ("MD041", &["first-line-heading", "first-line-h1"]),
    ("MD042", &["no-empty-links"]),
    ("MD043", &["required-headings"]),
    ("MD044", &["proper-names"]),
    ("MD045", &["no-alt-text"]),
    ("MD046", &["code-block-style"]),
    ("MD047", &["single-trailing-newline"]),
    ("MD048", &["code-fence-style"]),
    ("MD049", &["emphasis-style"]),
    ("MD050", &["strong-style"]),
    ("MD051", &["link-fragments"]),
    ("MD052", &["reference-links-images"]),
    ("MD053", &["link-image-reference-definitions"]),
    ("MD054", &["link-image-style"]),
    ("MD055", &["table-pipe-style"]),
    ("MD056", &["table-column-count"]),
    ("MD058", &["blanks-around-tables"]),
    ("MD059", &["descriptive-link-text"]),
    ("MD060", &["table-column-style"]),
];

#[derive(Deserialize)]
struct Case {
    name: String,
    source: String,
    config: Map<String, Value>,
    expected: Vec<(String, u32)>,
}

/// The fixture's violation case for every rule, with the rule's own setting.
fn violations() -> Vec<(&'static MarkdownlintRule, Case, Value)> {
    let cases: Vec<Case> =
        serde_json::from_str(include_str!("fixtures/markdownlint.json")).unwrap();
    let found: Vec<_> = cases
        .into_iter()
        .filter_map(|case| {
            let id = case.name.strip_suffix(" violation")?;
            let rule = markdownlint_rules().iter().find(|rule| rule.id == id)?;
            let setting = case.config[rule.id].clone();
            Some((rule, case, setting))
        })
        .collect();
    assert_eq!(found.len(), markdownlint_rules().len(), "one violation case per rule");
    found
}

fn pairs(source: &str, markdownlint: Value) -> Vec<(String, u32)> {
    let options: MarkdownLintOptions =
        serde_json::from_value(json!({ "markdownlint": markdownlint })).unwrap();
    let mut found: Vec<_> = lint_markdown(source, Some(options))
        .diagnostics
        .into_iter()
        .map(|value| (value.rule_id, value.line))
        .collect();
    found.sort_by(|a, b| (a.1, &a.0).cmp(&(b.1, &b.0)));
    found
}

fn only(key: &str, setting: &Value) -> Value {
    json!({ "default": false, key: setting })
}

#[test]
fn catalog_lists_the_upstream_identifiers_and_aliases_in_order() {
    let rules = markdownlint_rules();
    let actual: Vec<_> = rules.iter().map(|rule| (rule.id, rule.aliases)).collect();
    assert_eq!(actual, NAMES);
    let mut names = BTreeSet::new();
    let tags: BTreeSet<_> = rules.iter().flat_map(|rule| rule.tags).copied().collect();
    for rule in rules {
        assert_eq!(rule.id, format!("MD{:03}", rule.number));
        assert!(!rule.description.is_empty() && !rule.description.ends_with('.'), "{}", rule.id);
        assert!(!rule.tags.is_empty(), "{}", rule.id);
        for name in std::iter::once(&rule.id).chain(rule.aliases) {
            assert!(names.insert(name.to_ascii_lowercase()), "{name} is ambiguous");
            assert!(!tags.contains(name), "{name} is both a rule name and a tag");
        }
    }
    assert!(tags.iter().all(|tag| *tag == tag.to_ascii_lowercase()));
    // Removed upstream; they must stay unknown rather than silently accepted.
    for removed in ["MD002", "MD006", "MD008", "MD015", "MD016", "MD017", "MD057", "MD061"] {
        assert!(serde_json::from_value::<MarkdownlintConfig>(json!({ removed: true })).is_err());
    }
    assert_eq!(
        serde_json::to_value(&rules[0]).unwrap(),
        json!({
            "number": 1,
            "id": "MD001",
            "description": "Heading levels should only increment by one level at a time",
            "aliases": ["heading-increment"],
            "tags": ["headings"]
        })
    );
}

#[test]
fn every_alias_in_any_letter_case_selects_the_same_rule_as_its_identifier() {
    for (rule, case, setting) in violations() {
        assert_eq!(pairs(&case.source, only(rule.id, &setting)), case.expected, "{}", rule.id);
        let mut keys = vec![rule.id.to_ascii_lowercase()];
        for alias in rule.aliases {
            keys.extend([(*alias).to_string(), alias.to_ascii_uppercase()]);
        }
        for key in keys {
            assert_eq!(pairs(&case.source, only(&key, &setting)), case.expected, "{key}");
        }
    }
}

#[test]
fn a_tag_enables_exactly_its_member_rules() {
    let cases = violations();
    let tags: BTreeSet<_> = markdownlint_rules().iter().flat_map(|rule| rule.tags).collect();
    for tag in tags {
        let members: Vec<_> =
            markdownlint_rules().iter().filter(|rule| rule.tags.contains(tag)).collect();
        for (rule, case, setting) in &cases {
            let found = pairs(&case.source, only(tag, &json!(true)));
            assert!(
                found.iter().all(|(id, _)| members.iter().any(|member| member.id == id)),
                "{tag} reported a non-member on the {} case: {found:?}",
                rule.id
            );
            // Rules that need no options fire through their tag exactly as alone.
            if rule.tags.contains(tag) && *setting == json!(true) {
                let own: Vec<_> = found.into_iter().filter(|(id, _)| id == rule.id).collect();
                assert_eq!(own, case.expected, "{tag} -> {}", rule.id);
            }
        }
        // The specific rule wins over its tag in both directions.
        for member in members {
            let off = json!({ "default": false, *tag: true, member.id: false });
            let on = json!({ "default": false, *tag: false, member.id: true });
            let (_, case, setting) = cases.iter().find(|(rule, ..)| rule.id == member.id).unwrap();
            assert!(pairs(&case.source, off).iter().all(|(id, _)| id != member.id), "{tag}");
            if *setting == json!(true) {
                assert_eq!(pairs(&case.source, on), case.expected, "{tag} -> {}", member.id);
            }
        }
    }
}

#[test]
fn switching_one_rule_off_leaves_every_other_default_diagnostic_alone() {
    for (rule, case, setting) in violations() {
        let all = pairs(&case.source, json!({}));
        assert_eq!(all, pairs(&case.source, json!(true)), "{}", rule.id);
        assert_eq!(all, pairs(&case.source, json!({ "default": true })), "{}", rule.id);
        if setting == json!(true) {
            let own: Vec<_> = all.iter().filter(|(id, _)| id == rule.id).cloned().collect();
            assert_eq!(own, case.expected, "{} in the default profile", rule.id);
        }
        let others: Vec<_> = all.into_iter().filter(|(id, _)| id != rule.id).collect();
        for off in [json!(false), json!({ "enabled": false })] {
            assert_eq!(pairs(&case.source, json!({ rule.id: off })), others, "{}", rule.id);
        }
        assert!(pairs(&case.source, json!({ "default": false })).is_empty());
    }
    // `false` hands the document back to the native structure rules.
    let native = pairs("#  Title!\ntext \n", json!(false));
    assert!(!native.is_empty() && native.iter().all(|(id, _)| !id.starts_with("MD")), "{native:?}");
}

#[test]
fn every_rule_reports_only_itself_and_honours_warning_severity() {
    for (rule, case, setting) in violations() {
        let mut warning = setting.as_object().cloned().unwrap_or_default();
        warning.insert("severity".into(), json!("warning"));
        let options: MarkdownLintOptions = serde_json::from_value(
            json!({ "markdownlint": { "default": false, rule.id: warning } }),
        )
        .unwrap();
        let result = lint_markdown(&case.source, Some(options));
        assert_eq!(result.diagnostics.len(), case.expected.len(), "{}", rule.id);
        for value in &result.diagnostics {
            assert_eq!((value.rule_id.as_str(), value.severity.as_str()), (rule.id, "warning"));
            assert_eq!(value.message, rule.description);
        }
        assert_eq!((result.error_count, result.warning_count), (0, case.expected.len() as u32));
    }
}

#[test]
fn invalid_configuration_names_the_offending_key() {
    for (config, message) in [
        (json!("strict"), "must be a boolean or configuration object"),
        (json!([]), "must be a boolean or configuration object"),
        (json!(null), "must be a boolean or configuration object"),
        (json!({ "MD999": true }), "Unknown markdownlint rule or tag: MD999"),
        (json!({ "no-such-alias": false }), "Unknown markdownlint rule or tag: no-such-alias"),
        (json!({ "default": "yes" }), "Unknown markdownlint rule or tag: default"),
        (json!({ "$schema": 1 }), "Unknown markdownlint rule or tag: $schema"),
        (json!({ "MD009": 1 }), "Invalid configuration for MD009"),
        (json!({ "MD009": "info" }), "Invalid configuration for MD009"),
        (json!({ "MD009": null }), "Invalid configuration for MD009"),
        (json!({ "MD009": [] }), "Invalid configuration for MD009"),
        (json!({ "MD009": { "br_spaces": -1 } }), "MD009.br_spaces"),
        (json!({ "MD009": { "br_spaces": "2" } }), "MD009.br_spaces"),
        (json!({ "MD009": { "br_spaces": 1.5 } }), "MD009.br_spaces"),
        (json!({ "MD009": { "strict": 1 } }), "MD009.strict"),
        (json!({ "MD009": { "severity": "info" } }), "MD009.severity"),
        (json!({ "MD009": { "enabled": "no" } }), "MD009.enabled"),
        (json!({ "MD013": { "line_length": 0 } }), "MD013.line_length"),
        (json!({ "MD025": { "level": 7 } }), "MD025.level"),
        (json!({ "MD003": { "style": "ATX" } }), "MD003.style"),
        (json!({ "MD043": { "headings": "# Title" } }), "MD043.headings"),
        (json!({ "MD043": { "headings": [1] } }), "MD043.headings"),
        (json!({ "MD022": { "lines_above": -2 } }), "MD022.lines_above"),
        (json!({ "MD022": { "lines_above": [1, "x"] } }), "MD022.lines_above"),
        (json!({ "MD041": { "front_matter_title": "[" } }), "Invalid native regex"),
        (json!({ "line-length": { "lenght": 80 } }), "line-length.lenght"),
        // Options given to a tag must exist on every rule the tag selects.
        (json!({ "headings": { "style": "atx" } }), "headings.style"),
    ] {
        let error = serde_json::from_value::<MarkdownlintConfig>(config.clone())
            .map(|_| ())
            .unwrap_err()
            .to_string();
        assert!(error.contains(message), "{config}: {error}");
        let nested =
            serde_json::from_value::<MarkdownLintOptions>(json!({ "markdownlint": config }));
        assert_eq!(nested.is_err(), !config.is_null(), "{config}");
    }
    for valid in [
        json!(true),
        json!(false),
        json!({}),
        json!({ "$schema": "https://example.com/schema.json", "default": false }),
        json!({ "whitespace": { "severity": "warning" } }),
        json!({ "line-length": { "line_length": 100, "stern": true } }),
        json!({ "MD022": { "lines_above": [1, 2, 0], "lines_below": -1 } }),
        json!({ "MD051": { "ignored_pattern": "" }, "MD041": { "front_matter_title": "" } }),
    ] {
        assert!(serde_json::from_value::<MarkdownlintConfig>(valid.clone()).is_ok(), "{valid}");
    }
}

#[test]
fn the_option_schema_matches_the_catalog_and_every_default_is_accepted() {
    let schema: Map<String, Value> =
        serde_json::from_str(include_str!("../src/lint/markdownlint/parameters.json")).unwrap();
    let ids: Vec<_> = markdownlint_rules().iter().map(|rule| rule.id).collect();
    assert_eq!(schema.keys().map(String::as_str).collect::<Vec<_>>(), ids);
    let accepts = |rule: &str, option: &str, value: Value| {
        serde_json::from_value::<MarkdownlintConfig>(json!({ rule: { option: value } })).is_ok()
    };
    let mut options = 0;
    for (rule, parameters) in &schema {
        let parameters = parameters.as_object().unwrap();
        assert!(parameters.contains_key("enabled") && parameters.contains_key("severity"));
        for (option, definition) in parameters {
            options += 1;
            let at = format!("{rule}.{option}");
            assert!(accepts(rule, option, definition["default"].clone()), "{at} default");
            assert!(!accepts(rule, option, json!({})), "{at} object");
            assert!(!accepts(rule, option, json!(null)), "{at} null");
            for value in definition["enum"].as_array().into_iter().flatten() {
                assert!(accepts(rule, option, value.clone()), "{at} = {value}");
            }
            let kinds: Vec<_> = match &definition["type"] {
                Value::Array(kinds) => kinds.iter().filter_map(Value::as_str).collect(),
                kind => kind.as_str().into_iter().collect(),
            };
            assert_eq!(accepts(rule, option, json!(true)), kinds.contains(&"boolean"), "{at}");
            assert_eq!(accepts(rule, option, json!(3)), kinds.contains(&"integer"), "{at}");
            assert_eq!(accepts(rule, option, json!([])), kinds.contains(&"array"), "{at}");
            let free_text = kinds.contains(&"string") && definition["enum"].is_null();
            assert_eq!(accepts(rule, option, json!("-")), free_text, "{at} string");
            if let Some(minimum) = definition["minimum"].as_i64() {
                assert!(accepts(rule, option, json!(minimum)), "{at} minimum");
                assert!(!accepts(rule, option, json!(minimum - 1)), "{at} below minimum");
            }
            if let Some(maximum) = definition["maximum"].as_i64() {
                assert!(accepts(rule, option, json!(maximum)), "{at} maximum");
                assert!(!accepts(rule, option, json!(maximum + 1)), "{at} above maximum");
            }
            if kinds.contains(&"array") && definition["items"]["type"] == "string" {
                assert!(!accepts(rule, option, json!([1])), "{at} items");
            }
        }
    }
    assert_eq!(options, 175);
}
