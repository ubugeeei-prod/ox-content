//! Rebuilds an arena AST from mdast JSON.
//!
//! [`crate::mdast::to_mdast_json`] hands a document to JavaScript; this is the
//! way back, so a caller that rewrites the tree there can have the result
//! rendered by the same renderer that would have rendered the original.
//!
//! Position information is not part of the payload. Spans therefore come back
//! empty, which the HTML renderer only uses to size its output buffer.

use ox_content_allocator::{Allocator, Vec as ArenaVec};
use ox_content_ast::{
    AlignKind, BlockQuote, Break, CodeBlock, Definition, Delete, Document, Emphasis,
    FootnoteDefinition, FootnoteReference, Heading, Html, Image, InlineCode, Link, List, ListItem,
    MdxFlowExpression, MdxJsxFlowElement, MdxJsxTextElement, MdxTextExpression, MdxjsEsm, Node,
    Paragraph, Span, Strong, Table, TableCell, TableRow, Text, ThematicBreak,
};
use serde_json::Value;

mod mdx;

use mdx::mdx_attributes;

/// Why an mdast payload could not be turned back into a document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MdastJsonError {
    message: String,
}

impl MdastJsonError {
    fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }

    /// The human-readable reason.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl std::fmt::Display for MdastJsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for MdastJsonError {}

/// Rebuilds a document from the mdast JSON of [`crate::mdast::to_mdast_json`].
///
/// # Errors
///
/// Returns an error when the payload is not valid JSON, is not an mdast root,
/// or holds a node whose `type` this AST has no counterpart for. A rewrite
/// that invents a node type is a mistake worth reporting rather than
/// silently dropping part of the page.
pub fn from_mdast_json<'a>(
    allocator: &'a Allocator,
    json: &str,
) -> Result<Document<'a>, MdastJsonError> {
    let value: Value =
        serde_json::from_str(json).map_err(|error| MdastJsonError::new(error.to_string()))?;

    let root =
        value.as_object().ok_or_else(|| MdastJsonError::new("mdast root is not an object"))?;
    match root.get("type").and_then(Value::as_str) {
        Some("root") => {}
        Some(other) => {
            return Err(MdastJsonError::new(format!(
                "mdast root has type {other:?}, not \"root\""
            )));
        }
        None => return Err(MdastJsonError::new("mdast root has no \"type\"")),
    }

    let children = nodes(allocator, root.get("children"))?;
    Ok(Document { children, span: Span::new(0, 0) })
}

fn nodes<'a>(
    allocator: &'a Allocator,
    value: Option<&Value>,
) -> Result<ArenaVec<'a, Node<'a>>, MdastJsonError> {
    let items = children_array(value)?;
    let mut out = allocator.new_vec_with_capacity(items.len());
    for item in items {
        out.push(node(allocator, item)?);
    }
    Ok(out)
}

fn node<'a>(allocator: &'a Allocator, value: &Value) -> Result<Node<'a>, MdastJsonError> {
    let kind = value
        .get("type")
        .and_then(Value::as_str)
        .ok_or_else(|| MdastJsonError::new("mdast node has no \"type\""))?;
    let span = Span::new(0, 0);

    Ok(match kind {
        "paragraph" => Node::Paragraph(
            allocator.boxed(Paragraph { children: nodes(allocator, value.get("children"))?, span }),
        ),
        "heading" => Node::Heading(allocator.boxed(Heading {
            depth: depth(value)?,
            children: nodes(allocator, value.get("children"))?,
            span,
        })),
        "thematicBreak" => Node::ThematicBreak(ThematicBreak { span }),
        "blockquote" => Node::BlockQuote(
            allocator
                .boxed(BlockQuote { children: nodes(allocator, value.get("children"))?, span }),
        ),
        "list" => Node::List(allocator.boxed(List {
            ordered: flag(value, "ordered"),
            start: value.get("start").and_then(Value::as_u64).and_then(|n| u32::try_from(n).ok()),
            spread: flag(value, "spread"),
            children: list_items(allocator, value.get("children"))?,
            span,
        })),
        "listItem" => Node::ListItem(allocator.boxed(list_item(allocator, value)?)),
        "code" => Node::CodeBlock(allocator.boxed(CodeBlock {
            lang: optional_str(allocator, value, "lang"),
            meta: optional_str(allocator, value, "meta"),
            value: required_str(allocator, value, "value")?,
            span,
        })),
        "html" => Node::Html(Html { value: required_str(allocator, value, "value")?, span }),
        "table" => Node::Table(allocator.boxed(Table {
            align: align(allocator, value.get("align")),
            children: table_rows(allocator, value.get("children"))?,
            span,
        })),
        "text" => Node::Text(Text { value: required_str(allocator, value, "value")?, span }),
        "emphasis" => Node::Emphasis(
            allocator.boxed(Emphasis { children: nodes(allocator, value.get("children"))?, span }),
        ),
        "strong" => Node::Strong(
            allocator.boxed(Strong { children: nodes(allocator, value.get("children"))?, span }),
        ),
        "inlineCode" => {
            Node::InlineCode(InlineCode { value: required_str(allocator, value, "value")?, span })
        }
        "break" => Node::Break(Break { span }),
        "link" => Node::Link(allocator.boxed(Link {
            url: required_str(allocator, value, "url")?,
            title: optional_str(allocator, value, "title"),
            children: nodes(allocator, value.get("children"))?,
            span,
        })),
        "image" => Node::Image(allocator.boxed(Image {
            url: required_str(allocator, value, "url")?,
            alt: optional_str(allocator, value, "alt").unwrap_or(""),
            title: optional_str(allocator, value, "title"),
            span,
        })),
        "delete" => Node::Delete(
            allocator.boxed(Delete { children: nodes(allocator, value.get("children"))?, span }),
        ),
        "footnoteReference" => Node::FootnoteReference(allocator.boxed(FootnoteReference {
            identifier: required_str(allocator, value, "identifier")?,
            label: optional_str(allocator, value, "label"),
            span,
        })),
        "definition" => Node::Definition(allocator.boxed(Definition {
            identifier: required_str(allocator, value, "identifier")?,
            label: optional_str(allocator, value, "label"),
            url: required_str(allocator, value, "url")?,
            title: optional_str(allocator, value, "title"),
            span,
        })),
        "footnoteDefinition" => Node::FootnoteDefinition(allocator.boxed(FootnoteDefinition {
            identifier: required_str(allocator, value, "identifier")?,
            label: optional_str(allocator, value, "label"),
            children: nodes(allocator, value.get("children"))?,
            span,
        })),
        "mdxJsxFlowElement" => Node::MdxJsxFlowElement(allocator.boxed(MdxJsxFlowElement {
            name: optional_str(allocator, value, "name"),
            attributes: mdx_attributes(allocator, value.get("attributes"))?,
            children: nodes(allocator, value.get("children"))?,
            self_closing: flag(value, "selfClosing"),
            span,
        })),
        "mdxJsxTextElement" => Node::MdxJsxTextElement(allocator.boxed(MdxJsxTextElement {
            name: optional_str(allocator, value, "name"),
            attributes: mdx_attributes(allocator, value.get("attributes"))?,
            children: nodes(allocator, value.get("children"))?,
            self_closing: flag(value, "selfClosing"),
            span,
        })),
        "mdxjsEsm" => {
            Node::MdxjsEsm(MdxjsEsm { value: required_str(allocator, value, "value")?, span })
        }
        "mdxFlowExpression" => Node::MdxFlowExpression(MdxFlowExpression {
            value: required_str(allocator, value, "value")?,
            span,
        }),
        "mdxTextExpression" => Node::MdxTextExpression(MdxTextExpression {
            value: required_str(allocator, value, "value")?,
            span,
        }),
        other => return Err(MdastJsonError::new(format!("unknown mdast node type {other:?}"))),
    })
}

fn list_items<'a>(
    allocator: &'a Allocator,
    value: Option<&Value>,
) -> Result<ArenaVec<'a, ListItem<'a>>, MdastJsonError> {
    let items = children_array(value)?;
    let mut out = allocator.new_vec_with_capacity(items.len());
    for item in items {
        out.push(list_item(allocator, item)?);
    }
    Ok(out)
}

fn list_item<'a>(allocator: &'a Allocator, value: &Value) -> Result<ListItem<'a>, MdastJsonError> {
    Ok(ListItem {
        spread: flag(value, "spread"),
        checked: value.get("checked").and_then(Value::as_bool),
        children: nodes(allocator, value.get("children"))?,
        span: Span::new(0, 0),
    })
}

fn table_rows<'a>(
    allocator: &'a Allocator,
    value: Option<&Value>,
) -> Result<ArenaVec<'a, TableRow<'a>>, MdastJsonError> {
    let rows = children_array(value)?;
    let mut out = allocator.new_vec_with_capacity(rows.len());
    for row in rows {
        let cells = children_array(row.get("children"))?;
        let mut children = allocator.new_vec_with_capacity(cells.len());
        for cell in cells {
            children.push(TableCell {
                children: nodes(allocator, cell.get("children"))?,
                span: Span::new(0, 0),
            });
        }
        out.push(TableRow { children, span: Span::new(0, 0) });
    }
    Ok(out)
}

fn align<'a>(allocator: &'a Allocator, value: Option<&Value>) -> ArenaVec<'a, AlignKind> {
    let items = value.and_then(Value::as_array).map(Vec::as_slice).unwrap_or_default();
    let mut out = allocator.new_vec_with_capacity(items.len());
    for item in items {
        out.push(match item.as_str() {
            Some("left") => AlignKind::Left,
            Some("center") => AlignKind::Center,
            Some("right") => AlignKind::Right,
            _ => AlignKind::None,
        });
    }
    out
}

fn children_array(value: Option<&Value>) -> Result<&[Value], MdastJsonError> {
    match value {
        None | Some(Value::Null) => Ok(&[]),
        Some(Value::Array(items)) => Ok(items),
        Some(_) => Err(MdastJsonError::new("mdast \"children\" is not an array")),
    }
}

fn depth(value: &Value) -> Result<u8, MdastJsonError> {
    let depth = value
        .get("depth")
        .and_then(Value::as_u64)
        .ok_or_else(|| MdastJsonError::new("mdast heading has no \"depth\""))?;
    u8::try_from(depth)
        .map(|depth| depth.clamp(1, 6))
        .map_err(|_| MdastJsonError::new(format!("mdast heading depth {depth} is out of range")))
}

fn flag(value: &Value, key: &str) -> bool {
    value.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn optional_str<'a>(allocator: &'a Allocator, value: &Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str).map(|text| allocator.alloc_str(text))
}

fn required_str<'a>(
    allocator: &'a Allocator,
    value: &Value,
    key: &str,
) -> Result<&'a str, MdastJsonError> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(|text| allocator.alloc_str(text))
        .ok_or_else(|| MdastJsonError::new(format!("mdast node has no {key:?} string")))
}

#[cfg(test)]
mod tests;
