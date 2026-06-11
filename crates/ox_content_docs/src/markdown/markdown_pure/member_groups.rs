use super::super::{MarkdownDocsOptions, MarkdownLinkContext};
use super::index_signatures::render_index_signature_group_pure;
use super::member_tables::render_member_group_pure;
use crate::model::ApiDocEntry;

/// Renders the member tables for an entry, grouped to match the HTML renderer.
///
/// Each member group (`Properties`, `Methods`, …) is emitted as a real heading
/// at `section_level` — the same level as the entry's other sections — matching
/// TypeDoc, which renders `## Properties` directly rather than nesting member
/// tables under a separate "Members" heading.
pub(super) fn render_members_pure(
    entry: &ApiDocEntry,
    options: &MarkdownDocsOptions,
    context: Option<&MarkdownLinkContext<'_>>,
    section_level: usize,
) -> String {
    let mut out = String::new();
    let heading = "#".repeat(section_level);
    let group_context = MemberGroupRenderContext {
        entry_name: &entry.name,
        entry_kind: &entry.kind,
        options,
        link_context: context,
        parameter_section_level: section_level + 1,
    };

    match entry.kind.as_str() {
        "class" => {
            let mut constructors = Vec::new();
            let mut static_methods = Vec::new();
            let mut methods = Vec::new();
            let mut index_signatures = Vec::new();
            let mut static_properties = Vec::new();
            let mut properties = Vec::new();

            for member in &entry.members {
                match member.kind.as_str() {
                    "constructor" => constructors.push(member),
                    "method" | "getter" | "setter" if member.r#static => {
                        static_methods.push(member);
                    }
                    "method" | "getter" | "setter" => methods.push(member),
                    "indexSignature" => index_signatures.push(member),
                    "property" if member.r#static => static_properties.push(member),
                    "property" => properties.push(member),
                    _ => {}
                }
            }
            render_member_group_pure(
                &mut out,
                &heading,
                "Constructors",
                &constructors,
                &group_context,
            );
            render_member_group_pure(
                &mut out,
                &heading,
                "Static Methods",
                &static_methods,
                &group_context,
            );
            render_member_group_pure(&mut out, &heading, "Methods", &methods, &group_context);
            render_index_signature_group_pure(
                &mut out,
                &heading,
                &index_signatures,
                &group_context,
            );
            render_member_group_pure(
                &mut out,
                &heading,
                "Static Properties",
                &static_properties,
                &group_context,
            );
            render_member_group_pure(&mut out, &heading, "Properties", &properties, &group_context);
        }
        "interface" => {
            let mut properties = Vec::new();
            let mut methods = Vec::new();
            let mut index_signatures = Vec::new();

            for member in &entry.members {
                match member.kind.as_str() {
                    "indexSignature" => index_signatures.push(member),
                    "method" | "getter" | "setter" if !member.r#static => methods.push(member),
                    "property" if !member.r#static => properties.push(member),
                    _ => {}
                }
            }
            render_index_signature_group_pure(
                &mut out,
                &heading,
                &index_signatures,
                &group_context,
            );
            render_member_group_pure(&mut out, &heading, "Properties", &properties, &group_context);
            render_member_group_pure(&mut out, &heading, "Methods", &methods, &group_context);
        }
        "type" => {
            let mut properties = Vec::new();
            let mut methods = Vec::new();
            let mut index_signatures = Vec::new();
            let mut enum_members = Vec::new();

            for member in &entry.members {
                match member.kind.as_str() {
                    "indexSignature" => index_signatures.push(member),
                    "method" | "getter" | "setter" if !member.r#static => methods.push(member),
                    "property" if !member.r#static => properties.push(member),
                    "enumMember" => enum_members.push(member),
                    _ => {}
                }
            }
            render_index_signature_group_pure(
                &mut out,
                &heading,
                &index_signatures,
                &group_context,
            );
            render_member_group_pure(&mut out, &heading, "Properties", &properties, &group_context);
            render_member_group_pure(&mut out, &heading, "Methods", &methods, &group_context);
            render_member_group_pure(
                &mut out,
                &heading,
                "Enum Members",
                &enum_members,
                &group_context,
            );
        }
        "enum" => {
            let mut enum_members = Vec::new();

            for member in &entry.members {
                if member.kind == "enumMember" {
                    enum_members.push(member);
                }
            }
            render_member_group_pure(
                &mut out,
                &heading,
                "Enum Members",
                &enum_members,
                &group_context,
            );
        }
        _ => {
            let members = entry.members.iter().collect::<Vec<_>>();
            render_member_group_pure(&mut out, &heading, "Members", &members, &group_context);
        }
    }
    out
}

pub(super) struct MemberGroupRenderContext<'a, 'ctx> {
    pub(super) entry_name: &'a str,
    pub(super) entry_kind: &'a str,
    pub(super) options: &'a MarkdownDocsOptions,
    pub(super) link_context: Option<&'a MarkdownLinkContext<'ctx>>,
    pub(super) parameter_section_level: usize,
}
