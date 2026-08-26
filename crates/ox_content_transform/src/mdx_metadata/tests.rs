use ox_content_allocator::Allocator;
use ox_content_parser::{Parser, ParserOptions};

use super::{
    MdxImport, MdxImportSpecifier, MdxImportSpecifierKind, MdxMetadata, extract_mdx_metadata,
    specifier,
};

fn parse_doc(source: &str) -> MdxMetadata {
    let allocator = Allocator::for_source_len(source.len());
    let document = Parser::with_options(&allocator, source, ParserOptions::mdx()).parse().unwrap();
    extract_mdx_metadata(&document)
}

fn spec(imported: &str, local: &str, kind: MdxImportSpecifierKind) -> MdxImportSpecifier {
    specifier(imported, local, kind)
}

#[test]
fn collects_default_named_and_namespace_imports() {
    let meta = parse_doc(
        "import Alert from './Alert'\n\
         import { Chart, Foo as Bar } from './Chart'\n\
         import * as Icons from './icons'\n\
         import Def, { A } from './mixed'\n\
         import './side-effect'\n",
    );

    assert_eq!(
        meta.imports,
        vec![
            MdxImport {
                source: "./Alert".into(),
                specifiers: vec![spec("default", "Alert", MdxImportSpecifierKind::Default)],
            },
            MdxImport {
                source: "./Chart".into(),
                specifiers: vec![
                    spec("Chart", "Chart", MdxImportSpecifierKind::Named),
                    spec("Foo", "Bar", MdxImportSpecifierKind::Named),
                ],
            },
            MdxImport {
                source: "./icons".into(),
                specifiers: vec![spec("*", "Icons", MdxImportSpecifierKind::Namespace)],
            },
            MdxImport {
                source: "./mixed".into(),
                specifiers: vec![
                    spec("default", "Def", MdxImportSpecifierKind::Default),
                    spec("A", "A", MdxImportSpecifierKind::Named),
                ],
            },
            MdxImport { source: "./side-effect".into(), specifiers: vec![] },
        ]
    );
}

#[test]
fn collects_export_names_without_evaluating_initializers() {
    let meta = parse_doc(
        "export const title = (() => { throw new Error('nope') })()\n\
         export function helper() {}\n\
         export default function Page() {}\n\
         export { a, b as c }\n\
         export * as ns from './mod'\n",
    );
    assert_eq!(meta.exports, vec!["title", "helper", "default", "a", "c", "ns"]);
}

#[test]
fn collects_type_only_imports_and_export_declarations() {
    let meta = parse_doc(
        "import type Props from './Props'\n\
         import { type ChartData, Series as ChartSeries } from './chart'\n\
         export type PageData = { title: string }\n\
         export interface TocEntry { text: string }\n\
         export enum LayoutKind { Docs }\n\
         export async function load() {}\n\
         export class ViewModel {}\n",
    );

    assert_eq!(
        meta.imports,
        vec![
            MdxImport {
                source: "./Props".into(),
                specifiers: vec![spec("default", "Props", MdxImportSpecifierKind::Default)],
            },
            MdxImport {
                source: "./chart".into(),
                specifiers: vec![
                    spec("ChartData", "ChartData", MdxImportSpecifierKind::Named),
                    spec("Series", "ChartSeries", MdxImportSpecifierKind::Named),
                ],
            },
        ]
    );
    assert_eq!(meta.exports, vec!["PageData", "TocEntry", "LayoutKind", "load", "ViewModel"]);
}

#[test]
fn collects_unique_component_names_and_skips_fragments() {
    let meta = parse_doc(
        "<Alert />\n\
         Hello <Badge>ok</Badge>\n\
         <Icons.Star />\n\
         <Alert title=\"again\" />\n\
         <>ignored</>\n",
    );
    assert_eq!(meta.components, vec!["Alert", "Badge", "Icons.Star"]);
}

#[test]
fn ignores_imports_inside_fences() {
    let meta = parse_doc("```js\nimport foo from 'bar'\n```\n\n<Alert />\n");
    assert!(meta.imports.is_empty());
    assert_eq!(meta.components, vec!["Alert"]);
}

#[test]
fn stores_hostile_import_source_without_executing() {
    let meta = parse_doc("import x from \"<script>alert(1)</script>\"\n");
    assert_eq!(meta.imports[0].source, "<script>alert(1)</script>");
}

#[test]
fn empty_when_mdx_nodes_are_absent() {
    let allocator = Allocator::for_source_len(7);
    let document = Parser::new(&allocator, "# Hello").parse().unwrap();
    let meta = extract_mdx_metadata(&document);
    assert_eq!(meta, MdxMetadata::default());
}
