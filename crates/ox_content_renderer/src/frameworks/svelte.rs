/// Escapes Svelte expression delimiters before emitting static compiled markup.
pub fn escape_markup(html: &str) -> String {
    if !html.contains('{') && !html.contains('}') {
        return String::from(html);
    }

    let mut output = String::with_capacity(html.len());
    for ch in html.chars() {
        match ch {
            '{' => output.push_str("&#123;"),
            '}' => output.push_str("&#125;"),
            _ => output.push(ch),
        }
    }
    output
}
