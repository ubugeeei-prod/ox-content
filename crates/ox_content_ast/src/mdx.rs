//! MDX AST nodes, aligned with `mdast-util-mdx`.
//!
//! These types are arena-allocated and borrow from the source. Expression
//! values store source text only; estree evaluation is out of scope here.

use ox_content_allocator::Vec;

use crate::{Node, Span};

/// MDX JSX attribute value: a quoted literal or a `{expression}`.
#[derive(Debug)]
pub enum MdxJsxAttributeValue<'a> {
    /// Quoted attribute value: `title="hello"`.
    Literal(&'a str),
    /// Expression attribute value: `title={foo}`.
    Expression(MdxJsxAttributeValueExpression<'a>),
}

/// `{expression}` used as a JSX attribute value.
#[derive(Debug)]
pub struct MdxJsxAttributeValueExpression<'a> {
    /// Expression source, without the surrounding braces.
    pub value: &'a str,
    /// Source span covering the braces.
    pub span: Span,
}

/// Named JSX attribute: `title="hello"`, `disabled`, or `title={foo}`.
#[derive(Debug)]
pub struct MdxJsxAttribute<'a> {
    /// Attribute name.
    pub name: &'a str,
    /// Attribute value. `None` is a boolean attribute.
    pub value: Option<MdxJsxAttributeValue<'a>>,
    /// Source span.
    pub span: Span,
}

/// Spread or other expression attribute: `{...props}`.
#[derive(Debug)]
pub struct MdxJsxExpressionAttribute<'a> {
    /// Expression source, without the surrounding braces.
    pub value: &'a str,
    /// Source span covering the braces.
    pub span: Span,
}

/// One attribute on an MDX JSX element.
#[derive(Debug)]
pub enum MdxJsxAttributeEntry<'a> {
    /// Named attribute.
    Attribute(MdxJsxAttribute<'a>),
    /// Expression / spread attribute.
    Expression(MdxJsxExpressionAttribute<'a>),
}

/// Block JSX element: `<Alert />` or `<Alert>children</Alert>`.
///
/// `name` is `None` for fragments (`<>...</>`).
#[derive(Debug)]
pub struct MdxJsxFlowElement<'a> {
    /// Tag name, or `None` for a fragment.
    pub name: Option<&'a str>,
    /// Attributes.
    pub attributes: Vec<'a, MdxJsxAttributeEntry<'a>>,
    /// Block or phrasing children.
    pub children: Vec<'a, Node<'a>>,
    /// Whether the tag was self-closing.
    pub self_closing: bool,
    /// Source span.
    pub span: Span,
}

/// Inline JSX element: `Hello <Badge />`.
///
/// `name` is `None` for fragments (`<>...</>`).
#[derive(Debug)]
pub struct MdxJsxTextElement<'a> {
    /// Tag name, or `None` for a fragment.
    pub name: Option<&'a str>,
    /// Attributes.
    pub attributes: Vec<'a, MdxJsxAttributeEntry<'a>>,
    /// Phrasing children.
    pub children: Vec<'a, Node<'a>>,
    /// Whether the tag was self-closing.
    pub self_closing: bool,
    /// Source span.
    pub span: Span,
}

/// Module-level `import` / `export` (`mdxjsEsm`).
#[derive(Debug)]
pub struct MdxjsEsm<'a> {
    /// Raw ESM source.
    pub value: &'a str,
    /// Source span.
    pub span: Span,
}

/// Block `{expression}` (`mdxFlowExpression`).
#[derive(Debug)]
pub struct MdxFlowExpression<'a> {
    /// Expression source, without the surrounding braces.
    pub value: &'a str,
    /// Source span covering the braces.
    pub span: Span,
}

/// Inline `{expression}` (`mdxTextExpression`).
#[derive(Debug)]
pub struct MdxTextExpression<'a> {
    /// Expression source, without the surrounding braces.
    pub value: &'a str,
    /// Source span covering the braces.
    pub span: Span,
}
