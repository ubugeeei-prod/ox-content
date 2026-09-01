use napi_derive::napi;

#[napi(js_name = "runLspStdio")]
pub fn run_lsp_stdio() -> napi::Result<()> {
    let runtime =
        tokio::runtime::Builder::new_multi_thread().enable_all().build().map_err(|error| {
            napi::Error::from_reason(format!("Failed to start ox-content-lsp runtime: {error}"))
        })?;

    runtime.block_on(ox_content_lsp::run_stdio());
    Ok(())
}
