use std::cmp::Ordering;

use crate::model::{ApiDocEntry, ApiDocMember, ApiDocModule, ApiReturnDoc};

use super::{file_name, MarkdownDocsOptions, DOC_KIND_ORDER};

/// A TypeDoc-compatible sort strategy that maps onto ox-content's data model.
/// Strategies whose required data is unavailable (`enum-value-*`, `documents-*`)
/// are not represented here and are dropped during parsing, so they act as no-ops.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortStrategy {
    SourceOrder,
    Alphabetical,
    Kind,
    StaticFirst,
    InstanceFirst,
    RequiredFirst,
    Visibility,
    ExternalLast,
}

/// Parses TypeDoc `sort` strategy names, dropping unsupported/unknown ones so they
/// fall through to the next strategy (matching TypeDoc's tie-breaking semantics).
/// `alphabetical-ignoring-documents` maps to `Alphabetical` (ox-content has no
/// document reflections).
pub fn parse_sort_strategies(raw: &[String]) -> Vec<SortStrategy> {
    raw.iter()
        .filter_map(|strategy| match strategy.as_str() {
            "source-order" => Some(SortStrategy::SourceOrder),
            "alphabetical" | "alphabetical-ignoring-documents" => Some(SortStrategy::Alphabetical),
            "kind" => Some(SortStrategy::Kind),
            "static-first" => Some(SortStrategy::StaticFirst),
            "instance-first" => Some(SortStrategy::InstanceFirst),
            "required-first" => Some(SortStrategy::RequiredFirst),
            "visibility" => Some(SortStrategy::Visibility),
            "external-last" => Some(SortStrategy::ExternalLast),
            _ => None,
        })
        .collect()
}

/// The declaration-kind ranking used by the `kind` strategy and as the base order
/// for module index sections / nav groups: `kind_sort_order` when provided, else
/// the historical [`DOC_KIND_ORDER`].
pub fn kind_order_slice(kind_sort_order: Option<&[String]>) -> Vec<&str> {
    match kind_sort_order {
        Some(order) => order.iter().map(String::as_str).collect(),
        None => DOC_KIND_ORDER.to_vec(),
    }
}

/// Rank of `kind` within `kind_order`; unlisted kinds sort after listed ones.
fn kind_rank(kind: &str, kind_order: &[&str]) -> usize {
    kind_order.iter().position(|candidate| *candidate == kind).unwrap_or(kind_order.len())
}

/// Case-insensitive name comparison with a case-sensitive tiebreak. Used as the
/// final, always-decisive tiebreak so sorts are total and stable.
fn compare_names(a: &str, b: &str) -> Ordering {
    a.to_lowercase().cmp(&b.to_lowercase()).then_with(|| a.cmp(b))
}

/// Compares two entries under an ordered list of sort strategies (later strategies
/// only break ties). A trailing name comparison guarantees a total order.
pub fn compare_entries(
    a: &ApiDocEntry,
    b: &ApiDocEntry,
    sort: &[SortStrategy],
    kind_order: &[&str],
) -> Ordering {
    for strategy in sort {
        let ordering = match strategy {
            SortStrategy::SourceOrder => a.line.cmp(&b.line),
            SortStrategy::Alphabetical => compare_names(&a.name, &b.name),
            SortStrategy::Kind => {
                kind_rank(&a.kind, kind_order).cmp(&kind_rank(&b.kind, kind_order))
            }
            SortStrategy::ExternalLast => a.file.is_empty().cmp(&b.file.is_empty()),
            SortStrategy::StaticFirst
            | SortStrategy::InstanceFirst
            | SortStrategy::RequiredFirst
            | SortStrategy::Visibility => Ordering::Equal,
        };
        if ordering != Ordering::Equal {
            return ordering;
        }
    }
    compare_names(&a.name, &b.name)
}

fn compare_members(
    a: &ApiDocMember,
    b: &ApiDocMember,
    sort: &[SortStrategy],
    kind_order: &[&str],
) -> Ordering {
    for strategy in sort {
        let ordering = match strategy {
            SortStrategy::SourceOrder => a.line.cmp(&b.line),
            SortStrategy::Alphabetical => compare_names(&a.name, &b.name),
            SortStrategy::Kind => {
                kind_rank(&a.kind, kind_order).cmp(&kind_rank(&b.kind, kind_order))
            }
            SortStrategy::StaticFirst => b.r#static.cmp(&a.r#static),
            SortStrategy::InstanceFirst => a.r#static.cmp(&b.r#static),
            SortStrategy::RequiredFirst => a.optional.cmp(&b.optional),
            SortStrategy::Visibility => a.private.cmp(&b.private),
            SortStrategy::ExternalLast => Ordering::Equal,
        };
        if ordering != Ordering::Equal {
            return ordering;
        }
    }
    compare_names(&a.name, &b.name)
}

pub(super) fn sort_extracted_docs(
    docs: &[ApiDocModule],
    options: &MarkdownDocsOptions,
) -> Vec<ApiDocModule> {
    let mut sorted = docs.to_vec();
    let strategies = options.sort.as_deref().map(parse_sort_strategies);
    let kind_order = kind_order_slice(options.kind_sort_order.as_deref());

    for doc in &mut sorted {
        if let Some(strategies) = &strategies {
            doc.entries.sort_by(|a, b| compare_entries(a, b, strategies, &kind_order));
            for entry in &mut doc.entries {
                entry.members.sort_by(|a, b| compare_members(a, b, strategies, &kind_order));
                sort_api_doc_return_members(entry, Some(strategies), &kind_order);
            }
        } else {
            doc.entries.sort_by_cached_key(|entry| (entry.name.to_lowercase(), entry.name.clone()));
            for entry in &mut doc.entries {
                sort_api_doc_members(entry);
                sort_api_doc_return_members(entry, None, &kind_order);
            }
        }
    }

    if options.sort_entry_points {
        sorted.sort_by_cached_key(|module| {
            let name = file_name(&module.file);
            (name.to_lowercase(), name)
        });
    }
    sorted
}

fn sort_api_doc_members(entry: &mut ApiDocEntry) {
    if matches!(entry.kind.as_str(), "class" | "interface" | "type") {
        entry
            .members
            .sort_by_cached_key(|member| (member.name.to_lowercase(), member.name.clone()));
    }
}

fn sort_api_doc_return_members(
    entry: &mut ApiDocEntry,
    strategies: Option<&[SortStrategy]>,
    kind_order: &[&str],
) {
    sort_return_members(entry.returns.as_mut(), strategies, kind_order);
    for member in &mut entry.members {
        sort_return_members(member.returns.as_mut(), strategies, kind_order);
    }
}

fn sort_return_members(
    returns: Option<&mut ApiReturnDoc>,
    strategies: Option<&[SortStrategy]>,
    kind_order: &[&str],
) {
    let Some(returns) = returns else {
        return;
    };
    if let Some(strategies) = strategies {
        returns.members.sort_by(|a, b| compare_members(a, b, strategies, kind_order));
    } else {
        returns
            .members
            .sort_by_cached_key(|member| (member.name.to_lowercase(), member.name.clone()));
    }
    for member in &mut returns.members {
        sort_return_members(member.returns.as_mut(), strategies, kind_order);
    }
}
