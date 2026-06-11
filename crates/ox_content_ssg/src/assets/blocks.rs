pub(super) const STYLE_BLOCK_START: &str = "<!-- ox-content:styles:start -->";
pub(super) const STYLE_BLOCK_END: &str = "<!-- ox-content:styles:end -->";
pub(super) const SCRIPT_BLOCK_START: &str = "<!-- ox-content:scripts:start -->";
pub(super) const SCRIPT_BLOCK_END: &str = "<!-- ox-content:scripts:end -->";
pub(super) const STYLE_OPEN: &str = "<style>";
pub(super) const STYLE_CLOSE: &str = "</style>";
pub(super) const SCRIPT_OPEN: &str = "<script>";
pub(super) const SCRIPT_CLOSE: &str = "</script>";
pub(super) const BODY_CLOSE: &str = "</body>";

pub(super) struct BlockMatch {
    pub(super) start: usize,
    pub(super) end: usize,
    pub(super) content: String,
}

pub(super) fn find_marked_block(
    html: &str,
    block_start_marker: &str,
    block_end_marker: &str,
    inner_open: &str,
    inner_close: &str,
) -> Option<BlockMatch> {
    let marker_start = html.find(block_start_marker)?;
    let start = include_leading_horizontal_ws(html, marker_start);
    let open_start = marker_start + html[marker_start..].find(inner_open)?;
    let content_start = open_start + inner_open.len();
    let content_end = content_start + html[content_start..].find(inner_close)?;
    let close_end = content_end + inner_close.len();
    let block_end = close_end + html[close_end..].find(block_end_marker)? + block_end_marker.len();

    Some(BlockMatch {
        start,
        end: block_end,
        content: html[content_start..content_end].to_string(),
    })
}

pub(super) fn find_first_tag_block(html: &str, open: &str, close: &str) -> Option<BlockMatch> {
    let open_start = html.find(open)?;
    let start = include_leading_horizontal_ws(html, open_start);
    let content_start = open_start + open.len();
    let content_end = content_start + html[content_start..].find(close)?;
    Some(BlockMatch {
        start,
        end: content_end + close.len(),
        content: html[content_start..content_end].to_string(),
    })
}

pub(super) fn find_last_body_script_block(html: &str) -> Option<BlockMatch> {
    let body_start = html.rfind(BODY_CLOSE)?;
    let before_body = &html[..body_start];
    let script_start = before_body.rfind(SCRIPT_OPEN)?;
    let content_start = script_start + SCRIPT_OPEN.len();
    let content_end = content_start + html[content_start..body_start].find(SCRIPT_CLOSE)?;
    let script_end = content_end + SCRIPT_CLOSE.len();
    if !html[script_end..body_start].trim().is_empty() {
        return None;
    }

    Some(BlockMatch {
        start: include_leading_horizontal_ws(html, script_start),
        end: body_start + BODY_CLOSE.len(),
        content: html[content_start..content_end].to_string(),
    })
}

pub(super) fn include_leading_horizontal_ws(value: &str, start: usize) -> usize {
    let bytes = value.as_bytes();
    let mut cursor = start;
    while cursor > 0 && matches!(bytes[cursor - 1], b' ' | b'\t') {
        cursor -= 1;
    }
    cursor
}
