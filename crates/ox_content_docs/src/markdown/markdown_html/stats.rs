use super::super::{doc_kind_plural, EntryStats, DOC_KIND_ORDER};
use crate::string_builder::StringBuilder;

pub(in crate::markdown) fn render_details_controls_html(target_selector: &str) -> String {
    let mut out = StringBuilder::with_capacity(260 + target_selector.len());
    out.push_str("<div class=\"ox-api-controls\" data-ox-api-target=\"");
    out.push_str(target_selector);
    out.push_str("\" role=\"toolbar\" aria-label=\"Reference display controls\">
<button type=\"button\" class=\"ox-api-controls__button\" data-ox-api-toggle=\"expand\">Open all</button>
<button type=\"button\" class=\"ox-api-controls__button\" data-ox-api-toggle=\"collapse\">Close all</button>
</div>");
    out.into_string()
}

pub(in crate::markdown) fn render_stats_html(
    stats: &EntryStats,
    module_count: Option<usize>,
) -> String {
    let mut rendered_items = StringBuilder::new();
    if let Some(module_count) = module_count {
        push_stat_html(&mut rendered_items, "modules", module_count, None);
    }

    push_stat_html(&mut rendered_items, "symbols", stats.entries, None);

    for (index, kind) in DOC_KIND_ORDER.iter().enumerate() {
        let count = stats.by_kind[index];
        if count > 0 {
            push_stat_html(&mut rendered_items, doc_kind_plural(kind), count, None);
        }
    }

    if stats.params > 0 {
        push_stat_html(&mut rendered_items, "parameters", stats.params, None);
    }
    if stats.members > 0 {
        push_stat_html(&mut rendered_items, "members", stats.members, None);
    }
    if stats.returns > 0 {
        push_stat_html(&mut rendered_items, "returns", stats.returns, None);
    }
    if stats.examples > 0 {
        push_stat_html(&mut rendered_items, "examples", stats.examples, None);
    }
    if stats.deprecated > 0 {
        push_stat_html(&mut rendered_items, "deprecated", stats.deprecated, Some("warning"));
    }

    let rendered_items = rendered_items.into_string();
    let mut out = StringBuilder::with_capacity(rendered_items.len() + 80);
    out.push_str("<div class=\"ox-api-stats\" aria-label=\"API reference summary\">\n");
    out.push_str(&rendered_items);
    out.push_str("\n</div>");
    out.into_string()
}

fn push_stat_html(out: &mut StringBuilder, label: &str, value: usize, tone: Option<&str>) {
    if !out.is_empty() {
        out.push_char('\n');
    }
    out.push_str("<span class=\"ox-api-stat");
    if let Some(tone) = tone {
        out.push_str(" ox-api-stat--");
        out.push_str(tone);
    }
    out.push_str("\">\n  <strong>");
    out.push_usize(value);
    out.push_str("</strong>\n  <span>");
    out.push_str(label);
    out.push_str("</span>\n</span>");
}
