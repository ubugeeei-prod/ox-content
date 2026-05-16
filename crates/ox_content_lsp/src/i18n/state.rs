use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::sync::RwLock;
use tower_lsp::lsp_types::Url;

use ox_content_i18n::dictionary::{self, DictionarySet};
use ox_content_i18n_checker::key_collector::{KeyCollector, KeyUsage};
use oxc_span::SourceType;

#[derive(Clone)]
pub struct I18nState {
    inner: Arc<RwLock<Inner>>,
}

#[derive(Default)]
struct Inner {
    dict_dir: Option<PathBuf>,
    dict_set: DictionarySet,
    file_keys: HashMap<String, Vec<KeyUsage>>,
    all_keys: HashSet<String>,
    open_uris: Vec<Url>,
}

impl I18nState {
    #[must_use]
    pub fn new() -> Self {
        Self { inner: Arc::new(RwLock::new(Inner::default())) }
    }

    pub async fn set_root(&self, root: Option<PathBuf>) {
        let (dict_dir, dict_set) = load_dicts(root.as_deref());
        let mut inner = self.inner.write().await;
        inner.dict_dir = dict_dir;
        inner.dict_set = dict_set;
    }

    pub async fn reload_dictionaries(&self) {
        let mut inner = self.inner.write().await;
        let (dict_dir, dict_set) = load_dicts(inner.dict_dir.as_deref().and_then(Path::parent));
        inner.dict_dir = dict_dir;
        inner.dict_set = dict_set;
    }

    pub async fn update_file_keys(&self, file_path: &str, source: &str) {
        let collector = KeyCollector::new();
        let source_type = SourceType::from_path(Path::new(file_path)).unwrap_or_default();
        let usages = collector.collect_source(source, file_path, source_type).unwrap_or_default();

        let mut inner = self.inner.write().await;
        inner.file_keys.insert(file_path.to_string(), usages);
        rebuild_all_keys(&mut inner);
        drop(inner);
    }

    pub async fn remove_file(&self, file_path: &str) {
        let mut inner = self.inner.write().await;
        inner.file_keys.remove(file_path);
        inner.open_uris.retain(|uri| {
            uri.to_file_path().map_or(true, |path| path.to_string_lossy() != file_path)
        });
        rebuild_all_keys(&mut inner);
        drop(inner);
    }

    pub async fn add_open_uri(&self, uri: Url) {
        let mut inner = self.inner.write().await;
        if !inner.open_uris.contains(&uri) {
            inner.open_uris.push(uri);
        }
    }

    pub async fn get_open_uris(&self) -> Vec<Url> {
        let inner = self.inner.read().await;
        inner.open_uris.clone()
    }

    pub async fn all_dictionary_keys(&self) -> Vec<String> {
        let inner = self.inner.read().await;
        let mut keys = HashSet::new();
        for locale in inner.dict_set.locales() {
            if let Some(dict) = inner.dict_set.get(locale) {
                keys.extend(dict.keys().map(std::string::ToString::to_string));
            }
        }
        let mut sorted: Vec<String> = keys.into_iter().collect();
        sorted.sort();
        sorted
    }

    pub async fn translations_for_key(&self, key: &str) -> Vec<(String, String)> {
        let inner = self.inner.read().await;
        let mut translations = Vec::new();
        for locale in inner.dict_set.locales() {
            if let Some(dict) = inner.dict_set.get(locale) {
                if let Some(value) = dict.get(key) {
                    translations.push((locale.to_string(), value.to_string()));
                }
            }
        }
        translations
    }

    pub async fn default_translation(&self, key: &str) -> Option<String> {
        let inner = self.inner.read().await;
        if let Some(locale) =
            inner.dict_set.default_locale().map(|locale| locale.as_str().to_string())
        {
            if let Some(dict) = inner.dict_set.get(&locale) {
                if let Some(value) = dict.get(key) {
                    return Some(value.to_string());
                }
            }
        }

        let fallback = inner
            .dict_set
            .locales()
            .find_map(|locale| inner.dict_set.get(locale).and_then(|dict| dict.get(key)))
            .map(std::string::ToString::to_string);

        fallback
    }

    pub async fn find_key_definition(&self, key: &str) -> Option<String> {
        let inner = self.inner.read().await;
        let dict_dir = inner.dict_dir.clone()?;
        let locales: Vec<String> = inner
            .dict_set
            .locales()
            .filter(|locale| inner.dict_set.get(locale).and_then(|dict| dict.get(key)).is_some())
            .map(String::from)
            .collect();
        drop(inner);
        let namespace = key.split('.').next().unwrap_or(key);

        for locale in &locales {
            for ext in ["json", "yaml", "yml"] {
                let candidate = dict_dir.join(locale).join(format!("{namespace}.{ext}"));
                if candidate.exists() {
                    return Some(candidate.to_string_lossy().to_string());
                }
            }
        }
        None
    }

    pub async fn check_diagnostics(&self) -> Vec<ox_content_i18n::checker::Diagnostic> {
        let inner = self.inner.read().await;
        ox_content_i18n::checker::check_all(&inner.all_keys, &inner.dict_set)
    }

    pub async fn get_file_key_usages(&self, file_path: &str) -> Vec<KeyUsage> {
        let inner = self.inner.read().await;
        inner.file_keys.get(file_path).cloned().unwrap_or_default()
    }
}

impl Default for I18nState {
    fn default() -> Self {
        Self::new()
    }
}

fn load_dicts(root: Option<&Path>) -> (Option<PathBuf>, DictionarySet) {
    let Some(root) = root else {
        return (None, DictionarySet::new());
    };

    let dict_dir = root.join("content/i18n");
    let dict_set = if dict_dir.exists() {
        dictionary::load_from_dir(&dict_dir).unwrap_or_default()
    } else {
        DictionarySet::new()
    };

    (Some(dict_dir), dict_set)
}

fn rebuild_all_keys(inner: &mut Inner) {
    inner.all_keys = inner.file_keys.values().flatten().map(|usage| usage.key.clone()).collect();
}
