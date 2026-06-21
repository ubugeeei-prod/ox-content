use std::{collections::HashMap, str::FromStr};

use thiserror::Error;

pub mod angular;
mod parser;
pub mod react;
mod render;
mod shared;
pub mod solid;
pub mod svelte;
pub mod vue;

#[cfg(test)]
mod tests;

/// Component island metadata used when rendering framework-native code.
#[derive(Clone, Debug, PartialEq)]
pub struct FrameworkComponentIsland {
    pub name: String,
    pub props: HashMap<String, serde_json::Value>,
    pub id: String,
    pub content: Option<String>,
}

/// Framework target for HTML-to-component code generation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrameworkCodegenTarget {
    React,
    Vue,
}

impl FrameworkCodegenTarget {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::React => "react",
            Self::Vue => "vue",
        }
    }
}

impl FromStr for FrameworkCodegenTarget {
    type Err = FrameworkCodegenError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "react" => Ok(Self::React),
            "vue" => Ok(Self::Vue),
            _ => Err(FrameworkCodegenError { target: String::from(value) }),
        }
    }
}

/// Error returned when a framework codegen target is not supported.
#[derive(Debug, Error, Eq, PartialEq)]
#[error("Unsupported framework component render target: {target}")]
pub struct FrameworkCodegenError {
    target: String,
}

impl FrameworkCodegenError {
    pub fn target(&self) -> &str {
        &self.target
    }
}

/// Renders already-produced Markdown HTML into framework-native component code.
pub fn render_framework_component_code(
    html: &str,
    target: FrameworkCodegenTarget,
    islands: &[FrameworkComponentIsland],
) -> String {
    let nodes = parser::HtmlFragmentParser::new(html).parse();
    render::FrameworkCodegen { target, islands }.render_root(&nodes)
}

/// Escapes Svelte expression delimiters before emitting static compiled markup.
pub fn escape_svelte_markup(html: &str) -> String {
    svelte::escape_markup(html)
}
