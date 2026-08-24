//! Render MDX JSX elements as island placeholders with serialized props.
//!
//! Named PascalCase / member-name tags become `data-ox-island` wrappers.
//! Markdown children render as HTML *inside* that wrapper. Fragments render
//! children only. Document-level `{expression}` and ESM stay silent. Pages
//! without named components emit no `<script>`. Raw HTML under a named
//! island is tagfiltered so `<script>` in children cannot execute.

use ox_content_ast::{MdxJsxAttributeEntry, MdxJsxFlowElement, MdxJsxTextElement, Node};

use super::super::escape::write_escaped_into;
use super::super::mdx_payload::{collect_island_payload, stringify_xss_safe};
use super::HtmlRenderer;

impl HtmlRenderer {
    pub(in crate::html::renderer) fn render_mdx_jsx_flow_element(
        &mut self,
        node: &MdxJsxFlowElement<'_>,
    ) {
        self.render_mdx_jsx(node.name, &node.attributes, &node.children, "div", true);
    }

    pub(in crate::html::renderer) fn render_mdx_jsx_text_element(
        &mut self,
        node: &MdxJsxTextElement<'_>,
    ) {
        self.render_mdx_jsx(node.name, &node.attributes, &node.children, "span", false);
    }

    fn render_mdx_jsx(
        &mut self,
        name: Option<&str>,
        attributes: &[MdxJsxAttributeEntry<'_>],
        children: &[Node<'_>],
        tag: &str,
        block: bool,
    ) {
        let Some(name) = name else {
            self.render_mdx_children(children, block);
            return;
        };

        let previous_child_html = self.in_mdx_island_children;
        self.in_mdx_island_children = false;

        self.output.push('<');
        self.output.push_str(tag);
        self.output.push_str(" class=\"ox-island\" data-ox-island=\"");
        write_escaped_into(&mut self.output, name);
        self.output.push('"');

        let payload = collect_island_payload(attributes);
        let json = (!payload.is_empty()).then(|| stringify_xss_safe(&payload.into_json_value()));
        if let Some(json) = json.as_deref() {
            self.output.push_str(" data-ox-props=\"");
            write_escaped_into(&mut self.output, json);
            self.output.push('"');
        }

        self.output.push('>');
        if let Some(json) = json.as_deref() {
            self.output.push_str("<script type=\"application/json\">");
            self.output.push_str(json);
            self.output.push_str("</script>");
        }
        self.in_mdx_island_children = true;
        self.render_mdx_children(children, block);
        self.output.push_str("</");
        self.output.push_str(tag);
        self.output.push('>');
        self.in_mdx_island_children = previous_child_html;
        if block {
            self.output.push('\n');
        }
    }

    fn render_mdx_children(&mut self, children: &[Node<'_>], block: bool) {
        for child in children {
            if block {
                self.render_node(child);
            } else {
                self.visit_inline_node(child);
            }
        }
    }
}
