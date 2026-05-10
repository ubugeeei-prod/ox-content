// @author kazuya kawaguchi (a.k.a. kazupon)
// @license MIT
//

//! Walk a [`TypeNodeData`] tree in DFS pre-order and emit it to a
//! [`BinaryWriter`]. One `emit_type_node` per Kind; the helpers compute
//! the Children bitmask up-front (the writer needs it before children are
//! written), then recursively emit each present child with the parent's
//! `NodeIndex`.
//!
//! Mirrors the layout assumed by `crates/ox_jsdoc_binary/src/decoder/nodes/type_node.rs`:
//! - Single children (`element`, `left`, `right`, etc.) are direct children
//!   of the parent at the matching visitor-bit slot.
//! - Variable-length `elements` lists are emitted as direct children of the
//!   parent with `(head_index, count)` patched into the parent's Extended
//!   Data block at [`TYPE_LIST_PARENT_SLOT`]. The Kind 0x7F NodeList wrapper
//!   that previously sat between parent and elements is no longer emitted.
//! - Pattern 3 mixed nodes (`TypeKeyValue` etc.) place the optional child
//!   directly after the parent without a Children bitmask.

use crate::writer::nodes::type_node::*;
use crate::writer::{BinaryWriter, ExtOffset, NodeIndex};

use super::type_data::*;

/// Emit a TypeNodeData node into `writer` and return the assigned NodeIndex.
pub fn emit_type_node(
    writer: &mut BinaryWriter<'_>,
    node: &TypeNodeData<'_>,
    parent_index: u32,
) -> u32 {
    match node {
        TypeNodeData::Name(n) => {
            let v = writer.intern_source_or_string_for_leaf_payload(n.value, n.span);
            write_type_name(writer, n.span, parent_index, v).as_u32()
        }
        TypeNodeData::Number(n) => {
            let v = writer.intern_source_or_string_for_leaf_payload(n.value, n.span);
            write_type_number(writer, n.span, parent_index, v).as_u32()
        }
        TypeNodeData::StringValue(n) => {
            let v = writer.intern_source_or_string_for_leaf_payload(n.value, n.span);
            write_type_string_value(writer, n.span, parent_index, quote_to_u8(Some(n.quote)), v)
                .as_u32()
        }
        TypeNodeData::Property(n) => {
            // length-mismatch fallback handles the unquoted-property case
            // (`"foo"` → value `foo`, span `"foo"`).
            let v = writer.intern_source_or_string_for_leaf_payload(n.value, n.span);
            write_type_property(writer, n.span, parent_index, quote_to_u8(n.quote), v).as_u32()
        }
        TypeNodeData::SpecialNamePath(n) => {
            // Span here covers `module:foo` while value is just `foo` →
            // length-mismatch fallback.
            let v = writer.intern_source_or_string_for_leaf_payload(n.value, n.span);
            let st = match n.special_type {
                SpecialPathType::Module => 0,
                SpecialPathType::Event => 1,
                SpecialPathType::External => 2,
            };
            write_type_special_name_path(writer, n.span, parent_index, st, quote_to_u8(n.quote), v)
                .as_u32()
        }
        TypeNodeData::Null(n) => write_type_null(writer, n.span, parent_index).as_u32(),
        TypeNodeData::Undefined(n) => write_type_undefined(writer, n.span, parent_index).as_u32(),
        TypeNodeData::Any(n) => write_type_any(writer, n.span, parent_index).as_u32(),
        TypeNodeData::Unknown(n) => write_type_unknown(writer, n.span, parent_index).as_u32(),
        TypeNodeData::UniqueSymbol(n) => {
            write_type_unique_symbol(writer, n.span, parent_index).as_u32()
        }

        TypeNodeData::Union(n) => {
            emit_elements_only(writer, parent_index, n.span, &n.elements, |w, s, p| {
                write_type_union(w, s, p)
            })
        }
        TypeNodeData::Intersection(n) => {
            emit_elements_only(writer, parent_index, n.span, &n.elements, |w, s, p| {
                write_type_intersection(w, s, p)
            })
        }
        TypeNodeData::Tuple(n) => {
            emit_elements_only(writer, parent_index, n.span, &n.elements, |w, s, p| {
                write_type_tuple(w, s, p)
            })
        }
        TypeNodeData::TypeParameter(n) => {
            // TypeTypeParameter wraps name + optional constraint + optional default
            // as a single elements list per the lazy decoder shape.
            let mut wrapped: Vec<&TypeNodeData<'_>> = Vec::with_capacity(3);
            wrapped.push(&n.name);
            if let Some(c) = n.constraint.as_ref() {
                wrapped.push(c);
            }
            if let Some(d) = n.default_value.as_ref() {
                wrapped.push(d);
            }
            emit_borrowed_elements(writer, parent_index, n.span, &wrapped, |w, s, p| {
                write_type_type_parameter(w, s, p)
            })
        }
        TypeNodeData::ParameterList(n) => {
            emit_elements_only(writer, parent_index, n.span, &n.elements, |w, s, p| {
                write_type_parameter_list(w, s, p)
            })
        }

        TypeNodeData::Object(n) => {
            let sep = match n.separator {
                Some(ObjectSeparator::Comma) => 1,
                Some(ObjectSeparator::Semicolon) => 2,
                Some(ObjectSeparator::Linebreak) => 3,
                Some(ObjectSeparator::CommaAndLinebreak) => 4,
                Some(ObjectSeparator::SemicolonAndLinebreak) => 5,
                None => 0,
            };
            let (idx, ext) = write_type_object(writer, n.span, parent_index, sep);
            let parent = idx.as_u32();
            if !n.elements.is_empty() {
                emit_node_list_into_parent(writer, ext, parent, &n.elements);
            }
            parent
        }

        TypeNodeData::Generic(n) => {
            let (idx, ext) = write_type_generic(
                writer,
                n.span,
                parent_index,
                match n.brackets {
                    GenericBrackets::Angle => 0,
                    GenericBrackets::Square => 1,
                },
                n.dot,
            );
            let parent = idx.as_u32();
            emit_type_node(writer, &n.left, parent);
            if !n.elements.is_empty() {
                emit_node_list_into_parent(writer, ext, parent, &n.elements);
            }
            parent
        }

        TypeNodeData::Function(n) => emit_function_like(
            writer,
            parent_index,
            n.span,
            &n.parameters,
            n.return_type.as_deref(),
            &n.type_parameters,
            |w, s, p, b| {
                write_type_function(w, s, p, n.constructor, n.arrow, n.parenthesis, b).as_u32()
            },
        ),

        TypeNodeData::CallSignature(n) => emit_function_like(
            writer,
            parent_index,
            n.span,
            &n.parameters,
            Some(&n.return_type),
            &n.type_parameters,
            |w, s, p, b| write_type_call_signature(w, s, p, b).as_u32(),
        ),

        TypeNodeData::ConstructorSignature(n) => emit_function_like(
            writer,
            parent_index,
            n.span,
            &n.parameters,
            Some(&n.return_type),
            &n.type_parameters,
            |w, s, p, b| write_type_constructor_signature(w, s, p, b).as_u32(),
        ),

        TypeNodeData::Parenthesis(n) => {
            emit_single_child(writer, parent_index, n.span, &n.element, |w, s, p, b| {
                write_type_parenthesis(w, s, p, b).as_u32()
            })
        }
        TypeNodeData::Infer(n) => {
            emit_single_child(writer, parent_index, n.span, &n.element, |w, s, p, b| {
                write_type_infer(w, s, p, b).as_u32()
            })
        }
        TypeNodeData::KeyOf(n) => {
            emit_single_child(writer, parent_index, n.span, &n.element, |w, s, p, b| {
                write_type_key_of(w, s, p, b).as_u32()
            })
        }
        TypeNodeData::TypeOf(n) => {
            emit_single_child(writer, parent_index, n.span, &n.element, |w, s, p, b| {
                write_type_type_of(w, s, p, b).as_u32()
            })
        }
        TypeNodeData::Import(n) => {
            emit_single_child(writer, parent_index, n.span, &n.element, |w, s, p, b| {
                write_type_import(w, s, p, b).as_u32()
            })
        }
        TypeNodeData::AssertsPlain(n) => {
            emit_single_child(writer, parent_index, n.span, &n.element, |w, s, p, b| {
                write_type_asserts_plain(w, s, p, b).as_u32()
            })
        }
        TypeNodeData::ReadonlyArray(n) => {
            emit_single_child(writer, parent_index, n.span, &n.element, |w, s, p, b| {
                write_type_readonly_array(w, s, p, b).as_u32()
            })
        }
        TypeNodeData::IndexedAccessIndex(n) => {
            emit_single_child(writer, parent_index, n.span, &n.right, |w, s, p, b| {
                write_type_indexed_access_index(w, s, p, b).as_u32()
            })
        }
        TypeNodeData::ReadonlyProperty(n) => {
            emit_single_child(writer, parent_index, n.span, &n.element, |w, s, p, b| {
                write_type_readonly_property(w, s, p, b).as_u32()
            })
        }

        TypeNodeData::Nullable(n) => emit_modifier_child(
            writer,
            parent_index,
            n.span,
            &n.element,
            n.position,
            |w, s, p, pos, b| write_type_nullable(w, s, p, pos, b).as_u32(),
        ),
        TypeNodeData::NotNullable(n) => emit_modifier_child(
            writer,
            parent_index,
            n.span,
            &n.element,
            n.position,
            |w, s, p, pos, b| write_type_not_nullable(w, s, p, pos, b).as_u32(),
        ),
        TypeNodeData::Optional(n) => emit_modifier_child(
            writer,
            parent_index,
            n.span,
            &n.element,
            n.position,
            |w, s, p, pos, b| write_type_optional(w, s, p, pos, b).as_u32(),
        ),
        TypeNodeData::Variadic(n) => {
            let pos = n.position.map_or(0, |p| match p {
                VariadicPosition::Prefix => 0,
                VariadicPosition::Suffix => 1,
            });
            let bitmask = if n.element.is_some() { 0b1 } else { 0 };
            let parent =
                write_type_variadic(writer, n.span, parent_index, pos, n.square_brackets, bitmask)
                    .as_u32();
            if let Some(el) = n.element.as_ref() {
                emit_type_node(writer, el, parent);
            }
            parent
        }

        TypeNodeData::Conditional(n) => {
            let bitmask = 0b1111;
            let parent = write_type_conditional(writer, n.span, parent_index, bitmask).as_u32();
            emit_type_node(writer, &n.checks_type, parent);
            emit_type_node(writer, &n.extends_type, parent);
            emit_type_node(writer, &n.true_type, parent);
            emit_type_node(writer, &n.false_type, parent);
            parent
        }

        TypeNodeData::Predicate(n) => {
            emit_left_right(writer, parent_index, n.span, &n.left, &n.right, |w, s, p, b| {
                write_type_predicate(w, s, p, b).as_u32()
            })
        }
        TypeNodeData::Asserts(n) => {
            emit_left_right(writer, parent_index, n.span, &n.left, &n.right, |w, s, p, b| {
                write_type_asserts(w, s, p, b).as_u32()
            })
        }

        TypeNodeData::NamePath(n) => {
            let pt = match n.path_type {
                NamePathType::Property => 0,
                NamePathType::Instance => 1,
                NamePathType::Inner => 2,
                NamePathType::PropertyBrackets => 3,
            };
            let bitmask = 0b11;
            let parent = write_type_name_path(writer, n.span, parent_index, pt, bitmask).as_u32();
            emit_type_node(writer, &n.left, parent);
            emit_type_node(writer, &n.right, parent);
            parent
        }

        TypeNodeData::ObjectField(n) => {
            let mut bitmask = 0b1; // key
            if n.right.is_some() {
                bitmask |= 0b10;
            }
            let parent = write_type_object_field(
                writer,
                n.span,
                parent_index,
                n.optional,
                n.readonly,
                quote_to_u8(n.quote),
                bitmask,
            )
            .as_u32();
            emit_type_node(writer, &n.key, parent);
            if let Some(r) = n.right.as_ref() {
                emit_type_node(writer, r, parent);
            }
            parent
        }
        TypeNodeData::JsdocObjectField(n) => {
            let bitmask = 0b11;
            let parent =
                write_type_jsdoc_object_field(writer, n.span, parent_index, bitmask).as_u32();
            emit_type_node(writer, &n.left, parent);
            emit_type_node(writer, &n.right, parent);
            parent
        }

        // Pattern 3: Mixed (Extended Data + optional first child).
        TypeNodeData::KeyValue(n) => {
            // Span covers `key: value` (or `...key: value`); length-mismatch
            // fallback when `key` is just the leading identifier.
            let key = writer.intern_source_or_string(n.key, n.span);
            let parent =
                write_type_key_value(writer, n.span, parent_index, n.optional, n.variadic, key)
                    .as_u32();
            if let Some(r) = n.right.as_ref() {
                emit_type_node(writer, r, parent);
            }
            parent
        }
        TypeNodeData::IndexSignature(n) => {
            // Span covers `[key: K]: V`; key is just the leading identifier
            // → length-mismatch fallback.
            let key = writer.intern_source_or_string(n.key, n.span);
            let parent = write_type_index_signature(writer, n.span, parent_index, key).as_u32();
            emit_type_node(writer, &n.right, parent);
            parent
        }
        TypeNodeData::MappedType(n) => {
            // Same parent-spanning shape as IndexSignature.
            let key = writer.intern_source_or_string(n.key, n.span);
            let parent = write_type_mapped_type(writer, n.span, parent_index, key).as_u32();
            emit_type_node(writer, &n.right, parent);
            parent
        }
        TypeNodeData::MethodSignature(n) => {
            // Span covers the entire method signature; name is just the
            // leading identifier → length-mismatch fallback.
            let name = writer.intern_source_or_string(n.name, n.span);
            let has_params = !n.parameters.is_empty();
            let has_tparams = !n.type_parameters.is_empty();
            let parent = write_type_method_signature(
                writer,
                n.span,
                parent_index,
                quote_to_u8(n.quote),
                has_params,
                has_tparams,
                name,
            )
            .as_u32();
            // Currently the Rust lazy decoder treats MethodSignature as a leaf
            // (no exposed children accessors). For round-trip parity with the
            // typed parser we still emit the parameters/return/type-params
            // sub-tree as siblings; this matches the design's "code generation
            // will fill in the rest at Phase 4" note in
            // `crates/ox_jsdoc_binary/src/decoder/visitor.rs`.
            for p in &n.parameters {
                emit_type_node(writer, p, parent);
            }
            emit_type_node(writer, &n.return_type, parent);
            for tp in &n.type_parameters {
                emit_type_node(writer, tp, parent);
            }
            parent
        }
        TypeNodeData::TemplateLiteral(n) => {
            let interned: Vec<crate::writer::StringField> =
                n.literals.iter().map(|s| writer.intern_string(s)).collect();
            let parent =
                write_type_template_literal(writer, n.span, parent_index, &interned).as_u32();
            for interp in &n.interpolations {
                emit_type_node(writer, interp, parent);
            }
            parent
        }
        TypeNodeData::Symbol(n) => {
            // Span covers `Symbol(...)` while value is the leading name →
            // length-mismatch fallback.
            let value = writer.intern_source_or_string(n.value, n.span);
            let parent =
                write_type_symbol(writer, n.span, parent_index, n.element.is_some(), value)
                    .as_u32();
            if let Some(el) = n.element.as_ref() {
                emit_type_node(writer, el, parent);
            }
            parent
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn quote_to_u8(quote: Option<QuoteStyle>) -> u8 {
    match quote {
        None => 0,
        Some(QuoteStyle::Single) => 1,
        Some(QuoteStyle::Double) => 2,
    }
}

/// Emit `elements` as direct children of `parent_index` and patch the
/// `(head, count)` metadata into the parent's Extended Data block at
/// [`TYPE_LIST_PARENT_SLOT`].
fn emit_node_list_into_parent(
    writer: &mut BinaryWriter<'_>,
    parent_ext: ExtOffset,
    parent_index: u32,
    elements: &[Box<TypeNodeData<'_>>],
) {
    let mut list = writer.begin_node_list_at(parent_ext, TYPE_LIST_PARENT_SLOT);
    for el in elements {
        let child_idx = emit_type_node(writer, el, parent_index);
        writer.record_list_child(&mut list, child_idx);
    }
    writer.finalize_node_list(list);
}

fn emit_borrowed_node_list_into_parent(
    writer: &mut BinaryWriter<'_>,
    parent_ext: ExtOffset,
    parent_index: u32,
    elements: &[&TypeNodeData<'_>],
) {
    let mut list = writer.begin_node_list_at(parent_ext, TYPE_LIST_PARENT_SLOT);
    for el in elements {
        let child_idx = emit_type_node(writer, el, parent_index);
        writer.record_list_child(&mut list, child_idx);
    }
    writer.finalize_node_list(list);
}

fn emit_elements_only<F>(
    writer: &mut BinaryWriter<'_>,
    parent_index: u32,
    span: oxc_span::Span,
    elements: &[Box<TypeNodeData<'_>>],
    write_parent: F,
) -> u32
where
    F: FnOnce(&mut BinaryWriter<'_>, oxc_span::Span, u32) -> (NodeIndex, ExtOffset),
{
    let (idx, ext) = write_parent(writer, span, parent_index);
    let parent = idx.as_u32();
    if !elements.is_empty() {
        emit_node_list_into_parent(writer, ext, parent, elements);
    }
    parent
}

fn emit_borrowed_elements<F>(
    writer: &mut BinaryWriter<'_>,
    parent_index: u32,
    span: oxc_span::Span,
    elements: &[&TypeNodeData<'_>],
    write_parent: F,
) -> u32
where
    F: FnOnce(&mut BinaryWriter<'_>, oxc_span::Span, u32) -> (NodeIndex, ExtOffset),
{
    let (idx, ext) = write_parent(writer, span, parent_index);
    let parent = idx.as_u32();
    if !elements.is_empty() {
        emit_borrowed_node_list_into_parent(writer, ext, parent, elements);
    }
    parent
}

fn emit_single_child<F>(
    writer: &mut BinaryWriter<'_>,
    parent_index: u32,
    span: oxc_span::Span,
    child: &TypeNodeData<'_>,
    write_parent: F,
) -> u32
where
    F: FnOnce(&mut BinaryWriter<'_>, oxc_span::Span, u32, u32) -> u32,
{
    let parent = write_parent(writer, span, parent_index, 0b1);
    emit_type_node(writer, child, parent);
    parent
}

fn emit_modifier_child<F>(
    writer: &mut BinaryWriter<'_>,
    parent_index: u32,
    span: oxc_span::Span,
    child: &TypeNodeData<'_>,
    position: ModifierPosition,
    write_parent: F,
) -> u32
where
    F: FnOnce(&mut BinaryWriter<'_>, oxc_span::Span, u32, u8, u32) -> u32,
{
    let pos = match position {
        ModifierPosition::Prefix => 0,
        ModifierPosition::Suffix => 1,
    };
    let parent = write_parent(writer, span, parent_index, pos, 0b1);
    emit_type_node(writer, child, parent);
    parent
}

fn emit_left_right<F>(
    writer: &mut BinaryWriter<'_>,
    parent_index: u32,
    span: oxc_span::Span,
    left: &TypeNodeData<'_>,
    right: &TypeNodeData<'_>,
    write_parent: F,
) -> u32
where
    F: FnOnce(&mut BinaryWriter<'_>, oxc_span::Span, u32, u32) -> u32,
{
    let parent = write_parent(writer, span, parent_index, 0b11);
    emit_type_node(writer, left, parent);
    emit_type_node(writer, right, parent);
    parent
}

fn emit_function_like<F>(
    writer: &mut BinaryWriter<'_>,
    parent_index: u32,
    span: oxc_span::Span,
    parameters: &[Box<TypeNodeData<'_>>],
    return_type: Option<&TypeNodeData<'_>>,
    type_parameters: &[Box<TypeNodeData<'_>>],
    write_parent: F,
) -> u32
where
    F: FnOnce(&mut BinaryWriter<'_>, oxc_span::Span, u32, u32) -> u32,
{
    let mut bitmask = 0u32;
    if !parameters.is_empty() {
        bitmask |= 0b001;
    }
    if return_type.is_some() {
        bitmask |= 0b010;
    }
    if !type_parameters.is_empty() {
        bitmask |= 0b100;
    }
    let parent = write_parent(writer, span, parent_index, bitmask);

    if !parameters.is_empty() {
        // parameters child is itself a TypeParameterList that owns the
        // elements list metadata in its own Extended Data block.
        let (plist_idx, plist_ext) = write_type_parameter_list(writer, span, parent);
        emit_node_list_into_parent(writer, plist_ext, plist_idx.as_u32(), parameters);
    }
    if let Some(rt) = return_type {
        emit_type_node(writer, rt, parent);
    }
    if !type_parameters.is_empty() {
        let (tplist_idx, tplist_ext) = write_type_parameter_list(writer, span, parent);
        emit_node_list_into_parent(writer, tplist_ext, tplist_idx.as_u32(), type_parameters);
    }
    parent
}
