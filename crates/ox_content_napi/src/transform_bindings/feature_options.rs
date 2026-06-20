use std::collections::HashMap;

use napi_derive::napi;
use ox_content_transform::{
    AttrsOptions, CodeBlockLintOptions, CodeImportOptions, DocsTestOptions, EditThisPageOptions,
    EmojiShortcodeOptions, MediaEmbedsOptions, SanitizeOptions, WikiLinkOptions,
};

/// Wiki-link transform options.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsWikiLinkOptions {
    /// Enable `[[target]]` and `[[target|label]]` expansion.
    ///
    /// Default: `false`.
    pub enabled: Option<bool>,

    /// Base URL used for site-relative wiki links.
    ///
    /// Default: `"/"`.
    pub base_url: Option<String>,
}

impl From<JsWikiLinkOptions> for WikiLinkOptions {
    fn from(value: JsWikiLinkOptions) -> Self {
        Self { enabled: value.enabled, base_url: value.base_url }
    }
}

/// Emoji-shortcode transform options.
#[napi(object)]
#[derive(Default, Clone)]
#[allow(clippy::disallowed_types)]
pub struct JsEmojiShortcodeOptions {
    /// Enable `:shortcode:` expansion.
    ///
    /// Default: `false`.
    pub enabled: Option<bool>,

    /// Custom shortcode map. Values are emitted verbatim.
    ///
    /// Default: `{}`.
    pub custom: Option<HashMap<String, String>>,
}

impl From<JsEmojiShortcodeOptions> for EmojiShortcodeOptions {
    fn from(value: JsEmojiShortcodeOptions) -> Self {
        Self {
            enabled: value.enabled,
            custom: value.custom.map(|values| values.into_iter().collect()),
        }
    }
}

/// Attribute syntax transform options.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsAttrsOptions {
    /// Enable markdown-it-attrs style `{#id .class key=value}`.
    ///
    /// Default: `false`.
    pub enabled: Option<bool>,
}

impl From<JsAttrsOptions> for AttrsOptions {
    fn from(value: JsAttrsOptions) -> Self {
        Self { enabled: value.enabled }
    }
}

/// Code import / snippet injection options.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsCodeImportOptions {
    /// Enable `<<< path{selector}` snippet injection.
    ///
    /// Default: `false`.
    pub enabled: Option<bool>,

    /// Root directory used for `@/` and absolute snippet imports.
    ///
    /// Default: project root from the JavaScript caller.
    pub root_dir: Option<String>,
}

impl From<JsCodeImportOptions> for CodeImportOptions {
    fn from(value: JsCodeImportOptions) -> Self {
        Self { enabled: value.enabled, root_dir: value.root_dir }
    }
}

/// HTML sanitizer options.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsSanitizeOptions {
    /// Enable sanitizer. When omitted, passing this object enables it.
    ///
    /// Default: `false` when the whole option is omitted; `true` when this object is present.
    pub enabled: Option<bool>,

    /// Allowed tag names. Omit for safe defaults.
    ///
    /// Default: built-in safe tag allow list.
    pub allowed_tags: Option<Vec<String>>,

    /// Allowed attribute names. Omit for safe defaults.
    ///
    /// Default: built-in safe attribute allow list.
    pub allowed_attributes: Option<Vec<String>>,

    /// Allowed URL schemes for URL-bearing attributes. Omit for safe defaults.
    ///
    /// Default: built-in safe URL scheme allow list.
    pub allowed_url_schemes: Option<Vec<String>>,
}

impl From<JsSanitizeOptions> for SanitizeOptions {
    fn from(value: JsSanitizeOptions) -> Self {
        Self {
            enabled: value.enabled,
            allowed_tags: value.allowed_tags,
            allowed_attributes: value.allowed_attributes,
            allowed_url_schemes: value.allowed_url_schemes,
        }
    }
}

/// Edit-this-page link options.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsEditThisPageOptions {
    /// Enable edit link generation.
    ///
    /// Default: `false` unless `repo_url` is provided by the JavaScript resolver.
    pub enabled: Option<bool>,

    /// GitHub repository URL, e.g. `https://github.com/owner/repo`.
    pub repo_url: Option<String>,

    /// Branch used in edit URLs.
    ///
    /// Default: `"main"`.
    pub branch: Option<String>,

    /// Root directory used to relativize `sourcePath`.
    ///
    /// Default: no extra root prefix.
    pub root_dir: Option<String>,

    /// Link label.
    ///
    /// Default: `"Edit this page"`.
    pub label: Option<String>,
}

impl From<JsEditThisPageOptions> for EditThisPageOptions {
    fn from(value: JsEditThisPageOptions) -> Self {
        Self {
            enabled: value.enabled,
            repo_url: value.repo_url,
            branch: value.branch,
            root_dir: value.root_dir,
            label: value.label,
        }
    }
}

/// Code block linting options.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsCodeBlockLintOptions {
    /// Enable code block linting.
    ///
    /// Default: `false` when the whole option is omitted.
    pub enabled: Option<bool>,

    /// Restrict linting to these language identifiers.
    ///
    /// Default: all fenced block languages.
    pub languages: Option<Vec<String>>,

    /// Report fences without a language identifier.
    ///
    /// Default: `false`.
    pub require_language: Option<bool>,

    /// Report trailing whitespace in code block lines.
    ///
    /// Default: `true`.
    pub trailing_spaces: Option<bool>,
}

impl From<JsCodeBlockLintOptions> for CodeBlockLintOptions {
    fn from(value: JsCodeBlockLintOptions) -> Self {
        Self {
            enabled: value.enabled,
            languages: value.languages,
            require_language: value.require_language,
            trailing_spaces: value.trailing_spaces,
        }
    }
}

/// Docs-as-tests extraction options.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsDocsTestOptions {
    /// Enable docs test extraction.
    ///
    /// Default: `false` when the whole option is omitted.
    pub enabled: Option<bool>,

    /// Languages that can be emitted as test cases.
    ///
    /// Default: `["js", "jsx", "ts", "tsx", "mjs", "mts"]`.
    pub languages: Option<Vec<String>>,

    /// Require fence meta such as `test`, `runnable`, or `vitest`.
    ///
    /// Default: `true`.
    pub require_meta: Option<bool>,
}

impl From<JsDocsTestOptions> for DocsTestOptions {
    fn from(value: JsDocsTestOptions) -> Self {
        Self {
            enabled: value.enabled,
            languages: value.languages,
            require_meta: value.require_meta,
        }
    }
}

/// Built-in media embed transform switches.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsMediaEmbedsOptions {
    /// Render `<Spotify>` embeds.
    ///
    /// Default: `false`.
    pub spotify: Option<bool>,

    /// Render `<StackBlitz>` embeds.
    ///
    /// Default: `false`.
    pub stack_blitz: Option<bool>,

    /// Render `<Tweet>` / `<XPost>` static cards.
    ///
    /// Default: `false`.
    pub twitter: Option<bool>,

    /// Render `<Bluesky>` static cards.
    ///
    /// Default: `false`.
    pub bluesky: Option<bool>,

    /// Render `<WebContainer>` lazy placeholder blocks.
    ///
    /// Default: `false`.
    pub web_container: Option<bool>,
}

impl From<JsMediaEmbedsOptions> for MediaEmbedsOptions {
    fn from(value: JsMediaEmbedsOptions) -> Self {
        Self {
            spotify: value.spotify,
            stack_blitz: value.stack_blitz,
            twitter: value.twitter,
            bluesky: value.bluesky,
            web_container: value.web_container,
        }
    }
}
