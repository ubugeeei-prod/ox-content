use napi_derive::napi;

#[napi]
pub fn run_lsp() -> napi::Result<()> {
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|error| napi::Error::from_reason(format!("failed to start LSP runtime: {error}")))?;
    runtime.block_on(ox_content_lsp::run());
    Ok(())
}
