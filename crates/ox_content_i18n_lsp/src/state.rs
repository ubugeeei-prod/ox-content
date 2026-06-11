use rustc_hash::{FxHashMap, FxHashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_lsp::lsp_types::Url;

use ox_content_i18n::dictionary::{self, DictionarySet};
use ox_content_i18n_checker::key_collector::{KeyCollector, KeyUsage};

/// Shared LSP state holding dictionaries and key caches.
#[derive(Clone)]
pub struct LspState {
    inner: Arc<RwLock<Inner>>,
}

struct Inner {
    /// Root directory of the workspace.
    root: Option<PathBuf>,
    /// i18n dictionary directory.
    dict_dir: Option<PathBuf>,
    /// Loaded dictionaries.
    dict_set: DictionarySet,
    /// Cache of file → collected key usages.
    file_keys: FxHashMap<String, Vec<KeyUsage>>,
    /// All used keys (union of file_keys values).
    all_keys: FxHashSet<String>,
    /// Text content of currently open documents.
    document_texts: FxHashMap<String, String>,
    /// URIs of currently open documents.
    open_uris: Vec<Url>,
}

impl LspState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(Inner {
                root: None,
                dict_dir: None,
                dict_set: DictionarySet::new(),
                file_keys: FxHashMap::default(),
                all_keys: FxHashSet::default(),
                document_texts: FxHashMap::default(),
                open_uris: Vec::new(),
            })),
        }
    }

    /// Sets the workspace root and attempts to load dictionaries.
    pub async fn set_root(&self, root: PathBuf) {
        let dict_dir = root.join("content/i18n");
        let dict_set = if dict_dir.exists() {
            dictionary::load_from_dir(&dict_dir).unwrap_or_default()
        } else {
            DictionarySet::new()
        };

        let mut inner = self.inner.write().await;
        inner.root = Some(root);
        inner.dict_dir = Some(dict_dir);
        inner.dict_set = dict_set;
    }

    /// Reloads dictionaries from disk.
    pub async fn reload_dictionaries(&self) {
        let mut inner = self.inner.write().await;
        if let Some(ref dict_dir) = inner.dict_dir {
            if dict_dir.exists() {
                if let Ok(set) = dictionary::load_from_dir(dict_dir) {
                    inner.dict_set = set;
                }
            }
        }
    }

    /// Updates the key cache and document text for a file.
    pub async fn update_file_keys(&self, file_path: &str, source: &str) {
        let collector = KeyCollector::new();
        let source_type = oxc_span::SourceType::from_path(Path::new(file_path)).unwrap_or_default();

        let usages = collector.collect_source(source, file_path, source_type).unwrap_or_default();

        let mut inner = self.inner.write().await;
        inner.file_keys.insert(file_path.to_string(), usages);
        inner.document_texts.insert(file_path.to_string(), source.to_string());

        // Rebuild all_keys
        inner.all_keys =
            inner.file_keys.values().flatten().map(|usage| usage.key.clone()).collect();
    }

    /// Removes cached keys, text, and URI for a file.
    pub async fn remove_file(&self, file_path: &str) {
        let mut inner = self.inner.write().await;
        inner.file_keys.remove(file_path);
        inner.document_texts.remove(file_path);

        // Remove matching URI
        inner
            .open_uris
            .retain(|uri| uri.to_file_path().map_or(true, |p| p.to_string_lossy() != file_path));

        inner.all_keys =
            inner.file_keys.values().flatten().map(|usage| usage.key.clone()).collect();
    }

    /// Returns all translation keys from all locales' dictionaries.
    pub async fn all_dictionary_keys(&self) -> Vec<String> {
        let inner = self.inner.read().await;
        let mut keys = FxHashSet::default();
        for locale in inner.dict_set.locales() {
            if let Some(dict) = inner.dict_set.get(locale) {
                for key in dict.keys() {
                    keys.insert(key.to_string());
                }
            }
        }
        let mut sorted: Vec<String> = keys.into_iter().collect();
        sorted.sort();
        sorted
    }

    /// Translates a key in all locales (for hover preview).
    #[allow(dead_code)]
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

    /// Gets the default locale translation for a key (for inlay hints).
    #[allow(dead_code)]
    pub async fn default_translation(&self, key: &str) -> Option<String> {
        let inner = self.inner.read().await;
        let default_locale = inner.dict_set.default_locale().map(|l| l.as_str().to_string());

        if let Some(locale) = default_locale {
            if let Some(dict) = inner.dict_set.get(&locale) {
                return dict.get(key).map(String::from);
            }
        }

        // Try first available locale
        for locale in inner.dict_set.locales() {
            if let Some(dict) = inner.dict_set.get(locale) {
                if let Some(value) = dict.get(key) {
                    return Some(value.to_string());
                }
            }
        }
        None
    }

    /// Finds the dictionary file path and position for a key.
    #[allow(dead_code)]
    pub async fn find_key_definition(&self, key: &str) -> Option<(String, String)> {
        let inner = self.inner.read().await;
        let dict_dir = inner.dict_dir.as_ref()?;

        // Find which locale/file contains this key
        for locale in inner.dict_set.locales() {
            if let Some(dict) = inner.dict_set.get(locale) {
                if dict.get(key).is_some() {
                    // Key found — determine source file from namespace
                    let namespace = key.split('.').next().unwrap_or(key);
                    let json_path = dict_dir.join(locale).join(format!("{namespace}.json"));

                    if json_path.exists() {
                        return Some((json_path.to_string_lossy().to_string(), locale.to_string()));
                    }

                    let yaml_path = dict_dir.join(locale).join(format!("{namespace}.yaml"));
                    if yaml_path.exists() {
                        return Some((yaml_path.to_string_lossy().to_string(), locale.to_string()));
                    }
                }
            }
        }
        None
    }

    /// Returns all locales.
    #[allow(dead_code)]
    pub async fn locales(&self) -> Vec<String> {
        let inner = self.inner.read().await;
        inner.dict_set.locales().map(String::from).collect()
    }

    /// Runs diagnostics and returns checker results.
    #[allow(dead_code)]
    pub async fn check_diagnostics(&self) -> Vec<ox_content_i18n::checker::Diagnostic> {
        let inner = self.inner.read().await;
        ox_content_i18n::checker::check_all(&inner.all_keys, &inner.dict_set)
    }

    /// Returns the stored document text for a file.
    #[allow(dead_code)]
    pub async fn get_document_text(&self, file_path: &str) -> Option<String> {
        let inner = self.inner.read().await;
        inner.document_texts.get(file_path).cloned()
    }

    /// Returns key usages for a specific file.
    pub async fn get_file_key_usages(&self, file_path: &str) -> Vec<KeyUsage> {
        let inner = self.inner.read().await;
        inner.file_keys.get(file_path).cloned().unwrap_or_default()
    }

    /// Returns all currently open URIs.
    pub async fn get_open_uris(&self) -> Vec<Url> {
        let inner = self.inner.read().await;
        inner.open_uris.clone()
    }

    /// Adds an open document URI.
    pub async fn add_open_uri(&self, uri: Url) {
        let mut inner = self.inner.write().await;
        if !inner.open_uris.contains(&uri) {
            inner.open_uris.push(uri);
        }
    }

    /// Removes an open document URI.
    pub async fn remove_open_uri(&self, uri: &Url) {
        let mut inner = self.inner.write().await;
        inner.open_uris.retain(|u| u != uri);
    }
}

impl Default for LspState {
    fn default() -> Self {
        Self::new()
    }
}
