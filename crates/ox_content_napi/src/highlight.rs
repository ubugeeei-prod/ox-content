#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedAttribute {
    name: String,
    value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedStartTag {
    name: String,
    attributes: Vec<ParsedAttribute>,
}

impl ParsedStartTag {
    fn parse(raw: &str) -> Option<Self> {
        if !raw.starts_with('<') || !raw.ends_with('>') || raw.starts_with("</") {
            return None;
        }

        let inner = &raw[1..raw.len() - 1];
        let bytes = inner.as_bytes();
        let mut index = 0;

        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }

        let name_start = index;
        while index < bytes.len() && !bytes[index].is_ascii_whitespace() && bytes[index] != b'/' {
            index += 1;
        }

        if name_start == index {
            return None;
        }

        let name = inner[name_start..index].to_string();
        let mut attributes = Vec::new();

        while index < bytes.len() {
            while index < bytes.len() && bytes[index].is_ascii_whitespace() {
                index += 1;
            }

            if index >= bytes.len() || bytes[index] == b'/' {
                break;
            }

            let attr_start = index;
            while index < bytes.len()
                && !bytes[index].is_ascii_whitespace()
                && bytes[index] != b'='
                && bytes[index] != b'/'
            {
                index += 1;
            }

            if attr_start == index {
                break;
            }

            let attr_name = inner[attr_start..index].to_string();

            while index < bytes.len() && bytes[index].is_ascii_whitespace() {
                index += 1;
            }

            let value = if index < bytes.len() && bytes[index] == b'=' {
                index += 1;
                while index < bytes.len() && bytes[index].is_ascii_whitespace() {
                    index += 1;
                }

                if index >= bytes.len() {
                    Some(String::new())
                } else if bytes[index] == b'"' || bytes[index] == b'\'' {
                    let quote = bytes[index];
                    index += 1;
                    let value_start = index;

                    while index < bytes.len() && bytes[index] != quote {
                        index += 1;
                    }

                    let value = inner[value_start..index].to_string();
                    if index < bytes.len() {
                        index += 1;
                    }
                    Some(value)
                } else {
                    let value_start = index;
                    while index < bytes.len()
                        && !bytes[index].is_ascii_whitespace()
                        && bytes[index] != b'/'
                    {
                        index += 1;
                    }
                    Some(inner[value_start..index].to_string())
                }
            } else {
                None
            };

            attributes.push(ParsedAttribute { name: attr_name, value });
        }

        Some(Self { name, attributes })
    }

    fn to_html(&self) -> String {
        let mut html = String::new();
        html.push('<');
        html.push_str(&self.name);

        for attribute in &self.attributes {
            html.push(' ');
            html.push_str(&attribute.name);
            if let Some(value) = &attribute.value {
                html.push_str("=\"");
                html.push_str(value);
                html.push('"');
            }
        }

        html.push('>');
        html
    }

    fn attribute_value(&self, name: &str) -> Option<&str> {
        self.attributes.iter().find_map(|attribute| {
            if attribute.name == name {
                attribute.value.as_deref()
            } else {
                None
            }
        })
    }

    fn set_attribute(&mut self, name: &str, value: &str) {
        if let Some(attribute) = self.attributes.iter_mut().find(|attribute| attribute.name == name)
        {
            attribute.value = Some(value.to_string());
            return;
        }

        self.attributes
            .push(ParsedAttribute { name: name.to_string(), value: Some(value.to_string()) });
    }

    fn class_names(&self) -> Vec<String> {
        self.attribute_value("class")
            .map(|value| value.split_whitespace().map(ToString::to_string).collect())
            .unwrap_or_default()
    }

    fn merge_class_names(&mut self, class_names: &[String]) {
        if class_names.is_empty() {
            return;
        }

        let mut merged = self.class_names();
        for class_name in class_names {
            if !merged.contains(class_name) {
                merged.push(class_name.clone());
            }
        }
        self.set_class_names(&merged);
    }

    fn set_class_names(&mut self, class_names: &[String]) {
        let value = class_names.join(" ");
        self.set_attribute("class", &value);
    }

    fn data_attributes(&self) -> Vec<ParsedAttribute> {
        self.attributes
            .iter()
            .filter(|attribute| attribute.name.starts_with("data-"))
            .cloned()
            .collect()
    }

    fn merge_data_attributes(&mut self, attributes: &[ParsedAttribute]) {
        for attribute in attributes {
            match &attribute.value {
                Some(value) => self.set_attribute(&attribute.name, value),
                None => {
                    if self.attributes.iter().all(|existing| existing.name != attribute.name) {
                        self.attributes.push(attribute.clone());
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LineMetadata {
    class_names: Vec<String>,
    data_attributes: Vec<ParsedAttribute>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CodeBlockMetadata {
    pre_classes: Vec<String>,
    pre_data_attributes: Vec<ParsedAttribute>,
    code_classes: Vec<String>,
    code_data_attributes: Vec<ParsedAttribute>,
    language: Option<String>,
    lines: Vec<LineMetadata>,
}

#[derive(Debug, Clone)]
struct TagMatch {
    start: usize,
    end: usize,
    tag: ParsedStartTag,
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

fn find_next_start_tag(html: &str, from: usize) -> Option<TagMatch> {
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

fn collect_pre_blocks(html: &str) -> Vec<(usize, usize)> {
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

fn extract_code_block_metadata(original_block: &str) -> CodeBlockMetadata {
    let mut index = 0;
    let mut pre_classes = Vec::new();
    let mut pre_data_attributes = Vec::new();
    let mut code_classes = Vec::new();
    let mut code_data_attributes = Vec::new();
    let mut language = None;
    let mut lines = Vec::new();

    while let Some(tag_match) = find_next_start_tag(original_block, index) {
        match tag_match.tag.name.as_str() {
            "pre" if pre_classes.is_empty() => {
                pre_classes = tag_match.tag.class_names();
                pre_data_attributes = tag_match.tag.data_attributes();
            }
            "code" if code_classes.is_empty() => {
                code_classes = tag_match.tag.class_names();
                code_data_attributes = tag_match.tag.data_attributes();
                language = code_classes
                    .iter()
                    .find_map(|class_name| class_name.strip_prefix("language-"))
                    .map(ToString::to_string);
            }
            "span" => {
                let class_names = tag_match.tag.class_names();
                if class_names.iter().any(|class_name| class_name == "line") {
                    lines.push(LineMetadata {
                        class_names,
                        data_attributes: tag_match.tag.data_attributes(),
                    });
                }
            }
            _ => {}
        }

        index = tag_match.end;
    }

    CodeBlockMetadata {
        pre_classes,
        pre_data_attributes,
        code_classes,
        code_data_attributes,
        language,
        lines,
    }
}

fn merge_highlighted_code_block(original_block: &str, highlighted_block: &str) -> String {
    let metadata = extract_code_block_metadata(original_block);
    let mut merged = String::with_capacity(highlighted_block.len() + 64);
    let mut index = 0;
    let mut pre_updated = false;
    let mut code_updated = false;
    let mut line_index = 0;

    while let Some(tag_match) = find_next_start_tag(highlighted_block, index) {
        merged.push_str(&highlighted_block[index..tag_match.start]);

        let mut tag = tag_match.tag;
        match tag.name.as_str() {
            "pre" if !pre_updated => {
                tag.merge_class_names(&metadata.pre_classes);
                tag.merge_data_attributes(&metadata.pre_data_attributes);
                if let Some(language) = metadata.language.as_deref() {
                    tag.set_attribute("data-language", language);
                }
                merged.push_str(&tag.to_html());
                pre_updated = true;
            }
            "code" if !code_updated => {
                if !metadata.code_classes.is_empty() {
                    tag.set_class_names(&metadata.code_classes);
                }
                tag.merge_data_attributes(&metadata.code_data_attributes);
                if let Some(language) = metadata.language.as_deref() {
                    tag.set_attribute("data-language", language);
                }
                merged.push_str(&tag.to_html());
                code_updated = true;
            }
            "span" => {
                let is_line = tag.class_names().iter().any(|class_name| class_name == "line");
                if is_line {
                    if let Some(line_metadata) = metadata.lines.get(line_index) {
                        tag.merge_class_names(&line_metadata.class_names);
                        tag.merge_data_attributes(&line_metadata.data_attributes);
                    }
                    line_index += 1;
                    merged.push_str(&tag.to_html());
                } else {
                    merged.push_str(&highlighted_block[tag_match.start..tag_match.end]);
                }
            }
            _ => merged.push_str(&highlighted_block[tag_match.start..tag_match.end]),
        }

        index = tag_match.end;
    }

    merged.push_str(&highlighted_block[index..]);
    merged
}

pub fn merge_highlighted_code_blocks(original_html: &str, highlighted_html: &str) -> String {
    let original_blocks = collect_pre_blocks(original_html);
    let highlighted_blocks = collect_pre_blocks(highlighted_html);

    if highlighted_blocks.is_empty() {
        return highlighted_html.to_string();
    }

    let mut merged = String::with_capacity(highlighted_html.len() + 128);
    let mut cursor = 0;

    for (index, (highlight_start, highlight_end)) in highlighted_blocks.iter().enumerate() {
        merged.push_str(&highlighted_html[cursor..*highlight_start]);

        if let Some((original_start, original_end)) = original_blocks.get(index) {
            merged.push_str(&merge_highlighted_code_block(
                &original_html[*original_start..*original_end],
                &highlighted_html[*highlight_start..*highlight_end],
            ));
        } else {
            merged.push_str(&highlighted_html[*highlight_start..*highlight_end]);
        }

        cursor = *highlight_end;
    }

    merged.push_str(&highlighted_html[cursor..]);
    merged
}

#[cfg(test)]
mod tests {
    use super::merge_highlighted_code_blocks;

    #[test]
    fn merges_annotation_metadata_into_highlighted_html() {
        let original = r#"<p>Before</p><pre class="ox-code-block ox-code-block--annotated"><code class="language-ts"><span class="line ox-code-line ox-code-line--highlight" data-line="1">const first = 1;</span>
<span class="line ox-code-line ox-code-line--warning" data-line="2">const second = 2;</span>
<span class="line ox-code-line ox-code-line--error" data-line="3">throw new Error("boom");</span>
</code></pre><p>After</p>"#;
        let highlighted = r#"<p>Before</p><pre class="shiki github-dark" style="background-color:#24292e;color:#e1e4e8" tabindex="0"><code><span class="line"><span style="color:#E1E4E8">const first = 1;</span></span>
<span class="line"><span style="color:#E1E4E8">const second = 2;</span></span>
<span class="line"><span style="color:#E1E4E8">throw new Error("boom");</span></span>
</code></pre><p>After</p>"#;

        let merged = merge_highlighted_code_blocks(original, highlighted);

        assert!(merged
            .contains(r#"<pre class="shiki github-dark ox-code-block ox-code-block--annotated""#));
        assert!(merged.contains(r#"class="language-ts" data-language="ts""#));
        assert!(
            merged.contains(r#"class="line ox-code-line ox-code-line--highlight" data-line="1""#)
        );
        assert!(merged.contains(r#"class="line ox-code-line ox-code-line--warning" data-line="2""#));
        assert!(merged.contains(r#"class="line ox-code-line ox-code-line--error" data-line="3""#));
    }

    #[test]
    fn preserves_language_metadata_for_non_annotated_code_blocks() {
        let original = r#"<pre><code class="language-rs">fn main() {}
</code></pre>"#;
        let highlighted = r#"<pre class="shiki github-dark" style="background-color:#24292e;color:#e1e4e8" tabindex="0"><code><span class="line"><span style="color:#E1E4E8">fn main() {}</span></span>
</code></pre>"#;

        let merged = merge_highlighted_code_blocks(original, highlighted);

        assert!(merged.contains(r#"class="language-rs" data-language="rs""#));
        assert!(merged.contains(r#"<pre class="shiki github-dark" style="background-color:#24292e;color:#e1e4e8" tabindex="0" data-language="rs">"#));
    }
}
