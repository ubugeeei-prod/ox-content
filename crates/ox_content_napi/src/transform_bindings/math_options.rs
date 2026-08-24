use napi_derive::napi;
use ox_content_transform::MathOptions;

/// Opt-in `$…$` inline and `$$…$$` block math.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsMathOptions {
    /// Enable `$…$` inline and `$$…$$` block math.
    ///
    /// Default: `false` when the whole option is omitted; `true` when this object is present.
    pub enabled: Option<bool>,
}

impl From<JsMathOptions> for MathOptions {
    fn from(value: JsMathOptions) -> Self {
        Self { enabled: value.enabled }
    }
}
