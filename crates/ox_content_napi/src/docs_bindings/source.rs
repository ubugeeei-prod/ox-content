use ox_content_docs::{DocItem, DocItemKind, DocTag, ParamDoc};

use crate::{JsSourceDocItem, JsSourceDocParam, JsSourceDocTag};

fn doc_item_kind_to_string(kind: DocItemKind) -> String {
    match kind {
        DocItemKind::Module => "module",
        DocItemKind::Function => "function",
        DocItemKind::Class => "class",
        DocItemKind::Interface => "interface",
        DocItemKind::Type => "type",
        DocItemKind::Enum => "enum",
        DocItemKind::Variable => "variable",
        DocItemKind::Method => "method",
        DocItemKind::Property => "property",
        DocItemKind::Constructor => "constructor",
        DocItemKind::Getter => "getter",
        DocItemKind::Setter => "setter",
        DocItemKind::EnumMember => "enumMember",
        DocItemKind::IndexSignature => "indexSignature",
    }
    .to_string()
}

fn map_doc_tag(tag: DocTag) -> JsSourceDocTag {
    JsSourceDocTag { tag: tag.tag, value: tag.value }
}

fn map_param_doc(param: ParamDoc) -> JsSourceDocParam {
    JsSourceDocParam {
        name: param.name,
        type_annotation: param.type_annotation,
        optional: param.optional,
        default_value: param.default_value,
        description: param.description,
    }
}

pub(super) fn map_doc_item(item: DocItem) -> JsSourceDocItem {
    let return_members = (!item.return_members.is_empty())
        .then(|| item.return_members.into_iter().map(map_doc_item).collect());
    let members =
        (!item.children.is_empty()).then(|| item.children.into_iter().map(map_doc_item).collect());

    JsSourceDocItem {
        name: item.name,
        kind: doc_item_kind_to_string(item.kind),
        doc: item.doc,
        jsdoc: item.jsdoc,
        source_path: item.source_path,
        line: item.line,
        end_line: item.end_line,
        exported: item.exported,
        signature: item.signature,
        extends: (!item.extends.is_empty()).then_some(item.extends),
        implements: (!item.implements.is_empty()).then_some(item.implements),
        params: item.params.into_iter().map(map_param_doc).collect(),
        return_type: item.return_type,
        return_members,
        members,
        tags: item.tags.into_iter().map(map_doc_tag).collect(),
    }
}
