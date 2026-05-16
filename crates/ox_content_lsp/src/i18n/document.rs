use std::path::Path;

use ox_content_i18n_checker::key_collector::KeyUsage;
use tower_lsp::lsp_types::Position;

const DICT_SEGMENT_POSIX: &str = "/content/i18n/";
const DICT_SEGMENT_WINDOWS: &str = "\\content\\i18n\\";

#[must_use]
pub fn is_i18n_source_path(path: &Path) -> bool {
    matches!(path.extension().and_then(|ext| ext.to_str()), Some("ts" | "tsx" | "js" | "jsx"))
}

#[must_use]
pub fn is_i18n_dictionary_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    let in_dict_dir =
        path_str.contains(DICT_SEGMENT_POSIX) || path_str.contains(DICT_SEGMENT_WINDOWS);

    in_dict_dir
        && matches!(path.extension().and_then(|ext| ext.to_str()), Some("json" | "yaml" | "yml"))
}

pub fn key_at_position(usages: &[KeyUsage], position: Position) -> Option<String> {
    let cursor_line = position.line + 1;
    let cursor_col = position.character + 1;

    usages
        .iter()
        .find(|usage| {
            usage.line == cursor_line
                && cursor_col >= usage.column
                && cursor_col <= usage.end_column
        })
        .map(|usage| usage.key.clone())
}

pub fn find_key_line_in_file(file_path: &str, key: &str) -> Option<u32> {
    let content = std::fs::read_to_string(file_path).ok()?;
    let leaf_key = key.rsplit('.').next().unwrap_or(key);

    content.lines().enumerate().find_map(|(index, line)| {
        let trimmed = line.trim();
        (trimmed.starts_with(&format!("\"{leaf_key}\""))
            || trimmed.starts_with(&format!("{leaf_key}:")))
        .then_some(index as u32)
    })
}
