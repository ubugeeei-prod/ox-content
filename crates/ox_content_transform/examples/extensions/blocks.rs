use super::{Case, case};
use ox_content_transform::*;

pub fn cases() -> Vec<Case> {
    let mut cases = vec![];
    macro_rules! feature {
        ($field:ident, $source:expr, $marker:expr) => {
            cases.push(case(
                stringify!($field),
                $source,
                $marker,
                TransformOptions { $field: Some(Default::default()), ..Default::default() },
            ));
        };
    }
    feature!(containers, "::: tip Tip title\nBody with **emphasis**.\n:::\n\n", "tip");
    feature!(cards, "::: card\n### Install\nCopy the package.\n:::\n\n", "ox-card");
    feature!(
        steps,
        "::: steps\n### Install\nRun the command.\n### Build\nBuild the site.\n:::\n\n",
        "ox-steps"
    );
    feature!(
        code_groups,
        "::: code-group\n```js [a.js]\nlet a = 1;\n```\n```ts [a.ts]\nlet a: number = 1;\n```\n:::\n\n",
        "<tabs>"
    );
    feature!(
        image_galleries,
        "::: gallery\n![Alt](/a.png \"Caption\")\n![Other](/b.png)\n:::\n\n",
        "ox-image-gallery"
    );
    feature!(
        timelines,
        "::: timeline\n- 2026-08-26 Release\n- 2026-09-01 Updated\n:::\n\n",
        "ox-timeline"
    );
    feature!(conditional_blocks, "::: if true\nVisible\n::: else\nHidden\n:::\n\n", "Visible");
    feature!(
        file_tree,
        "```file-tree\nsrc/\n  index.ts\n  utils/\n    format.ts\n```\n\n",
        "ox-file-tree"
    );
    feature!(data_tables, "```csv-table\nName,Value\nalpha,1\nbeta,2\n```\n\n", "<table");
    cases.push(case("json_tables", "```json-table\n[{\"name\":\"alpha\",\"value\":1},{\"name\":\"beta\",\"value\":2}]\n```\n\n", "<table", TransformOptions { data_tables: Some(DataTableOptions::default()), ..Default::default() }));
    let all_blocks = TransformOptions {
        containers: Some(ContainerOptions::default()),
        cards: Some(CardOptions::default()),
        steps: Some(StepsOptions::default()),
        code_groups: Some(CodeGroupOptions::default()),
        image_galleries: Some(ImageGalleryOptions::default()),
        timelines: Some(TimelineOptions::default()),
        conditional_blocks: Some(ConditionalBlockOptions::default()),
        data_tables: Some(DataTableOptions::default()),
        ..Default::default()
    };
    cases.push(case(
        "all_blocks_tip",
        "::: tip\nOrdinary **content**.\n:::\n\n",
        "tip",
        all_blocks,
    ));
    cases
}
