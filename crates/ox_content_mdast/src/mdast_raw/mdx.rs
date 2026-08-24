use ox_content_ast::{
    MdxFlowExpression, MdxJsxFlowElement, MdxJsxTextElement, MdxTextExpression, MdxjsEsm,
};

use super::format::{
    FLAG_MDX_SELF_CLOSING, KIND_MDX_ESM, KIND_MDX_FLOW_EXPRESSION, KIND_MDX_JSX_FLOW,
    KIND_MDX_JSX_TEXT, KIND_MDX_TEXT_EXPRESSION, RawNodeRecord,
};
use super::serializer::MdastRawSerializer;

impl MdastRawSerializer {
    pub(super) fn write_mdx_jsx_flow_element(
        &mut self,
        node: &MdxJsxFlowElement<'_>,
    ) -> crate::transfer::Result<u32> {
        let mut record = RawNodeRecord::new(KIND_MDX_JSX_FLOW, node.span);
        self.write_child_nodes(&mut record, &node.children)?;
        self.write_string_into_slot(&mut record, 0, node.name)?;
        let attributes = crate::mdast::mdx_attributes_to_json(&node.attributes);
        self.write_string_into_slot(&mut record, 1, Some(&attributes))?;
        if node.self_closing {
            record.flags |= FLAG_MDX_SELF_CLOSING;
        }
        self.push_record(record)
    }

    pub(super) fn write_mdx_jsx_text_element(
        &mut self,
        node: &MdxJsxTextElement<'_>,
    ) -> crate::transfer::Result<u32> {
        let mut record = RawNodeRecord::new(KIND_MDX_JSX_TEXT, node.span);
        self.write_child_nodes(&mut record, &node.children)?;
        self.write_string_into_slot(&mut record, 0, node.name)?;
        let attributes = crate::mdast::mdx_attributes_to_json(&node.attributes);
        self.write_string_into_slot(&mut record, 1, Some(&attributes))?;
        if node.self_closing {
            record.flags |= FLAG_MDX_SELF_CLOSING;
        }
        self.push_record(record)
    }

    pub(super) fn write_mdxjs_esm(&mut self, node: &MdxjsEsm<'_>) -> crate::transfer::Result<u32> {
        let mut record = RawNodeRecord::new(KIND_MDX_ESM, node.span);
        self.write_string_into_slot(&mut record, 0, Some(node.value))?;
        self.push_record(record)
    }

    pub(super) fn write_mdx_flow_expression(
        &mut self,
        node: &MdxFlowExpression<'_>,
    ) -> crate::transfer::Result<u32> {
        let mut record = RawNodeRecord::new(KIND_MDX_FLOW_EXPRESSION, node.span);
        self.write_string_into_slot(&mut record, 0, Some(node.value))?;
        self.push_record(record)
    }

    pub(super) fn write_mdx_text_expression(
        &mut self,
        node: &MdxTextExpression<'_>,
    ) -> crate::transfer::Result<u32> {
        let mut record = RawNodeRecord::new(KIND_MDX_TEXT_EXPRESSION, node.span);
        self.write_string_into_slot(&mut record, 0, Some(node.value))?;
        self.push_record(record)
    }
}
