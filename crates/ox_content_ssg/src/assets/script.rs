use super::blocks::BlockMatch;
use super::chunk::{AssetCache, AssetKind};

const JS_SECTION_PREFIX: &str = "// ox-content:js:";
const JS_SECTION_START_SUFFIX: &str = ":start";
const JS_SECTION_END_SUFFIX: &str = ":end";
const SEARCH_CHUNK_START: &str = "// ox-content:search:start";
const SEARCH_CHUNK_END: &str = "// ox-content:search:end";
const SEARCH_CHUNK_PLACEHOLDER: &str = "__OX_CONTENT_SEARCH_CHUNK__";
const LAZY_JS_SECTIONS: [&str; 1] = ["search"];

#[derive(Debug, Clone)]
struct JsSection {
    name: String,
    content: String,
}

pub(super) fn build_script_replacement(
    js_content: &str,
    js_chunks: &mut AssetCache,
    out_dir: &str,
    base: &str,
) -> String {
    // Feature sections (`// ox-content:js:*`) become one file per widget, the
    // same way CSS sections become one file per plugin. Search stays lazy: the
    // chunk is written, but pages only get a `<script src>` after the user
    // opens search. Unmarked scripts fall back to one shared JS asset.
    let sections = extract_js_sections(js_content);
    if sections.is_empty() {
        return emit_legacy_script(js_content, js_chunks, out_dir, base);
    }

    let mut fragments = Vec::new();
    for section in sections {
        if LAZY_JS_SECTIONS.contains(&section.name.as_str()) {
            let _ = js_chunks.get_or_create(
                AssetKind::Js,
                &section.name,
                &section.content,
                out_dir,
                base,
            );
            continue;
        }

        let content = split_lazy_search_from_core(&section, js_chunks, out_dir, base);
        if content.is_empty() {
            continue;
        }
        let chunk = js_chunks.get_or_create(AssetKind::Js, &section.name, &content, out_dir, base);
        fragments.push(script_src_tag(&chunk.public_path));
    }

    fragments.join("\n")
}

fn emit_legacy_script(
    js_content: &str,
    js_chunks: &mut AssetCache,
    out_dir: &str,
    base: &str,
) -> String {
    let has_search =
        js_content.contains(SEARCH_CHUNK_START) && js_content.contains(SEARCH_CHUNK_PLACEHOLDER);
    let core = JsSection { name: "js".to_string(), content: js_content.to_string() };
    let content = split_lazy_search_from_core(&core, js_chunks, out_dir, base);
    if content.is_empty() {
        return String::new();
    }
    let label = if has_search { "core" } else { "js" };
    let chunk = js_chunks.get_or_create(AssetKind::Js, label, &content, out_dir, base);
    script_src_tag(&chunk.public_path)
}

fn split_lazy_search_from_core(
    section: &JsSection,
    js_chunks: &mut AssetCache,
    out_dir: &str,
    base: &str,
) -> String {
    // Search UI is large and only needed after the user opens search. Peel it
    // out of the core runtime, write a deferred chunk, and rewrite the
    // placeholder URL. Other feature sections pass through unchanged.
    if section.name != "core" && section.name != "js" {
        return section.content.trim().to_string();
    }
    let Some(search_chunk) =
        find_marked_js_chunk(&section.content, SEARCH_CHUNK_START, SEARCH_CHUNK_END)
    else {
        return section.content.trim().to_string();
    };
    if !section.content.contains(SEARCH_CHUNK_PLACEHOLDER) {
        return section.content.trim().to_string();
    }

    let search_content = search_chunk.content.trim();
    if search_content.is_empty() {
        return section.content.trim().to_string();
    }

    let search_public_path = js_chunks
        .get_or_create(AssetKind::Js, "search", search_content, out_dir, base)
        .public_path
        .clone();
    let mut core_content = String::new();
    core_content.push_str(&section.content[..search_chunk.start]);
    core_content.push_str(&section.content[search_chunk.end..]);
    core_content.replace(SEARCH_CHUNK_PLACEHOLDER, &search_public_path).trim().to_string()
}

fn extract_js_sections(js_content: &str) -> Vec<JsSection> {
    let mut sections = Vec::new();
    let mut cursor = 0;

    while let Some(start_rel) = js_content[cursor..].find(JS_SECTION_PREFIX) {
        let marker_start = cursor + start_rel;
        let name_start = marker_start + JS_SECTION_PREFIX.len();
        let Some(name_end_rel) = js_content[name_start..].find(JS_SECTION_START_SUFFIX) else {
            break;
        };
        let name_end = name_start + name_end_rel;
        let name = &js_content[name_start..name_end];
        if !name.chars().all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-') {
            cursor = name_end;
            continue;
        }

        let content_start = name_end + JS_SECTION_START_SUFFIX.len();
        let end_marker = format!("{JS_SECTION_PREFIX}{name}{JS_SECTION_END_SUFFIX}");
        let Some(end_rel) = js_content[content_start..].find(&end_marker) else {
            break;
        };
        let content_end = content_start + end_rel;
        let content = js_content[content_start..content_end].trim();
        if !content.is_empty() {
            sections.push(JsSection { name: name.to_string(), content: content.to_string() });
        }
        cursor = content_end + end_marker.len();
    }

    sections
}

fn find_marked_js_chunk(
    js_content: &str,
    start_marker: &str,
    end_marker: &str,
) -> Option<BlockMatch> {
    let start = js_content.find(start_marker)?;
    let content_start = start + start_marker.len();
    let end_start = content_start + js_content[content_start..].find(end_marker)?;
    Some(BlockMatch {
        start,
        end: end_start + end_marker.len(),
        content: js_content[content_start..end_start].trim().to_string(),
    })
}

fn script_src_tag(public_path: &str) -> String {
    format!("  <script defer src=\"{public_path}\"></script>")
}
