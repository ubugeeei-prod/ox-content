use napi::bindgen_prelude::*;
use napi::Task;
use ox_content_transform::transformer::MarkdownTransformer;

use crate::{JsTransformOptions, TransformResult};

pub struct TransformTask {
    pub(super) source: String,
    pub(super) options: JsTransformOptions,
}

impl Task for TransformTask {
    type Output = TransformResult;
    type JsValue = TransformResult;

    fn compute(&mut self) -> Result<Self::Output> {
        let options = self.options.clone().into();
        Ok(MarkdownTransformer::from_options(&options).transform(&self.source).into())
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
        Ok(output)
    }
}
