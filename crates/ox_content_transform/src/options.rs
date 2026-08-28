use rustc_hash::FxHashMap;

#[derive(Clone, Default)]
pub struct TransformOptions {
    pub gfm: Option<bool>,
    pub mdx: Option<bool>,
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
    pub semantic_footnotes: Option<bool>,
    /// Opt-in visible heading permalinks. Disabled when omitted.
    pub heading_permalinks: Option<bool>,
    pub wiki_links: Option<WikiLinkOptions>,
    pub emoji_shortcodes: Option<EmojiShortcodeOptions>,
    pub math: Option<MathOptions>,
    pub attributes: Option<AttrsOptions>,
    pub cjk_emphasis: Option<bool>,
    pub code_imports: Option<CodeImportOptions>,
    pub sanitize: Option<SanitizeOptions>,
    pub edit_this_page: Option<EditThisPageOptions>,
    /// Opt-in `::: tip` custom containers. Disabled when omitted.
    pub containers: Option<ContainerOptions>,
    /// Opt-in Markdown file includes. Disabled when omitted.
    pub includes: Option<IncludeOptions>,
    /// Opt-in parameterized Markdown partials. Disabled when omitted.
    pub partials: Option<PartialsOptions>,
    /// Opt-in `::: steps` ordered lists. Disabled when omitted.
    pub steps: Option<StepsOptions>,
    /// Opt-in `::: code-group` fence groups. Disabled when omitted.
    pub code_groups: Option<CodeGroupOptions>,
    /// Opt-in `{badge:variant}` inline badges. Disabled when omitted.
    pub badges: Option<BadgeOptions>,
    /// Opt-in `<NotByAI />` authorship badge. Disabled when omitted.
    pub not_by_ai: Option<NotByAiOptions>,
    /// Opt-in `{kbd:...}` inline keyboard keys. Disabled when omitted.
    pub keyboard_keys: Option<KeyboardKeysOptions>,
    /// Opt-in abbreviation / glossary expansion. Disabled when omitted.
    pub abbreviations: Option<AbbreviationsOptions>,
    /// Opt-in PHP Markdown Extra / mdBook-style definition lists. Disabled when omitted.
    pub definition_lists: Option<DefinitionListOptions>,
    /// Opt-in `{link:...}` rich magic links. Disabled when omitted.
    pub magic_links: Option<MagicLinkOptions>,
    /// Opt-in figures, captions, and lazy images. Disabled when omitted.
    pub images: Option<ImageOptions>,
    /// Opt-in static `::: gallery` image groups. Disabled when omitted.
    pub image_galleries: Option<ImageGalleryOptions>,
    /// Opt-in static `::: timeline` milestone lists. Disabled when omitted.
    pub timelines: Option<TimelineOptions>,
    /// Opt-in static `::: if` / `::: else` conditional blocks. Disabled when omitted.
    pub conditional_blocks: Option<ConditionalBlockOptions>,
    /// Opt-in `::: card` / `::: link-card` / `::: card-grid` blocks. Disabled when omitted.
    pub cards: Option<CardOptions>,
    /// Opt-in `file-tree` fences. Disabled when omitted.
    pub file_tree: Option<FileTreeOptions>,
    /// Opt-in `csv-table` / `json-table` fences. Disabled when omitted.
    pub data_tables: Option<DataTableOptions>,
}

#[derive(Clone, Default)]
pub struct BadgeOptions {
    pub enabled: Option<bool>,
}

#[derive(Clone, Default)]
pub struct NotByAiOptions {
    pub enabled: Option<bool>,
    pub label: Option<String>,
    pub href: Option<String>,
}

#[derive(Clone, Default)]
pub struct KeyboardKeysOptions {
    pub enabled: Option<bool>,
    pub aliases: Option<FxHashMap<String, String>>,
    pub style: Option<String>,
}

#[derive(Clone, Default)]
pub struct AbbreviationsOptions {
    pub enabled: Option<bool>,
    pub terms: Option<FxHashMap<String, String>>,
    pub first_use_only: Option<bool>,
}

#[derive(Clone, Default)]
pub struct DefinitionListOptions {
    pub enabled: Option<bool>,
}

#[derive(Clone, Default)]
pub struct MagicLinkOptions {
    pub enabled: Option<bool>,
    pub aliases: Option<FxHashMap<String, MagicLinkAlias>>,
    pub favicon: Option<bool>,
    pub favicon_template: Option<String>,
    pub image_overrides: Option<Vec<MagicLinkImageOverride>>,
}

#[derive(Clone, Default)]
pub struct MagicLinkAlias {
    pub href: String,
    pub label: Option<String>,
    pub image: Option<String>,
}

#[derive(Clone, Default)]
pub struct MagicLinkImageOverride {
    pub href: Option<String>,
    pub prefix: Option<String>,
    pub image: String,
}

#[derive(Clone, Default)]
pub struct ImageOptions {
    pub enabled: Option<bool>,
    pub lazy: Option<bool>,
}

#[derive(Clone, Default)]
pub struct ImageGalleryOptions {
    pub enabled: Option<bool>,
    pub lazy: Option<bool>,
    pub missing_alt: Option<String>,
    pub empty: Option<String>,
}

#[derive(Clone, Default)]
pub struct TimelineOptions {
    pub enabled: Option<bool>,
    pub ordered: Option<bool>,
    pub invalid_date: Option<String>,
    pub unknown_meta: Option<String>,
    pub empty: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct ConditionalBlockOptions {
    pub enabled: Option<bool>,
    pub values: Option<FxHashMap<String, serde_json::Value>>,
}

#[derive(Clone, Default)]
pub struct ContainerOptions {
    pub enabled: Option<bool>,
    pub types: Option<FxHashMap<String, ContainerTypeOptions>>,
}

#[derive(Clone, Default)]
pub struct ContainerTypeOptions {
    pub title: Option<String>,
    pub tag: Option<String>,
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
pub struct MathOptions {
    pub enabled: Option<bool>,
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
pub struct IncludeOptions {
    pub enabled: Option<bool>,
    pub root_dir: Option<String>,
}

#[derive(Clone, Default)]
pub struct PartialsOptions {
    pub enabled: Option<bool>,
    pub root_dir: Option<String>,
    pub root: Option<String>,
    pub missing: Option<String>,
}

#[derive(Clone, Default)]
pub struct CardOptions {
    pub enabled: Option<bool>,
}

#[derive(Clone, Default)]
pub struct StepsOptions {
    pub enabled: Option<bool>,
}

#[derive(Clone, Default)]
pub struct CodeGroupOptions {
    pub enabled: Option<bool>,
}

#[derive(Clone, Default)]
pub struct FileTreeOptions {
    pub enabled: Option<bool>,
    pub default_open: Option<bool>,
    pub icons: Option<bool>,
    pub icon_folder: Option<String>,
    pub icon_folder_open: Option<String>,
    pub icon_file: Option<String>,
    pub icon_files: Option<FxHashMap<String, String>>,
}

#[derive(Clone, Default)]
pub struct DataTableOptions {
    pub enabled: Option<bool>,
    pub root_dir: Option<String>,
    pub missing: Option<String>,
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
    /// Where the source root sits inside the repository, prefixed to the
    /// page path. Requires [`Self::src_dir`] to know where that root is on
    /// disk; without it the page path stays relative to the process's
    /// working directory.
    pub root_dir: Option<String>,
    /// Absolute path of the source root, supplied by the build rather than
    /// by the user, so [`Self::root_dir`] can be joined with the page's
    /// path inside that root.
    pub src_dir: Option<String>,
    /// Forge whose edit-URL shape to use: `github`, `gitlab`, `bitbucket`,
    /// or `gitea`. Inferred from the [`Self::repo_url`] host when omitted,
    /// and an unrecognized value is inferred the same way.
    pub provider: Option<String>,
    /// Edit-URL template, which wins over [`Self::provider`]. Understands
    /// `{repoUrl}`, `{branch}`, and `{path}`; anything else is literal.
    pub url_pattern: Option<String>,
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
    pub apple_music: Option<bool>,
    pub speaker_deck: Option<bool>,
    pub audio: Option<bool>,
    pub video: Option<bool>,
    pub stack_blitz: Option<bool>,
    pub twitter: Option<bool>,
    pub bluesky: Option<bool>,
    pub google_maps: Option<bool>,
    pub qiita: Option<bool>,
    pub zenn: Option<bool>,
    pub package_registry: Option<bool>,
    pub playgrounds: Option<bool>,
    pub vimeo: Option<bool>,
    pub twitch: Option<bool>,
    pub discord: Option<bool>,
    pub fediverse: Option<bool>,
    pub facebook: Option<bool>,
    pub threads: Option<bool>,
    pub instagram: Option<bool>,
    pub web_container: Option<bool>,
    pub loom: Option<bool>,
    pub asciinema: Option<bool>,
    pub figma: Option<bool>,
    pub note: Option<bool>,
    pub google_slides: Option<bool>,
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
    /// MDX `import` statements parsed from `MdxjsEsm` (empty when MDX is off).
    pub imports: Vec<crate::MdxImport>,
    /// Export names from `MdxjsEsm` (empty when MDX is off).
    pub exports: Vec<String>,
    /// Unique JSX component names, document order (empty when MDX is off).
    pub components: Vec<String>,
}
