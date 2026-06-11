use crate::model::ApiDocEntry;

/// Present declaration kinds in `kind_order` (the historical `DOC_KIND_ORDER` or
/// a caller-provided `kindSortOrder`), with any kinds not listed in `kind_order`
/// appended alphabetically. Shared by the module index sections and nav groups.
pub fn ordered_entry_kinds(entries: &[ApiDocEntry], kind_order: &[&str]) -> Vec<String> {
    let mut kinds = Vec::new();
    for kind in kind_order {
        if entries.iter().any(|entry| entry.kind == *kind) {
            kinds.push((*kind).to_string());
        }
    }
    let mut extra = entries
        .iter()
        .map(|entry| entry.kind.clone())
        .filter(|kind| !kind_order.contains(&kind.as_str()))
        .collect::<Vec<_>>();
    extra.sort();
    extra.dedup();
    kinds.extend(extra);
    kinds
}

/// Reorders `(group_title, payload)` sections by a TypeDoc-style `group_order`.
///
/// `None` returns the input unchanged (preserving the caller's default order).
/// Otherwise titles listed before `*` lead in the given order, titles after `*`
/// trail in the given order, and titles not listed are placed at the `*`
/// position (or the end when there is no `*`) sorted alphabetically. Listed
/// titles that are not present are ignored.
pub fn order_by_group_title<T>(
    sections: Vec<(String, T)>,
    group_order: Option<&[String]>,
) -> Vec<(String, T)> {
    let Some(group_order) = group_order else {
        return sections;
    };
    let star = group_order.iter().position(|group| group == "*");
    let (head, tail): (&[String], &[String]) = match star {
        Some(index) => (&group_order[..index], &group_order[index + 1..]),
        None => (group_order, &group_order[group_order.len()..]),
    };

    let mut remaining: Vec<Option<(String, T)>> = sections.into_iter().map(Some).collect();
    let mut result = Vec::with_capacity(remaining.len());

    for title in head {
        if let Some(section) = take_section(&mut remaining, title) {
            result.push(section);
        }
    }

    let mut unspecified = Vec::new();
    for slot in &mut remaining {
        let is_tail = slot.as_ref().is_some_and(|(title, _)| tail.iter().any(|t| t == title));
        if !is_tail {
            if let Some(section) = slot.take() {
                unspecified.push(section);
            }
        }
    }
    unspecified.sort_by(|a, b| a.0.cmp(&b.0));
    result.extend(unspecified);

    for title in tail {
        if let Some(section) = take_section(&mut remaining, title) {
            result.push(section);
        }
    }
    result
}

fn take_section<T>(remaining: &mut [Option<(String, T)>], title: &str) -> Option<(String, T)> {
    remaining
        .iter_mut()
        .find(|slot| slot.as_ref().is_some_and(|(t, _)| t == title))
        .and_then(Option::take)
}
