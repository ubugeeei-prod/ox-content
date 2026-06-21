use napi_derive::napi;
use ox_content_renderer::frameworks::{
    escape_svelte_markup as escape_svelte_markup_impl, render_framework_code,
    render_framework_component_code as render_framework_component_code_impl, FrameworkCodegenMode,
    FrameworkCodegenTarget, FrameworkComponentIsland,
};
use rustc_hash::FxHashMap;

#[napi(object)]
pub struct JsFrameworkComponentIsland {
    pub name: String,
    #[napi(ts_type = "Record<string, any>")]
    pub props: serde_json::Value,
    pub id: String,
    pub content: Option<String>,
}

impl From<JsFrameworkComponentIsland> for FrameworkComponentIsland {
    fn from(island: JsFrameworkComponentIsland) -> Self {
        Self {
            name: island.name,
            props: props_to_map(island.props),
            id: island.id,
            content: island.content,
        }
    }
}

fn props_to_map(value: serde_json::Value) -> FxHashMap<String, serde_json::Value> {
    let serde_json::Value::Object(properties) = value else {
        return FxHashMap::default();
    };

    properties.into_iter().collect()
}

/// Renders already-produced Markdown HTML into framework-native component code.
///
/// The `target` argument accepts `"react"`, `"vue"`, and `"svelte"`.
/// `mode` defaults to `"expression"` for backward compatibility.
#[napi(js_name = "renderFrameworkComponentCode")]
pub fn render_framework_component_code(
    html: String,
    target: String,
    islands: Option<Vec<JsFrameworkComponentIsland>>,
    mode: Option<String>,
) -> napi::Result<String> {
    let target = target
        .parse::<FrameworkCodegenTarget>()
        .map_err(|error| napi::Error::from_reason(error.message()))?;
    let mode = match mode {
        Some(mode) => mode
            .parse::<FrameworkCodegenMode>()
            .map_err(|error| napi::Error::from_reason(error.message()))?,
        None => FrameworkCodegenMode::Expression,
    };
    let islands = islands
        .unwrap_or_default()
        .into_iter()
        .map(FrameworkComponentIsland::from)
        .collect::<Vec<_>>();
    if mode == FrameworkCodegenMode::Expression {
        return Ok(render_framework_component_code_impl(&html, target, &islands));
    }
    render_framework_code(&html, target, mode, &islands)
        .map_err(|error| napi::Error::from_reason(error.message()))
}

/// Escapes Svelte expression delimiters before emitting static compiled markup.
#[napi(js_name = "escapeSvelteMarkup")]
pub fn escape_svelte_markup(html: String) -> String {
    escape_svelte_markup_impl(&html)
}
