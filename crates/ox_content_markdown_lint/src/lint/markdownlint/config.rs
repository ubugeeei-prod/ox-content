use super::catalog;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::sync::LazyLock;

static PARAMETERS: LazyLock<Value> =
    LazyLock::new(|| serde_json::from_str(include_str!("parameters.json")).unwrap_or(Value::Null));

/// Native markdownlint configuration: MD identifiers, aliases, and rule tags.
#[derive(Clone, Debug, Serialize)]
#[serde(transparent)]
pub struct MarkdownlintConfig(pub Value);

impl Default for MarkdownlintConfig {
    fn default() -> Self {
        Self(serde_json::json!({}))
    }
}

impl<'de> Deserialize<'de> for MarkdownlintConfig {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(deserializer)?;
        validate(&value).map_err(serde::de::Error::custom)?;
        Ok(Self(value))
    }
}

pub(super) fn validate(value: &Value) -> Result<(), String> {
    if value.is_boolean() {
        return Ok(());
    }
    let object =
        value.as_object().ok_or("markdownlint must be a boolean or configuration object")?;
    for (key, value) in object {
        if key == "$schema" && value.is_string() {
            continue;
        }
        if key == "default" && value.is_boolean() {
            continue;
        }
        let mask = catalog::mask(key);
        if mask == 0 {
            return Err(format!("Unknown markdownlint rule or tag: {key}"));
        }
        if value.is_boolean() || matches!(value.as_str(), Some("error" | "warning")) {
            continue;
        }
        let options =
            value.as_object().ok_or_else(|| format!("Invalid configuration for {key}"))?;
        for rule in catalog::RULES.iter().filter(|rule| mask & (1 << rule.number) != 0) {
            for (name, value) in options {
                let schema = &PARAMETERS[rule.id][name];
                if schema.is_null() || !matches_schema(value, schema) {
                    return Err(format!("Invalid markdownlint option {key}.{name}"));
                }
                if matches!(name.as_str(), "front_matter_title" | "ignored_pattern")
                    && let Some(pattern) = value.as_str()
                    && !pattern.is_empty()
                {
                    regex::Regex::new(pattern)
                        .map_err(|error| format!("Invalid native regex {key}.{name}: {error}"))?;
                }
            }
        }
    }
    Ok(())
}

fn matches_schema(value: &Value, schema: &Value) -> bool {
    if let Some(alternatives) = schema["oneOf"].as_array() {
        return alternatives.iter().any(|item| matches_schema(value, item));
    }
    let valid = if let Some(types) = schema["type"].as_array() {
        types.iter().filter_map(Value::as_str).any(|kind| matches_type(value, schema, kind))
    } else {
        schema["type"].as_str().is_some_and(|kind| matches_type(value, schema, kind))
    };
    valid && schema["enum"].as_array().is_none_or(|items| items.contains(value))
}

fn matches_type(value: &Value, schema: &Value, kind: &str) -> bool {
    match kind {
        "boolean" => value.is_boolean(),
        "integer" => value.as_i64().is_some_and(|n| {
            n >= schema["minimum"].as_i64().unwrap_or(i64::MIN)
                && n <= schema["maximum"].as_i64().unwrap_or(i64::MAX)
        }),
        "string" => value.is_string(),
        "array" => value.as_array().is_some_and(|items| {
            items.iter().all(|item| matches_schema(item, &schema["items"]))
                && items.len() >= schema["minItems"].as_u64().unwrap_or(0) as usize
                && items.len() <= schema["maxItems"].as_u64().unwrap_or(u64::MAX) as usize
        }),
        _ => false,
    }
}

#[derive(Clone)]
pub(in crate::lint) struct Settings {
    pub enabled: u64,
    pub inline_config: bool,
    pub configuration: MarkdownlintConfig,
    pub rules: [Value; 61],
    pub regexes: [Option<regex::Regex>; 61],
    pub proper_names: Vec<(String, regex::Regex)>,
}

impl Settings {
    pub fn new(config: &MarkdownlintConfig) -> Self {
        let mut result = Self {
            inline_config: true,
            configuration: config.clone(),
            enabled: if config.0 == false || config.0["default"] == false {
                0
            } else {
                catalog::all()
            },
            rules: std::array::from_fn(|_| Value::Null),
            regexes: std::array::from_fn(|_| None),
            proper_names: Vec::new(),
        };
        // Tags first, then aliases, then IDs; specific rules override groups.
        if let Some(object) = config.0.as_object() {
            for priority in 0..3 {
                for (key, value) in object {
                    let is_id = catalog::RULES.iter().any(|r| r.id.eq_ignore_ascii_case(key));
                    let is_alias = catalog::RULES
                        .iter()
                        .any(|r| r.aliases.iter().any(|v| v.eq_ignore_ascii_case(key)));
                    let rank = if is_id { 2 } else { i32::from(is_alias) };
                    if rank != priority {
                        continue;
                    }
                    for rule in
                        catalog::RULES.iter().filter(|r| catalog::mask(key) & (1 << r.number) != 0)
                    {
                        result.rules[rule.number] = value.clone();
                        if value == false || value["enabled"] == false {
                            result.enabled &= !(1 << rule.number);
                        } else {
                            result.enabled |= 1 << rule.number;
                        }
                    }
                }
            }
        }
        for number in [1, 25, 41, 51] {
            let key = if number == 51 { "ignored_pattern" } else { "front_matter_title" };
            let default = if number == 51 { "" } else { r"^\s*title\s*[:=]" };
            let pattern = result.string(number, key, default);
            result.regexes[number] =
                if pattern.is_empty() { None } else { regex::Regex::new(pattern).ok() };
        }
        result.proper_names = result
            .strings(44, "names")
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .filter(|v| !v.is_empty())
            .filter_map(|name| {
                regex::Regex::new(&format!("(?i){}", regex::escape(name)))
                    .ok()
                    .map(|pattern| (name.to_string(), pattern))
            })
            .collect();
        result
    }
    #[inline]
    pub fn on(&self, rule: usize) -> bool {
        self.enabled & (1 << rule) != 0
    }
    pub fn boolean(&self, rule: usize, name: &str, default: bool) -> bool {
        self.rules[rule][name].as_bool().unwrap_or(default)
    }
    pub fn number(&self, rule: usize, name: &str, default: usize) -> usize {
        self.rules[rule][name].as_u64().map_or(default, |n| n as usize)
    }
    pub fn string<'a>(&'a self, rule: usize, name: &str, default: &'a str) -> &'a str {
        self.rules[rule][name].as_str().unwrap_or(default)
    }
    pub fn strings(&self, rule: usize, name: &str) -> Option<&Vec<Value>> {
        self.rules[rule][name].as_array()
    }
    pub fn severity(&self, rule: usize) -> &str {
        self.rules[rule]["severity"].as_str().or(self.rules[rule].as_str()).unwrap_or("error")
    }
}
