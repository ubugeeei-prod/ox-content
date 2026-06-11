use crate::html_scan::find_ci;

/// A `<youtube ...>` element located in the source HTML.
///
/// Note: the `start` attribute is intentionally not read. The TS
/// implementation this ports parsed HTML via hast, which coerces `start`
/// (a known numeric attribute on `<ol>`) to a number and then dropped it in
/// its string-only attribute reader, so `start` never reached the embed URL.
pub(super) struct YouTubeElement {
    /// Byte range of the whole element (open tag through close tag or `/>`).
    pub(super) span: (usize, usize),
    pub(super) id: Option<String>,
    pub(super) url: Option<String>,
    pub(super) title: Option<String>,
}

/// Find the next `<youtube ...>` element at or after `from`. Recognises both
/// `<youtube ...></youtube>` and self-closing `<youtube ... />` forms.
pub(super) fn find_youtube_element(html: &str, from: usize) -> Option<YouTubeElement> {
    let bytes = html.as_bytes();
    let mut search = from;
    loop {
        let rel = find_ci(html, search, "<youtube")?;
        let tag_start = rel;
        let after_name = tag_start + "<youtube".len();
        let boundary = bytes.get(after_name).copied();
        let is_boundary =
            matches!(boundary, Some(b) if b == b'>' || b == b'/' || b.is_ascii_whitespace());
        if !is_boundary {
            search = after_name;
            continue;
        }

        let mut i = after_name;
        let mut quote: Option<u8> = None;
        let mut tag_end = None;
        while i < bytes.len() {
            let b = bytes[i];
            match quote {
                Some(q) => {
                    if b == q {
                        quote = None;
                    }
                }
                None => {
                    if b == b'"' || b == b'\'' {
                        quote = Some(b);
                    } else if b == b'>' {
                        tag_end = Some(i);
                        break;
                    }
                }
            }
            i += 1;
        }
        let tag_end = tag_end?;
        let self_closing = tag_end > after_name && bytes[tag_end - 1] == b'/';

        let inner_end = if self_closing { tag_end - 1 } else { tag_end };
        let (mut id, mut url, mut title) = (None, None, None);
        for (name, value) in parse_attributes(&html[after_name..inner_end]) {
            match name.as_str() {
                "id" if id.is_none() => id = Some(value),
                "url" if url.is_none() => url = Some(value),
                "title" if title.is_none() => title = Some(value),
                _ => {}
            }
        }

        let span_end = if self_closing {
            tag_end + 1
        } else {
            match find_ci(html, tag_end + 1, "</youtube>") {
                Some(close_start) => close_start + "</youtube>".len(),
                None => tag_end + 1,
            }
        };

        return Some(YouTubeElement { span: (tag_start, span_end), id, url, title });
    }
}

/// Parse `name="value"` / `name='value'` / `name=value` / bare `name`
/// attributes from the inside of a start tag. Names are lower-cased.
fn parse_attributes(inner: &str) -> Vec<(String, String)> {
    // Returning a small vector keeps first-wins semantics obvious at the call
    // site. The scan respects quoted values so it can run on raw HTML without
    // a rehype parse/stringify round-trip.
    let bytes = inner.as_bytes();
    let mut attrs = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() || bytes[i] == b'/' {
            break;
        }
        let name_start = i;
        while i < bytes.len()
            && !bytes[i].is_ascii_whitespace()
            && bytes[i] != b'='
            && bytes[i] != b'/'
        {
            i += 1;
        }
        if name_start == i {
            break;
        }
        let name = inner[name_start..i].to_ascii_lowercase();
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        let value = if i < bytes.len() && bytes[i] == b'=' {
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if i >= bytes.len() {
                String::new()
            } else if bytes[i] == b'"' || bytes[i] == b'\'' {
                let q = bytes[i];
                i += 1;
                let vs = i;
                while i < bytes.len() && bytes[i] != q {
                    i += 1;
                }
                let v = inner[vs..i].to_string();
                if i < bytes.len() {
                    i += 1;
                }
                v
            } else {
                let vs = i;
                while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'/' {
                    i += 1;
                }
                inner[vs..i].to_string()
            }
        } else {
            String::new()
        };
        attrs.push((name, value));
    }
    attrs
}
