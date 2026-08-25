use napi_derive::napi;

use crate::JsSectionIndexItem;

/// Renders escaped section-index listing HTML. Hostile hrefs are dropped.
#[napi(js_name = "renderSsgSectionIndex")]
pub fn render_ssg_section_index(
    title: String,
    items: Vec<JsSectionIndexItem>,
    style: String,
) -> String {
    let items: Vec<ox_content_ssg::SectionIndexItem> = items
        .into_iter()
        .map(|item| ox_content_ssg::SectionIndexItem {
            title: item.title,
            href: item.href,
            description: item.description,
        })
        .collect();
    ox_content_ssg::render_section_index(
        &title,
        &items,
        ox_content_ssg::SectionIndexStyle::parse(&style),
    )
}
