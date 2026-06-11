use super::super::{doc_kind_plural, EntryStats, DOC_KIND_ORDER};
use crate::string_builder::StringBuilder;

/// Renders the per-page stats summary as a single italic Markdown line.
pub(in crate::markdown) fn render_stats_markdown(
    stats: &EntryStats,
    module_count: Option<usize>,
) -> String {
    let mut out = StringBuilder::new();
    let mut has_parts = false;
    out.push_char('_');
    if let Some(module_count) = module_count {
        push_stat_part(&mut out, &mut has_parts, module_count, "modules");
    }
    push_stat_part(&mut out, &mut has_parts, stats.entries, "symbols");
    for (index, kind) in DOC_KIND_ORDER.iter().enumerate() {
        let count = stats.by_kind[index];
        if count > 0 {
            push_stat_part(&mut out, &mut has_parts, count, doc_kind_plural(kind));
        }
    }
    if stats.params > 0 {
        push_stat_part(&mut out, &mut has_parts, stats.params, "parameters");
    }
    if stats.members > 0 {
        push_stat_part(&mut out, &mut has_parts, stats.members, "members");
    }
    if stats.returns > 0 {
        push_stat_part(&mut out, &mut has_parts, stats.returns, "returns");
    }
    if stats.examples > 0 {
        push_stat_part(&mut out, &mut has_parts, stats.examples, "examples");
    }
    if stats.deprecated > 0 {
        push_stat_part(&mut out, &mut has_parts, stats.deprecated, "deprecated");
    }
    out.push_char('_');
    out.into_string()
}

fn push_stat_part(out: &mut StringBuilder, has_parts: &mut bool, count: usize, label: &str) {
    if *has_parts {
        out.push_str(" · ");
    }
    out.push_usize(count);
    out.push_char(' ');
    out.push_str(label);
    *has_parts = true;
}
