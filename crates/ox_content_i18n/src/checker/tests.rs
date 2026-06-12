use rustc_hash::FxHashSet;

use super::*;
use crate::dictionary::Dictionary;
use crate::key::KeyPath;
use crate::locale::Locale;

fn make_dict_set() -> DictionarySet {
    let mut set = DictionarySet::new();
    set.set_default_locale(Locale::new("en").unwrap());

    let mut en = Dictionary::new();
    en.insert(KeyPath::new("common.greeting"), "Hello {$name}".to_string());
    en.insert(KeyPath::new("common.farewell"), "Goodbye".to_string());
    set.insert(Locale::new("en").unwrap(), en);

    let mut ja = Dictionary::new();
    ja.insert(KeyPath::new("common.greeting"), "こんにちは {$name}".to_string());
    set.insert(Locale::new("ja").unwrap(), ja);

    set
}

#[test]
fn missing_keys() {
    let dict_set = make_dict_set();
    let mut used = FxHashSet::default();
    used.insert("common.greeting".to_string());
    used.insert("common.unknown".to_string());

    let diags = check_missing_keys(&used, &dict_set);
    assert!(!diags.is_empty());
    assert!(diags.iter().any(|d| d.message.contains("common.unknown")));
}

#[test]
fn unused_keys() {
    let dict_set = make_dict_set();
    let used: FxHashSet<String> = FxHashSet::default();

    let diags = check_unused_keys(&used, &dict_set);
    assert!(!diags.is_empty());
}

#[test]
fn type_mismatch() {
    let mut set = DictionarySet::new();

    let mut en = Dictionary::new();
    en.insert(KeyPath::new("msg"), "Hello {$name} {$count}".to_string());
    set.insert(Locale::new("en").unwrap(), en);

    let mut ja = Dictionary::new();
    ja.insert(KeyPath::new("msg"), "こんにちは {$name}".to_string());
    set.insert(Locale::new("ja").unwrap(), ja);

    let diags = check_type_mismatch(&set);
    assert!(!diags.is_empty());
    // Depending on hash iteration order, the diagnostic may report
    // "missing variables" or "extra variables".
    assert!(diags
        .iter()
        .any(|d| d.message.contains("missing variables") || d.message.contains("extra variables")));
}
