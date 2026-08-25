//! Emit static file-tree markup.

use super::super::escape_html_text;
use super::ResolvedFileTreeOptions;
use super::icons;
use super::parse::{TreeNode, parse_tree};

pub(super) fn emit_tree(body: &str, options: &ResolvedFileTreeOptions, out: &mut String) {
    out.push_str("<div class=\"ox-file-tree\">\n");
    emit_list(&parse_tree(body), options, out);
    out.push_str("</div>\n");
}

fn emit_list(nodes: &[TreeNode], options: &ResolvedFileTreeOptions, out: &mut String) {
    out.push_str("<ul>\n");
    for node in nodes {
        emit_node(node, options, out);
    }
    out.push_str("</ul>\n");
}

fn emit_node(node: &TreeNode, options: &ResolvedFileTreeOptions, out: &mut String) {
    out.push_str("<li class=\"");
    out.push_str(if node.is_dir { "ox-file-tree__dir" } else { "ox-file-tree__file" });
    if node.highlight {
        out.push_str(" ox-file-tree__highlight");
    }
    out.push_str("\">");

    if node.children.is_empty() {
        out.push_str("<span class=\"ox-file-tree__row\">");
        twisty(out, true);
        emit_entry_icon(node, options, out);
        emit_name(&node.name, out);
        out.push_str("</span>");
    } else {
        out.push_str("<details");
        if options.default_open {
            out.push_str(" open");
        }
        out.push_str("><summary>");
        twisty(out, false);
        emit_entry_icon(node, options, out);
        emit_name(&node.name, out);
        out.push_str("</summary>");
        emit_list(&node.children, options, out);
        out.push_str("</details>");
    }
    out.push_str("</li>\n");
}

fn emit_entry_icon(node: &TreeNode, options: &ResolvedFileTreeOptions, out: &mut String) {
    if node.is_dir {
        icons::emit_dir_icons(out, options, !node.children.is_empty());
    } else {
        icons::emit_file_icon(out, options, &node.name);
    }
}

fn emit_name(name: &str, out: &mut String) {
    out.push_str("<span class=\"ox-file-tree__name\">");
    escape_html_text(name, out);
    out.push_str("</span>");
}

fn twisty(out: &mut String, spacer: bool) {
    out.push_str("<span class=\"ox-file-tree__twisty");
    if spacer {
        out.push_str(" ox-file-tree__twisty--spacer");
    }
    out.push_str("\" aria-hidden=\"true\"></span>");
}
