use serde_json::Value;
use std::borrow::Cow;
use std::collections::HashMap;
use std::hash::BuildHasher;

use crate::features;
use crate::features::TransformFeatureOptions;

fn legacy<'a, S: BuildHasher>(
    mut current: Cow<'a, str>,
    options: &TransformFeatureOptions,
    frontmatter: &HashMap<String, Value, S>,
    errors: &mut Vec<String>,
) -> Cow<'a, str> {
    if let Some(conditionals) = &options.conditional_blocks {
        current = Cow::Owned(features::conditional_blocks::transform(
            &current,
            conditionals,
            frontmatter,
            errors,
        ));
    }
    if let Some(galleries) = &options.image_galleries {
        current = Cow::Owned(features::image_galleries::transform(&current, galleries, errors));
    }
    if let Some(timelines) = &options.timelines {
        current = Cow::Owned(features::timelines::transform(&current, timelines, errors));
    }
    if options.cards.is_some() {
        current = Cow::Owned(features::cards::transform(&current));
    }
    if options.steps.is_some() {
        current = Cow::Owned(features::steps::transform(&current));
    }
    if options.code_groups.is_some() {
        current = Cow::Owned(features::code_groups::transform(&current, errors));
    }
    if let Some(containers) = &options.containers {
        current = Cow::Owned(features::containers::transform(&current, containers));
    }
    current
}

#[test]
fn guarded_passes_match_unconditional_pipeline_for_every_option_combination() {
    let sources = [
        "Ordinary prose, no block markers.",
        "::: unknown\nBody\n:::\n",
        "::: TIP\nBody\n:::\n",
        "::: CARD\n### Title\nBody\n:::\n",
        "::: STEPS\n### One\nBody\n:::\n",
        "::: CODE-GROUP\n```js [a.js]\nx\n```\n:::\n",
        "::: gallery\n![Alt](/a.png)\n:::\n",
        "::: timeline\n- 2026-09-01 Release\n:::\n",
        "::: if true\n::: card\n### Title\nBody\n:::\n:::\n",
        "```md\n::: card\nHidden\n:::\n```\n",
        "::: tip\n::: steps\n### One\nText\n:::\n:::\n",
        "::: code-group\nNot a fence.\n:::\n",
        "İ 日本語 :: card steps code-group if gallery timeline",
    ];
    let frontmatter = HashMap::<String, Value>::new();
    for mask in 0u8..128 {
        let mut options = crate::TransformOptions::default();
        macro_rules! flag {
            ($bit:expr, $field:ident) => {
                if mask & (1 << $bit) != 0 {
                    options.$field = Some(Default::default());
                }
            };
        }
        flag!(0, conditional_blocks);
        flag!(1, image_galleries);
        flag!(2, timelines);
        flag!(3, cards);
        flag!(4, steps);
        flag!(5, code_groups);
        flag!(6, containers);
        let options = TransformFeatureOptions::from_options(&options);
        for source in sources {
            let mut actual_errors = Vec::new();
            let mut expected_errors = Vec::new();
            let actual =
                super::apply(Cow::Borrowed(source), &options, &frontmatter, &mut actual_errors);
            let expected =
                legacy(Cow::Borrowed(source), &options, &frontmatter, &mut expected_errors);
            assert_eq!(actual, expected, "options {mask} on {source}");
            assert_eq!(actual_errors, expected_errors, "options {mask} on {source}");
        }
    }
}

#[test]
fn enabled_data_tables_borrow_documents_without_table_fences() {
    let options = TransformFeatureOptions::from_options(&crate::TransformOptions {
        data_tables: Some(crate::DataTableOptions::default()),
        ..Default::default()
    });
    for source in
        ["# Prose\n\nOrdinary content.", "```rust\nlet a = 1;\n```\n", "日本語の文章です。"]
    {
        let result = features::preprocess_markdown(source, &options);
        assert!(matches!(result.source, Cow::Borrowed(_)));
        assert_eq!(result.source, source);
        assert!(result.errors.is_empty());
    }
}
