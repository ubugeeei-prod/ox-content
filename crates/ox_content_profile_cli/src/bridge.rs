use ox_content_docs::{
    ApiDocEntry, ApiDocMember, ApiDocModule, ApiDocTag, ApiParamDoc, ApiReturnDoc, ApiThrowsDoc,
    ApiTypeParamDoc, ExtractedDocModule, NormalizedDocEntry, NormalizedMember, NormalizedParamDoc,
    NormalizedReturnDoc, NormalizedThrowsDoc, NormalizedTypeParam,
};

// Bridge the normalized extraction output into the `ApiDocModule` render IR.
// `generate_markdown` consumes the IR that the NAPI layer reconstructs in JS
// between the `extractDocsFrom*` and `generateDocsMarkdown` calls; this mirrors
// that conversion so the full pipeline can be profiled in-process. The mapping
// tracks `ox_content_napi`'s `convert_markdown_*` helpers.

pub fn to_api_module(module: ExtractedDocModule) -> ApiDocModule {
    ApiDocModule {
        file: module.file,
        description: String::new(),
        source_path: String::new(),
        examples: Vec::new(),
        tags: Vec::new(),
        entries: module.entries.into_iter().map(to_api_entry).collect(),
    }
}

fn to_api_entry(entry: NormalizedDocEntry) -> ApiDocEntry {
    ApiDocEntry {
        name: entry.name,
        kind: entry.kind.as_str().to_string(),
        description: entry.description,
        params: entry.params.into_iter().map(to_api_param).collect(),
        returns: entry.returns.map(to_api_return),
        throws: entry.throws.into_iter().map(to_api_throws).collect(),
        examples: entry.examples,
        tags: entry.tags.into_iter().map(|(tag, value)| ApiDocTag { tag, value }).collect(),
        private: entry.private,
        file: entry.file,
        line: entry.line,
        end_line: entry.end_line,
        signature: entry.signature,
        extends: entry.extends,
        implements: entry.implements,
        has_body: entry.has_body,
        members: entry.members.into_iter().map(to_api_member).collect(),
        type_parameters: entry.type_parameters.into_iter().map(to_api_type_param).collect(),
    }
}

fn to_api_member(member: NormalizedMember) -> ApiDocMember {
    ApiDocMember {
        name: member.name,
        kind: member.kind.as_str().to_string(),
        description: member.description,
        signature: member.signature,
        type_annotation: member.type_annotation,
        default_value: member.default_value,
        params: member.params.into_iter().map(to_api_param).collect(),
        type_parameters: member.type_parameters.into_iter().map(to_api_type_param).collect(),
        returns: member.returns.map(to_api_return),
        throws: member.throws.into_iter().map(to_api_throws).collect(),
        members: member.members.into_iter().map(to_api_member).collect(),
        optional: member.optional,
        readonly: member.readonly,
        r#static: member.r#static,
        private: member.private,
        tags: member.tags.into_iter().map(|(tag, value)| ApiDocTag { tag, value }).collect(),
        implementation_of: Vec::new(),
        line: member.line,
        end_line: member.end_line,
    }
}

fn to_api_param(param: NormalizedParamDoc) -> ApiParamDoc {
    ApiParamDoc {
        name: param.name,
        type_annotation: param.type_annotation,
        description: param.description,
        optional: param.optional,
        default_value: param.default_value,
    }
}

fn to_api_return(return_doc: NormalizedReturnDoc) -> ApiReturnDoc {
    ApiReturnDoc {
        type_annotation: return_doc.type_annotation,
        description: return_doc.description,
        members: return_doc.members.into_iter().map(to_api_member).collect(),
    }
}

fn to_api_throws(throws: NormalizedThrowsDoc) -> ApiThrowsDoc {
    ApiThrowsDoc { type_annotation: throws.type_annotation, description: throws.description }
}

fn to_api_type_param(type_param: NormalizedTypeParam) -> ApiTypeParamDoc {
    ApiTypeParamDoc {
        name: type_param.name,
        constraint: type_param.constraint,
        default: type_param.default,
        description: type_param.description,
    }
}
