use rustc_hash::FxHashMap;

#[derive(Clone, Default)]
pub struct TransformOptions {
    pub gfm: Option<bool>,
    pub footnotes: Option<bool>,
    pub task_lists: Option<bool>,
    pub tables: Option<bool>,
    pub strikethrough: Option<bool>,
    pub autolinks: Option<bool>,
    pub frontmatter: Option<bool>,
    pub toc_max_depth: Option<u8>,
    pub convert_md_links: Option<bool>,
    pub base_url: Option<String>,
    pub source_path: Option<String>,
    pub code_annotations: Option<bool>,
    pub code_annotation_meta_key: Option<String>,
    pub code_annotation_syntax: Option<String>,
    pub code_annotation_default_line_numbers: Option<bool>,
    pub autolink_urls: Option<bool>,
    pub autolink_patterns: Option<Vec<String>>,
    pub autolink_target_blank: Option<bool>,
    pub wiki_links: Option<WikiLinkOptions>,
    pub emoji_shortcodes: Option<EmojiShortcodeOptions>,
    pub attributes: Option<AttrsOptions>,
    pub cjk_emphasis: Option<bool>,
    pub code_imports: Option<CodeImportOptions>,
    pub sanitize: Option<SanitizeOptions>,
    pub edit_this_page: Option<EditThisPageOptions>,
}

#[derive(Clone, Default)]
pub struct WikiLinkOptions {
    pub enabled: Option<bool>,
    pub base_url: Option<String>,
}

#[derive(Clone, Default)]
pub struct EmojiShortcodeOptions {
    pub enabled: Option<bool>,
    pub custom: Option<FxHashMap<String, String>>,
}

#[derive(Clone, Default)]
pub struct AttrsOptions {
    pub enabled: Option<bool>,
}

#[derive(Clone, Default)]
pub struct CodeImportOptions {
    pub enabled: Option<bool>,
    pub root_dir: Option<String>,
}

#[derive(Clone, Default)]
pub struct SanitizeOptions {
    pub enabled: Option<bool>,
    pub allowed_tags: Option<Vec<String>>,
    pub allowed_attributes: Option<Vec<String>>,
    pub allowed_url_schemes: Option<Vec<String>>,
}

#[derive(Clone, Default)]
pub struct EditThisPageOptions {
    pub enabled: Option<bool>,
    pub repo_url: Option<String>,
    pub branch: Option<String>,
    pub root_dir: Option<String>,
    pub label: Option<String>,
}

#[derive(Clone, Default)]
pub struct CodeBlockLintOptions {
    pub enabled: Option<bool>,
    pub languages: Option<Vec<String>>,
    pub require_language: Option<bool>,
    pub trailing_spaces: Option<bool>,
}

#[derive(Clone, Default)]
pub struct DocsTestOptions {
    pub enabled: Option<bool>,
    pub languages: Option<Vec<String>>,
    pub require_meta: Option<bool>,
}

#[derive(Clone, Default)]
pub struct MediaEmbedsOptions {
    pub spotify: Option<bool>,
    pub stack_blitz: Option<bool>,
    pub twitter: Option<bool>,
    pub bluesky: Option<bool>,
    pub web_container: Option<bool>,
}

#[derive(Clone)]
pub struct TocEntry {
    pub depth: u8,
    pub text: String,
    pub slug: String,
    pub children: Vec<TocEntry>,
}

pub struct TransformResult {
    pub html: String,
    pub frontmatter: String,
    pub toc: Vec<TocEntry>,
    pub errors: Vec<String>,
}
