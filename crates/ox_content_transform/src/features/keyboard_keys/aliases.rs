use rustc_hash::FxHashMap;
use std::borrow::Cow;

#[derive(Clone, Copy)]
pub(super) enum KeyboardKeyStyle {
    Words,
    Symbols,
}

impl KeyboardKeyStyle {
    pub(super) fn from_option(value: Option<&str>) -> Self {
        match value {
            Some("symbols") => Self::Symbols,
            _ => Self::Words,
        }
    }
}

pub(super) fn normalize_key<'a>(
    key: &'a str,
    aliases: &'a FxHashMap<String, String>,
    style: KeyboardKeyStyle,
) -> Cow<'a, str> {
    let lower = key.to_ascii_lowercase();
    if let Some(custom) = aliases.get(&lower) {
        return Cow::Owned(custom.clone());
    }
    if let Some(label) = builtin_alias(&lower, style) {
        return Cow::Borrowed(label);
    }
    Cow::Borrowed(key)
}

fn builtin_alias(lower: &str, style: KeyboardKeyStyle) -> Option<&'static str> {
    let words = matches!(style, KeyboardKeyStyle::Words);
    Some(match lower {
        "cmd" | "command" | "meta" => {
            if words {
                "Command"
            } else {
                "⌘"
            }
        }
        "ctrl" | "control" => {
            if words {
                "Ctrl"
            } else {
                "⌃"
            }
        }
        "alt" => {
            if words {
                "Alt"
            } else {
                "⌥"
            }
        }
        "opt" | "option" => {
            if words {
                "Option"
            } else {
                "⌥"
            }
        }
        "shift" => {
            if words {
                "Shift"
            } else {
                "⇧"
            }
        }
        "enter" | "return" => {
            if words {
                "Enter"
            } else {
                "↵"
            }
        }
        "esc" | "escape" => {
            if words {
                "Esc"
            } else {
                "⎋"
            }
        }
        "tab" => {
            if words {
                "Tab"
            } else {
                "⇥"
            }
        }
        "space" | "spacebar" => {
            if words {
                "Space"
            } else {
                "␣"
            }
        }
        "backspace" => {
            if words {
                "Backspace"
            } else {
                "⌫"
            }
        }
        "delete" | "del" => {
            if words {
                "Delete"
            } else {
                "⌦"
            }
        }
        "up" | "arrowup" => {
            if words {
                "Up"
            } else {
                "↑"
            }
        }
        "down" | "arrowdown" => {
            if words {
                "Down"
            } else {
                "↓"
            }
        }
        "left" | "arrowleft" => {
            if words {
                "Left"
            } else {
                "←"
            }
        }
        "right" | "arrowright" => {
            if words {
                "Right"
            } else {
                "→"
            }
        }
        "plus" => "+",
        _ => return None,
    })
}
