use std::collections::HashMap;

use napi_derive::napi;

mod parser;
mod render;
#[cfg(test)]
mod tests;

use parser::HtmlFragmentParser;
use render::FrameworkCodegen;

#[napi(object)]
pub struct JsFrameworkComponentIsland {
    pub name: String,
    pub props: HashMap<String, serde_json::Value>,
    pub id: String,
    pub content: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FrameworkCodegenTarget {
    React,
    Vue,
}

impl FrameworkCodegenTarget {
    fn parse(value: &str) -> napi::Result<Self> {
        match value {
            "react" => Ok(Self::React),
            "vue" => Ok(Self::Vue),
            _ => Err(napi::Error::from_reason(format!(
                "Unsupported framework component render target: {value}"
            ))),
        }
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
    let target = FrameworkCodegenTarget::parse(&target)?;
    let nodes = HtmlFragmentParser::new(&html).parse();
    let islands = islands.unwrap_or_default();
    Ok(FrameworkCodegen { target, islands: &islands }.render_root(&nodes))
}

/// Escapes Svelte expression delimiters before emitting static compiled markup.
#[napi(js_name = "escapeSvelteMarkup")]
pub fn escape_svelte_markup(html: String) -> String {
    if !html.contains('{') && !html.contains('}') {
        return html;
    }

    let mut output = String::with_capacity(html.len());
    for ch in html.chars() {
        match ch {
            '{' => output.push_str("&#123;"),
            '}' => output.push_str("&#125;"),
            _ => output.push(ch),
        }
    }
    output
}
