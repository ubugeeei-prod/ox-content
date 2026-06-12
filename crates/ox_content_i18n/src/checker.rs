use std::collections::HashSet;
use std::hash::BuildHasher;

use rustc_hash::FxHashSet;

use crate::dictionary::DictionarySet;
use crate::mf2;

#[cfg(test)]
mod tests;

/// Diagnostic severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// A single diagnostic produced by static analysis.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub key: Option<String>,
    pub locale: Option<String>,
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let level = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
        };
        write!(f, "[{level}] {}", self.message)?;
        if let Some(key) = &self.key {
            write!(f, " (key: {key})")?;
        }
        if let Some(locale) = &self.locale {
            write!(f, " (locale: {locale})")?;
        }
        Ok(())
    }
}

/// Checks for keys used in source code that are missing from dictionaries.
#[must_use]
#[allow(clippy::disallowed_types)]
pub fn check_missing_keys<S: BuildHasher>(
    used_keys: &HashSet<String, S>,
    dict_set: &DictionarySet,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for locale in dict_set.locales() {
        if let Some(dict) = dict_set.get(locale) {
            for key in used_keys {
                if dict.get(key).is_none() {
                    diagnostics.push(Diagnostic {
                        severity: Severity::Error,
                        message: format!("missing translation for key '{key}'"),
                        key: Some(key.clone()),
                        locale: Some(locale.to_string()),
                    });
                }
            }
        }
    }

    diagnostics
}

/// Checks for keys in dictionaries that are not used in source code.
#[must_use]
#[allow(clippy::disallowed_types)]
pub fn check_unused_keys<S: BuildHasher>(
    used_keys: &HashSet<String, S>,
    dict_set: &DictionarySet,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for locale in dict_set.locales() {
        if let Some(dict) = dict_set.get(locale) {
            for key in dict.keys() {
                if !used_keys.contains(key) {
                    diagnostics.push(Diagnostic {
                        severity: Severity::Warning,
                        message: format!("unused translation key '{key}'"),
                        key: Some(key.to_string()),
                        locale: Some(locale.to_string()),
                    });
                }
            }
        }
    }

    diagnostics
}

/// Checks that placeholder variables match across all locales for each key.
#[must_use]
pub fn check_type_mismatch(dict_set: &DictionarySet) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // Collect all keys from all locales
    let mut all_keys: FxHashSet<String> = FxHashSet::default();
    for locale in dict_set.locales() {
        if let Some(dict) = dict_set.get(locale) {
            for key in dict.keys() {
                all_keys.insert(key.to_string());
            }
        }
    }

    // For each key, compare variable sets across locales
    for key in &all_keys {
        let mut locale_vars: Vec<(String, FxHashSet<String>)> = Vec::new();

        for locale in dict_set.locales() {
            if let Some(dict) = dict_set.get(locale) {
                if let Some(value) = dict.get(key) {
                    if let Ok(msg) = mf2::parse(value) {
                        let vars = mf2::validator::extract_variables(&msg);
                        locale_vars.push((locale.to_string(), vars));
                    }
                }
            }
        }

        // Compare all variable sets against the first locale
        if locale_vars.len() > 1 {
            let (ref_locale, ref_vars) = &locale_vars[0];
            for (other_locale, other_vars) in &locale_vars[1..] {
                let missing: Vec<_> = ref_vars.difference(other_vars).collect();
                let extra: Vec<_> = other_vars.difference(ref_vars).collect();

                if !missing.is_empty() {
                    diagnostics.push(Diagnostic {
                        severity: Severity::Error,
                        message: format!(
                            "locale '{other_locale}' is missing variables {missing:?} \
                             (present in '{ref_locale}')"
                        ),
                        key: Some(key.clone()),
                        locale: Some(other_locale.clone()),
                    });
                }
                if !extra.is_empty() {
                    diagnostics.push(Diagnostic {
                        severity: Severity::Warning,
                        message: format!(
                            "locale '{other_locale}' has extra variables {extra:?} \
                             (not in '{ref_locale}')"
                        ),
                        key: Some(key.clone()),
                        locale: Some(other_locale.clone()),
                    });
                }
            }
        }
    }

    diagnostics
}

/// Checks all dictionary values for MF2 syntax errors.
#[must_use]
pub fn check_syntax_errors(dict_set: &DictionarySet) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for locale in dict_set.locales() {
        if let Some(dict) = dict_set.get(locale) {
            for (key, value) in dict.iter() {
                if let Err(e) = mf2::parse(value) {
                    diagnostics.push(Diagnostic {
                        severity: Severity::Error,
                        message: format!("MF2 syntax error: {e}"),
                        key: Some(key.to_string()),
                        locale: Some(locale.to_string()),
                    });
                }

                // Also run semantic validation
                if let Ok(msg) = mf2::parse(value) {
                    let errors = mf2::validator::validate(&msg);
                    for err in errors {
                        diagnostics.push(Diagnostic {
                            severity: Severity::Warning,
                            message: format!("MF2 validation: {err}"),
                            key: Some(key.to_string()),
                            locale: Some(locale.to_string()),
                        });
                    }
                }
            }
        }
    }

    diagnostics
}

/// Runs all checks and returns combined diagnostics.
#[must_use]
#[allow(clippy::disallowed_types)]
pub fn check_all<S: BuildHasher>(
    used_keys: &HashSet<String, S>,
    dict_set: &DictionarySet,
) -> Vec<Diagnostic> {
    let mut all = Vec::new();
    all.extend(check_missing_keys(used_keys, dict_set));
    all.extend(check_unused_keys(used_keys, dict_set));
    all.extend(check_type_mismatch(dict_set));
    all.extend(check_syntax_errors(dict_set));
    all
}
