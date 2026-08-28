//! MDX JSX attributes, on the way back from JSON.
//!
//! Attributes are the one part of the tree with a shape of their own — a
//! literal, an expression, or a whole spread — so they get their own module,
//! mirroring `mdast/mdx.rs` on the way out.

use ox_content_allocator::{Allocator, Vec as ArenaVec};
use ox_content_ast::{
    MdxJsxAttribute, MdxJsxAttributeEntry, MdxJsxAttributeValue, MdxJsxAttributeValueExpression,
    MdxJsxExpressionAttribute, Span,
};
use serde_json::Value;

use super::{MdastJsonError, children_array, required_str};

pub(super) fn mdx_attributes<'a>(
    allocator: &'a Allocator,
    value: Option<&Value>,
) -> Result<ArenaVec<'a, MdxJsxAttributeEntry<'a>>, MdastJsonError> {
    let items = children_array(value)?;
    let mut out = allocator.new_vec_with_capacity(items.len());
    for item in items {
        out.push(mdx_attribute(allocator, item)?);
    }
    Ok(out)
}

fn mdx_attribute<'a>(
    allocator: &'a Allocator,
    value: &Value,
) -> Result<MdxJsxAttributeEntry<'a>, MdastJsonError> {
    let span = Span::new(0, 0);
    match value.get("type").and_then(Value::as_str) {
        Some("mdxJsxExpressionAttribute") => {
            Ok(MdxJsxAttributeEntry::Expression(MdxJsxExpressionAttribute {
                value: required_str(allocator, value, "value")?,
                span,
            }))
        }
        Some("mdxJsxAttribute") => Ok(MdxJsxAttributeEntry::Attribute(MdxJsxAttribute {
            name: required_str(allocator, value, "name")?,
            value: mdx_attribute_value(allocator, value.get("value"))?,
            span,
        })),
        Some(other) => {
            Err(MdastJsonError::new(format!("unknown mdast JSX attribute type {other:?}")))
        }
        None => Err(MdastJsonError::new("mdast JSX attribute has no \"type\"")),
    }
}

fn mdx_attribute_value<'a>(
    allocator: &'a Allocator,
    value: Option<&Value>,
) -> Result<Option<MdxJsxAttributeValue<'a>>, MdastJsonError> {
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(literal)) => {
            Ok(Some(MdxJsxAttributeValue::Literal(allocator.alloc_str(literal))))
        }
        Some(object) => {
            Ok(Some(MdxJsxAttributeValue::Expression(MdxJsxAttributeValueExpression {
                value: required_str(allocator, object, "value")?,
                span: Span::new(0, 0),
            })))
        }
    }
}
