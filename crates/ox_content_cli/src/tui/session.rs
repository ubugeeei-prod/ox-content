use super::{
    render::{self, Document},
    screen::{self, State},
    style,
    terminal::Terminal,
};
use crate::{Result, files};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    terminal,
};
use std::{
    io::Write,
    path::PathBuf,
    sync::atomic::Ordering,
    time::{Duration, Instant},
};

pub fn run(files: Vec<PathBuf>, theme: String, color: bool, watch: bool) -> Result<()> {
    let mut source = files::read_document(&files[0])?;
    let guard = Terminal::open()?;
    let mut state = State { sidebar: true, theme, ..State::default() };
    let mut checked = Instant::now();
    let mut redraw = true;
    let mut doc = Document::default();
    let mut height = 20;
    while !guard.stopped.load(Ordering::Relaxed) {
        if redraw {
            let (columns, rows) = terminal::size()?;
            let size = screen::layout(columns, rows, state.sidebar);
            height = size.height;
            doc = render::render(&source, size.content);
            state.offset = state.offset.min(doc.lines.len().saturating_sub(height));
            write!(
                std::io::stdout(),
                "\x1b[H{}\x1b[J",
                screen::frame(&state, &doc, &files, &size, color)
            )?;
            std::io::stdout().flush()?;
            redraw = false;
        }
        if watch && checked.elapsed() >= Duration::from_millis(200) {
            checked = Instant::now();
            // Reopen the path to follow editor atomic-save replacements.
            match files::read_document(&files[state.current]) {
                Ok(next) if next != source => {
                    source = next;
                    state.message.clear();
                    redraw = true;
                }
                Err(error) => {
                    let message = error.to_string();
                    if message != state.message {
                        state.message = message;
                        redraw = true;
                    }
                }
                _ => {}
            }
        }
        if !event::poll(Duration::from_millis(100))? {
            continue;
        }
        match event::read()? {
            Event::Key(key) if key.kind != KeyEventKind::Release => {
                if handle_key(key, &mut state, &doc, &files, &mut source, height)? {
                    break;
                }
                redraw = true;
            }
            Event::Resize(_, _) => {
                redraw = true;
            }
            _ => {}
        }
    }
    Ok(())
}

fn handle_key(
    key: KeyEvent,
    state: &mut State,
    doc: &Document,
    files: &[PathBuf],
    source: &mut String,
    height: usize,
) -> Result<bool> {
    if (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
        || (key.code == KeyCode::Char('q') && !state.searching)
    {
        return Ok(true);
    }
    if state.searching {
        match key.code {
            KeyCode::Esc => {
                state.searching = false;
                state.query.clear();
            }
            KeyCode::Enter => {
                state.searching = false;
                search(state, doc, false);
            }
            KeyCode::Backspace => {
                state.query.pop();
            }
            KeyCode::Char(ch) if !ch.is_control() => {
                state.query.push(ch);
            }
            _ => {}
        }
        return Ok(false);
    }
    if state.help {
        if matches!(key.code, KeyCode::Esc | KeyCode::Enter | KeyCode::Char('?')) {
            state.help = false;
        }
        return Ok(false);
    }
    state.message.clear();
    match key.code {
        KeyCode::Char('?') => {
            state.help = true;
        }
        KeyCode::Char('/') => {
            state.searching = true;
            state.query.clear();
        }
        KeyCode::Char('n' | 'N') => {
            search(state, doc, key.code == KeyCode::Char('N'));
        }
        KeyCode::Char('b') => {
            state.sidebar = !state.sidebar;
        }
        KeyCode::Char('t') => {
            let index = style::THEMES.iter().position(|theme| *theme == state.theme).unwrap_or(0);
            state.theme = style::THEMES[(index + 1) % style::THEMES.len()].to_string();
        }
        KeyCode::Char('r') => {
            open(state.current, state, files, source, false);
        }
        KeyCode::Tab => {
            state.focus = (state.focus + 1) % 4;
            state.sidebar = true;
        }
        KeyCode::BackTab => {
            state.focus = (state.focus + 3) % 4;
            state.sidebar = true;
        }
        KeyCode::Esc => {
            state.focus = 0;
        }
        KeyCode::Enter => match state.focus {
            1 => {
                open(state.file_index, state, files, source, true);
                state.focus = 0;
            }
            2 => {
                state.offset =
                    doc.outline.get(state.outline_index).map_or(0, |heading| heading.line);
                state.focus = 0;
            }
            3 => {
                if let Some(link) = doc.links.get(state.link_index) {
                    let href = link.href.split(['?', '#']).next().unwrap_or("");
                    if href.contains(':') || href.starts_with("//") {
                        state.message = "External links are displayed as text".to_string();
                    } else {
                        let decoded = percent_encoding::percent_decode_str(href).decode_utf8()?;
                        let parent =
                            files[state.current].parent().ok_or("Missing document directory")?;
                        let target = files::absolute(&parent.join(decoded.as_ref()))?;
                        if let Some(index) = files.iter().position(|path| *path == target) {
                            open(index, state, files, source, true);
                            state.focus = 0;
                        } else {
                            state.message =
                                "Local link is outside the selected Markdown files".to_string();
                        }
                    }
                }
            }
            _ => {}
        },
        key => {
            let (value, count) = match state.focus {
                1 => (&mut state.file_index, files.len()),
                2 => (&mut state.outline_index, doc.outline.len()),
                3 => (&mut state.link_index, doc.links.len()),
                _ => (&mut state.offset, doc.lines.len()),
            };
            match key {
                KeyCode::Up | KeyCode::Char('k') => {
                    *value = value.saturating_sub(1);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    *value = value.saturating_add(1);
                }
                KeyCode::Home | KeyCode::Char('g') => {
                    *value = 0;
                }
                KeyCode::End | KeyCode::Char('G') => {
                    *value = count;
                }
                KeyCode::PageUp => {
                    *value = value.saturating_sub(height);
                }
                KeyCode::PageDown | KeyCode::Char(' ') => {
                    *value = value.saturating_add(height);
                }
                _ => {}
            }
            *value = (*value).min(count.saturating_sub(1));
        }
    }
    Ok(false)
}

fn search(state: &mut State, doc: &Document, backwards: bool) {
    if let Some(index) = screen::find_match(&doc.lines, &state.query, state.offset, backwards) {
        state.offset = index;
    } else {
        state.message = "No matching text".to_string();
    }
}

fn open(index: usize, state: &mut State, files: &[PathBuf], source: &mut String, reset: bool) {
    match files::read_document(&files[index]) {
        Ok(next) => {
            *source = next;
            state.current = index;
            state.file_index = index;
            if reset {
                state.offset = 0;
                state.outline_index = 0;
                state.link_index = 0;
            }
        }
        Err(error) => {
            state.message = error.to_string();
        }
    }
}
