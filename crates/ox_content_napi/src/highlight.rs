mod scan;
mod tag;
#[cfg(test)]
mod tests;

use scan::{collect_pre_blocks, find_next_start_tag};
use tag::ParsedAttribute;

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
