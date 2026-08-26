use serde_json::Value;
use std::borrow::Cow;
use std::collections::HashMap;
use std::hash::BuildHasher;

use super::TransformFeatureOptions;

pub(super) fn apply<'a, S: BuildHasher>(
    mut current: Cow<'a, str>,
    options: &TransformFeatureOptions,
    frontmatter: &HashMap<String, Value, S>,
    errors: &mut Vec<String>,
) -> Cow<'a, str> {
    if let Some(conditionals) = &options.conditional_blocks {
        current = Cow::Owned(super::conditional_blocks::transform(
            &current,
            conditionals,
            frontmatter,
            errors,
        ));
    }
    if let Some(galleries) = &options.image_galleries {
        current = Cow::Owned(super::image_galleries::transform(&current, galleries, errors));
    }
    if let Some(timelines) = &options.timelines {
        current = Cow::Owned(super::timelines::transform(&current, timelines, errors));
    }
    if options.cards.is_some() {
        current = Cow::Owned(super::cards::transform(&current));
    }
    if options.steps.is_some() {
        current = Cow::Owned(super::steps::transform(&current));
    }
    if options.code_groups.is_some() {
        current = Cow::Owned(super::code_groups::transform(&current, errors));
    }
    if let Some(containers) = &options.containers {
        current = Cow::Owned(super::containers::transform(&current, containers));
    }
    current
}
