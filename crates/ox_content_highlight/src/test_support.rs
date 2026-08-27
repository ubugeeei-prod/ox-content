//! Helpers shared by this crate's test modules.

/// Strips the generated markup and unescapes, leaving what a reader sees.
///
/// Highlighting may split a token across any number of spans, so the only
/// stable contract is that the visible text still equals the input. Asserting
/// on span boundaries instead would pin an implementation detail of whichever
/// grammar happens to be in use.
pub fn visible_text(html: &str) -> String {
    let mut text = String::new();
    let mut rest = html;
    while let Some(open) = rest.find('<') {
        text.push_str(&rest[..open]);
        let Some(close) = rest[open..].find('>') else { break };
        rest = &rest[open + close + 1..];
    }
    text.push_str(rest);
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
}
