use napi_derive::napi;

/// One provider tag as authors write it.
#[napi(object)]
pub struct JsEmbedTag {
    pub name: String,
    /// True when only the PascalCase spelling is an embed, because the
    /// lowercase one is a real HTML element.
    pub pascal_only: bool,
}

/// Every tag the media-embed transform can rewrite.
///
/// A caller that pre-scans a document to decide whether to run the transform
/// needs this list, and a hand-kept copy drifts — that is how `<CodeSandbox>`
/// came to be skipped on a page containing nothing else.
#[napi(js_name = "mediaEmbedTags")]
pub fn media_embed_tags() -> Vec<JsEmbedTag> {
    ox_content_transform::media_embeds::embed_tags()
        .into_iter()
        .map(|tag| JsEmbedTag { name: tag.name, pascal_only: tag.pascal_only })
        .collect()
}
