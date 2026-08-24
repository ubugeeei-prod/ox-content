use napi_derive::napi;
use rustc_hash::FxHashMap;
use serde_json::Value;

use ox_content_transform::{PublishDecision, PublishStateOptions, classify_publish_state};

/// JavaScript publish-state options.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsPublishStateOptions {
    /// When false, every page stays published and listed.
    pub enabled: Option<bool>,
    /// Injected ISO-8601 clock used for scheduled / expiry comparison.
    pub now: Option<String>,
    /// Keep draft and not-yet-scheduled pages visible (dev preview).
    pub include_drafts: Option<bool>,
}

/// Whether a page should be written and listed.
#[napi(object)]
pub struct JsPublishDecision {
    /// Write HTML for this page.
    pub output: bool,
    /// Include the page in nav, sitemap, and search.
    pub listed: bool,
}

impl From<JsPublishStateOptions> for PublishStateOptions {
    fn from(options: JsPublishStateOptions) -> Self {
        Self {
            enabled: options.enabled.unwrap_or(false),
            now: options.now,
            include_drafts: options.include_drafts.unwrap_or(false),
        }
    }
}

impl From<PublishDecision> for JsPublishDecision {
    fn from(decision: PublishDecision) -> Self {
        Self { output: decision.output, listed: decision.listed }
    }
}

/// Classifies already-parsed frontmatter JSON against publish-state options.
#[napi(js_name = "classifyPublishState")]
pub fn classify_publish_state_js(
    frontmatter_json: String,
    options: Option<JsPublishStateOptions>,
) -> JsPublishDecision {
    let frontmatter = parse_frontmatter_object(&frontmatter_json);
    classify_publish_state(
        &frontmatter,
        &options.map(PublishStateOptions::from).unwrap_or_default(),
    )
    .into()
}

fn parse_frontmatter_object(frontmatter_json: &str) -> FxHashMap<String, Value> {
    match serde_json::from_str::<Value>(frontmatter_json) {
        Ok(Value::Object(map)) => map.into_iter().collect(),
        _ => FxHashMap::default(),
    }
}
