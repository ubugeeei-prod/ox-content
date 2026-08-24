#![allow(clippy::disallowed_macros, clippy::disallowed_methods, clippy::disallowed_types)]

use super::scan::{EsmScan, scan_esm};

fn scanned(source: &str) -> EsmScan<'_> {
    scan_esm(source, 0).unwrap_or_else(|| panic!("expected ESM for {source:?}"))
}

#[test]
fn scan_default_import_keeps_source_and_specifier() {
    let esm = scanned("import Foo from \"./Foo\"\n");
    assert_eq!(esm.value, "import Foo from \"./Foo\"");
    assert_eq!(esm.module_source, Some("./Foo"));
    assert_eq!(esm.end, "import Foo from \"./Foo\"\n".len());
}

#[test]
fn scan_named_import_with_alias() {
    let esm = scanned("import { Foo, Bar as Baz } from \"./mod\"");
    assert_eq!(esm.value, "import { Foo, Bar as Baz } from \"./mod\"");
    assert_eq!(esm.module_source, Some("./mod"));
}

#[test]
fn scan_namespace_and_side_effect_imports() {
    let ns = scanned("import * as ns from './ns'");
    assert_eq!(ns.module_source, Some("./ns"));
    let side = scanned("import \"./side\"");
    assert_eq!(side.module_source, Some("./side"));
    assert_eq!(side.value, "import \"./side\"");
}

#[test]
fn scan_export_const_object_and_default() {
    let exported = scanned("export const meta = { title: \"Hi\" }");
    assert_eq!(exported.value, "export const meta = { title: \"Hi\" }");
    assert_eq!(exported.module_source, None);
    assert_eq!(
        scanned("export const compact = {title:\"Hi\"}").value,
        "export const compact = {title:\"Hi\"}"
    );
    assert_eq!(scanned("export default Foo").value, "export default Foo");
}

#[test]
fn scan_keeps_hostile_specifier_as_text() {
    let esm = scanned("import Foo from \"../../../../etc/passwd\"");
    assert_eq!(esm.module_source, Some("../../../../etc/passwd"));
    assert!(esm.value.contains("../../../../etc/passwd"));
}

#[test]
fn scan_rejects_invalid_and_unclosed_imports() {
    for source in [
        "import",
        "import Foo",
        "import Foo from",
        "import { Foo",
        "import Foo from \"./Foo",
        "important thing",
        "importantly Foo from \"./Foo\"",
        "import.meta",
        "export",
        "export const",
        "export const meta =",
        "export const meta = { title: \"Hi\"",
    ] {
        assert!(scan_esm(source, 0).is_none(), "should reject {source:?}");
    }
}

#[test]
fn scan_allows_optional_semicolon_and_single_quotes() {
    assert_eq!(scanned("import Foo from './Foo';").value, "import Foo from './Foo';");
}

#[test]
fn scan_allows_multiline_named_import() {
    let source = "import {\n  Foo,\n  Bar as Baz,\n} from \"./mod\"\n";
    let esm = scanned(source);
    assert!(esm.value.contains("Bar as Baz"));
    assert_eq!(esm.module_source, Some("./mod"));
}

#[test]
fn scan_rejects_blank_line_inside_statement() {
    assert!(scan_esm("import Foo from\n\n\"./Foo\"\n", 0).is_none());
    assert!(scan_esm("import {\n\nFoo } from \"./mod\"\n", 0).is_none());
}
