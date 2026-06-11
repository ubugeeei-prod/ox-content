use std::borrow::Cow;

use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};

use crate::model::ApiDocModule;
use crate::string_builder::StringBuilder;

pub(super) fn annotate_implementation_relationships(docs: &mut [ApiDocModule]) {
    let mut implemented_names = FxHashSet::default();
    for doc in docs.iter() {
        for entry in &doc.entries {
            if entry.kind != "class" || entry.implements.is_empty() {
                continue;
            }
            for implemented in &entry.implements {
                let display_name = heritage_display_name(implemented);
                implemented_names.insert(heritage_lookup_name(display_name.as_ref()).to_string());
            }
        }
    }
    if implemented_names.is_empty() {
        return;
    }

    let mut implementable_members: FxHashMap<String, FxHashSet<String>> =
        FxHashMap::with_capacity_and_hasher(implemented_names.len(), FxBuildHasher);
    for doc in docs.iter() {
        for entry in &doc.entries {
            if !matches!(entry.kind.as_str(), "interface" | "type") {
                continue;
            }
            if !implemented_names.contains(entry.name.as_str()) {
                continue;
            }
            let members =
                entry.members.iter().map(|member| member.name.clone()).collect::<FxHashSet<_>>();
            implementable_members.insert(entry.name.clone(), members);
        }
    }
    if implementable_members.is_empty() {
        return;
    }

    for doc in docs {
        for entry in &mut doc.entries {
            if entry.kind != "class" || entry.implements.is_empty() {
                continue;
            }
            for implemented in &entry.implements {
                let display_name = heritage_display_name(implemented);
                let lookup_name = heritage_lookup_name(display_name.as_ref());
                let Some(interface_members) = implementable_members.get(lookup_name) else {
                    continue;
                };
                for member in &mut entry.members {
                    if interface_members.contains(&member.name) {
                        let mut implementation = StringBuilder::new();
                        implementation.push_str(display_name.as_ref());
                        implementation.push_char('.');
                        implementation.push_str(&member.name);
                        let implementation = implementation.into_string();
                        if !member
                            .implementation_of
                            .iter()
                            .any(|existing| existing == &implementation)
                        {
                            member.implementation_of.push(implementation);
                        }
                    }
                }
            }
        }
    }
}

fn heritage_lookup_name(display_name: &str) -> &str {
    display_name.rsplit_once('.').map_or(display_name, |(_, tail)| tail)
}

fn heritage_display_name(name: &str) -> Cow<'_, str> {
    let trimmed = name.trim();
    if let Some(index) = trimmed.find('<') {
        Cow::Owned(trimmed[..index].trim().to_string())
    } else {
        Cow::Borrowed(trimmed)
    }
}
