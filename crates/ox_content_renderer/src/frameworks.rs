use std::{fmt, str::FromStr};

use rustc_hash::FxHashMap;
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
#[cfg(test)]
mod tests_parser;

/// Component island metadata used when rendering framework-native code.
#[derive(Clone, Debug, PartialEq)]
pub struct FrameworkComponentIsland {
    pub name: String,
    pub props: FxHashMap<String, serde_json::Value>,
    pub id: String,
    pub content: Option<String>,
}

/// Framework target for HTML-to-component code generation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrameworkCodegenTarget {
    React,
    Svelte,
    Vue,
}

impl FrameworkCodegenTarget {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::React => "react",
            Self::Svelte => "svelte",
            Self::Vue => "vue",
        }
    }
}

impl fmt::Display for FrameworkCodegenTarget {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for FrameworkCodegenTarget {
    type Err = FrameworkCodegenError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "react" => Ok(Self::React),
            "svelte" => Ok(Self::Svelte),
            "vue" => Ok(Self::Vue),
            _ => Err(FrameworkCodegenError::UnsupportedTarget { target: String::from(value) }),
        }
    }
}

/// Framework code generation output mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrameworkCodegenMode {
    InnerHtml,
    Expression,
    RenderFunction,
    Component,
}

impl FrameworkCodegenMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InnerHtml => "innerHtml",
            Self::Expression => "expression",
            Self::RenderFunction => "renderFunction",
            Self::Component => "component",
        }
    }
}

impl fmt::Display for FrameworkCodegenMode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for FrameworkCodegenMode {
    type Err = FrameworkCodegenError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "html" | "innerHtml" | "inner-html" => Ok(Self::InnerHtml),
            "expression" | "native" => Ok(Self::Expression),
            "renderFunction" | "render-function" => Ok(Self::RenderFunction),
            "component" => Ok(Self::Component),
            _ => Err(FrameworkCodegenError::UnsupportedMode { mode: String::from(value) }),
        }
    }
}

/// Error returned when a framework codegen target or mode is not supported.
#[derive(Debug, Error, Eq, PartialEq)]
pub enum FrameworkCodegenError {
    #[error("Unsupported framework component render target: {target}")]
    UnsupportedTarget { target: String },
    #[error("Unsupported framework component render mode: {mode}")]
    UnsupportedMode { mode: String },
    #[error("Unsupported framework component render mode {mode} for target {target}")]
    UnsupportedModeForTarget { mode: FrameworkCodegenMode, target: FrameworkCodegenTarget },
}

impl FrameworkCodegenError {
    pub fn message(&self) -> String {
        let mut output = String::with_capacity(96);
        match self {
            Self::UnsupportedTarget { target } => {
                output.push_str("Unsupported framework component render target: ");
                output.push_str(target);
            }
            Self::UnsupportedMode { mode } => {
                output.push_str("Unsupported framework component render mode: ");
                output.push_str(mode);
            }
            Self::UnsupportedModeForTarget { mode, target } => {
                output.push_str("Unsupported framework component render mode ");
                output.push_str(mode.as_str());
                output.push_str(" for target ");
                output.push_str(target.as_str());
            }
        }
        output
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

/// Renders already-produced Markdown HTML into the selected framework code shape.
pub fn render_framework_code(
    html: &str,
    target: FrameworkCodegenTarget,
    mode: FrameworkCodegenMode,
    islands: &[FrameworkComponentIsland],
) -> Result<String, FrameworkCodegenError> {
    match mode {
        FrameworkCodegenMode::InnerHtml => Ok(render_inner_html_component(html, target)),
        FrameworkCodegenMode::Expression => render_framework_expression(html, target, islands),
        FrameworkCodegenMode::RenderFunction => {
            if target == FrameworkCodegenTarget::Svelte {
                return Err(FrameworkCodegenError::UnsupportedModeForTarget { mode, target });
            }
            let expression = render_framework_expression(html, target, islands)?;
            render_framework_render_function(&expression, target)
        }
        FrameworkCodegenMode::Component => render_framework_component(html, target, islands),
    }
}

/// Escapes Svelte expression delimiters before emitting static compiled markup.
pub fn escape_svelte_markup(html: &str) -> String {
    svelte::escape_markup(html)
}

fn render_framework_expression(
    html: &str,
    target: FrameworkCodegenTarget,
    islands: &[FrameworkComponentIsland],
) -> Result<String, FrameworkCodegenError> {
    match target {
        FrameworkCodegenTarget::React | FrameworkCodegenTarget::Vue => {
            Ok(render_framework_component_code(html, target, islands))
        }
        FrameworkCodegenTarget::Svelte => Err(FrameworkCodegenError::UnsupportedModeForTarget {
            mode: FrameworkCodegenMode::Expression,
            target,
        }),
    }
}

fn render_framework_render_function(
    expression: &str,
    target: FrameworkCodegenTarget,
) -> Result<String, FrameworkCodegenError> {
    match target {
        FrameworkCodegenTarget::React => Ok(react::render_function_module(expression)),
        FrameworkCodegenTarget::Vue => Ok(vue::render_function_module(expression)),
        FrameworkCodegenTarget::Svelte => Err(FrameworkCodegenError::UnsupportedModeForTarget {
            mode: FrameworkCodegenMode::RenderFunction,
            target,
        }),
    }
}

fn render_framework_component(
    html: &str,
    target: FrameworkCodegenTarget,
    islands: &[FrameworkComponentIsland],
) -> Result<String, FrameworkCodegenError> {
    match target {
        FrameworkCodegenTarget::React => {
            let expression = render_framework_component_code(html, target, islands);
            Ok(react::component_module(&expression))
        }
        FrameworkCodegenTarget::Vue => {
            let expression = render_framework_component_code(html, target, islands);
            Ok(vue::component_module(&expression))
        }
        FrameworkCodegenTarget::Svelte => Ok(svelte::component_module(html)),
    }
}

fn render_inner_html_component(html: &str, target: FrameworkCodegenTarget) -> String {
    match target {
        FrameworkCodegenTarget::React => react::inner_html_component_module(html),
        FrameworkCodegenTarget::Svelte => svelte::inner_html_component_module(html),
        FrameworkCodegenTarget::Vue => vue::inner_html_component_module(html),
    }
}
