use super::{
    render::Document,
    style::{clean, pad, paint},
};
use std::path::PathBuf;

#[derive(Default)]
pub struct State {
    pub current: usize,
    pub file_index: usize,
    pub outline_index: usize,
    pub link_index: usize,
    pub offset: usize,
    pub focus: usize,
    pub sidebar: bool,
    pub theme: String,
    pub query: String,
    pub searching: bool,
    pub help: bool,
    pub message: String,
}

pub struct Layout {
    pub columns: usize,
    pub rows: usize,
    pub height: usize,
    pub sidebar: usize,
    pub content: usize,
}

pub fn layout(columns: u16, rows: u16, sidebar: bool) -> Layout {
    let columns = usize::from(columns).max(1);
    let rows = usize::from(rows).max(3);
    let sidebar = if sidebar && columns >= 80 { 30.min(columns / 3) } else { 0 };
    Layout {
        columns,
        rows,
        height: rows.saturating_sub(3),
        sidebar,
        content: columns.saturating_sub(sidebar).max(1),
    }
}

pub fn frame(
    state: &State,
    doc: &Document,
    files: &[PathBuf],
    size: &Layout,
    color: bool,
) -> String {
    let filename = files
        .get(state.current)
        .and_then(|path| path.file_name())
        .map_or_else(String::new, |name| clean(&name.to_string_lossy()));
    let mut lines = vec![paint(
        &pad(&format!(" ◆ OX CONTENT · {filename}"), size.columns),
        &state.theme,
        color,
        true,
    )];
    let pane = ["READER", "FILES", "OUTLINE", "LINKS"][state.focus];
    lines.push(paint(
        &pad(&format!(" {pane} · Tab panes · / search · ? help · q quit"), size.columns),
        &state.theme,
        color,
        false,
    ));
    let entries = entries(state, doc, files);
    let selected = match state.focus {
        1 => state.file_index,
        2 => state.outline_index,
        3 => state.link_index,
        _ => 0,
    };
    let start = selected.saturating_sub(size.height / 2);
    for row in 0..size.height {
        let content = if state.help {
            [
                "READER KEYS",
                "Tab / Shift-Tab: switch panes",
                "j/k or arrows: move",
                "Enter: open file, heading or link",
                "/: search, n/N: next/previous",
                "b: zen mode, t: theme, r: reload",
                "?: help, q or Ctrl-C: quit",
            ]
            .get(row)
            .copied()
            .unwrap_or("")
        } else {
            doc.lines.get(state.offset + row).map_or("", String::as_str)
        };
        let content = pad(content, size.content);
        let text = if size.sidebar > 0 {
            let entry = entries.get(start + row).map_or("", String::as_str);
            let left = pad(entry, size.sidebar.saturating_sub(1));
            let left =
                paint(&left, &state.theme, color, start + row == selected && state.focus > 0);
            format!("{left}│{content}")
        } else if state.focus > 0 && !state.help {
            pad(entries.get(start + row).map_or("", String::as_str), size.columns)
        } else {
            content
        };
        lines.push(text);
    }
    let status = if state.searching {
        format!("/{}", state.query)
    } else if !state.message.is_empty() {
        clean(&state.message)
    } else {
        format!(" {} / {} · {} · {}", state.current + 1, files.len(), state.theme, pane)
    };
    lines.push(paint(&pad(&status, size.columns), &state.theme, color, false));
    lines.truncate(size.rows);
    lines.join("\r\n")
}

fn entries(state: &State, doc: &Document, files: &[PathBuf]) -> Vec<String> {
    match state.focus {
        2 => doc.outline.iter().map(|heading| heading.title.clone()).collect(),
        3 => doc.links.iter().map(|link| format!("{} ({})", link.title, link.href)).collect(),
        _ => files
            .iter()
            .map(|path| clean(&path.file_name().unwrap_or_default().to_string_lossy()))
            .collect(),
    }
}

pub fn find_match(lines: &[String], query: &str, offset: usize, backwards: bool) -> Option<usize> {
    if query.is_empty() || lines.is_empty() {
        return None;
    }
    let query = query.to_lowercase();
    (1..=lines.len())
        .map(|step| {
            if backwards {
                (offset + lines.len() - step % lines.len()) % lines.len()
            } else {
                (offset + step) % lines.len()
            }
        })
        .find(|index| lines[*index].to_lowercase().contains(&query))
}
