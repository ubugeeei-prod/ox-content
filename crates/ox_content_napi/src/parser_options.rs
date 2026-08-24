//! Parser options as seen from JavaScript, plus a hand-written reader for them.
//!
//! Every Markdown entry point takes this object, so how fast it crosses the
//! boundary sets the floor for a `parseAndRender` call on a short document. The
//! derived reader spends four Node-API calls per field, one of which allocates a
//! fresh JavaScript string for the property name — seven fields of that measured
//! at roughly as much as parsing a 500-byte document. [`FromNapiValue`] is
//! therefore written by hand below; the struct keeps `#[napi(object)]` so the
//! generated TypeScript declaration and the JavaScript-facing conversion stay
//! exactly as they were.

// Reading a property off a JavaScript value is raw Node-API: `FromNapiValue` is
// an unsafe trait and `napi_sys` is a C ABI. The unsafety is confined to this
// module, which holds nothing but the options type and its reader.
#![allow(unsafe_code)]

use std::ffi::CStr;
use std::ptr;

use napi::bindgen_prelude::{FromNapiValue, ValidateNapiValue};
use napi::sys;
use napi_derive::napi;
use ox_content_parser::ParserOptions;

/// Parser options for JavaScript.
///
/// When `gfm` is `true`, omitted extension flags inherit the GFM profile.
#[napi(object, object_from_js = false)]
#[derive(Default, Clone)]
pub struct JsParserOptions {
    /// Enable the GFM convenience profile.
    ///
    /// Default: `false`.
    pub gfm: Option<bool>,

    /// Enable MDX JSX, ESM, and expression nodes.
    ///
    /// Default: `false`.
    pub mdx: Option<bool>,

    /// Enable footnote references and definitions.
    ///
    /// Default: `false`, or `true` when `gfm` is `true`.
    pub footnotes: Option<bool>,

    /// Enable GFM task-list item markers.
    ///
    /// Default: `false`, or `true` when `gfm` is `true`.
    pub task_lists: Option<bool>,

    /// Enable GFM pipe tables.
    ///
    /// Default: `false`, or `true` when `gfm` is `true`.
    pub tables: Option<bool>,

    /// Enable GFM strikethrough spans.
    ///
    /// Default: `false`, or `true` when `gfm` is `true`.
    pub strikethrough: Option<bool>,

    /// Enable GFM autolinks.
    ///
    /// Default: `false`, or `true` when `gfm` is `true`.
    pub autolinks: Option<bool>,
}

impl From<JsParserOptions> for ParserOptions {
    fn from(opts: JsParserOptions) -> Self {
        let mut options =
            if opts.gfm.unwrap_or(false) { ParserOptions::gfm() } else { ParserOptions::default() };

        if let Some(v) = opts.footnotes {
            options.footnotes = v;
        }
        if let Some(v) = opts.task_lists {
            options.task_lists = v;
        }
        if let Some(v) = opts.tables {
            options.tables = v;
        }
        if let Some(v) = opts.strikethrough {
            options.strikethrough = v;
        }
        if let Some(v) = opts.autolinks {
            options.autolinks = v;
        }
        if let Some(v) = opts.mdx {
            options.mdx = v;
        }

        options
    }
}

/// Reads one optional boolean property from a JavaScript object.
///
/// `napi_get_named_property` takes the key as a C string and lets Node
/// internalize it, which skips the `napi_create_string_utf8` the derived reader
/// pays per field. An absent property arrives as `undefined` and reads as
/// `None`; any other non-boolean value is a type error, exactly as before.
///
/// # Safety
///
/// `env` must be the environment of the running call, and `object` a live
/// handle to a JavaScript object in it.
unsafe fn read_flag(
    env: sys::napi_env,
    object: sys::napi_value,
    key: &CStr,
) -> napi::Result<Option<bool>> {
    let mut value = ptr::null_mut();
    napi::check_status!(
        unsafe { sys::napi_get_named_property(env, object, key.as_ptr(), &raw mut value) },
        "Failed to read parser option `{}`",
        key.to_string_lossy()
    )?;

    let mut value_type = sys::ValueType::napi_undefined;
    napi::check_status!(
        unsafe { sys::napi_typeof(env, value, &raw mut value_type) },
        "Failed to read parser option `{}`",
        key.to_string_lossy()
    )?;
    if matches!(value_type, sys::ValueType::napi_undefined | sys::ValueType::napi_null) {
        return Ok(None);
    }

    let mut flag = false;
    napi::check_status!(
        unsafe { sys::napi_get_value_bool(env, value, &raw mut flag) },
        "Parser option `{}` must be a boolean",
        key.to_string_lossy()
    )?;
    Ok(Some(flag))
}

impl FromNapiValue for JsParserOptions {
    unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> napi::Result<Self> {
        Ok(Self {
            gfm: unsafe { read_flag(env, napi_val, c"gfm") }?,
            mdx: unsafe { read_flag(env, napi_val, c"mdx") }?,
            footnotes: unsafe { read_flag(env, napi_val, c"footnotes") }?,
            task_lists: unsafe { read_flag(env, napi_val, c"taskLists") }?,
            tables: unsafe { read_flag(env, napi_val, c"tables") }?,
            strikethrough: unsafe { read_flag(env, napi_val, c"strikethrough") }?,
            autolinks: unsafe { read_flag(env, napi_val, c"autolinks") }?,
        })
    }
}

/// Uses the default object check from [`ValidateNapiValue`], matching what the
/// derived implementation installed before the reader was written by hand.
impl ValidateNapiValue for JsParserOptions {}

#[cfg(test)]
mod tests {
    use ox_content_parser::ParserOptions;

    use super::JsParserOptions;

    #[test]
    fn mdx_flag_maps_to_parser_options_without_changing_the_default() {
        let enabled =
            ParserOptions::from(JsParserOptions { mdx: Some(true), ..JsParserOptions::default() });
        assert!(enabled.mdx);

        let defaults = ParserOptions::from(JsParserOptions::default());
        assert!(!defaults.mdx);
    }
}
