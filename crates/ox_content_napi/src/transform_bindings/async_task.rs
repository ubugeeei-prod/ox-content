use napi::bindgen_prelude::*;
use napi::Task;

use crate::transformer::MarkdownTransformer;
use crate::{JsTransformOptions, TransformResult};

pub struct TransformTask {
    pub(super) source: String,
    pub(super) options: JsTransformOptions,
}

impl Task for TransformTask {
    type Output = TransformResult;
    type JsValue = TransformResult;

    fn compute(&mut self) -> Result<Self::Output> {
        Ok(MarkdownTransformer::from_options(&self.options).transform(&self.source))
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
        Ok(output)
    }
}
