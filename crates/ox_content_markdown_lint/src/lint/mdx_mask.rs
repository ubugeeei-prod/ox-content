use ox_content_allocator::Allocator;
use ox_content_ast::{
    Document, MdxFlowExpression, MdxJsxFlowElement, MdxJsxTextElement, MdxTextExpression, MdxjsEsm,
    Node, Span, Visit,
};
use ox_content_parser::{Parser, ParserOptions};

#[derive(Clone, Copy)]
struct MaskAction {
    mask: bool,
    span: Span,
}

struct MdxSyntaxMask {
    actions: Vec<MaskAction>,
}

impl MdxSyntaxMask {
    fn push(&mut self, span: Span, mask: bool) {
        if !span.is_empty() {
            self.actions.push(MaskAction { mask, span });
        }
    }

    fn mask_jsx_children<'a>(&mut self, span: Span, children: &[Node<'a>]) {
        self.push(span, true);
        for child in children {
            self.push(child.span(), false);
            self.visit_node(child);
        }
    }
}

impl<'a> Visit<'a> for MdxSyntaxMask {
    fn visit_mdx_jsx_flow_element(&mut self, node: &MdxJsxFlowElement<'a>) {
        self.mask_jsx_children(node.span, &node.children);
    }

    fn visit_mdx_jsx_text_element(&mut self, node: &MdxJsxTextElement<'a>) {
        self.mask_jsx_children(node.span, &node.children);
    }

    fn visit_mdxjs_esm(&mut self, node: &MdxjsEsm<'a>) {
        self.push(node.span, true);
    }

    fn visit_mdx_flow_expression(&mut self, node: &MdxFlowExpression<'a>) {
        self.push(node.span, true);
    }

    fn visit_mdx_text_expression(&mut self, node: &MdxTextExpression<'a>) {
        self.push(node.span, true);
    }
}

pub(super) fn mask_mdx_syntax(source: &str) -> String {
    let allocator = Allocator::for_source_len(source.len());
    let mut options = ParserOptions::gfm();
    options.mdx = true;
    let Ok(document) = Parser::with_options(&allocator, source, options).parse() else {
        return source.to_string();
    };

    apply_mask(source, collect_actions(&document))
}

fn collect_actions(document: &Document<'_>) -> Vec<MaskAction> {
    let mut visitor = MdxSyntaxMask { actions: Vec::new() };
    visitor.visit_document(document);
    visitor.actions
}

fn apply_mask(source: &str, actions: Vec<MaskAction>) -> String {
    let mut masked = vec![false; source.len()];
    for action in actions {
        let start = (action.span.start as usize).min(masked.len());
        let end = (action.span.end as usize).min(masked.len());
        masked[start..end].fill(action.mask);
    }

    source
        .char_indices()
        .map(
            |(offset, character)| {
                if character == '\n' || !masked[offset] { character } else { ' ' }
            },
        )
        .collect()
}
