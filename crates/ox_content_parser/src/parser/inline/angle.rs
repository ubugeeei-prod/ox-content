//! What a `<` in inline content opens: an autolink, a JSX tag, raw HTML, or
//! nothing.

use ox_content_allocator::Vec;
use ox_content_ast::Node;

use crate::error::ParseResult;
use crate::parser::Parser;

impl<'a> Parser<'a> {
    pub(super) fn parse_inline_html_or_text(
        &self,
        content: &'a str,
        offset: usize,
        children: &mut Vec<'a, Node<'a>>,
        pos: &mut usize,
    ) -> ParseResult<()> {
        // An autolink, a JSX tag and an inline HTML tag all have to close
        // with `>`. Each of the three parsers below only reports that there
        // is none by scanning to the end of the content, so a line holding
        // `<` with no `>` after it paid three walks per `<` — quadratic over
        // a run of them, and `a < b` is ordinary prose.
        if !self.has_closer_from(content, *pos + 1, b'>') {
            Self::push_text(children, "<", offset + *pos, offset + *pos + 1);
            *pos += 1;
            return Ok(());
        }

        if let Some((link, end)) = self.parse_autolink(content, *pos, offset) {
            children.push(link);
            *pos = end;
        } else if let Some((node, end)) = self.try_parse_mdx_jsx_text(content, *pos, offset)? {
            children.push(node);
            *pos = end;
        } else if let Some((html, end)) = Self::parse_inline_html(content, *pos, offset) {
            children.push(Node::Html(html));
            *pos = end;
        } else {
            Self::push_text(children, "<", offset + *pos, offset + *pos + 1);
            *pos += 1;
        }
        Ok(())
    }
}
