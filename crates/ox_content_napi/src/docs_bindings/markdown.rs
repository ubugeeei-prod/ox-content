use ox_content_docs::{
    ApiDocEntry, ApiDocMember, ApiDocModule, ApiDocTag, ApiParamDoc, ApiReturnDoc, ApiThrowsDoc,
    ApiTypeParamDoc,
};

use crate::{
    JsDocMember, JsDocParam, JsDocReturn, JsDocThrows, JsDocsMarkdownEntry, JsDocsMarkdownModule,
    JsDocsMarkdownTag, JsTypeParam,
};

fn convert_markdown_param(param: JsDocParam) -> ApiParamDoc {
    ApiParamDoc {
        name: param.name,
        type_annotation: param.r#type,
        description: param.description,
        optional: param.optional.unwrap_or(false),
        default_value: param.r#default,
    }
}

fn convert_markdown_return(return_doc: JsDocReturn) -> ApiReturnDoc {
    ApiReturnDoc {
        type_annotation: return_doc.r#type,
        description: return_doc.description,
        members: return_doc
            .members
            .unwrap_or_default()
            .into_iter()
            .map(convert_markdown_member)
            .collect(),
    }
}

fn convert_markdown_throws(throws_doc: JsDocThrows) -> ApiThrowsDoc {
    ApiThrowsDoc { type_annotation: throws_doc.r#type, description: throws_doc.description }
}

fn convert_markdown_tag(tag: JsDocsMarkdownTag) -> ApiDocTag {
    ApiDocTag { tag: tag.tag, value: tag.value }
}

pub(super) fn map_api_doc_tag(tag: ApiDocTag) -> JsDocsMarkdownTag {
    JsDocsMarkdownTag { tag: tag.tag, value: tag.value }
}

fn convert_markdown_member(member: JsDocMember) -> ApiDocMember {
    ApiDocMember {
        name: member.name,
        kind: member.kind,
        description: member.description,
        signature: member.signature,
        type_annotation: member.r#type,
        default_value: member.r#default,
        params: member.params.unwrap_or_default().into_iter().map(convert_markdown_param).collect(),
        type_parameters: member
            .type_parameters
            .unwrap_or_default()
            .into_iter()
            .map(convert_markdown_type_param)
            .collect(),
        returns: member.returns.map(convert_markdown_return),
        throws: member
            .throws
            .unwrap_or_default()
            .into_iter()
            .map(convert_markdown_throws)
            .collect(),
        members: member
            .members
            .unwrap_or_default()
            .into_iter()
            .map(convert_markdown_member)
            .collect(),
        optional: member.optional.unwrap_or(false),
        readonly: member.readonly.unwrap_or(false),
        r#static: member.r#static.unwrap_or(false),
        private: member.private.unwrap_or(false),
        tags: member
            .tags
            .unwrap_or_default()
            .into_iter()
            .map(|(tag, value)| ApiDocTag { tag, value })
            .collect(),
        implementation_of: member.implementation_of.unwrap_or_default(),
        line: member.line,
        end_line: member.end_line,
    }
}

pub fn convert_markdown_entry(entry: JsDocsMarkdownEntry) -> ApiDocEntry {
    ApiDocEntry {
        name: entry.name,
        kind: entry.kind,
        description: entry.description,
        params: entry.params.unwrap_or_default().into_iter().map(convert_markdown_param).collect(),
        returns: entry.returns.map(convert_markdown_return),
        throws: entry.throws.unwrap_or_default().into_iter().map(convert_markdown_throws).collect(),
        examples: entry.examples.unwrap_or_default(),
        tags: entry.tags.unwrap_or_default().into_iter().map(convert_markdown_tag).collect(),
        private: entry.private,
        file: entry.file,
        line: entry.line,
        end_line: entry.end_line,
        signature: entry.signature,
        extends: entry.extends.unwrap_or_default(),
        implements: entry.implements.unwrap_or_default(),
        has_body: entry.has_body.unwrap_or(false),
        members: entry
            .members
            .unwrap_or_default()
            .into_iter()
            .map(convert_markdown_member)
            .collect(),
        type_parameters: entry
            .type_parameters
            .unwrap_or_default()
            .into_iter()
            .map(convert_markdown_type_param)
            .collect(),
    }
}

fn convert_markdown_type_param(type_param: JsTypeParam) -> ApiTypeParamDoc {
    ApiTypeParamDoc {
        name: type_param.name,
        constraint: type_param.constraint,
        default: type_param.r#default,
        description: type_param.description,
    }
}

pub(super) fn convert_markdown_module(module: JsDocsMarkdownModule) -> ApiDocModule {
    ApiDocModule {
        file: module.file,
        description: module.description.unwrap_or_default(),
        source_path: module.source_path.unwrap_or_default(),
        examples: module.examples.unwrap_or_default(),
        tags: module.tags.unwrap_or_default().into_iter().map(convert_markdown_tag).collect(),
        entries: module.entries.into_iter().map(convert_markdown_entry).collect(),
    }
}
