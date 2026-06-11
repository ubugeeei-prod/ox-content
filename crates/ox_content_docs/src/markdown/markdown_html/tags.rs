use super::super::{is_structured_tag, MarkdownLinkContext};
use super::inline::{escape_html, render_doc_inline_html};
use crate::model::ApiDocTag;
use crate::string_builder::StringBuilder;

pub(super) fn render_tag_list_html(
    tags: &[ApiDocTag],
    context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let mut items = StringBuilder::new();
    for tag in tags {
        // Structured tags (lifecycle / since / version) are surfaced as badges, so
        // exclude them here to avoid duplicating them in the generic tag list.
        if is_structured_tag(&tag.tag) {
            continue;
        }
        items.push_str("<li><span class=\"ox-api-entry__tag-name\">@");
        items.push_str(&escape_html(&tag.tag));
        items.push_str("</span><span class=\"ox-api-entry__tag-value\">");
        items.push_str(&render_doc_inline_html(&tag.value, context));
        items.push_str("</span></li>");
    }
    let items = items.into_string();

    let mut out = StringBuilder::with_capacity(items.len() + 115);
    out.push_str(
        "<div class=\"ox-api-entry__section ox-api-entry__section--tags\">
<h4>Tags</h4>
<ul class=\"ox-api-entry__tags\">",
    );
    out.push_str(&items);
    out.push_str(
        "</ul>
</div>",
    );
    out.into_string()
}
