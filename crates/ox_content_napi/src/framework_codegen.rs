use std::collections::HashMap;

use napi_derive::napi;
use ox_content_renderer::frameworks::{
    escape_svelte_markup as escape_svelte_markup_impl,
    render_framework_component_code as render_framework_component_code_impl,
    FrameworkCodegenTarget, FrameworkComponentIsland,
};

#[napi(object)]
pub struct JsFrameworkComponentIsland {
    pub name: String,
    pub props: HashMap<String, serde_json::Value>,
    pub id: String,
    pub content: Option<String>,
}

impl From<JsFrameworkComponentIsland> for FrameworkComponentIsland {
    fn from(island: JsFrameworkComponentIsland) -> Self {
        Self { name: island.name, props: island.props, id: island.id, content: island.content }
    }
}

/// Renders already-produced Markdown HTML into framework-native component code.
///
/// The `target` argument currently accepts `"react"` and `"vue"`. React output
/// uses `createElement(...)`; Vue output uses `h(...)`.
#[napi(js_name = "renderFrameworkComponentCode")]
pub fn render_framework_component_code(
    html: String,
    target: String,
    islands: Option<Vec<JsFrameworkComponentIsland>>,
) -> napi::Result<String> {
    let target = target
        .parse::<FrameworkCodegenTarget>()
        .map_err(|error| napi::Error::from_reason(error.to_string()))?;
    let islands = islands
        .unwrap_or_default()
        .into_iter()
        .map(FrameworkComponentIsland::from)
        .collect::<Vec<_>>();
    Ok(render_framework_component_code_impl(&html, target, &islands))
}

/// Escapes Svelte expression delimiters before emitting static compiled markup.
#[napi(js_name = "escapeSvelteMarkup")]
pub fn escape_svelte_markup(html: String) -> String {
    escape_svelte_markup_impl(&html)
}
