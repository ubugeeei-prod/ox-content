// @author kazuya kawaguchi (a.k.a. kazupon)
// @license MIT
//

//! `LazyJsdocVisitor` — depth-first walker over the lazy decoder.
//!
//! See `design/007-binary-ast/rust-impl.md#lazy-visitor-trait-code-generation`.
//!
//! Each lazy node kind exposes two methods:
//!
//! - `visit_xxx(&mut self, node)` — the override hook. The default
//!   implementation forwards to `visit_xxx_default(node)` for container
//!   nodes (or is a no-op for leaves), so an empty trait impl already
//!   walks the entire tree.
//! - `visit_xxx_default(&mut self, node)` (container nodes only) —
//!   recursively walks every accessor-exposed child via the corresponding
//!   `visit_*` method. Override `visit_xxx` and call this from inside it to
//!   mix custom logic with the default traversal.
//!
//! Two helpers dispatch sum-typed children:
//!
//! - [`LazyJsdocVisitor::visit_type_node`] — match on a [`LazyTypeNode`] and
//!   forward to the variant-specific `visit_type_*` method.
//! - [`LazyJsdocVisitor::visit_tag_body`] — match on a [`LazyJsdocTagBody`].
//!
//! Phase 4 will replace this hand-written file with a code-generated
//! version derived from the same schema that emits the writer/decoder.

use super::nodes::comment_ast::{
    LazyJsdocBlock, LazyJsdocBorrowsTagBody, LazyJsdocDescriptionLine, LazyJsdocGenericTagBody,
    LazyJsdocIdentifier, LazyJsdocInlineTag, LazyJsdocNamepathSource, LazyJsdocParameterName,
    LazyJsdocRawTagBody, LazyJsdocTag, LazyJsdocTagBody, LazyJsdocTagName, LazyJsdocTagNameValue,
    LazyJsdocText, LazyJsdocTypeLine, LazyJsdocTypeSource,
};
use super::nodes::type_node::{
    LazyTypeAny, LazyTypeAsserts, LazyTypeAssertsPlain, LazyTypeCallSignature, LazyTypeConditional,
    LazyTypeConstructorSignature, LazyTypeFunction, LazyTypeGeneric, LazyTypeImport,
    LazyTypeIndexSignature, LazyTypeIndexedAccessIndex, LazyTypeInfer, LazyTypeIntersection,
    LazyTypeJsdocObjectField, LazyTypeKeyOf, LazyTypeKeyValue, LazyTypeMappedType,
    LazyTypeMethodSignature, LazyTypeName, LazyTypeNamePath, LazyTypeNode, LazyTypeNotNullable,
    LazyTypeNull, LazyTypeNullable, LazyTypeNumber, LazyTypeObject, LazyTypeObjectField,
    LazyTypeOptional, LazyTypeParameterList, LazyTypeParenthesis, LazyTypePredicate,
    LazyTypeProperty, LazyTypeReadonlyArray, LazyTypeReadonlyProperty, LazyTypeSpecialNamePath,
    LazyTypeStringValue, LazyTypeSymbol, LazyTypeTemplateLiteral, LazyTypeTuple, LazyTypeTypeOf,
    LazyTypeTypeParameter, LazyTypeUndefined, LazyTypeUnion, LazyTypeUniqueSymbol, LazyTypeUnknown,
    LazyTypeVariadic,
};

/// Hand-written depth-first visitor over the lazy decoder. The default
/// implementations together produce a complete pre-order walk; users can
/// hook any subset of the methods to inject custom behaviour.
///
/// See the module-level docs for the `visit_xxx` / `visit_xxx_default`
/// pattern.
pub trait LazyJsdocVisitor<'a> {
    // =====================================================================
    // Comment AST
    // =====================================================================

    /// Visit a `JsdocBlock`. Default: walk every accessor-exposed child.
    fn visit_block(&mut self, block: LazyJsdocBlock<'a>) {
        self.visit_block_default(block);
    }

    /// Default `JsdocBlock` traversal — descriptionLines → tags → inlineTags.
    fn visit_block_default(&mut self, block: LazyJsdocBlock<'a>) {
        for line in block.description_lines() {
            self.visit_description_line(line);
        }
        for tag in block.tags() {
            self.visit_tag(tag);
        }
        for inline in block.inline_tags() {
            self.visit_inline_tag(inline);
        }
    }

    /// Visit a `JsdocDescriptionLine` leaf.
    fn visit_description_line(&mut self, _node: LazyJsdocDescriptionLine<'a>) {}

    /// Visit a `JsdocTag`. Default: walk every accessor-exposed child.
    fn visit_tag(&mut self, tag: LazyJsdocTag<'a>) {
        self.visit_tag_default(tag);
    }

    /// Default `JsdocTag` traversal in visitor-bit order
    /// (tag → rawType → name → parsedType → body → typeLines →
    /// descriptionLines → inlineTags).
    fn visit_tag_default(&mut self, tag: LazyJsdocTag<'a>) {
        self.visit_tag_name(tag.tag());
        if let Some(rt) = tag.raw_type() {
            self.visit_type_source(rt);
        }
        if let Some(name) = tag.name() {
            self.visit_tag_name_value(name);
        }
        if let Some(pt) = tag.parsed_type() {
            self.visit_type_node(pt);
        }
        if let Some(body) = tag.body() {
            self.visit_tag_body(body);
        }
        for line in tag.type_lines() {
            self.visit_type_line(line);
        }
        for line in tag.description_lines() {
            self.visit_description_line(line);
        }
        for inline in tag.inline_tags() {
            self.visit_inline_tag(inline);
        }
    }

    /// Dispatch a `JsdocTagBody` sum to the variant-specific method.
    fn visit_tag_body(&mut self, body: LazyJsdocTagBody<'a>) {
        match body {
            LazyJsdocTagBody::Generic(b) => self.visit_generic_tag_body(b),
            LazyJsdocTagBody::Borrows(b) => self.visit_borrows_tag_body(b),
            LazyJsdocTagBody::Raw(b) => self.visit_raw_tag_body(b),
        }
    }

    /// Visit a `JsdocTagName` leaf.
    fn visit_tag_name(&mut self, _node: LazyJsdocTagName<'a>) {}
    /// Visit a `JsdocTagNameValue` leaf.
    fn visit_tag_name_value(&mut self, _node: LazyJsdocTagNameValue<'a>) {}
    /// Visit a `JsdocTypeSource` leaf.
    fn visit_type_source(&mut self, _node: LazyJsdocTypeSource<'a>) {}
    /// Visit a `JsdocTypeLine` leaf.
    fn visit_type_line(&mut self, _node: LazyJsdocTypeLine<'a>) {}
    /// Visit a `JsdocInlineTag` leaf.
    fn visit_inline_tag(&mut self, _node: LazyJsdocInlineTag<'a>) {}
    /// Visit a `JsdocGenericTagBody`. Currently a leaf — child accessors
    /// (description tag-list etc.) land in Phase 1.2a.
    fn visit_generic_tag_body(&mut self, _node: LazyJsdocGenericTagBody<'a>) {}
    /// Visit a `JsdocBorrowsTagBody`. Currently a leaf — `source`/`target`
    /// child accessors land in Phase 1.2a.
    fn visit_borrows_tag_body(&mut self, _node: LazyJsdocBorrowsTagBody<'a>) {}
    /// Visit a `JsdocRawTagBody` leaf.
    fn visit_raw_tag_body(&mut self, _node: LazyJsdocRawTagBody<'a>) {}
    /// Visit a `JsdocParameterName` leaf.
    fn visit_parameter_name(&mut self, _node: LazyJsdocParameterName<'a>) {}
    /// Visit a `JsdocNamepathSource` leaf.
    fn visit_namepath_source(&mut self, _node: LazyJsdocNamepathSource<'a>) {}
    /// Visit a `JsdocIdentifier` leaf.
    fn visit_identifier(&mut self, _node: LazyJsdocIdentifier<'a>) {}
    /// Visit a `JsdocText` leaf.
    fn visit_text(&mut self, _node: LazyJsdocText<'a>) {}

    // =====================================================================
    // TypeNode dispatch
    // =====================================================================

    /// Dispatch a `LazyTypeNode` sum to the variant-specific method.
    fn visit_type_node(&mut self, node: LazyTypeNode<'a>) {
        match node {
            LazyTypeNode::Name(n) => self.visit_type_name(n),
            LazyTypeNode::Number(n) => self.visit_type_number(n),
            LazyTypeNode::StringValue(n) => self.visit_type_string_value(n),
            LazyTypeNode::Property(n) => self.visit_type_property(n),
            LazyTypeNode::SpecialNamePath(n) => self.visit_type_special_name_path(n),
            LazyTypeNode::Union(n) => self.visit_type_union(n),
            LazyTypeNode::Intersection(n) => self.visit_type_intersection(n),
            LazyTypeNode::Generic(n) => self.visit_type_generic(n),
            LazyTypeNode::Function(n) => self.visit_type_function(n),
            LazyTypeNode::Object(n) => self.visit_type_object(n),
            LazyTypeNode::Tuple(n) => self.visit_type_tuple(n),
            LazyTypeNode::Parenthesis(n) => self.visit_type_parenthesis(n),
            LazyTypeNode::NamePath(n) => self.visit_type_name_path(n),
            LazyTypeNode::Nullable(n) => self.visit_type_nullable(n),
            LazyTypeNode::NotNullable(n) => self.visit_type_not_nullable(n),
            LazyTypeNode::Optional(n) => self.visit_type_optional(n),
            LazyTypeNode::Variadic(n) => self.visit_type_variadic(n),
            LazyTypeNode::Conditional(n) => self.visit_type_conditional(n),
            LazyTypeNode::Infer(n) => self.visit_type_infer(n),
            LazyTypeNode::KeyOf(n) => self.visit_type_key_of(n),
            LazyTypeNode::TypeOf(n) => self.visit_type_type_of(n),
            LazyTypeNode::Import(n) => self.visit_type_import(n),
            LazyTypeNode::Predicate(n) => self.visit_type_predicate(n),
            LazyTypeNode::Asserts(n) => self.visit_type_asserts(n),
            LazyTypeNode::AssertsPlain(n) => self.visit_type_asserts_plain(n),
            LazyTypeNode::ReadonlyArray(n) => self.visit_type_readonly_array(n),
            LazyTypeNode::ObjectField(n) => self.visit_type_object_field(n),
            LazyTypeNode::JsdocObjectField(n) => self.visit_type_jsdoc_object_field(n),
            LazyTypeNode::IndexedAccessIndex(n) => self.visit_type_indexed_access_index(n),
            LazyTypeNode::CallSignature(n) => self.visit_type_call_signature(n),
            LazyTypeNode::ConstructorSignature(n) => self.visit_type_constructor_signature(n),
            LazyTypeNode::TypeParameter(n) => self.visit_type_type_parameter(n),
            LazyTypeNode::ParameterList(n) => self.visit_type_parameter_list(n),
            LazyTypeNode::ReadonlyProperty(n) => self.visit_type_readonly_property(n),
            LazyTypeNode::KeyValue(n) => self.visit_type_key_value(n),
            LazyTypeNode::IndexSignature(n) => self.visit_type_index_signature(n),
            LazyTypeNode::MappedType(n) => self.visit_type_mapped_type(n),
            LazyTypeNode::MethodSignature(n) => self.visit_type_method_signature(n),
            LazyTypeNode::TemplateLiteral(n) => self.visit_type_template_literal(n),
            LazyTypeNode::Symbol(n) => self.visit_type_symbol(n),
            LazyTypeNode::Null(n) => self.visit_type_null(n),
            LazyTypeNode::Undefined(n) => self.visit_type_undefined(n),
            LazyTypeNode::Any(n) => self.visit_type_any(n),
            LazyTypeNode::Unknown(n) => self.visit_type_unknown(n),
            LazyTypeNode::UniqueSymbol(n) => self.visit_type_unique_symbol(n),
        }
    }

    // =====================================================================
    // TypeNode — Pattern 1 (string-only leaves)
    // =====================================================================

    /// Visit a `TypeName` leaf.
    fn visit_type_name(&mut self, _node: LazyTypeName<'a>) {}
    /// Visit a `TypeNumber` leaf.
    fn visit_type_number(&mut self, _node: LazyTypeNumber<'a>) {}
    /// Visit a `TypeStringValue` leaf.
    fn visit_type_string_value(&mut self, _node: LazyTypeStringValue<'a>) {}
    /// Visit a `TypeProperty` leaf.
    fn visit_type_property(&mut self, _node: LazyTypeProperty<'a>) {}
    /// Visit a `TypeSpecialNamePath` leaf.
    fn visit_type_special_name_path(&mut self, _node: LazyTypeSpecialNamePath<'a>) {}

    // =====================================================================
    // TypeNode — Pattern 2 (children-only containers)
    // =====================================================================

    /// Visit a `TypeUnion`. Default: walk `elements`.
    fn visit_type_union(&mut self, node: LazyTypeUnion<'a>) {
        self.visit_type_union_default(node);
    }
    /// Default `TypeUnion` traversal.
    fn visit_type_union_default(&mut self, node: LazyTypeUnion<'a>) {
        for el in node.elements() {
            self.visit_type_node(el);
        }
    }

    /// Visit a `TypeIntersection`. Default: walk `elements`.
    fn visit_type_intersection(&mut self, node: LazyTypeIntersection<'a>) {
        self.visit_type_intersection_default(node);
    }
    /// Default `TypeIntersection` traversal.
    fn visit_type_intersection_default(&mut self, node: LazyTypeIntersection<'a>) {
        for el in node.elements() {
            self.visit_type_node(el);
        }
    }

    /// Visit a `TypeGeneric`. Default: walk `left` then `elements`.
    fn visit_type_generic(&mut self, node: LazyTypeGeneric<'a>) {
        self.visit_type_generic_default(node);
    }
    /// Default `TypeGeneric` traversal.
    fn visit_type_generic_default(&mut self, node: LazyTypeGeneric<'a>) {
        if let Some(l) = node.left() {
            self.visit_type_node(l);
        }
        for el in node.elements() {
            self.visit_type_node(el);
        }
    }

    /// Visit a `TypeFunction`. Default: walk
    /// `parameters` → `return_type` → `type_parameters`.
    fn visit_type_function(&mut self, node: LazyTypeFunction<'a>) {
        self.visit_type_function_default(node);
    }
    /// Default `TypeFunction` traversal.
    fn visit_type_function_default(&mut self, node: LazyTypeFunction<'a>) {
        if let Some(params) = node.parameters() {
            self.visit_type_parameter_list(params);
        }
        if let Some(ret) = node.return_type() {
            self.visit_type_node(ret);
        }
        if let Some(tp) = node.type_parameters() {
            self.visit_type_parameter_list(tp);
        }
    }

    /// Visit a `TypeObject`. Default: walk `elements`.
    fn visit_type_object(&mut self, node: LazyTypeObject<'a>) {
        self.visit_type_object_default(node);
    }
    /// Default `TypeObject` traversal.
    fn visit_type_object_default(&mut self, node: LazyTypeObject<'a>) {
        for el in node.elements() {
            self.visit_type_node(el);
        }
    }

    /// Visit a `TypeTuple`. Default: walk `elements`.
    fn visit_type_tuple(&mut self, node: LazyTypeTuple<'a>) {
        self.visit_type_tuple_default(node);
    }
    /// Default `TypeTuple` traversal.
    fn visit_type_tuple_default(&mut self, node: LazyTypeTuple<'a>) {
        for el in node.elements() {
            self.visit_type_node(el);
        }
    }

    /// Visit a `TypeParenthesis`. Default: walk `element`.
    fn visit_type_parenthesis(&mut self, node: LazyTypeParenthesis<'a>) {
        self.visit_type_parenthesis_default(node);
    }
    /// Default `TypeParenthesis` traversal.
    fn visit_type_parenthesis_default(&mut self, node: LazyTypeParenthesis<'a>) {
        if let Some(el) = node.element() {
            self.visit_type_node(el);
        }
    }

    /// Visit a `TypeNamePath`. Default: walk `left` then `right`.
    fn visit_type_name_path(&mut self, node: LazyTypeNamePath<'a>) {
        self.visit_type_name_path_default(node);
    }
    /// Default `TypeNamePath` traversal.
    fn visit_type_name_path_default(&mut self, node: LazyTypeNamePath<'a>) {
        if let Some(l) = node.left() {
            self.visit_type_node(l);
        }
        if let Some(r) = node.right() {
            self.visit_type_node(r);
        }
    }

    /// Visit a `TypeNullable`. Default: walk `element`.
    fn visit_type_nullable(&mut self, node: LazyTypeNullable<'a>) {
        self.visit_type_nullable_default(node);
    }
    /// Default `TypeNullable` traversal.
    fn visit_type_nullable_default(&mut self, node: LazyTypeNullable<'a>) {
        if let Some(el) = node.element() {
            self.visit_type_node(el);
        }
    }

    /// Visit a `TypeNotNullable`. Default: walk `element`.
    fn visit_type_not_nullable(&mut self, node: LazyTypeNotNullable<'a>) {
        self.visit_type_not_nullable_default(node);
    }
    /// Default `TypeNotNullable` traversal.
    fn visit_type_not_nullable_default(&mut self, node: LazyTypeNotNullable<'a>) {
        if let Some(el) = node.element() {
            self.visit_type_node(el);
        }
    }

    /// Visit a `TypeOptional`. Default: walk `element`.
    fn visit_type_optional(&mut self, node: LazyTypeOptional<'a>) {
        self.visit_type_optional_default(node);
    }
    /// Default `TypeOptional` traversal.
    fn visit_type_optional_default(&mut self, node: LazyTypeOptional<'a>) {
        if let Some(el) = node.element() {
            self.visit_type_node(el);
        }
    }

    /// Visit a `TypeVariadic`. Default: walk `element`.
    fn visit_type_variadic(&mut self, node: LazyTypeVariadic<'a>) {
        self.visit_type_variadic_default(node);
    }
    /// Default `TypeVariadic` traversal.
    fn visit_type_variadic_default(&mut self, node: LazyTypeVariadic<'a>) {
        if let Some(el) = node.element() {
            self.visit_type_node(el);
        }
    }

    /// Visit a `TypeConditional`. Default: walk
    /// check → extends → true → false.
    fn visit_type_conditional(&mut self, node: LazyTypeConditional<'a>) {
        self.visit_type_conditional_default(node);
    }
    /// Default `TypeConditional` traversal.
    fn visit_type_conditional_default(&mut self, node: LazyTypeConditional<'a>) {
        if let Some(t) = node.check_type() {
            self.visit_type_node(t);
        }
        if let Some(t) = node.extends_type() {
            self.visit_type_node(t);
        }
        if let Some(t) = node.true_type() {
            self.visit_type_node(t);
        }
        if let Some(t) = node.false_type() {
            self.visit_type_node(t);
        }
    }

    /// Visit a `TypeInfer`. Default: walk `element`.
    fn visit_type_infer(&mut self, node: LazyTypeInfer<'a>) {
        self.visit_type_infer_default(node);
    }
    /// Default `TypeInfer` traversal.
    fn visit_type_infer_default(&mut self, node: LazyTypeInfer<'a>) {
        if let Some(el) = node.element() {
            self.visit_type_node(el);
        }
    }

    /// Visit a `TypeKeyOf`. Default: walk `element`.
    fn visit_type_key_of(&mut self, node: LazyTypeKeyOf<'a>) {
        self.visit_type_key_of_default(node);
    }
    /// Default `TypeKeyOf` traversal.
    fn visit_type_key_of_default(&mut self, node: LazyTypeKeyOf<'a>) {
        if let Some(el) = node.element() {
            self.visit_type_node(el);
        }
    }

    /// Visit a `TypeTypeOf`. Default: walk `element`.
    fn visit_type_type_of(&mut self, node: LazyTypeTypeOf<'a>) {
        self.visit_type_type_of_default(node);
    }
    /// Default `TypeTypeOf` traversal.
    fn visit_type_type_of_default(&mut self, node: LazyTypeTypeOf<'a>) {
        if let Some(el) = node.element() {
            self.visit_type_node(el);
        }
    }

    /// Visit a `TypeImport`. Default: walk `element`.
    fn visit_type_import(&mut self, node: LazyTypeImport<'a>) {
        self.visit_type_import_default(node);
    }
    /// Default `TypeImport` traversal.
    fn visit_type_import_default(&mut self, node: LazyTypeImport<'a>) {
        if let Some(el) = node.element() {
            self.visit_type_node(el);
        }
    }

    /// Visit a `TypePredicate`. Default: walk `left` then `right`.
    fn visit_type_predicate(&mut self, node: LazyTypePredicate<'a>) {
        self.visit_type_predicate_default(node);
    }
    /// Default `TypePredicate` traversal.
    fn visit_type_predicate_default(&mut self, node: LazyTypePredicate<'a>) {
        if let Some(l) = node.left() {
            self.visit_type_node(l);
        }
        if let Some(r) = node.right() {
            self.visit_type_node(r);
        }
    }

    /// Visit a `TypeAsserts`. Default: walk `left` then `right`.
    fn visit_type_asserts(&mut self, node: LazyTypeAsserts<'a>) {
        self.visit_type_asserts_default(node);
    }
    /// Default `TypeAsserts` traversal.
    fn visit_type_asserts_default(&mut self, node: LazyTypeAsserts<'a>) {
        if let Some(l) = node.left() {
            self.visit_type_node(l);
        }
        if let Some(r) = node.right() {
            self.visit_type_node(r);
        }
    }

    /// Visit a `TypeAssertsPlain`. Default: walk `element`.
    fn visit_type_asserts_plain(&mut self, node: LazyTypeAssertsPlain<'a>) {
        self.visit_type_asserts_plain_default(node);
    }
    /// Default `TypeAssertsPlain` traversal.
    fn visit_type_asserts_plain_default(&mut self, node: LazyTypeAssertsPlain<'a>) {
        if let Some(el) = node.element() {
            self.visit_type_node(el);
        }
    }

    /// Visit a `TypeReadonlyArray`. Default: walk `element`.
    fn visit_type_readonly_array(&mut self, node: LazyTypeReadonlyArray<'a>) {
        self.visit_type_readonly_array_default(node);
    }
    /// Default `TypeReadonlyArray` traversal.
    fn visit_type_readonly_array_default(&mut self, node: LazyTypeReadonlyArray<'a>) {
        if let Some(el) = node.element() {
            self.visit_type_node(el);
        }
    }

    /// Visit a `TypeObjectField`. Default: walk `key` then `right`.
    fn visit_type_object_field(&mut self, node: LazyTypeObjectField<'a>) {
        self.visit_type_object_field_default(node);
    }
    /// Default `TypeObjectField` traversal.
    fn visit_type_object_field_default(&mut self, node: LazyTypeObjectField<'a>) {
        if let Some(k) = node.key() {
            self.visit_type_node(k);
        }
        if let Some(r) = node.right() {
            self.visit_type_node(r);
        }
    }

    /// Visit a `TypeJsdocObjectField`. Default: walk `key` then `right`.
    fn visit_type_jsdoc_object_field(&mut self, node: LazyTypeJsdocObjectField<'a>) {
        self.visit_type_jsdoc_object_field_default(node);
    }
    /// Default `TypeJsdocObjectField` traversal.
    fn visit_type_jsdoc_object_field_default(&mut self, node: LazyTypeJsdocObjectField<'a>) {
        if let Some(k) = node.key() {
            self.visit_type_node(k);
        }
        if let Some(r) = node.right() {
            self.visit_type_node(r);
        }
    }

    /// Visit a `TypeIndexedAccessIndex`. Default: walk `element`.
    fn visit_type_indexed_access_index(&mut self, node: LazyTypeIndexedAccessIndex<'a>) {
        self.visit_type_indexed_access_index_default(node);
    }
    /// Default `TypeIndexedAccessIndex` traversal.
    fn visit_type_indexed_access_index_default(&mut self, node: LazyTypeIndexedAccessIndex<'a>) {
        if let Some(el) = node.element() {
            self.visit_type_node(el);
        }
    }

    /// Visit a `TypeCallSignature`. Default: walk
    /// parameters → return_type → type_parameters.
    fn visit_type_call_signature(&mut self, node: LazyTypeCallSignature<'a>) {
        self.visit_type_call_signature_default(node);
    }
    /// Default `TypeCallSignature` traversal.
    fn visit_type_call_signature_default(&mut self, node: LazyTypeCallSignature<'a>) {
        if let Some(params) = node.parameters() {
            self.visit_type_parameter_list(params);
        }
        if let Some(ret) = node.return_type() {
            self.visit_type_node(ret);
        }
        if let Some(tp) = node.type_parameters() {
            self.visit_type_parameter_list(tp);
        }
    }

    /// Visit a `TypeConstructorSignature`. Default: walk
    /// parameters → return_type → type_parameters.
    fn visit_type_constructor_signature(&mut self, node: LazyTypeConstructorSignature<'a>) {
        self.visit_type_constructor_signature_default(node);
    }
    /// Default `TypeConstructorSignature` traversal.
    fn visit_type_constructor_signature_default(&mut self, node: LazyTypeConstructorSignature<'a>) {
        if let Some(params) = node.parameters() {
            self.visit_type_parameter_list(params);
        }
        if let Some(ret) = node.return_type() {
            self.visit_type_node(ret);
        }
        if let Some(tp) = node.type_parameters() {
            self.visit_type_parameter_list(tp);
        }
    }

    /// Visit a `TypeTypeParameter`. Default: walk `elements`.
    fn visit_type_type_parameter(&mut self, node: LazyTypeTypeParameter<'a>) {
        self.visit_type_type_parameter_default(node);
    }
    /// Default `TypeTypeParameter` traversal.
    fn visit_type_type_parameter_default(&mut self, node: LazyTypeTypeParameter<'a>) {
        for el in node.elements() {
            self.visit_type_node(el);
        }
    }

    /// Visit a `TypeParameterList`. Default: walk `elements`.
    fn visit_type_parameter_list(&mut self, node: LazyTypeParameterList<'a>) {
        self.visit_type_parameter_list_default(node);
    }
    /// Default `TypeParameterList` traversal.
    fn visit_type_parameter_list_default(&mut self, node: LazyTypeParameterList<'a>) {
        for el in node.elements() {
            self.visit_type_node(el);
        }
    }

    /// Visit a `TypeReadonlyProperty`. Default: walk `element`.
    fn visit_type_readonly_property(&mut self, node: LazyTypeReadonlyProperty<'a>) {
        self.visit_type_readonly_property_default(node);
    }
    /// Default `TypeReadonlyProperty` traversal.
    fn visit_type_readonly_property_default(&mut self, node: LazyTypeReadonlyProperty<'a>) {
        if let Some(el) = node.element() {
            self.visit_type_node(el);
        }
    }

    // =====================================================================
    // TypeNode — Pattern 3 (mixed string + children)
    // =====================================================================

    /// Visit a `TypeKeyValue`. Default: walk `right`.
    fn visit_type_key_value(&mut self, node: LazyTypeKeyValue<'a>) {
        self.visit_type_key_value_default(node);
    }
    /// Default `TypeKeyValue` traversal.
    fn visit_type_key_value_default(&mut self, node: LazyTypeKeyValue<'a>) {
        if let Some(r) = node.right() {
            self.visit_type_node(r);
        }
    }

    /// Visit a `TypeIndexSignature`. Default: walk `right`.
    fn visit_type_index_signature(&mut self, node: LazyTypeIndexSignature<'a>) {
        self.visit_type_index_signature_default(node);
    }
    /// Default `TypeIndexSignature` traversal.
    fn visit_type_index_signature_default(&mut self, node: LazyTypeIndexSignature<'a>) {
        if let Some(r) = node.right() {
            self.visit_type_node(r);
        }
    }

    /// Visit a `TypeMappedType`. Default: walk `right`.
    fn visit_type_mapped_type(&mut self, node: LazyTypeMappedType<'a>) {
        self.visit_type_mapped_type_default(node);
    }
    /// Default `TypeMappedType` traversal.
    fn visit_type_mapped_type_default(&mut self, node: LazyTypeMappedType<'a>) {
        if let Some(r) = node.right() {
            self.visit_type_node(r);
        }
    }

    /// Visit a `TypeMethodSignature`. Currently a leaf — child accessors
    /// (`parameters` / `return_type` / `type_parameters`) land in Phase 1.2a
    /// alongside the parser's MethodSignature emission.
    fn visit_type_method_signature(&mut self, _node: LazyTypeMethodSignature<'a>) {}

    /// Visit a `TypeTemplateLiteral` leaf (literals live in Extended Data,
    /// not as child nodes).
    fn visit_type_template_literal(&mut self, _node: LazyTypeTemplateLiteral<'a>) {}

    /// Visit a `TypeSymbol`. Default: walk `element` when `has_element`.
    fn visit_type_symbol(&mut self, node: LazyTypeSymbol<'a>) {
        self.visit_type_symbol_default(node);
    }
    /// Default `TypeSymbol` traversal.
    fn visit_type_symbol_default(&mut self, node: LazyTypeSymbol<'a>) {
        if let Some(el) = node.element() {
            self.visit_type_node(el);
        }
    }

    // =====================================================================
    // TypeNode — Others (pure leaves, no payload)
    // =====================================================================

    /// Visit a `TypeNull` leaf.
    fn visit_type_null(&mut self, _node: LazyTypeNull<'a>) {}
    /// Visit a `TypeUndefined` leaf.
    fn visit_type_undefined(&mut self, _node: LazyTypeUndefined<'a>) {}
    /// Visit a `TypeAny` leaf.
    fn visit_type_any(&mut self, _node: LazyTypeAny<'a>) {}
    /// Visit a `TypeUnknown` leaf.
    fn visit_type_unknown(&mut self, _node: LazyTypeUnknown<'a>) {}
    /// Visit a `TypeUniqueSymbol` leaf.
    fn visit_type_unique_symbol(&mut self, _node: LazyTypeUniqueSymbol<'a>) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoder::source_file::LazySourceFile;
    use crate::format::kind::Kind;
    use crate::writer::BinaryWriter;
    use crate::writer::nodes::comment_ast::{
        JSDOC_BLOCK_TAGS_SLOT, write_jsdoc_block, write_jsdoc_tag, write_jsdoc_tag_name,
        write_jsdoc_tag_name_value,
    };
    use crate::writer::nodes::type_node::TYPE_LIST_PARENT_SLOT;
    use crate::writer::nodes::type_node::write_type_name;
    use oxc_allocator::Allocator;
    use oxc_span::Span;
    use std::collections::HashMap;

    /// Visitor that counts every visited node bucketed by Kind.
    struct CountVisitor {
        counts: HashMap<Kind, usize>,
    }

    impl CountVisitor {
        fn bump(&mut self, kind: Kind) {
            *self.counts.entry(kind).or_insert(0) += 1;
        }
    }

    impl<'a> LazyJsdocVisitor<'a> for CountVisitor {
        fn visit_block(&mut self, b: LazyJsdocBlock<'a>) {
            self.bump(Kind::JsdocBlock);
            self.visit_block_default(b);
        }
        fn visit_tag(&mut self, t: LazyJsdocTag<'a>) {
            self.bump(Kind::JsdocTag);
            self.visit_tag_default(t);
        }
        fn visit_tag_name(&mut self, _n: LazyJsdocTagName<'a>) {
            self.bump(Kind::JsdocTagName);
        }
        fn visit_tag_name_value(&mut self, _n: LazyJsdocTagNameValue<'a>) {
            self.bump(Kind::JsdocTagNameValue);
        }
        fn visit_type_name(&mut self, _n: LazyTypeName<'a>) {
            self.bump(Kind::TypeName);
        }
    }

    /// Build a `/** @param {string} id */` buffer with the new no-NodeList
    /// hierarchy: block → tag (direct child) → tag-name + parsedType +
    /// tag-name-value. The block's `tags` list metadata slot is patched to
    /// `(head=tag_index, count=1)` once the tag has been emitted.
    fn build_single_tag_buffer<'a>(arena: &'a Allocator) -> Vec<u8> {
        let mut writer = BinaryWriter::new(arena);
        let _ = writer.append_source_text("/** @param {string} id */");

        let empty = writer.intern_string("");
        let star = writer.intern_string("*");
        let space = writer.intern_string(" ");
        let close = writer.intern_string("*/");
        let nl = writer.intern_string("\n");
        let tag_name_str = writer.intern_string_payload("param");
        let type_name_str = writer.intern_string_payload("string");
        let param_name_str = writer.intern_string_payload("id");

        let (block, block_ext) = write_jsdoc_block(
            &mut writer,
            Span::new(0, 25),
            0,
            None,
            star,
            space,
            close,
            nl,
            empty,
            nl,
            empty,
            0b010, // bit1 = tags
            None,  // description_raw_span — Phase 5 opt-in, off here
        );
        let mut tags_list = writer.begin_node_list_at(block_ext, JSDOC_BLOCK_TAGS_SLOT);
        // Tag children bitmask: bit0 (tag) + bit2 (name) + bit3 (parsedType) = 0b1101.
        let (tag, _tag_ext) = write_jsdoc_tag(
            &mut writer,
            Span::new(4, 23),
            block.as_u32(),
            false,
            None,
            None,
            None,
            0b0000_1101,
            None, // description_raw_span — Phase 5 opt-in, off here
        );
        writer.record_list_child(&mut tags_list, tag.as_u32());
        writer.finalize_node_list(tags_list);
        // Children of tag are emitted in visitor-bit order:
        // bit0 (JsdocTagName), then bit2 (JsdocTagNameValue), then bit3 (TypeName).
        let _ = write_jsdoc_tag_name(&mut writer, Span::new(4, 9), tag.as_u32(), tag_name_str);
        let _ = write_jsdoc_tag_name_value(
            &mut writer,
            Span::new(20, 22),
            tag.as_u32(),
            param_name_str,
        );
        let _ = write_type_name(&mut writer, Span::new(11, 17), tag.as_u32(), type_name_str);

        writer.push_root(block.as_u32(), 0, 0);
        writer.finish()
    }

    #[test]
    fn visitor_walks_block_and_children() {
        let arena = Allocator::default();
        let bytes = build_single_tag_buffer(&arena);
        let sf = LazySourceFile::new(&bytes).unwrap();

        let mut v = CountVisitor { counts: HashMap::new() };
        for opt in sf.asts() {
            if let Some(block) = opt {
                v.visit_block(block);
            }
        }

        assert_eq!(v.counts.get(&Kind::JsdocBlock).copied(), Some(1));
        assert_eq!(v.counts.get(&Kind::JsdocTag).copied(), Some(1));
        assert_eq!(v.counts.get(&Kind::JsdocTagName).copied(), Some(1));
        assert_eq!(v.counts.get(&Kind::JsdocTagNameValue).copied(), Some(1));
        // The TypeName is reached via `tag.parsed_type()` which feeds into
        // `visit_type_node` → `visit_type_name`.
        assert_eq!(v.counts.get(&Kind::TypeName).copied(), Some(1));
    }

    /// Empty visitor — verifies that `LazyJsdocVisitor` is implementable
    /// with zero overrides and traverses cleanly without panicking.
    struct NoopVisitor;
    impl<'a> LazyJsdocVisitor<'a> for NoopVisitor {}

    #[test]
    fn empty_visitor_traverses_without_panic() {
        let arena = Allocator::default();
        let bytes = build_single_tag_buffer(&arena);
        let sf = LazySourceFile::new(&bytes).unwrap();

        let mut v = NoopVisitor;
        for opt in sf.asts() {
            if let Some(block) = opt {
                v.visit_block(block);
            }
        }
    }

    /// Visitor that records every TypeNode variant it visits; verifies the
    /// `visit_type_node` dispatch wires each variant to its method.
    #[test]
    fn visit_type_node_dispatches_each_variant() {
        let arena = Allocator::default();
        let mut writer = BinaryWriter::new(&arena);
        let s = writer.intern_string_payload("Foo");
        // Build a TypeUnion with 2 TypeName children. Elements are direct
        // children of the union; their (head, count) is patched into the
        // union's ED list-metadata slot.
        use crate::writer::nodes::type_node::{write_type_name, write_type_union};
        let (union_idx, union_ext) = write_type_union(&mut writer, Span::new(0, 10), 0);
        let union = union_idx.as_u32();
        let mut list = writer.begin_node_list_at(union_ext, TYPE_LIST_PARENT_SLOT);
        let n1 = write_type_name(&mut writer, Span::new(0, 3), union, s);
        writer.record_list_child(&mut list, n1.as_u32());
        let n2 = write_type_name(&mut writer, Span::new(4, 7), union, s);
        writer.record_list_child(&mut list, n2.as_u32());
        writer.finalize_node_list(list);
        writer.push_root(union, 0, 0);

        let bytes = writer.finish();
        let sf = LazySourceFile::new(&bytes).unwrap();
        use crate::decoder::nodes::type_node::LazyTypeNode;

        struct TypeVisitor {
            saw_union: bool,
            type_name_count: usize,
        }
        impl<'a> LazyJsdocVisitor<'a> for TypeVisitor {
            fn visit_type_union(&mut self, n: LazyTypeUnion<'a>) {
                self.saw_union = true;
                self.visit_type_union_default(n);
            }
            fn visit_type_name(&mut self, _n: LazyTypeName<'a>) {
                self.type_name_count += 1;
            }
        }
        let mut v = TypeVisitor { saw_union: false, type_name_count: 0 };
        let root = LazyTypeNode::from_index(&sf, union, 0).expect("union root");
        v.visit_type_node(root);

        assert!(v.saw_union);
        assert_eq!(v.type_name_count, 2);
    }
}
