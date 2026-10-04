use napi_derive::napi;

/// Run native authoring commands using the same implementation as the Rust binary.
#[napi(js_name = "runAuthoringCli")]
pub fn run_authoring_cli(args: Vec<String>, editor_assets: String) -> napi::Result<i32> {
    ox_content_cli::run(&args, std::path::Path::new(&editor_assets))
        .map_err(|error| napi::Error::from_reason(error.to_string()))
}
