use std::collections::HashMap;

use napi_derive::napi;

#[napi(object)]
pub struct I18nLoadResult {
    /// Number of locales loaded.
    pub locale_count: u32,
    /// All locale tags.
    pub locales: Vec<String>,
    /// Errors encountered during loading.
    pub errors: Vec<String>,
}

/// Locale metadata for generated i18n runtime modules.
#[napi(object)]
#[derive(Clone)]
pub struct JsI18nRuntimeLocale {
    /// BCP 47 locale tag.
    pub code: String,
    /// Display name for this locale.
    pub name: String,
    /// Text direction.
    pub dir: Option<String>,
}

/// Configuration for generated i18n runtime modules.
#[napi(object)]
pub struct JsI18nRuntimeConfig {
    /// Default locale tag.
    pub default_locale: String,
    /// Available locales.
    pub locales: Vec<JsI18nRuntimeLocale>,
    /// Whether URLs should omit the default locale prefix.
    pub hide_default_locale: bool,
}

/// Result of MF2 validation.
#[napi(object)]
pub struct Mf2ValidateResult {
    /// Whether the message is valid.
    pub valid: bool,
    /// Validation errors.
    pub errors: Vec<String>,
    /// AST as JSON (if parsing succeeded).
    pub ast_json: Option<String>,
}

/// A single i18n diagnostic.
#[napi(object)]
pub struct I18nDiagnostic {
    /// Severity: "error", "warning", or "info".
    pub severity: String,
    /// Diagnostic message.
    pub message: String,
    /// Related translation key, if any.
    pub key: Option<String>,
    /// Related locale, if any.
    pub locale: Option<String>,
}

/// Result of i18n checking.
#[napi(object)]
pub struct I18nCheckResult {
    /// All diagnostics.
    pub diagnostics: Vec<I18nDiagnostic>,
    /// Number of errors.
    pub error_count: u32,
    /// Number of warnings.
    pub warning_count: u32,
}

fn i18n_check_result_from_diagnostics(
    diagnostics: Vec<ox_content_i18n::checker::Diagnostic>,
) -> I18nCheckResult {
    let mut error_count = 0u32;
    let mut warning_count = 0u32;
    let js_diagnostics: Vec<I18nDiagnostic> = diagnostics
        .into_iter()
        .map(|d| {
            let severity = match d.severity {
                ox_content_i18n::checker::Severity::Error => {
                    error_count += 1;
                    "error"
                }
                ox_content_i18n::checker::Severity::Warning => {
                    warning_count += 1;
                    "warning"
                }
                ox_content_i18n::checker::Severity::Info => "info",
            };
            I18nDiagnostic {
                severity: severity.to_string(),
                message: d.message,
                key: d.key,
                locale: d.locale,
            }
        })
        .collect();

    I18nCheckResult { diagnostics: js_diagnostics, error_count, warning_count }
}

fn i18n_check_error(message: String) -> I18nCheckResult {
    I18nCheckResult {
        diagnostics: vec![I18nDiagnostic {
            severity: "error".to_string(),
            message,
            key: None,
            locale: None,
        }],
        error_count: 1,
        warning_count: 0,
    }
}

/// Loads dictionaries from the given directory.
///
/// The directory should contain locale subdirectories (e.g., `en/`, `ja/`)
/// with JSON or YAML translation files.
#[napi]
pub fn load_dictionaries(dir: String) -> I18nLoadResult {
    let path = std::path::Path::new(&dir);
    match ox_content_i18n::dictionary::load_from_dir(path) {
        Ok(set) => {
            let locales: Vec<String> = set.locales().map(String::from).collect();
            I18nLoadResult { locale_count: locales.len() as u32, locales, errors: vec![] }
        }
        Err(e) => I18nLoadResult { locale_count: 0, locales: vec![], errors: vec![e.to_string()] },
    }
}

/// Loads dictionaries from the given directory and returns a flat key-value map per locale.
///
/// Each locale maps to a flat `{ "namespace.key": "value" }` structure.
/// Supports both JSON and YAML dictionary files.
#[napi]
pub fn load_dictionaries_flat(dir: String) -> HashMap<String, HashMap<String, String>> {
    let path = std::path::Path::new(&dir);
    ox_content_i18n::runtime::load_flat_dictionaries(path).unwrap_or_default()
}

/// Generates the `virtual:ox-content/i18n` runtime module.
#[napi(js_name = "generateI18nModule")]
pub fn generate_i18n_module(dict_dir: String, config: JsI18nRuntimeConfig) -> String {
    let config = ox_content_i18n::runtime::I18nRuntimeConfig {
        default_locale: config.default_locale,
        locales: config
            .locales
            .into_iter()
            .map(|locale| ox_content_i18n::runtime::I18nRuntimeLocale {
                code: locale.code,
                name: locale.name,
                dir: locale.dir,
            })
            .collect(),
        hide_default_locale: config.hide_default_locale,
    };
    let dictionaries =
        ox_content_i18n::runtime::load_flat_dictionaries(std::path::Path::new(&dict_dir))
            .unwrap_or_default();

    ox_content_i18n::runtime::generate_runtime_module(&config, &dictionaries)
}

/// Validates an MF2 message string.
///
/// Returns parsing and semantic validation results.
#[napi]
pub fn validate_mf2(message: String) -> Mf2ValidateResult {
    match ox_content_i18n::mf2::parse_and_validate(&message) {
        Ok((ast, validation_errors)) => {
            let ast_json = serde_json::to_string(&ast).ok();
            let errors: Vec<String> = validation_errors.iter().map(ToString::to_string).collect();
            Mf2ValidateResult { valid: errors.is_empty(), errors, ast_json }
        }
        Err(e) => Mf2ValidateResult { valid: false, errors: vec![e.to_string()], ast_json: None },
    }
}

/// Runs i18n checks on dictionaries against used translation keys.
///
/// `dict_dir` is the path to the i18n directory with locale subdirectories.
/// `used_keys` is a list of translation keys found in source code.
#[napi(js_name = "checkI18n")]
pub fn check_i18n(dict_dir: String, used_keys: Vec<String>) -> I18nCheckResult {
    let path = std::path::Path::new(&dict_dir);
    let dict_set = match ox_content_i18n::dictionary::load_from_dir(path) {
        Ok(set) => set,
        Err(e) => return i18n_check_error(e.to_string()),
    };

    let keys_set: std::collections::HashSet<String> = used_keys.into_iter().collect();
    let diagnostics = ox_content_i18n::checker::check_all(&keys_set, &dict_set);

    i18n_check_result_from_diagnostics(diagnostics)
}

/// Runs project-level i18n checks by collecting source keys and validating dictionaries.
///
/// `dict_dir` is the path to the i18n directory with locale subdirectories.
/// `src_dirs` are source/content directories to scan recursively.
/// `function_names` are translation call names to collect from JS/TS source.
/// `default_locale` is used for dictionary fallback rules.
#[napi(js_name = "checkI18nProject")]
pub fn check_i18n_project(
    dict_dir: String,
    src_dirs: Vec<String>,
    function_names: Vec<String>,
    default_locale: String,
) -> I18nCheckResult {
    let config = ox_content_i18n_checker::CheckConfig {
        dict_dir,
        src_dirs,
        function_names,
        default_locale: Some(default_locale),
        ..Default::default()
    };

    match ox_content_i18n_checker::check(&config) {
        Ok(result) => i18n_check_result_from_diagnostics(result.diagnostics),
        Err(error) => i18n_check_error(error),
    }
}

/// A translation key usage found in source code.
#[napi(object)]
pub struct I18nKeyUsage {
    /// The translation key.
    pub key: String,
    /// Source file path.
    pub file_path: String,
    /// Line number.
    pub line: u32,
    /// Column number.
    pub column: u32,
    /// End column number.
    pub end_column: u32,
}

/// Extracts translation keys from a TypeScript/JavaScript source string.
///
/// Finds calls like `t('key')` and `$t('key')`.
#[napi]
pub fn extract_translation_keys(
    source: String,
    file_path: String,
    function_names: Option<Vec<String>>,
) -> Vec<I18nKeyUsage> {
    let collector = if let Some(names) = function_names {
        ox_content_i18n_checker::key_collector::KeyCollector::with_function_names(names)
    } else {
        ox_content_i18n_checker::key_collector::KeyCollector::new()
    };

    let source_type =
        oxc_span::SourceType::from_path(std::path::Path::new(&file_path)).unwrap_or_default();

    match collector.collect_source(&source, &file_path, source_type) {
        Ok(usages) => usages
            .into_iter()
            .map(|u| I18nKeyUsage {
                key: u.key,
                file_path: u.file_path,
                line: u.line,
                column: u.column,
                end_column: u.end_column,
            })
            .collect(),
        Err(_) => vec![],
    }
}
