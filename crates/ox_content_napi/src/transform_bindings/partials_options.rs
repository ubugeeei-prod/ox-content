use napi_derive::napi;
use ox_content_transform::PartialsOptions;

/// Opt-in parameterized Markdown partials.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsPartialsOptions {
    /// Enable `<!-- @partial: PATH k="v" -->` expansion.
    ///
    /// Default: `false`.
    pub enabled: Option<bool>,

    /// Root directory used for `@/` and absolute partial paths.
    ///
    /// Default: project root from the JavaScript caller.
    pub root_dir: Option<String>,

    /// Directory used for bare partial names such as `install.md`.
    ///
    /// Default: `"_partials"`.
    pub root: Option<String>,

    /// Missing `{{ name }}` substitutions: `"literal"` (default) or `"error"`.
    ///
    /// Default: `"literal"`.
    pub missing: Option<String>,
}

impl From<JsPartialsOptions> for PartialsOptions {
    fn from(value: JsPartialsOptions) -> Self {
        Self {
            enabled: value.enabled,
            root_dir: value.root_dir,
            root: value.root,
            missing: value.missing,
        }
    }
}
