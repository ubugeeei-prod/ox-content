use super::{Case, case};
use ox_content_transform::*;

pub fn cases() -> Vec<Case> {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/extensions/assets");
    let host = format!("{root}/host.md");
    vec![
        case(
            "includes",
            "<!-- @include: ./snippet.md -->\n\n",
            "Included <strong>snippet</strong>",
            TransformOptions {
                source_path: Some(host.clone()),
                includes: Some(IncludeOptions {
                    root_dir: Some(root.into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
        ),
        case(
            "partials",
            "<!-- @partial: ./partial.md package=\"ox-content\" -->\n\n",
            "Install ox-content today",
            TransformOptions {
                source_path: Some(host.clone()),
                partials: Some(PartialsOptions {
                    root_dir: Some(root.into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
        ),
        case(
            "code_imports",
            "<<< ./snippet.rs\n\n",
            "pub fn example",
            TransformOptions {
                source_path: Some(host.clone()),
                code_imports: Some(CodeImportOptions {
                    root_dir: Some(root.into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
        ),
        case(
            "external_data_tables",
            "```csv-table src=\"./data.csv\"\n```\n\n",
            "<table",
            TransformOptions {
                source_path: Some(host.clone()),
                data_tables: Some(DataTableOptions {
                    root_dir: Some(root.into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
        ),
        case(
            "edit_this_page",
            "# Guide\n\nProse.\n\n",
            "Edit this page",
            TransformOptions {
                source_path: Some(host),
                edit_this_page: Some(EditThisPageOptions {
                    repo_url: Some("https://github.com/example/docs".into()),
                    root_dir: Some(root.into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
        ),
        Case {
            name: "frontmatter",
            source: "---\ntitle: Extension benchmark\nlabels: [one, two]\n---\n\n# Visible\n"
                .into(),
            marker: "Visible",
            options: TransformOptions { frontmatter: Some(true), ..Default::default() },
        },
        case(
            "semantic_footnotes",
            "Sentence[^note].\n\n[^note]: Note text.\n\n",
            "aria-label=\"Footnotes\"",
            TransformOptions {
                footnotes: Some(true),
                semantic_footnotes: Some(true),
                ..Default::default()
            },
        ),
        case(
            "autolink_urls",
            "Visit https://example.com/docs now.\n\n",
            "href=",
            TransformOptions { autolink_urls: Some(true), ..Default::default() },
        ),
    ]
}
