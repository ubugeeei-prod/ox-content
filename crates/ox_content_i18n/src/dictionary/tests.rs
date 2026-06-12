use super::*;

#[test]
fn dictionary_basic_ops() {
    let mut dict = Dictionary::new();
    assert!(dict.is_empty());

    dict.insert(KeyPath::new("greeting"), "Hello".to_string());
    assert_eq!(dict.len(), 1);
    assert_eq!(dict.get("greeting"), Some("Hello"));
    assert_eq!(dict.get("missing"), None);
}

#[test]
fn dictionary_set_translate() {
    let mut set = DictionarySet::new();
    set.set_default_locale(Locale::new("en").unwrap());

    let mut en = Dictionary::new();
    en.insert(KeyPath::new("greeting"), "Hello".to_string());
    en.insert(KeyPath::new("farewell"), "Goodbye".to_string());
    set.insert(Locale::new("en").unwrap(), en);

    let mut ja = Dictionary::new();
    ja.insert(KeyPath::new("greeting"), "こんにちは".to_string());
    set.insert(Locale::new("ja").unwrap(), ja);

    assert_eq!(set.translate("ja", "greeting"), Some("こんにちは"));
    assert_eq!(set.translate("ja", "farewell"), Some("Goodbye"));
    assert_eq!(set.translate("ja", "nonexistent"), None);
}

#[test]
fn dictionary_set_locales() {
    let mut set = DictionarySet::new();
    set.insert(Locale::new("en").unwrap(), Dictionary::new());
    set.insert(Locale::new("ja").unwrap(), Dictionary::new());

    let mut locales: Vec<&str> = set.locales().collect();
    locales.sort_unstable();
    assert_eq!(locales, vec!["en", "ja"]);
}
