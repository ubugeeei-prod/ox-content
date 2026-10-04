use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

pub const THEMES: &[&str] = &["nord", "light", "mono"];

pub fn clean(text: &str) -> String {
    text.chars().filter(|ch| !ch.is_control() || matches!(ch, '\n' | '\r' | '\t')).collect()
}

pub fn width(text: &str) -> usize {
    UnicodeWidthStr::width(text)
}

pub fn clip(text: &str, columns: usize) -> String {
    let mut used = 0;
    text.graphemes(true)
        .take_while(|glyph| {
            used += width(glyph);
            used <= columns
        })
        .collect()
}

pub fn pad(text: &str, columns: usize) -> String {
    let clipped = clip(text, columns);
    let spaces = columns.saturating_sub(width(&clipped));
    clipped + &" ".repeat(spaces)
}

pub fn wrap(text: &str, columns: usize) -> Vec<String> {
    let columns = columns.max(1);
    let mut result = Vec::new();
    for line in text.split('\n') {
        let mut current = String::new();
        let mut used = 0;
        // Keep words together where possible; split long words at grapheme boundaries.
        for word in line.split_inclusive(' ') {
            if !current.is_empty() && used + width(word.trim_end()) > columns {
                result.push(current.trim_end().to_string());
                current.clear();
                used = 0;
            }
            for glyph in word.graphemes(true) {
                let size = width(glyph);
                if used + size > columns && !current.is_empty() {
                    result.push(current.trim_end().to_string());
                    current.clear();
                    used = 0;
                }
                current.push_str(glyph);
                used += size;
            }
        }
        result.push(current.trim_end().to_string());
    }
    result
}

pub fn paint(text: &str, theme: &str, color: bool, selected: bool) -> String {
    if !color {
        return text.to_string();
    }
    let accent = match theme {
        "light" => "0;96;128",
        "mono" => "255;255;255",
        _ => "136;192;208",
    };
    let code = if selected { "7".to_string() } else { format!("38;2;{accent}") };
    format!("\x1b[{code}m{text}\x1b[0m")
}
