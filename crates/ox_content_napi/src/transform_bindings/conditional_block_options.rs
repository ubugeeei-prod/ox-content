use napi_derive::napi;
use ox_content_transform::ConditionalBlockOptions;
use rustc_hash::FxHashMap;
use serde_json::Value;

/// Opt-in static `::: if` / `::: else` conditional blocks.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsConditionalBlockOptions {
    /// Enable conditional block evaluation.
    ///
    /// Default: `true` when the options object is supplied.
    pub enabled: Option<bool>,

    /// Build-time values available as `config.*` or bare identifiers.
    ///
    /// Default: `{}`.
    #[napi(ts_type = "Record<string, any>")]
    pub values: Option<Value>,
}

impl From<JsConditionalBlockOptions> for ConditionalBlockOptions {
    fn from(value: JsConditionalBlockOptions) -> Self {
        Self { enabled: value.enabled, values: value.values.map(values_to_map) }
    }
}

fn values_to_map(value: Value) -> FxHashMap<String, Value> {
    let Value::Object(values) = value else {
        return FxHashMap::default();
    };
    values.into_iter().collect()
}
