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

pub(super) fn component_module(html: &str) -> String {
    let escaped = escape_markup(html);
    let mut output = String::with_capacity(34 + escaped.len());
    output.push_str("<div class=\"ox-content\">\n");
    output.push_str(&escaped);
    output.push_str("\n</div>\n");
    output
}

pub(super) fn inner_html_component_module(html: &str) -> String {
    let mut output = String::with_capacity(76 + html.len());
    output.push_str("<script>\n");
    output.push_str("  const rawHtml = ");
    super::shared::push_raw_html_js_string_literal(&mut output, html);
    output.push_str(";\n");
    output.push_str("</script>\n\n");
    output.push_str("<div class=\"ox-content\">\n");
    output.push_str("  {@html rawHtml}\n");
    output.push_str("</div>\n");
    output
}
