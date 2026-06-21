use compact_str::CompactString;
use smallvec::SmallVec;

pub(super) type HtmlNodes = Vec<HtmlNode>;
type HtmlAttributes = Vec<HtmlAttribute>;
type OpenElements = SmallVec<[OpenElement; 8]>;

#[derive(Debug, Eq, PartialEq)]
pub(super) enum HtmlNode {
    Element(HtmlElement),
    Text(CompactString),
}

#[derive(Debug, Eq, PartialEq)]
pub(super) struct HtmlElement {
    pub tag_name: CompactString,
    pub attributes: HtmlAttributes,
    pub children: HtmlNodes,
}

#[derive(Debug, Eq, PartialEq)]
pub(super) struct HtmlAttribute {
    pub name: CompactString,
    pub value: Option<CompactString>,
}

#[derive(Debug)]
struct OpenElement {
    tag_name: CompactString,
    attributes: HtmlAttributes,
    children: HtmlNodes,
}

impl OpenElement {
    fn into_node(self) -> HtmlNode {
        HtmlNode::Element(HtmlElement {
            tag_name: self.tag_name,
            attributes: self.attributes,
            children: self.children,
        })
    }
}

pub(super) struct HtmlFragmentParser<'a> {
    html: &'a str,
    pos: usize,
}

impl<'a> HtmlFragmentParser<'a> {
    pub(super) fn new(html: &'a str) -> Self {
        Self { html, pos: 0 }
    }

    pub(super) fn parse(mut self) -> HtmlNodes {
        let mut root = HtmlNodes::new();
        let mut stack = OpenElements::new();

        while self.pos < self.html.len() {
            let Some(tag_start) = find_byte(self.html, self.pos, b'<') else {
                push_text(&mut root, &mut stack, &self.html[self.pos..]);
                self.pos = self.html.len();
                break;
            };

            if tag_start > self.pos {
                push_text(&mut root, &mut stack, &self.html[self.pos..tag_start]);
            }

            if self.html[tag_start..].starts_with("<!--") {
                self.pos = find_comment_end(self.html, tag_start).unwrap_or(self.html.len());
                continue;
            }

            let Some(tag_end) = find_tag_end(self.html, tag_start) else {
                push_text(&mut root, &mut stack, &self.html[tag_start..]);
                self.pos = self.html.len();
                break;
            };

            let raw_tag = &self.html[tag_start + 1..tag_end];
            if raw_tag.trim_start().starts_with(['!', '?']) {
                self.pos = tag_end + 1;
                continue;
            }

            if let Some(tag_name) = parse_end_tag(raw_tag) {
                close_element(&mut root, &mut stack, &tag_name);
                self.pos = tag_end + 1;
                continue;
            }

            if let Some((tag_name, attributes, self_closing)) = parse_start_tag(raw_tag) {
                let is_void = is_void_element(&tag_name);
                let open = OpenElement { tag_name, attributes, children: HtmlNodes::new() };
                if self_closing || is_void {
                    append_node(&mut root, &mut stack, open.into_node());
                } else {
                    stack.push(open);
                }
                self.pos = tag_end + 1;
                continue;
            }

            push_text(&mut root, &mut stack, &self.html[tag_start..=tag_end]);
            self.pos = tag_end + 1;
        }

        while let Some(open) = stack.pop() {
            append_node(&mut root, &mut stack, open.into_node());
        }

        root
    }
}

fn find_byte(haystack: &str, from: usize, byte: u8) -> Option<usize> {
    memchr::memchr(byte, &haystack.as_bytes()[from..]).map(|index| from + index)
}

fn find_comment_end(html: &str, start: usize) -> Option<usize> {
    html[start + 4..].find("-->").map(|offset| start + 4 + offset + 3)
}

fn find_tag_end(html: &str, start: usize) -> Option<usize> {
    let bytes = html.as_bytes();
    let mut quote = None;

    for (index, byte) in bytes.iter().enumerate().skip(start + 1) {
        match (quote, *byte) {
            (Some(current), value) if value == current => quote = None,
            (Some(_), _) => {}
            (None, b'\'' | b'"') => quote = Some(*byte),
            (None, b'>') => return Some(index),
            _ => {}
        }
    }

    None
}

fn parse_end_tag(raw_tag: &str) -> Option<CompactString> {
    let raw_tag = raw_tag.trim_start();
    let rest = raw_tag.strip_prefix('/')?.trim_start();
    let end = rest
        .bytes()
        .position(|byte| byte.is_ascii_whitespace() || byte == b'/')
        .unwrap_or(rest.len());
    (end != 0).then(|| CompactString::from(&rest[..end]))
}

fn parse_start_tag(raw_tag: &str) -> Option<(CompactString, HtmlAttributes, bool)> {
    let bytes = raw_tag.as_bytes();
    let mut cursor = skip_ascii_whitespace(bytes, 0);
    if cursor >= bytes.len() || matches!(bytes[cursor], b'/' | b'!' | b'?') {
        return None;
    }

    let name_start = cursor;
    while cursor < bytes.len()
        && !bytes[cursor].is_ascii_whitespace()
        && !matches!(bytes[cursor], b'/' | b'=')
    {
        cursor += 1;
    }
    if cursor == name_start {
        return None;
    }

    let tag_name = CompactString::from(&raw_tag[name_start..cursor]);
    let mut attributes = HtmlAttributes::new();
    let mut self_closing = false;

    while cursor < bytes.len() {
        cursor = skip_ascii_whitespace(bytes, cursor);
        if cursor >= bytes.len() {
            break;
        }

        if bytes[cursor] == b'/' {
            self_closing = true;
            cursor += 1;
            continue;
        }

        let attr_start = cursor;
        while cursor < bytes.len()
            && !bytes[cursor].is_ascii_whitespace()
            && !matches!(bytes[cursor], b'=' | b'/')
        {
            cursor += 1;
        }
        if cursor == attr_start {
            cursor += 1;
            continue;
        }

        let name = CompactString::from(&raw_tag[attr_start..cursor]);
        cursor = skip_ascii_whitespace(bytes, cursor);
        let value = if cursor < bytes.len() && bytes[cursor] == b'=' {
            cursor += 1;
            cursor = skip_ascii_whitespace(bytes, cursor);
            Some(parse_attribute_value(raw_tag, bytes, &mut cursor))
        } else {
            None
        };

        attributes.push(HtmlAttribute { name, value });
    }

    Some((tag_name, attributes, self_closing))
}

fn parse_attribute_value(raw_tag: &str, bytes: &[u8], cursor: &mut usize) -> CompactString {
    if *cursor < bytes.len() && matches!(bytes[*cursor], b'\'' | b'"') {
        let quote = bytes[*cursor];
        *cursor += 1;
        let value_start = *cursor;
        while *cursor < bytes.len() && bytes[*cursor] != quote {
            *cursor += 1;
        }
        let value = decode_html_entities(&raw_tag[value_start..*cursor]);
        if *cursor < bytes.len() {
            *cursor += 1;
        }
        return value;
    }

    let value_start = *cursor;
    while *cursor < bytes.len() && !bytes[*cursor].is_ascii_whitespace() && bytes[*cursor] != b'/' {
        *cursor += 1;
    }
    decode_html_entities(&raw_tag[value_start..*cursor])
}

fn skip_ascii_whitespace(bytes: &[u8], mut cursor: usize) -> usize {
    while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
        cursor += 1;
    }
    cursor
}

fn push_text(root: &mut HtmlNodes, stack: &mut [OpenElement], value: &str) {
    if !value.is_empty() {
        append_node(root, stack, HtmlNode::Text(decode_html_entities(value)));
    }
}

fn append_node(root: &mut HtmlNodes, stack: &mut [OpenElement], node: HtmlNode) {
    if let Some(parent) = stack.last_mut() {
        append_child(&mut parent.children, node);
    } else {
        append_child(root, node);
    }
}

fn append_child(children: &mut HtmlNodes, node: HtmlNode) {
    if let HtmlNode::Text(value) = node {
        if let Some(HtmlNode::Text(previous)) = children.last_mut() {
            previous.push_str(&value);
        } else {
            children.push(HtmlNode::Text(value));
        }
        return;
    }

    children.push(node);
}

fn close_element(root: &mut HtmlNodes, stack: &mut OpenElements, tag_name: &str) {
    let Some(position) =
        stack.iter().rposition(|element| element.tag_name.eq_ignore_ascii_case(tag_name))
    else {
        return;
    };

    while stack.len() > position {
        let open = stack.pop().expect("stack length checked above");
        append_node(root, stack, open.into_node());
    }
}

fn is_void_element(tag_name: &str) -> bool {
    tag_name.eq_ignore_ascii_case("area")
        || tag_name.eq_ignore_ascii_case("base")
        || tag_name.eq_ignore_ascii_case("br")
        || tag_name.eq_ignore_ascii_case("col")
        || tag_name.eq_ignore_ascii_case("embed")
        || tag_name.eq_ignore_ascii_case("hr")
        || tag_name.eq_ignore_ascii_case("img")
        || tag_name.eq_ignore_ascii_case("input")
        || tag_name.eq_ignore_ascii_case("link")
        || tag_name.eq_ignore_ascii_case("meta")
        || tag_name.eq_ignore_ascii_case("param")
        || tag_name.eq_ignore_ascii_case("source")
        || tag_name.eq_ignore_ascii_case("track")
        || tag_name.eq_ignore_ascii_case("wbr")
}

fn decode_html_entities(value: &str) -> CompactString {
    if !value.contains('&') {
        return CompactString::from(value);
    }

    let mut output = String::with_capacity(value.len());
    let mut cursor = 0;
    while let Some(relative_amp) = value[cursor..].find('&') {
        let amp = cursor + relative_amp;
        output.push_str(&value[cursor..amp]);
        let Some(relative_semicolon) = value[amp + 1..].find(';') else {
            output.push('&');
            cursor = amp + 1;
            continue;
        };
        let semicolon = amp + 1 + relative_semicolon;
        let entity = &value[amp + 1..semicolon];
        if entity.len() > 32 {
            output.push('&');
            cursor = amp + 1;
            continue;
        }
        if let Some(decoded) = decode_html_entity(entity) {
            output.push(decoded);
            cursor = semicolon + 1;
        } else {
            output.push('&');
            cursor = amp + 1;
        }
    }
    output.push_str(&value[cursor..]);
    output.into()
}

fn decode_html_entity(entity: &str) -> Option<char> {
    match entity {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        "nbsp" => Some('\u{00a0}'),
        value if value.starts_with("#x") || value.starts_with("#X") => {
            u32::from_str_radix(&value[2..], 16).ok().and_then(char::from_u32)
        }
        value if value.starts_with('#') => value[1..].parse::<u32>().ok().and_then(char::from_u32),
        _ => None,
    }
}
