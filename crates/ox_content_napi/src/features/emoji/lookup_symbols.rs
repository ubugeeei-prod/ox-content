pub(super) fn lookup(name: &str) -> Option<&'static str> {
    match name {
        "+1" | "thumbs_up" | "thumbsup" => Some("\u{1F44D}"),
        "-1" | "down" | "thumbs_down" | "thumbsdown" => Some("\u{1F44E}"),
        "100" => Some("\u{1F4AF}"),
        "8ball" => Some("\u{1F3B1}"),
        _ => None,
    }
}
