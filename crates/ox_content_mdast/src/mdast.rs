use ox_content_allocator::Vec as ArenaVec;
use ox_content_ast::{
    AlignKind, BlockQuote, CodeBlock, Definition, Delete, Document, Emphasis, FootnoteDefinition,
    FootnoteReference, Heading, Html, Image, InlineCode, Link, List, ListItem, Node, Paragraph,
    Strong, Table, TableCell, TableRow, Text, ThematicBreak,
};

pub fn to_mdast_json(document: &Document<'_>) -> String {
    // mdast JSON expands well past the source: every text run carries
    // `{"type":"text","value":...}` framing and structural nodes carry
    // their own objects. Real corpora land between 3× and 4.5× of the
    // source span; 5× keeps the buffer single-allocation (the old 2×
    // guaranteed two growth copies on markup-dense documents).
    let estimated_len = (document.span.len() as usize).saturating_mul(5).max(128);
    let mut serializer = MdastJsonSerializer { output: String::with_capacity(estimated_len) };
    serializer.write_document(document);
    serializer.output
}

struct MdastJsonSerializer {
    output: String,
}

impl MdastJsonSerializer {
    fn write_document(&mut self, document: &Document<'_>) {
        self.output.push_str("{\"type\":\"root\",\"children\":");
        self.write_nodes(&document.children);
        self.output.push('}');
    }

    fn write_nodes<'a>(&mut self, nodes: &ArenaVec<'a, Node<'a>>) {
        self.output.push('[');
        for (idx, node) in nodes.iter().enumerate() {
            if idx > 0 {
                self.output.push(',');
            }
            self.write_node(node);
        }
        self.output.push(']');
    }

    fn write_list_items<'a>(&mut self, items: &ArenaVec<'a, ListItem<'a>>) {
        self.output.push('[');
        for (idx, item) in items.iter().enumerate() {
            if idx > 0 {
                self.output.push(',');
            }
            self.write_list_item(item);
        }
        self.output.push(']');
    }

    fn write_table_rows<'a>(&mut self, rows: &ArenaVec<'a, TableRow<'a>>) {
        self.output.push('[');
        for (idx, row) in rows.iter().enumerate() {
            if idx > 0 {
                self.output.push(',');
            }
            self.write_table_row(row);
        }
        self.output.push(']');
    }

    fn write_table_cells<'a>(&mut self, cells: &ArenaVec<'a, TableCell<'a>>) {
        self.output.push('[');
        for (idx, cell) in cells.iter().enumerate() {
            if idx > 0 {
                self.output.push(',');
            }
            self.write_table_cell(cell);
        }
        self.output.push(']');
    }

    fn write_node(&mut self, node: &Node<'_>) {
        match node {
            Node::Paragraph(node) => self.write_paragraph(node),
            Node::Heading(node) => self.write_heading(node),
            Node::ThematicBreak(node) => self.write_thematic_break(node),
            Node::BlockQuote(node) => self.write_block_quote(node),
            Node::List(node) => self.write_list(node),
            Node::ListItem(node) => self.write_list_item(node),
            Node::CodeBlock(node) => self.write_code_block(node),
            Node::Html(node) => self.write_html(node),
            Node::Table(node) => self.write_table(node),
            Node::Text(node) => self.write_text(node),
            Node::Emphasis(node) => self.write_emphasis(node),
            Node::Strong(node) => self.write_strong(node),
            Node::InlineCode(node) => self.write_inline_code(node),
            Node::Break(_) => self.output.push_str("{\"type\":\"break\"}"),
            Node::Link(node) => self.write_link(node),
            Node::Image(node) => self.write_image(node),
            Node::Delete(node) => self.write_delete(node),
            Node::FootnoteReference(node) => self.write_footnote_reference(node),
            Node::Definition(node) => self.write_definition(node),
            Node::FootnoteDefinition(node) => self.write_footnote_definition(node),
        }
    }

    fn write_paragraph(&mut self, paragraph: &Paragraph<'_>) {
        self.output.push_str("{\"type\":\"paragraph\",\"children\":");
        self.write_nodes(&paragraph.children);
        self.output.push('}');
    }

    fn write_heading(&mut self, heading: &Heading<'_>) {
        self.output.push_str("{\"type\":\"heading\",\"depth\":");
        self.write_u32(u32::from(heading.depth));
        self.output.push_str(",\"children\":");
        self.write_nodes(&heading.children);
        self.output.push('}');
    }

    fn write_thematic_break(&mut self, _thematic_break: &ThematicBreak) {
        self.output.push_str("{\"type\":\"thematicBreak\"}");
    }

    fn write_block_quote(&mut self, block_quote: &BlockQuote<'_>) {
        self.output.push_str("{\"type\":\"blockquote\",\"children\":");
        self.write_nodes(&block_quote.children);
        self.output.push('}');
    }

    fn write_list(&mut self, list: &List<'_>) {
        self.output.push_str("{\"type\":\"list\",\"ordered\":");
        self.output.push_str(if list.ordered { "true" } else { "false" });
        self.output.push_str(",\"spread\":");
        self.output.push_str(if list.spread { "true" } else { "false" });
        if let Some(start) = list.start {
            self.output.push_str(",\"start\":");
            self.write_u32(start);
        }
        self.output.push_str(",\"children\":");
        self.write_list_items(&list.children);
        self.output.push('}');
    }

    fn write_list_item(&mut self, list_item: &ListItem<'_>) {
        self.output.push_str("{\"type\":\"listItem\",\"spread\":");
        self.output.push_str(if list_item.spread { "true" } else { "false" });
        if let Some(checked) = list_item.checked {
            self.output.push_str(",\"checked\":");
            self.output.push_str(if checked { "true" } else { "false" });
        }
        self.output.push_str(",\"children\":");
        self.write_nodes(&list_item.children);
        self.output.push('}');
    }

    fn write_code_block(&mut self, code_block: &CodeBlock<'_>) {
        self.output.push_str("{\"type\":\"code\"");
        if let Some(lang) = code_block.lang {
            self.output.push_str(",\"lang\":");
            self.write_string(lang);
        }
        if let Some(meta) = code_block.meta {
            self.output.push_str(",\"meta\":");
            self.write_string(meta);
        }
        self.output.push_str(",\"value\":");
        self.write_string(code_block.value);
        self.output.push('}');
    }

    fn write_html(&mut self, html: &Html<'_>) {
        self.output.push_str("{\"type\":\"html\",\"value\":");
        self.write_string(html.value);
        self.output.push('}');
    }

    fn write_table(&mut self, table: &Table<'_>) {
        self.output.push_str("{\"type\":\"table\",\"align\":");
        self.write_align(&table.align);
        self.output.push_str(",\"children\":");
        self.write_table_rows(&table.children);
        self.output.push('}');
    }

    fn write_table_row(&mut self, row: &TableRow<'_>) {
        self.output.push_str("{\"type\":\"tableRow\",\"children\":");
        self.write_table_cells(&row.children);
        self.output.push('}');
    }

    fn write_table_cell(&mut self, cell: &TableCell<'_>) {
        self.output.push_str("{\"type\":\"tableCell\",\"children\":");
        self.write_nodes(&cell.children);
        self.output.push('}');
    }

    fn write_text(&mut self, text: &Text<'_>) {
        self.output.push_str("{\"type\":\"text\",\"value\":");
        self.write_string(text.value);
        self.output.push('}');
    }

    fn write_emphasis(&mut self, emphasis: &Emphasis<'_>) {
        self.output.push_str("{\"type\":\"emphasis\",\"children\":");
        self.write_nodes(&emphasis.children);
        self.output.push('}');
    }

    fn write_strong(&mut self, strong: &Strong<'_>) {
        self.output.push_str("{\"type\":\"strong\",\"children\":");
        self.write_nodes(&strong.children);
        self.output.push('}');
    }

    fn write_inline_code(&mut self, inline_code: &InlineCode<'_>) {
        self.output.push_str("{\"type\":\"inlineCode\",\"value\":");
        self.write_string(inline_code.value);
        self.output.push('}');
    }

    fn write_link(&mut self, link: &Link<'_>) {
        self.output.push_str("{\"type\":\"link\",\"url\":");
        self.write_string(link.url);
        if let Some(title) = link.title {
            self.output.push_str(",\"title\":");
            self.write_string(title);
        }
        self.output.push_str(",\"children\":");
        self.write_nodes(&link.children);
        self.output.push('}');
    }

    fn write_image(&mut self, image: &Image<'_>) {
        self.output.push_str("{\"type\":\"image\",\"url\":");
        self.write_string(image.url);
        self.output.push_str(",\"alt\":");
        self.write_string(image.alt);
        if let Some(title) = image.title {
            self.output.push_str(",\"title\":");
            self.write_string(title);
        }
        self.output.push('}');
    }

    fn write_delete(&mut self, delete: &Delete<'_>) {
        self.output.push_str("{\"type\":\"delete\",\"children\":");
        self.write_nodes(&delete.children);
        self.output.push('}');
    }

    fn write_footnote_reference(&mut self, footnote_ref: &FootnoteReference<'_>) {
        self.output.push_str("{\"type\":\"footnoteReference\",\"identifier\":");
        self.write_string(footnote_ref.identifier);
        if let Some(label) = footnote_ref.label {
            self.output.push_str(",\"label\":");
            self.write_string(label);
        }
        self.output.push('}');
    }

    fn write_definition(&mut self, definition: &Definition<'_>) {
        self.output.push_str("{\"type\":\"definition\",\"identifier\":");
        self.write_string(definition.identifier);
        if let Some(label) = definition.label {
            self.output.push_str(",\"label\":");
            self.write_string(label);
        }
        self.output.push_str(",\"url\":");
        self.write_string(definition.url);
        if let Some(title) = definition.title {
            self.output.push_str(",\"title\":");
            self.write_string(title);
        }
        self.output.push('}');
    }

    fn write_footnote_definition(&mut self, footnote_definition: &FootnoteDefinition<'_>) {
        self.output.push_str("{\"type\":\"footnoteDefinition\",\"identifier\":");
        self.write_string(footnote_definition.identifier);
        if let Some(label) = footnote_definition.label {
            self.output.push_str(",\"label\":");
            self.write_string(label);
        }
        self.output.push_str(",\"children\":");
        self.write_nodes(&footnote_definition.children);
        self.output.push('}');
    }

    fn write_align(&mut self, align: &ArenaVec<'_, AlignKind>) {
        self.output.push('[');
        for (idx, item) in align.iter().enumerate() {
            if idx > 0 {
                self.output.push(',');
            }
            match item {
                AlignKind::None => self.output.push_str("null"),
                AlignKind::Left => self.write_string("left"),
                AlignKind::Center => self.write_string("center"),
                AlignKind::Right => self.write_string("right"),
            }
        }
        self.output.push(']');
    }

    fn write_string(&mut self, value: &str) {
        self.output.push('"');
        let bytes = value.as_bytes();
        let mut start = 0usize;
        let mut i = 0usize;

        while i < bytes.len() {
            // 8-byte chunk fast-skip over bytes that never need escaping
            // (everything except `"`, `\`, and control bytes < 0x20) —
            // the common case for prose, URLs, and code values. Same
            // shape as the renderer's HTML escape scan.
            while i + 8 <= bytes.len() {
                let chunk = &bytes[i..i + 8];
                let mask = JSON_ESCAPE_FLAG[chunk[0] as usize]
                    | JSON_ESCAPE_FLAG[chunk[1] as usize]
                    | JSON_ESCAPE_FLAG[chunk[2] as usize]
                    | JSON_ESCAPE_FLAG[chunk[3] as usize]
                    | JSON_ESCAPE_FLAG[chunk[4] as usize]
                    | JSON_ESCAPE_FLAG[chunk[5] as usize]
                    | JSON_ESCAPE_FLAG[chunk[6] as usize]
                    | JSON_ESCAPE_FLAG[chunk[7] as usize];
                if mask != 0 {
                    break;
                }
                i += 8;
            }
            while i < bytes.len() && JSON_ESCAPE_FLAG[bytes[i] as usize] == 0 {
                i += 1;
            }
            if i >= bytes.len() {
                break;
            }

            if start < i {
                self.output.push_str(&value[start..i]);
            }
            let byte = bytes[i];
            match byte {
                b'"' => self.output.push_str("\\\""),
                b'\\' => self.output.push_str("\\\\"),
                b'\n' => self.output.push_str("\\n"),
                b'\r' => self.output.push_str("\\r"),
                b'\t' => self.output.push_str("\\t"),
                b'\x08' => self.output.push_str("\\b"),
                b'\x0c' => self.output.push_str("\\f"),
                _ => push_json_byte_escape(&mut self.output, byte),
            }
            i += 1;
            start = i;
        }

        if start < value.len() {
            self.output.push_str(&value[start..]);
        }
        self.output.push('"');
    }

    /// Writes a decimal integer without the `to_string` heap allocation
    /// the previous implementation paid per heading depth / list start.
    fn write_u32(&mut self, mut n: u32) {
        let mut buf = [0u8; 10];
        let mut at = buf.len();
        loop {
            at -= 1;
            buf[at] = b'0' + (n % 10) as u8;
            n /= 10;
            if n == 0 {
                break;
            }
        }
        for &digit in &buf[at..] {
            self.output.push(char::from(digit));
        }
    }
}

/// `JSON_ESCAPE_FLAG[b] == 1` iff `b` must be escaped inside a JSON
/// string: the two structural bytes and every control byte below 0x20.
static JSON_ESCAPE_FLAG: [u8; 256] = {
    let mut t = [0u8; 256];
    let mut b = 0usize;
    while b < 0x20 {
        t[b] = 1;
        b += 1;
    }
    t[b'"' as usize] = 1;
    t[b'\\' as usize] = 1;
    t
};

fn push_json_byte_escape(output: &mut String, byte: u8) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    output.push_str("\\u00");
    output.push(char::from(HEX[usize::from((byte >> 4) & 0x0f)]));
    output.push(char::from(HEX[usize::from(byte & 0x0f)]));
}

#[cfg(test)]
mod tests;
