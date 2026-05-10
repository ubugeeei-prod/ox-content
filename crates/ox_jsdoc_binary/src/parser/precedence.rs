// @author kazuya kawaguchi (a.k.a. kazupon)
// @license MIT
//

//! Pratt parser precedence levels for JSDoc type expressions.
//!
//! Verbatim port of `crates/ox_jsdoc/src/type_parser/precedence.rs`.
//! Higher numeric value = higher precedence = tighter binding.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum Precedence {
    All = 0,
    ParameterList = 1,
    Object = 2,
    KeyValue = 3,
    IndexBrackets = 4,
    Union = 5,
    Intersection = 6,
    Prefix = 7,
    Infix = 8,
    Tuple = 9,
    Symbol = 10,
    Optional = 11,
    Nullable = 12,
    KeyOfTypeOf = 13,
    Function = 14,
    Arrow = 15,
    ArrayBrackets = 16,
    Generic = 17,
    NamePath = 18,
    Parenthesis = 19,
    SpecialTypes = 20,
}
