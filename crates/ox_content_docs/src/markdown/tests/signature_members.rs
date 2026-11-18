use super::*;

fn param(name: &str, type_annotation: &str, description: &str) -> ApiParamDoc {
    ApiParamDoc {
        name: name.to_string(),
        type_annotation: type_annotation.to_string(),
        description: description.to_string(),
        ..ApiParamDoc::default()
    }
}

fn returns(type_annotation: &str, description: &str) -> Option<ApiReturnDoc> {
    Some(ApiReturnDoc {
        type_annotation: type_annotation.to_string(),
        description: description.to_string(),
        members: Vec::new(),
    })
}

fn signature_member(kind: &str, signature: &str, description: &str, line: u32) -> ApiDocMember {
    ApiDocMember {
        kind: kind.to_string(),
        description: description.to_string(),
        signature: Some(signature.to_string()),
        line,
        end_line: line,
        ..ApiDocMember::default()
    }
}

/// An interface with two call signatures (one generic), a construct signature,
/// a property and a method, as `extractDocsFromEntryPoints()` returns them.
fn translate_docs() -> Vec<ApiDocModule> {
    let mut generic = signature_member(
        "callSignature",
        "<Key extends string>(key: Key): string",
        "Translate a key.",
        3,
    );
    generic.type_parameters = vec![ApiTypeParamDoc {
        name: "Key".to_string(),
        constraint: Some("string".to_string()),
        description: "The key type.".to_string(),
        ..ApiTypeParamDoc::default()
    }];
    generic.params = vec![param("key", "Key", "A key.")];
    generic.returns = returns("string", "A message.");

    let mut plural = signature_member(
        "callSignature",
        "(key: string, plural: number): string",
        "Translate with a plural.",
        5,
    );
    plural.params = vec![param("key", "string", "A key."), param("plural", "number", "A count.")];
    plural.returns = returns("string", "");

    let mut construct =
        signature_member("constructSignature", "new (locale: string): Translate", "Create it.", 7);
    construct.params = vec![param("locale", "string", "")];
    construct.returns = returns("Translate", "");

    let mut reset = signature_member("method", "reset(): void", "Reset it.", 11);
    reset.name = "reset".to_string();
    reset.returns = returns("void", "");

    vec![ApiDocModule {
        file: "mod".to_string(),
        entries: vec![ApiDocEntry {
            name: "Translate".to_string(),
            kind: "interface".to_string(),
            description: "Translate messages.".to_string(),
            file: "/repo/src/translate.ts".to_string(),
            end_line: 12,
            signature: Some("export interface Translate".to_string()),
            members: vec![
                generic,
                plural,
                construct,
                ApiDocMember {
                    name: "locale".to_string(),
                    kind: "property".to_string(),
                    description: "The locale.".to_string(),
                    type_annotation: Some("string".to_string()),
                    line: 9,
                    end_line: 9,
                    ..ApiDocMember::default()
                },
                reset,
            ],
            ..ApiDocEntry::default()
        }],
        ..ApiDocModule::default()
    }]
}

fn translate_page(options: &MarkdownDocsOptions) -> String {
    generate_markdown(&translate_docs(), options)
        .into_iter()
        .find(|(path, _)| path.ends_with("Translate.md"))
        .map(|(_, page)| page)
        .expect("the Translate page")
}

#[test]
fn typedoc_markdown_renders_call_and_construct_signatures() {
    let page = translate_page(&markdown_typedoc_options());

    assert_no_heading_level_skips(&page);
    assert_markdown_snapshot("typedoc_markdown_renders_call_and_construct_signatures", &page);
}

#[test]
fn typedoc_html_renders_call_and_construct_signature_groups() {
    let page = translate_page(&html_typedoc_options());

    assert_markdown_snapshot("typedoc_html_renders_call_and_construct_signature_groups", &page);
}

#[test]
fn flat_anchors_name_signatures_by_kind() {
    let markdown = generate_markdown(
        &translate_docs(),
        &MarkdownDocsOptions {
            render_style: MarkdownRenderStyle::Html,
            ..MarkdownDocsOptions::default()
        },
    );
    let page = markdown.values().find(|page| page.contains("id=\"translate\"")).unwrap();

    assert!(page.contains("id=\"translate-call-signature\""));
    assert!(page.contains("id=\"translate-constructor\""));
    assert!(!page.contains("id=\"translate-\""));
}

#[test]
fn alphabetical_sort_keeps_call_signatures_in_source_order() {
    let page = translate_page(&MarkdownDocsOptions {
        sort: Some(vec!["alphabetical".to_string()]),
        ..markdown_typedoc_options()
    });

    let generic = page.find("Translate<Key extends string>(key: Key): string;").unwrap();
    let plural = page.find("Translate(key: string, plural: number): string;").unwrap();
    assert!(generic < plural);
}

#[test]
fn return_members_label_call_signatures() {
    let mut call = signature_member("callSignature", "(value: number): string", "Format it.", 3);
    call.params = vec![param("value", "number", "")];
    call.returns = returns("string", "");
    let docs = vec![ApiDocModule {
        file: "mod".to_string(),
        entries: vec![ApiDocEntry {
            name: "createFormatter".to_string(),
            kind: "function".to_string(),
            description: "Create a formatter.".to_string(),
            file: "/repo/src/format.ts".to_string(),
            end_line: 8,
            signature: Some("export function createFormatter(): Formatter".to_string()),
            returns: Some(ApiReturnDoc {
                type_annotation: "object".to_string(),
                description: "A formatter.".to_string(),
                members: vec![call],
            }),
            ..ApiDocEntry::default()
        }],
        ..ApiDocModule::default()
    }];

    let markdown = generate_markdown(&docs, &markdown_typedoc_options());
    let page = markdown.iter().find(|(path, _)| path.ends_with("createFormatter.md")).unwrap().1;
    assert!(page.contains("### Call Signature\n\n```ts\n(value: number): string;\n```"));

    let html = generate_markdown(&docs, &html_typedoc_options());
    let page = html.iter().find(|(path, _)| path.ends_with("createFormatter.md")).unwrap().1;
    assert!(page.contains("<h5>Call Signature</h5>"));
}

#[test]
fn signature_members_are_not_link_targets() {
    let symbols = build_symbol_map(&translate_docs(), &markdown_typedoc_options(), None);

    assert!(symbols.contains_key("Translate.locale"));
    assert!(!symbols.contains_key("Translate."));
}

#[test]
fn nested_member_tables_label_call_signatures() {
    let mut call = signature_member("callSignature", "(event: string): void", "Call it.", 4);
    call.returns = returns("void", "");
    let mut reset = signature_member("method", "reset(): void", "Reset it.", 5);
    reset.name = "reset".to_string();
    reset.returns = returns("void", "");
    let index = ApiDocMember {
        name: "[key: string]".to_string(),
        kind: "indexSignature".to_string(),
        signature: Some("[key: string]: string".to_string()),
        type_annotation: Some("string".to_string()),
        line: 6,
        end_line: 6,
        ..ApiDocMember::default()
    };
    let docs = vec![ApiDocModule {
        file: "mod".to_string(),
        entries: vec![ApiDocEntry {
            name: "Options".to_string(),
            kind: "interface".to_string(),
            description: "Options.".to_string(),
            file: "/repo/src/options.ts".to_string(),
            end_line: 8,
            signature: Some("export interface Options".to_string()),
            members: vec![ApiDocMember {
                name: "handler".to_string(),
                kind: "property".to_string(),
                description: "A handler.".to_string(),
                type_annotation: Some("{ (event: string): void }".to_string()),
                members: vec![call, reset, index],
                line: 3,
                end_line: 6,
                ..ApiDocMember::default()
            }],
            ..ApiDocEntry::default()
        }],
        ..ApiDocModule::default()
    }];

    let html = generate_markdown(
        &docs,
        &MarkdownDocsOptions {
            property_members_format: MarkdownDisplayFormat::Table,
            ..html_typedoc_options()
        },
    );
    let page = html.iter().find(|(path, _)| path.ends_with("Options.md")).unwrap().1;

    assert!(page.contains("<td>Call Signature</td>"));
    assert!(page.contains(">(event: string): void</code>"));
    // Other nested members keep their type, not their whole signature.
    assert!(page.contains(">void</code>"));
    assert!(!page.contains(">reset(): void</code>"));
    assert!(page.contains(">string</code>"));
    assert!(!page.contains(">[key: string]: string</code>"));
}
