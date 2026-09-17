use super::*;

#[test]
fn component_symbol_range_hugs_the_tag_name() {
    let document = TextDocumentState::new("<Alert tone=\"info\">Hello</Alert>\n".into());
    let symbol = symbol_at(&document, Position { line: 0, character: 2 }).expect("symbol");
    match symbol {
        SymbolAt::Component { name, range } => {
            assert_eq!(name, "Alert");
            assert_eq!(range.start.character, 1);
            assert_eq!(range.end.character, 6);
        }
        other @ SymbolAt::Attribute { .. } => panic!("expected component, got {other:?}"),
    }
}

#[test]
fn attribute_symbol_range_hugs_the_prop_name() {
    let document = TextDocumentState::new("<Alert tone=\"info\">Hello</Alert>\n".into());
    let symbol = symbol_at(&document, Position { line: 0, character: 8 }).expect("symbol");
    match symbol {
        SymbolAt::Attribute { component, name, range } => {
            assert_eq!(component, "Alert");
            assert_eq!(name, "tone");
            assert_eq!(range.start.character, 7);
            assert_eq!(range.end.character, 11);
        }
        other @ SymbolAt::Component { .. } => panic!("expected attribute, got {other:?}"),
    }
}

#[test]
fn multiline_tag_attribute_symbol_is_supported() {
    let document = TextDocumentState::new("<Alert\n  tone=\"info\"\n/>\n".into());
    let symbol = symbol_at(&document, Position { line: 1, character: 4 }).expect("symbol");
    assert!(matches!(symbol, SymbolAt::Attribute { component: "Alert", name: "tone", .. }));
}
