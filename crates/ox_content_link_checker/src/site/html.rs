use rustc_hash::FxHashSet;

#[derive(Debug)]
pub struct HtmlDocument {
    pub anchors: FxHashSet<String>,
    pub links: Vec<HtmlLink>,
    pub is_redirect: bool,
}

#[derive(Debug)]
pub struct HtmlLink {
    pub target: String,
    pub offset: usize,
    pub is_redirect_destination: bool,
}

#[derive(Debug)]
struct Attribute {
    name: String,
    value: String,
    offset: usize,
}

pub fn parse_html(source: &str) -> HtmlDocument {
    let mut document =
        HtmlDocument { anchors: FxHashSet::default(), links: Vec::new(), is_redirect: false };
    let bytes = source.as_bytes();
    let mut cursor = 0;

    while let Some(relative) = source[cursor..].find('<') {
        let start = cursor + relative;
        if source[start..].starts_with("<!--") {
            cursor = source[start + 4..].find("-->").map_or(bytes.len(), |end| start + 4 + end + 3);
            continue;
        }

        let Some((end, tag_name, attributes, closing)) = parse_tag(source, start) else {
            cursor = start + 1;
            continue;
        };
        cursor = end + 1;
        if closing {
            continue;
        }

        collect_tag(&tag_name, &attributes, &mut document);
        if matches!(tag_name.as_str(), "script" | "style") {
            let closing_tag = format!("</{tag_name}");
            if let Some(found) = find_ascii_case_insensitive(&source[cursor..], &closing_tag) {
                cursor += found + closing_tag.len();
            }
        }
    }

    document
}

fn collect_tag(tag: &str, attributes: &[Attribute], document: &mut HtmlDocument) {
    if let Some(id) = attr(attributes, "id") {
        document.anchors.insert(id.value.clone());
    }
    if tag == "a"
        && let Some(name) = attr(attributes, "name")
    {
        document.anchors.insert(name.value.clone());
    }

    let is_refresh = tag == "meta"
        && attr(attributes, "http-equiv")
            .is_some_and(|value| value.value.eq_ignore_ascii_case("refresh"));
    if is_refresh {
        document.is_redirect = true;
        if let Some(content) = attr(attributes, "content")
            && let Some((target, relative_offset)) = refresh_target(&content.value)
        {
            document.links.push(HtmlLink {
                target,
                offset: content.offset + relative_offset,
                is_redirect_destination: true,
            });
        }
    }

    for attribute in attributes {
        match attribute.name.as_str() {
            "href" | "src" | "poster" | "action" => document.links.push(HtmlLink {
                target: attribute.value.clone(),
                offset: attribute.offset,
                is_redirect_destination: false,
            }),
            "srcset" => collect_srcset(attribute, &mut document.links),
            _ => {}
        }
    }
}

fn collect_srcset(attribute: &Attribute, links: &mut Vec<HtmlLink>) {
    let mut consumed = 0;
    for candidate in attribute.value.split(',') {
        let leading = candidate.len() - candidate.trim_start().len();
        let value = candidate.trim().split_ascii_whitespace().next().unwrap_or_default();
        if !value.is_empty() {
            links.push(HtmlLink {
                target: value.to_string(),
                offset: attribute.offset + consumed + leading,
                is_redirect_destination: false,
            });
        }
        consumed += candidate.len() + 1;
    }
}

fn refresh_target(content: &str) -> Option<(String, usize)> {
    let lower = content.to_ascii_lowercase();
    let marker = lower.find("url=")?;
    let start = marker + 4;
    let raw = content[start..].trim();
    let trimmed = raw.trim_matches(['\'', '"']);
    let quote_offset = raw.len() - raw.trim_start_matches(['\'', '"']).len();
    Some((trimmed.to_string(), start + quote_offset))
}

fn attr<'a>(attributes: &'a [Attribute], name: &str) -> Option<&'a Attribute> {
    attributes.iter().find(|attribute| attribute.name == name)
}

fn parse_tag(source: &str, start: usize) -> Option<(usize, String, Vec<Attribute>, bool)> {
    let bytes = source.as_bytes();
    let mut cursor = start + 1;
    let closing = bytes.get(cursor) == Some(&b'/');
    if closing {
        cursor += 1;
    }
    while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
        cursor += 1;
    }
    let name_start = cursor;
    while bytes.get(cursor).is_some_and(|byte| is_name_byte(*byte)) {
        cursor += 1;
    }
    if cursor == name_start {
        return None;
    }
    let tag_name = source[name_start..cursor].to_ascii_lowercase();
    let mut attributes = Vec::new();

    loop {
        while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
            cursor += 1;
        }
        match bytes.get(cursor) {
            Some(b'>') => return Some((cursor, tag_name, attributes, closing)),
            Some(b'/') if bytes.get(cursor + 1) == Some(&b'>') => {
                return Some((cursor + 1, tag_name, attributes, closing));
            }
            None => return None,
            _ => {}
        }

        let attr_start = cursor;
        while bytes.get(cursor).is_some_and(|byte| is_attr_byte(*byte)) {
            cursor += 1;
        }
        if cursor == attr_start {
            cursor += 1;
            continue;
        }
        let name = source[attr_start..cursor].to_ascii_lowercase();
        while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
            cursor += 1;
        }
        if bytes.get(cursor) != Some(&b'=') {
            attributes.push(Attribute { name, value: String::new(), offset: cursor });
            continue;
        }
        cursor += 1;
        while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
            cursor += 1;
        }
        let quote = bytes.get(cursor).copied().filter(|byte| matches!(byte, b'\'' | b'"'));
        if quote.is_some() {
            cursor += 1;
        }
        let value_start = cursor;
        while let Some(byte) = bytes.get(cursor) {
            if quote
                .map_or_else(|| byte.is_ascii_whitespace() || *byte == b'>', |quote| *byte == quote)
            {
                break;
            }
            cursor += 1;
        }
        let value = decode_entities(&source[value_start..cursor]);
        attributes.push(Attribute { name, value, offset: value_start });
        if quote.is_some() && bytes.get(cursor).is_some() {
            cursor += 1;
        }
    }
}

fn is_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b':' | b'_')
}

fn is_attr_byte(byte: u8) -> bool {
    is_name_byte(byte) || byte == b'.'
}

fn decode_entities(value: &str) -> String {
    value
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
}

fn find_ascii_case_insensitive(haystack: &str, needle: &str) -> Option<usize> {
    haystack
        .as_bytes()
        .windows(needle.len())
        .position(|window| window.eq_ignore_ascii_case(needle.as_bytes()))
}
