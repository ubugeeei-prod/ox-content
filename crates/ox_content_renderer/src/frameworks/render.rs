use super::{
    parser::{HtmlElement, HtmlNode},
    react, shared, vue, FrameworkCodegenTarget, FrameworkComponentIsland,
};
use smallvec::SmallVec;

type RenderedChildren = SmallVec<[String; 8]>;

pub(super) struct FrameworkCodegen<'a> {
    pub target: FrameworkCodegenTarget,
    pub islands: &'a [FrameworkComponentIsland],
}

impl FrameworkCodegen<'_> {
    pub(super) fn render_root(&self, nodes: &[HtmlNode]) -> String {
        let children = self.render_children(nodes);

        match self.target {
            FrameworkCodegenTarget::React => react::render_root(&children),
            FrameworkCodegenTarget::Vue => vue::render_root(&children),
        }
    }

    fn render_children(&self, nodes: &[HtmlNode]) -> RenderedChildren {
        let mut children = RenderedChildren::new();
        for node in nodes {
            if let Some(child) = self.render_node(node) {
                children.push(child);
            }
        }
        children
    }

    fn render_node(&self, node: &HtmlNode) -> Option<String> {
        match node {
            HtmlNode::Text(value) if value.is_empty() => None,
            HtmlNode::Text(value) => Some(shared::js_string_literal(value)),
            HtmlNode::Element(element) => Some(self.render_element(element)),
        }
    }

    fn render_element(&self, element: &HtmlElement) -> String {
        if let Some(island) = shared::find_island(element, self.islands) {
            return self.render_island(island);
        }

        let children = self.render_children(&element.children);

        match self.target {
            FrameworkCodegenTarget::React => react::render_element(element, &children),
            FrameworkCodegenTarget::Vue => vue::render_element(element, &children),
        }
    }

    fn render_island(&self, island: &FrameworkComponentIsland) -> String {
        match self.target {
            FrameworkCodegenTarget::React => react::render_island(island),
            FrameworkCodegenTarget::Vue => vue::render_island(island),
        }
    }
}
