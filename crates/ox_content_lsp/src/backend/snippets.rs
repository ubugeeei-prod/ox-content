use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionTextEdit, InsertTextFormat, Position, TextEdit,
};

use crate::document::TextDocumentState;

#[allow(clippy::literal_string_with_formatting_args)]
pub(super) fn markdown_snippet_items(
    document: &TextDocumentState,
    position: Position,
) -> Vec<CompletionItem> {
    let line = document.line_text(position.line);
    let prefix = line[..document
        .position_to_offset(position)
        .saturating_sub(document.line_start_offset(position.line as usize))
        .min(line.len())]
        .trim();

    if !prefix.is_empty() && !prefix.chars().all(|ch| ch.is_ascii_alphanumeric()) {
        return Vec::new();
    }

    let replace = document.word_range_at(position, |ch| ch.is_ascii_alphanumeric());
    [
        ("h1", "Page heading", "# ${1:Title}\n\n$0"),
        ("h2", "Section heading", "## ${1:Section}\n\n$0"),
        ("quote", "Blockquote", "> ${1:Quoted text}\n\n$0"),
        ("list", "Bullet list", "- ${1:First item}\n- ${2:Second item}\n$0"),
        ("task", "Task list", "- [ ] ${1:Open item}\n- [x] ${2:Done item}\n$0"),
        ("link", "Markdown link", "[${1:label}](${2:https://example.com})$0"),
        ("image", "Markdown image", "![${1:alt text}](${2:/path/to/image.png})$0"),
        ("code", "TypeScript code fence", "```ts\n${1:const value = true}\n```\n$0"),
        (
            "table",
            "Simple table",
            "| ${1:Column} | ${2:Column} |\n| --- | --- |\n| ${3:Value} | ${4:Value} |\n$0",
        ),
    ]
    .into_iter()
    .enumerate()
    .map(|(index, (label, detail, insert_text))| CompletionItem {
        label: label.to_string(),
        kind: Some(CompletionItemKind::SNIPPET),
        detail: Some(detail.to_string()),
        insert_text: Some(insert_text.to_string()),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        sort_text: Some(format!("9{index:02}")),
        text_edit: Some(CompletionTextEdit::Edit(TextEdit {
            range: replace,
            new_text: insert_text.to_string(),
        })),
        ..Default::default()
    })
    .collect()
}
