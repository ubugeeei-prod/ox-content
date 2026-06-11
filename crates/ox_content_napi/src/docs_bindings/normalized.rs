use ox_content_docs::{
    NormalizedDocEntry, NormalizedMember, NormalizedParamDoc, NormalizedReturnDoc,
    NormalizedThrowsDoc, NormalizedTypeParam,
};

use crate::{JsDocEntry, JsDocMember, JsDocParam, JsDocReturn, JsDocThrows, JsTypeParam};

fn map_normalized_param_doc(param: NormalizedParamDoc) -> JsDocParam {
    JsDocParam {
        name: param.name,
        r#type: param.type_annotation,
        description: param.description,
        optional: param.optional.then_some(true),
        r#default: param.default_value,
    }
}

fn map_normalized_return_doc(return_doc: NormalizedReturnDoc) -> JsDocReturn {
    JsDocReturn {
        r#type: return_doc.type_annotation,
        description: return_doc.description,
        members: (!return_doc.members.is_empty())
            .then(|| return_doc.members.into_iter().map(map_normalized_member).collect()),
    }
}

fn map_normalized_throws_doc(throws_doc: NormalizedThrowsDoc) -> JsDocThrows {
    JsDocThrows { r#type: throws_doc.type_annotation, description: throws_doc.description }
}

fn map_normalized_member(member: NormalizedMember) -> JsDocMember {
    JsDocMember {
        name: member.name,
        kind: member.kind.as_str().to_string(),
        description: member.description,
        signature: member.signature,
        r#type: member.type_annotation,
        r#default: member.default_value,
        params: (!member.params.is_empty())
            .then(|| member.params.into_iter().map(map_normalized_param_doc).collect()),
        type_parameters: (!member.type_parameters.is_empty())
            .then(|| member.type_parameters.into_iter().map(map_normalized_type_param).collect()),
        returns: member.returns.map(map_normalized_return_doc),
        throws: (!member.throws.is_empty())
            .then(|| member.throws.into_iter().map(map_normalized_throws_doc).collect()),
        members: (!member.members.is_empty())
            .then(|| member.members.into_iter().map(map_normalized_member).collect()),
        optional: member.optional.then_some(true),
        readonly: member.readonly.then_some(true),
        r#static: member.r#static.then_some(true),
        private: member.private.then_some(true),
        tags: (!member.tags.is_empty()).then(|| member.tags.into_iter().collect()),
        implementation_of: None,
        line: member.line,
        end_line: member.end_line,
    }
}

pub fn map_normalized_doc_entry(entry: NormalizedDocEntry) -> JsDocEntry {
    JsDocEntry {
        name: entry.name,
        kind: entry.kind.as_str().to_string(),
        description: entry.description,
        params: (!entry.params.is_empty())
            .then(|| entry.params.into_iter().map(map_normalized_param_doc).collect()),
        returns: entry.returns.map(map_normalized_return_doc),
        throws: (!entry.throws.is_empty())
            .then(|| entry.throws.into_iter().map(map_normalized_throws_doc).collect()),
        examples: (!entry.examples.is_empty()).then_some(entry.examples),
        tags: (!entry.tags.is_empty()).then(|| entry.tags.into_iter().collect()),
        private: entry.private,
        file: entry.file,
        line: entry.line,
        end_line: entry.end_line,
        signature: entry.signature,
        extends: (!entry.extends.is_empty()).then_some(entry.extends),
        implements: (!entry.implements.is_empty()).then_some(entry.implements),
        has_body: entry.has_body,
        members: (!entry.members.is_empty())
            .then(|| entry.members.into_iter().map(map_normalized_member).collect()),
        type_parameters: (!entry.type_parameters.is_empty())
            .then(|| entry.type_parameters.into_iter().map(map_normalized_type_param).collect()),
    }
}

fn map_normalized_type_param(type_param: NormalizedTypeParam) -> JsTypeParam {
    JsTypeParam {
        name: type_param.name,
        constraint: type_param.constraint,
        r#default: type_param.default,
        description: type_param.description,
    }
}
