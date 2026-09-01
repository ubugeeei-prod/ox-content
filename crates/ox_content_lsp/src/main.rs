#[tokio::main]
async fn main() {
    ox_content_lsp::run_stdio().await;
}
