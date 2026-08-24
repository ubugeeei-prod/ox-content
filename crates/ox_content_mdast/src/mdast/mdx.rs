use ox_content_allocator::Vec as ArenaVec;
use ox_content_ast::{
    MdxFlowExpression, MdxJsxAttributeEntry, MdxJsxAttributeValue, MdxJsxFlowElement,
    MdxJsxTextElement, MdxTextExpression, MdxjsEsm, Node,
};

use super::MdastJsonSerializer;

impl MdastJsonSerializer {
    pub(super) fn write_mdx_jsx_flow_element(&mut self, node: &MdxJsxFlowElement<'_>) {
        self.write_mdx_jsx("mdxJsxFlowElement", node.name, &node.attributes, &node.children);
    }

    pub(super) fn write_mdx_jsx_text_element(&mut self, node: &MdxJsxTextElement<'_>) {
        self.write_mdx_jsx("mdxJsxTextElement", node.name, &node.attributes, &node.children);
    }

    fn write_mdx_jsx<'a>(
        &mut self,
        kind: &str,
        name: Option<&str>,
        attributes: &ArenaVec<'a, MdxJsxAttributeEntry<'a>>,
        children: &ArenaVec<'a, Node<'a>>,
    ) {
        self.output.push_str("{\"type\":\"");
        self.output.push_str(kind);
        self.output.push('"');
        if let Some(name) = name {
            self.output.push_str(",\"name\":");
            self.write_string(name);
        } else {
            self.output.push_str(",\"name\":null");
        }
        self.output.push_str(",\"attributes\":");
        self.write_mdx_attributes(attributes);
        self.output.push_str(",\"children\":");
        self.write_nodes(children);
        self.output.push('}');
    }

    fn write_mdx_attributes<'a>(&mut self, attributes: &ArenaVec<'a, MdxJsxAttributeEntry<'a>>) {
        self.output.push('[');
        for (idx, entry) in attributes.iter().enumerate() {
            if idx > 0 {
                self.output.push(',');
            }
            match entry {
                MdxJsxAttributeEntry::Attribute(attribute) => {
                    self.output.push_str("{\"type\":\"mdxJsxAttribute\",\"name\":");
                    self.write_string(attribute.name);
                    self.output.push_str(",\"value\":");
                    match &attribute.value {
                        None => self.output.push_str("null"),
                        Some(MdxJsxAttributeValue::Literal(value)) => self.write_string(value),
                        Some(MdxJsxAttributeValue::Expression(expr)) => {
                            self.output.push_str(
                                "{\"type\":\"mdxJsxAttributeValueExpression\",\"value\":",
                            );
                            self.write_string(expr.value);
                            self.output.push('}');
                        }
                    }
                    self.output.push('}');
                }
                MdxJsxAttributeEntry::Expression(expr) => {
                    self.output.push_str("{\"type\":\"mdxJsxExpressionAttribute\",\"value\":");
                    self.write_string(expr.value);
                    self.output.push('}');
                }
            }
        }
        self.output.push(']');
    }

    pub(super) fn write_mdxjs_esm(&mut self, node: &MdxjsEsm<'_>) {
        self.output.push_str("{\"type\":\"mdxjsEsm\",\"value\":");
        self.write_string(node.value);
        self.output.push('}');
    }

    pub(super) fn write_mdx_flow_expression(&mut self, node: &MdxFlowExpression<'_>) {
        self.output.push_str("{\"type\":\"mdxFlowExpression\",\"value\":");
        self.write_string(node.value);
        self.output.push('}');
    }

    pub(super) fn write_mdx_text_expression(&mut self, node: &MdxTextExpression<'_>) {
        self.output.push_str("{\"type\":\"mdxTextExpression\",\"value\":");
        self.write_string(node.value);
        self.output.push('}');
    }
}
