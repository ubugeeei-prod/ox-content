use oxc_ast::ast::{Function, TSType, TSTypeName};

use super::model::{DocItem, DocTag, FunctionTypeMetadata};
use super::DocVisitor;

impl<'a> DocVisitor<'a> {
    fn extract_return_type_from_annotation(
        &self,
        return_type: Option<&oxc_ast::ast::TSTypeAnnotation<'a>>,
        tags: &[DocTag],
    ) -> Option<String> {
        return_type.map(|r| self.format_ts_type(&r.type_annotation)).or_else(|| {
            tags.iter().find(|tag| tag.tag == "returns" || tag.tag == "return").and_then(|tag| {
                let (type_annotation, description) = Self::parse_return_tag(tag);
                type_annotation.or(description)
            })
        })
    }

    pub(super) fn extract_return_from_annotation(
        &self,
        return_type: Option<&oxc_ast::ast::TSTypeAnnotation<'a>>,
        tags: &[DocTag],
    ) -> (Option<String>, Vec<DocItem>) {
        if let Some(return_type) = return_type {
            if let TSType::TSTypeLiteral(type_literal) = &return_type.type_annotation {
                let members = self.extract_ts_signature_members(&type_literal.members);
                let type_annotation = if members.is_empty() {
                    self.format_ts_type(&return_type.type_annotation)
                } else {
                    "object".to_string()
                };
                return (Some(type_annotation), members);
            }
            return (Some(self.format_ts_type(&return_type.type_annotation)), Vec::new());
        }

        (self.extract_return_type_from_annotation(None, tags), Vec::new())
    }

    pub(super) fn extract_return(
        &self,
        func: &Function,
        tags: &[DocTag],
    ) -> (Option<String>, Vec<DocItem>) {
        self.extract_return_from_annotation(func.return_type.as_ref().map(AsRef::as_ref), tags)
    }

    pub(super) fn extract_function_type_metadata(
        &self,
        ts_type: &TSType<'a>,
        tags: &[DocTag],
    ) -> Option<FunctionTypeMetadata> {
        match ts_type {
            TSType::TSFunctionType(func) => {
                let (return_type, return_members) =
                    self.extract_return_from_annotation(Some(&func.return_type), tags);
                Some(FunctionTypeMetadata {
                    params: self.extract_params_from_formals(&func.params, tags),
                    return_type,
                    return_members,
                    type_parameters: self.extract_type_parameters(func.type_parameters.as_ref()),
                })
            }
            TSType::TSParenthesizedType(paren) => {
                self.extract_function_type_metadata(&paren.type_annotation, tags)
            }
            TSType::TSTypeReference(ref_type) => {
                let name = Self::type_reference_identifier_name(ref_type)?;
                self.type_alias_function_metadata
                    .get(&name)
                    .map(FunctionTypeMetadata::as_reference_metadata)
            }
            TSType::TSIntersectionType(inter) => inter
                .types
                .iter()
                .find_map(|ts_type| self.extract_function_type_metadata(ts_type, tags)),
            _ => None,
        }
    }

    fn type_reference_identifier_name(
        ref_type: &oxc_ast::ast::TSTypeReference<'a>,
    ) -> Option<String> {
        match &ref_type.type_name {
            TSTypeName::IdentifierReference(id) => Some(id.name.to_string()),
            TSTypeName::QualifiedName(_) | TSTypeName::ThisExpression(_) => None,
        }
    }
}
