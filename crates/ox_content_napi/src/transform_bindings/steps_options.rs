use napi_derive::napi;
use ox_content_transform::StepsOptions;

/// Opt-in `::: steps` wrappers. `enabled` defaults to `false`.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsStepsOptions {
    pub enabled: Option<bool>,
}

impl From<JsStepsOptions> for StepsOptions {
    fn from(value: JsStepsOptions) -> Self {
        Self { enabled: value.enabled }
    }
}
