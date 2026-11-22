use super::*;

mod content;
mod safety;

const DESCRIPTION: &str = "A longer description.\n\n- first point\n- second point";
const REMARKS: &str = "The first paragraph.\n\nThe second paragraph.\n\nSee also:\n- [Guide](https://example.com/guide)\n- [API](https://example.com/api)\n\n```ts\ngreet('Vue')\n```";

fn tag(name: &str, value: &str) -> ApiDocTag {
    ApiDocTag { tag: name.to_string(), value: value.to_string() }
}

fn remarks(value: &str) -> Vec<ApiDocTag> {
    vec![tag("remarks", value)]
}

fn details(description: &str, remarks: &str) -> Vec<ApiDocTag> {
    vec![tag("remarks", remarks), tag("description", description)]
}

fn returns(type_annotation: &str) -> Option<ApiReturnDoc> {
    Some(ApiReturnDoc { type_annotation: type_annotation.to_string(), ..ApiReturnDoc::default() })
}

fn module(entries: Vec<ApiDocEntry>) -> Vec<ApiDocModule> {
    vec![ApiDocModule { file: "mod".to_string(), entries, ..ApiDocModule::default() }]
}

fn page(docs: &[ApiDocModule], options: &MarkdownDocsOptions, name: &str) -> String {
    let file = join2(name, ".md");
    generate_markdown(docs, options)
        .into_iter()
        .find(|(path, _)| path.ends_with(&file))
        .map(|(_, page)| page)
        .expect("the symbol page")
}

fn position(page: &str, needle: &str) -> usize {
    page.find(needle).unwrap_or_else(|| panic!("`{needle}` is not in:\n{page}"))
}

fn assert_in_order(page: &str, needles: &[&str]) {
    let mut rest = page;
    for needle in needles {
        let at = rest
            .find(needle)
            .unwrap_or_else(|| panic!("`{needle}` is missing or out of order in:\n{page}"));
        rest = &rest[at + needle.len()..];
    }
}

fn greet() -> ApiDocEntry {
    let mut entry = test_entry("greet", "function", "/repo/src/greet.ts", "Greets a user.");
    entry.signature = Some("export function greet(name: string): string".to_string());
    entry.params = vec![ApiParamDoc {
        name: "name".to_string(),
        type_annotation: "string".to_string(),
        ..ApiParamDoc::default()
    }];
    entry.returns = returns("string");
    entry.tags = vec![
        tag("see", "https://example.com"),
        tag("remarks", REMARKS),
        tag("since", "1.0.0"),
        tag("description", DESCRIPTION),
    ];
    entry
}

fn members_entry() -> ApiDocEntry {
    let mut entry = test_entry("Options", "interface", "/repo/src/options.ts", "Options.");
    entry.signature = Some("export interface Options".to_string());
    entry.tags = details("Interface description.", "Interface remarks.");
    entry.members = vec![
        ApiDocMember {
            name: "locale".to_string(),
            kind: "property".to_string(),
            description: "The locale.".to_string(),
            type_annotation: Some("string".to_string()),
            tags: details("Property description.", "Property remarks.\n\n- first\n- second"),
            ..ApiDocMember::default()
        },
        ApiDocMember {
            name: "reset".to_string(),
            kind: "method".to_string(),
            description: "Resets it.".to_string(),
            signature: Some("reset(): void".to_string()),
            returns: returns("void"),
            tags: details("Method description.", "Method remarks."),
            ..ApiDocMember::default()
        },
    ];
    entry
}

#[test]
fn typedoc_markdown_renders_detail_tags_after_the_summary() {
    let page = page(&module(vec![greet()]), &markdown_typedoc_options(), "greet");

    assert_no_heading_level_skips(&page);
    assert!(!page.contains("@description"));
    assert!(!page.contains("@remarks"));
    assert_in_order(
        &page,
        &[
            "Greets a user.",
            "## Description",
            "## Remarks",
            "## Since",
            "## Signature",
            "## Parameters",
            "## Returns",
            "## Tags",
        ],
    );
    assert_markdown_snapshot("typedoc_markdown_renders_detail_tags", &page);
}

#[test]
fn typedoc_html_renders_detail_tags_after_the_summary() {
    let page = page(&module(vec![greet()]), &html_typedoc_options(), "greet");

    assert!(!page.contains("@description"));
    assert!(!page.contains("@remarks"));
    assert_in_order(
        &page,
        &[
            "Greets a user.",
            "<h4>Description</h4>",
            "<h4>Remarks</h4>",
            "<h4>Signature</h4>",
            "<h4>Parameters</h4>",
            "<h4>Returns</h4>",
            "<h4>Tags</h4>",
        ],
    );
    assert_markdown_snapshot("typedoc_html_renders_detail_tags", &page);
}

#[test]
fn desc_is_a_synonym_of_description() {
    let mut entry = greet();
    entry.tags = vec![tag("desc", "The short form.")];
    let docs = module(vec![entry]);

    let markdown = page(&docs, &markdown_typedoc_options(), "greet");
    assert!(markdown.contains("## Description\n\nThe short form.\n"));
    assert!(!markdown.contains("@desc"));
    let html = page(&docs, &html_typedoc_options(), "greet");
    assert!(html.contains("<h4>Description</h4>"));
    assert!(!html.contains("@desc"));
}

#[test]
fn a_description_tag_without_a_summary_follows_the_title() {
    let mut entry = greet();
    entry.description = String::new();
    entry.tags = vec![tag("description", "Only the tag.")];

    let page = page(&module(vec![entry]), &markdown_typedoc_options(), "greet");
    assert!(page.contains("# Function: greet()\n\n## Description\n\nOnly the tag.\n"));
}

#[test]
fn declaration_details_come_before_members_and_member_details_keep_their_blocks() {
    let options = MarkdownDocsOptions {
        interface_properties_format: MarkdownDisplayFormat::Table,
        ..markdown_typedoc_options()
    };
    let page = page(&module(vec![members_entry()]), &options, "Options");

    assert_no_heading_level_skips(&page);
    assert_in_order(
        &page,
        &[
            "Options.",
            "## Description",
            "## Remarks",
            "## Signature",
            "## Properties",
            "| `locale` | `string` | The locale. |\n",
            "### locale Description\n\nProperty description.\n",
            "### locale Remarks\n\nProperty remarks.\n\n- first\n- second\n",
            "## Methods",
            "Resets it.",
            "#### Description\n\nMethod description.\n",
            "#### Remarks\n\nMethod remarks.\n",
            "#### Returns",
        ],
    );
}

#[test]
fn html_member_rows_and_details_render_detail_tags_as_blocks() {
    let options = MarkdownDocsOptions {
        interface_properties_format: MarkdownDisplayFormat::Table,
        ..html_typedoc_options()
    };
    let mut entry = members_entry();
    entry.members[0].default_value = Some("'en'".to_string());
    let page = page(&module(vec![entry]), &options, "Options");

    assert_in_order(
        &page,
        &[
            "The locale.",
            "ox-api-entry__member-section--description\"><span>Description</span>",
            "Property description.",
            "ox-api-entry__member-section--remarks\"><span>Remarks</span>",
            "<li>first</li>",
            "ox-api-entry__member-default",
            "Resets it.",
            "ox-api-entry__member-detail-section--description\">\n<h6>Description</h6>",
            "ox-api-entry__member-detail-section--remarks\">\n<h6>Remarks</h6>",
            "<h6>Returns</h6>",
        ],
    );
}

#[test]
fn enum_member_and_return_member_details_are_rendered() {
    let mut color = test_entry("Color", "enum", "/repo/src/color.ts", "Colors.");
    color.members = vec![ApiDocMember {
        name: "Red".to_string(),
        kind: "enumMember".to_string(),
        description: "Red.".to_string(),
        tags: remarks("Enum member remarks."),
        ..ApiDocMember::default()
    }];
    let mut make = test_entry("make", "function", "/repo/src/make.ts", "Makes it.");
    make.returns = Some(ApiReturnDoc {
        type_annotation: "object".to_string(),
        members: vec![
            ApiDocMember {
                name: "value".to_string(),
                kind: "property".to_string(),
                type_annotation: Some("string".to_string()),
                tags: remarks("Return member remarks."),
                ..ApiDocMember::default()
            },
            ApiDocMember {
                name: "[key: string]".to_string(),
                kind: "indexSignature".to_string(),
                signature: Some("[key: string]: string".to_string()),
                type_annotation: Some("string".to_string()),
                tags: remarks("Return index remarks."),
                ..ApiDocMember::default()
            },
        ],
        ..ApiReturnDoc::default()
    });
    let docs = module(vec![color, make]);

    let color = page(&docs, &markdown_typedoc_options(), "Color");
    assert!(color.contains("### Red Remarks\n\nEnum member remarks.\n"));
    let make = page(&docs, &markdown_typedoc_options(), "make");
    assert!(make.contains("#### Remarks\n\nReturn member remarks.\n"));
    assert!(make.contains("Remarks\n\nReturn index remarks.\n"));
    let html = page(&docs, &html_typedoc_options(), "make");
    assert!(html.contains("Return member remarks."));
    assert!(html.contains(
        "<h6>Remarks</h6>\n<div class=\"ox-api-entry__prose\">\n<p>Return index remarks.</p>"
    ));
}

#[test]
fn index_signature_details_come_before_its_since() {
    let mut entry = test_entry("Dictionary", "interface", "/repo/src/dictionary.ts", "Words.");
    entry.members = vec![ApiDocMember {
        name: "[key: string]".to_string(),
        kind: "indexSignature".to_string(),
        signature: Some("[key: string]: string".to_string()),
        type_annotation: Some("string".to_string()),
        tags: vec![tag("since", "1.0.0"), tag("remarks", "Index remarks.")],
        ..ApiDocMember::default()
    }];
    let docs = module(vec![entry]);

    let markdown = page(&docs, &markdown_typedoc_options(), "Dictionary");
    assert_in_order(&markdown, &["Remarks\n\nIndex remarks.", "Since"]);
    assert!(page(&docs, &html_typedoc_options(), "Dictionary").contains("<h6>Remarks</h6>"));
}

#[test]
fn overload_details_follow_each_summary() {
    let overload = |signature: &str, has_body: bool, text: &str| {
        let mut entry = test_entry("pick", "function", "/repo/src/pick.ts", "");
        entry.signature = Some(signature.to_string());
        entry.has_body = has_body;
        entry.returns = returns("string");
        entry.tags = remarks(text);
        entry
    };
    let mut implementation =
        overload("export function pick(a: unknown): string", true, "Symbol remarks.");
    implementation.tags.push(tag("since", "1.0.0"));
    let docs = module(vec![
        overload("export function pick(a: string): string", false, "First remarks."),
        overload("export function pick(a: number): string", false, "Second remarks."),
        implementation,
    ]);

    let markdown = page(&docs, &markdown_typedoc_options(), "pick");
    assert_in_order(
        &markdown,
        &[
            "## Remarks\n\nSymbol remarks.",
            "## Since",
            "## Call Signature",
            "### Remarks\n\nFirst remarks.",
            "### Returns",
            "## Call Signature",
            "### Remarks\n\nSecond remarks.",
            "### Returns",
        ],
    );

    let html = page(&docs, &html_typedoc_options(), "pick");
    assert_in_order(
        &html,
        &[
            "Symbol remarks.",
            "<h4>Call Signature</h4>",
            "First remarks.",
            "<h4>Returns</h4>",
            "<h4>Call Signature</h4>",
            "Second remarks.",
        ],
    );
}

#[test]
fn module_details_render_on_the_module_page() {
    let docs = vec![ApiDocModule {
        file: "mod".to_string(),
        description: "A module.".to_string(),
        tags: details("Module description.", "Module remarks.\n\n- item"),
        entries: vec![test_entry("run", "function", "/repo/src/run.ts", "Run.")],
        ..ApiDocModule::default()
    }];

    let markdown = generate_markdown(&docs, &markdown_typedoc_options());
    let index = markdown.get("mod/index.md").expect("the module page");
    assert!(index.contains(
        "A module.\n\n## Description\n\nModule description.\n\n## Remarks\n\nModule remarks.\n\n- item\n"
    ));
}
