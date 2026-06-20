use super::tag::ParsedStartTag;

#[derive(Debug, Clone)]
pub(super) struct TagMatch {
    pub(super) start: usize,
    pub(super) end: usize,
    pub(super) tag: ParsedStartTag,
}

fn find_tag_end(html: &str, start: usize) -> Option<usize> {
    let bytes = html.as_bytes();
    let mut index = start;
    let mut quote: Option<u8> = None;

    while index < bytes.len() {
        let byte = bytes[index];

        if let Some(current_quote) = quote {
            if byte == current_quote {
                quote = None;
            }
        } else if byte == b'"' || byte == b'\'' {
            quote = Some(byte);
        } else if byte == b'>' {
            return Some(index + 1);
        }

        index += 1;
    }

    None
}

pub(super) fn find_next_start_tag(html: &str, from: usize) -> Option<TagMatch> {
    let bytes = html.as_bytes();
    let mut index = from;

    while index < bytes.len() {
        let relative = html[index..].find('<')?;
        let start = index + relative;
        let next = bytes.get(start + 1)?;

        if !next.is_ascii_alphabetic() {
            index = start + 1;
            continue;
        }

        let end = find_tag_end(html, start)?;
        let raw = &html[start..end];
        if let Some(tag) = ParsedStartTag::parse(raw) {
            return Some(TagMatch { start, end, tag });
        }

        index = start + 1;
    }

    None
}

pub(super) fn collect_pre_blocks(html: &str) -> Vec<(usize, usize)> {
    let mut blocks = Vec::new();
    let mut index = 0;

    while let Some(tag_match) = find_next_start_tag(html, index) {
        if tag_match.tag.name == "pre" {
            let Some(relative_end) = html[tag_match.end..].find("</pre>") else {
                break;
            };
            let end = tag_match.end + relative_end + "</pre>".len();
            blocks.push((tag_match.start, end));
            index = end;
        } else {
            index = tag_match.end;
        }
    }

    blocks
}
