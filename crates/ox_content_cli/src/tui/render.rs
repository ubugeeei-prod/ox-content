use super::style::{clean, pad, width, wrap};
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

#[derive(Debug)]
pub struct Heading {
    pub title: String,
    pub line: usize,
}
#[derive(Debug)]
pub struct Link {
    pub title: String,
    pub href: String,
}
#[derive(Default)]
pub struct Document {
    pub lines: Vec<String>,
    pub outline: Vec<Heading>,
    pub links: Vec<Link>,
}

pub fn render(source: &str, columns: usize) -> Document {
    let source = clean(source).replace("\r\n", "\n").replace('\t', "    ");
    let mut document = Document::default();
    let mut source = source.as_str();
    if let Some(rest) = source.strip_prefix("---\n") {
        let mut offset = 0;
        for line in rest.split_inclusive('\n') {
            if ["---", "..."].contains(&line.trim()) {
                document.lines.push("◇ Frontmatter".to_string());
                for line in rest[..offset].lines().take(8) {
                    put(&mut document, line, "  ", columns);
                }
                document.lines.push(String::new());
                source = &rest[offset + line.len()..];
                break;
            }
            offset += line.len();
        }
    }
    let mut text = String::new();
    let mut heading = None;
    let mut code = false;
    let mut quote_depth = 0;
    let mut lists = Vec::<Option<u64>>::new();
    let mut prefix = String::new();
    let mut link = None;
    let mut link_start = 0;
    let mut table = Vec::<Vec<String>>::new();
    let mut row = Vec::<String>::new();
    for event in Parser::new_ext(
        source,
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS,
    ) {
        let indent = "│ ".repeat(quote_depth) + &"  ".repeat(lists.len().saturating_sub(1));
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                heading = Some(level as usize);
            }
            Event::End(TagEnd::Heading(_)) => {
                document.outline.push(Heading { title: text.clone(), line: document.lines.len() });
                put(
                    &mut document,
                    &format!("{} {text}", "#".repeat(heading.unwrap_or(1))),
                    &indent,
                    columns,
                );
                if heading == Some(1) {
                    put(
                        &mut document,
                        &"─".repeat(columns.saturating_sub(width(&indent))),
                        &indent,
                        columns,
                    );
                }
                document.lines.push(String::new());
                text.clear();
                heading = None;
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                let language = match kind {
                    pulldown_cmark::CodeBlockKind::Fenced(value) => value.into_string(),
                    pulldown_cmark::CodeBlockKind::Indented => "code".to_string(),
                };
                put(&mut document, &format!("╭─ {language}"), &indent, columns);
                code = true;
            }
            Event::End(TagEnd::CodeBlock) => {
                for line in text.trim_end_matches('\n').lines() {
                    put(&mut document, line, &(indent.clone() + "│ "), columns);
                }
                put(&mut document, "╰────────────────", &indent, columns);
                document.lines.push(String::new());
                text.clear();
                code = false;
            }
            Event::Start(Tag::BlockQuote(_)) => {
                quote_depth += 1;
            }
            Event::End(TagEnd::BlockQuote(_)) => {
                quote_depth = quote_depth.saturating_sub(1);
            }
            Event::Start(Tag::List(start)) => {
                if !text.is_empty() {
                    put(&mut document, &(prefix.clone() + &text), &indent, columns);
                    text.clear();
                    prefix.clear();
                }
                lists.push(start);
            }
            Event::End(TagEnd::List(_)) => {
                lists.pop();
            }
            Event::Start(Tag::Item) => {
                prefix = if let Some(Some(number)) = lists.last_mut() {
                    let p = format!("{number}. ");
                    *number += 1;
                    p
                } else {
                    "• ".to_string()
                };
            }
            Event::TaskListMarker(checked) => {
                prefix = if checked { "☑ " } else { "☐ " }.to_string();
            }
            Event::End(TagEnd::Paragraph | TagEnd::Item) if !text.is_empty() => {
                put(&mut document, &(prefix.clone() + &text), &indent, columns);
                document.lines.push(String::new());
                text.clear();
                prefix.clear();
            }
            Event::Start(Tag::Link { dest_url, .. }) => {
                link = Some(dest_url.into_string());
                link_start = text.len();
            }
            Event::Start(Tag::Image { .. }) => text.push_str("[image: "),
            Event::End(TagEnd::Image) => text.push(']'),
            Event::End(TagEnd::Link) => {
                if let Some(href) = link.take() {
                    document
                        .links
                        .push(Link { title: text[link_start..].to_string(), href: href.clone() });
                    text.push_str(" (");
                    text.push_str(&href);
                    text.push(')');
                }
            }
            Event::Start(Tag::Table(_)) => {
                table.clear();
            }
            Event::End(TagEnd::TableCell) => {
                row.push(std::mem::take(&mut text));
            }
            Event::End(TagEnd::TableHead | TagEnd::TableRow) => {
                table.push(std::mem::take(&mut row));
            }
            Event::End(TagEnd::Table) => {
                render_table(&mut document, &table, columns);
            }
            Event::Text(value) | Event::Code(value) => {
                text.push_str(&value);
            }
            Event::SoftBreak => {
                text.push(if code { '\n' } else { ' ' });
            }
            Event::HardBreak => {
                text.push('\n');
            }
            Event::Rule => {
                put(&mut document, &"─".repeat(columns), "", columns);
            }
            _ => {}
        }
    }
    if !text.is_empty() {
        put(&mut document, &text, "", columns);
    }
    document
}

// Content remains plain until drawing so widths and search never see terminal escapes.
fn put(document: &mut Document, text: &str, indent: &str, columns: usize) {
    for line in wrap(text, columns.saturating_sub(width(indent))) {
        document.lines.push(indent.to_string() + &line);
    }
}

fn render_table(document: &mut Document, rows: &[Vec<String>], columns: usize) {
    let count = rows.first().map_or(0, Vec::len);
    if count == 0 {
        return;
    }
    let available = columns.saturating_sub(count * 3 + 1) / count;
    let sizes: Vec<_> = (0..count)
        .map(|index| {
            rows.iter()
                .filter_map(|row| row.get(index))
                .map(|cell| width(cell))
                .max()
                .unwrap_or(1)
                .min(available)
                .max(1)
        })
        .collect();
    for (index, row) in rows.iter().enumerate() {
        let cells: Vec<_> = row.iter().zip(&sizes).map(|(cell, size)| wrap(cell, *size)).collect();
        for line in 0..cells.iter().map(Vec::len).max().unwrap_or(1) {
            let value = cells
                .iter()
                .zip(&sizes)
                .map(|(cell, size)| pad(cell.get(line).map_or("", String::as_str), *size))
                .collect::<Vec<_>>()
                .join(" │ ");
            put(document, &format!("│ {value} │"), "", columns);
        }
        if index == 0 {
            put(
                document,
                &format!(
                    "├{}┤",
                    sizes.iter().map(|size| "─".repeat(*size + 2)).collect::<Vec<_>>().join("┼")
                ),
                "",
                columns,
            );
        }
    }
    document.lines.push(String::new());
}
