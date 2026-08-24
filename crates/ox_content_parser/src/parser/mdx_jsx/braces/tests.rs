#![allow(clippy::disallowed_macros, clippy::disallowed_methods, clippy::disallowed_types)]

use super::{scan_balanced_braces, skip_braces};

#[test]
fn scan_keeps_inner_source_without_braces() {
    let (value, end) = scan_balanced_braces("{items.map}", 0).expect("balanced");
    assert_eq!(value, "items.map");
    assert_eq!(end, 11);
}

#[test]
fn scan_skips_block_comment_with_braces_inside() {
    let (value, _) = scan_balanced_braces("{/* hide } */}", 0).expect("comment");
    assert_eq!(value, "/* hide } */");
}

#[test]
fn scan_skips_nested_braces_and_strings() {
    let source = "{...{html:\"<script>\"}}";
    let (value, end) = scan_balanced_braces(source, 0).expect("nested");
    assert_eq!(value, "...{html:\"<script>\"}");
    assert_eq!(end, source.len());
}

#[test]
fn unclosed_brace_or_comment_is_none() {
    assert!(skip_braces(b"{items.map", 0).is_none());
    assert!(skip_braces(b"{/* hide", 0).is_none());
    assert!(skip_braces(b"{foo /* bar", 0).is_none());
}
