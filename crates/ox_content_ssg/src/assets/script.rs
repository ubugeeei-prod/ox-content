use super::blocks::BlockMatch;
use super::chunk::{AssetCache, AssetKind};

const SEARCH_CHUNK_START: &str = "// ox-content:search:start";
const SEARCH_CHUNK_END: &str = "// ox-content:search:end";
const SEARCH_CHUNK_PLACEHOLDER: &str = "__OX_CONTENT_SEARCH_CHUNK__";

pub(super) fn build_script_replacement(
    js_content: &str,
    js_chunks: &mut AssetCache,
    out_dir: &str,
    base: &str,
) -> String {
    // The search UI runtime is large and only needed after the user opens
    // search. Split it into its own deferred chunk and replace the placeholder
    // in the core runtime with that chunk URL. If the marker is absent, fall
    // back to one shared JS asset for the whole script block.
    if let Some(search_chunk) = find_search_chunk(js_content) {
        if js_content.contains(SEARCH_CHUNK_PLACEHOLDER) {
            let search_content = search_chunk.content.trim();
            if !search_content.is_empty() {
                let search_public_path = js_chunks
                    .get_or_create(AssetKind::Js, "search", search_content, out_dir, base)
                    .public_path
                    .clone();
                let mut core_content = String::new();
                core_content.push_str(&js_content[..search_chunk.start]);
                core_content.push_str(&js_content[search_chunk.end..]);
                let core_content = core_content
                    .replace(SEARCH_CHUNK_PLACEHOLDER, &search_public_path)
                    .trim()
                    .to_string();

                if !core_content.is_empty() {
                    let core_chunk = js_chunks.get_or_create(
                        AssetKind::Js,
                        "core",
                        &core_content,
                        out_dir,
                        base,
                    );
                    return format!("  <script defer src=\"{}\"></script>", core_chunk.public_path);
                }
            }
        }
    }

    let fallback_content = js_content.trim();
    if fallback_content.is_empty() {
        return String::new();
    }

    let chunk = js_chunks.get_or_create(AssetKind::Js, "js", fallback_content, out_dir, base);
    format!("  <script defer src=\"{}\"></script>", chunk.public_path)
}

fn find_search_chunk(js_content: &str) -> Option<BlockMatch> {
    let start = js_content.find(SEARCH_CHUNK_START)?;
    let content_start = start + SEARCH_CHUNK_START.len();
    let end_start = content_start + js_content[content_start..].find(SEARCH_CHUNK_END)?;
    Some(BlockMatch {
        start,
        end: end_start + SEARCH_CHUNK_END.len(),
        content: js_content[content_start..end_start].trim().to_string(),
    })
}
