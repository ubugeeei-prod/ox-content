use serde::{Deserialize, Serialize};

pub const DOC_KIND_ORDER: [&str; 7] =
    ["function", "class", "interface", "type", "enum", "variable", "module"];

/// Options for generated API Markdown.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MarkdownDocsOptions {
    /// Grouping mode: `file` or `category`.
    #[serde(default = "default_group_by")]
    pub group_by: String,
    /// GitHub repository URL for source links.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub github_url: Option<String>,
    /// Internal documentation link style.
    #[serde(default)]
    pub link_style: MarkdownLinkStyle,
    /// Optional absolute route prefix for generated documentation links.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_path: Option<String>,
    /// Output path strategy.
    ///
    /// Only applies when `group_by` is `"file"`. Category grouping always emits
    /// flat `{kind}s.md` pages regardless of this setting.
    #[serde(default)]
    pub path_strategy: MarkdownPathStrategy,
    /// Rendering style: HTML-laced Markdown (default) or pure Markdown.
    #[serde(default)]
    pub render_style: MarkdownRenderStyle,
    /// Display format for index items.
    #[serde(default)]
    pub index_format: MarkdownDisplayFormat,
    /// Display format for value and type parameters.
    #[serde(default)]
    pub parameters_format: MarkdownDisplayFormat,
    /// Display format for interface property groups.
    #[serde(default)]
    pub interface_properties_format: MarkdownDisplayFormat,
    /// Display format for class property groups.
    #[serde(default)]
    pub class_properties_format: MarkdownDisplayFormat,
    /// Display format for type alias property groups.
    #[serde(default)]
    pub type_alias_properties_format: MarkdownDisplayFormat,
    /// Display format for enum member groups.
    #[serde(default)]
    pub enum_members_format: MarkdownDisplayFormat,
    /// Display format for property-owned object literal members.
    #[serde(default)]
    pub property_members_format: MarkdownDisplayFormat,
    /// Display format for type declaration members.
    #[serde(default)]
    pub type_declaration_format: MarkdownDisplayFormat,
    /// Whether to emit the stats summary line on index/overview pages. Defaults
    /// to `true` (historical behavior); set to `false` for TypeDoc-like output
    /// without stats.
    #[serde(default = "default_render_stats")]
    pub render_stats: bool,
    /// Whether to emit the generated-by attribution on root index pages. Defaults
    /// to `true` (historical behavior); set to `false` for TypeDoc-like output
    /// without ox-content attribution.
    #[serde(default = "default_render_generated_by")]
    pub render_generated_by: bool,
    /// TypeDoc-style group order for module index sections and nav groups. `None`
    /// keeps the historical fixed order; `Some` reorders groups by title, placing
    /// unlisted groups alphabetically at `*` (or at the end when `*` is absent).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_order: Option<Vec<String>>,
    /// TypeDoc-style `sort`: ordered list of sort strategies applied to entries and
    /// members. Later strategies break ties left by earlier ones. `None` keeps the
    /// historical behavior (entries/members alphabetical, enum members in
    /// declaration order). Unsupported strategies are ignored.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sort: Option<Vec<String>>,
    /// TypeDoc-style `sortEntryPoints`: when `true` (default) modules/entry points
    /// are ordered alphabetically; when `false` the caller-provided input order is
    /// preserved.
    #[serde(default = "default_sort_entry_points")]
    pub sort_entry_points: bool,
    /// TypeDoc-style `kindSortOrder`: declaration kind ranking used by the `kind`
    /// sort strategy and as the base order for module index sections / nav groups
    /// (before `group_order` is applied). `None` keeps the historical kind order.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind_sort_order: Option<Vec<String>>,
    /// Single-entry root handling for TypeDoc-style file output.
    ///
    /// `Preserve` keeps the historical root index plus module index hierarchy.
    /// `Flatten` uses the root index as the single module landing page when only
    /// one module is present, while keeping symbol page paths stable.
    #[serde(default)]
    pub single_entry_root: MarkdownSingleEntryRoot,
}

/// Internal documentation link style.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum MarkdownLinkStyle {
    /// Link to emitted Markdown files, such as `./context.md#symbol`.
    #[default]
    Markdown,
    /// Link to clean routes, such as `./context#symbol`.
    Clean,
}

/// Generated Markdown output path strategy.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum MarkdownPathStrategy {
    /// Keep the historical flat module/category files with entry anchors.
    #[default]
    Flat,
    /// Emit TypeDoc-style module/kind/symbol pages.
    TypeDoc,
}

/// Single-entry root handling for generated TypeDoc-style API docs.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum MarkdownSingleEntryRoot {
    /// Preserve the historical root/module hierarchy.
    #[default]
    Preserve,
    /// Flatten a single module's groups into the root page/nav level.
    Flatten,
}

/// API documentation rendering style.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum MarkdownRenderStyle {
    /// Emit HTML-laced Markdown: collapsible `<details>` entries, stat blocks,
    /// member tables and other ox-content theme scaffolding. This is the default
    /// and preserves the historical output.
    #[default]
    Html,
    /// Emit pure Markdown: headings, tables and fenced code with no raw HTML
    /// scaffolding. Suitable for plain Markdown hosts such as VitePress.
    Markdown,
}

/// TypeDoc-compatible Markdown display format.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum MarkdownDisplayFormat {
    /// Use the renderer's default behavior.
    #[default]
    None,
    /// Render supported sections as Markdown lists.
    List,
    /// Render supported sections as Markdown tables.
    Table,
}

impl Default for MarkdownDocsOptions {
    fn default() -> Self {
        Self {
            group_by: default_group_by(),
            github_url: None,
            link_style: MarkdownLinkStyle::Markdown,
            base_path: None,
            path_strategy: MarkdownPathStrategy::Flat,
            render_style: MarkdownRenderStyle::Html,
            index_format: MarkdownDisplayFormat::None,
            parameters_format: MarkdownDisplayFormat::None,
            interface_properties_format: MarkdownDisplayFormat::None,
            class_properties_format: MarkdownDisplayFormat::None,
            type_alias_properties_format: MarkdownDisplayFormat::None,
            enum_members_format: MarkdownDisplayFormat::None,
            property_members_format: MarkdownDisplayFormat::None,
            type_declaration_format: MarkdownDisplayFormat::None,
            render_stats: true,
            render_generated_by: true,
            group_order: None,
            sort: None,
            sort_entry_points: default_sort_entry_points(),
            kind_sort_order: None,
            single_entry_root: MarkdownSingleEntryRoot::Preserve,
        }
    }
}

fn default_group_by() -> String {
    "file".to_string()
}

fn default_render_stats() -> bool {
    true
}

fn default_render_generated_by() -> bool {
    true
}

fn default_sort_entry_points() -> bool {
    true
}
