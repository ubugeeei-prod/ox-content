use serde::Serialize;

#[derive(Clone, Copy)]
pub(crate) enum ThemePackageKind {
    Color,
    Skin,
}

pub(crate) fn theme_package_json(
    kind: ThemePackageKind,
    id: &str,
    version: &str,
    description: &str,
    keywords: &[String],
) -> PackageJson {
    let (name, directory, description) = match kind {
        ThemePackageKind::Color => (
            format!("@ox-content/theme-color-{id}"),
            format!("npm/theme-color/{id}"),
            format!(
                "{description} — an Ox Content color scheme that composes with any @ox-content/theme-* skin"
            ),
        ),
        ThemePackageKind::Skin => (
            format!("@ox-content/theme-{id}"),
            format!("npm/theme/{id}"),
            format!(
                "{description} — an Ox Content skin that composes with any @ox-content/theme-color-* scheme"
            ),
        ),
    };
    let mut all_keywords = match kind {
        ThemePackageKind::Color => vec![
            "ox-content",
            "theme",
            "color-scheme",
            "palette",
            "ssg",
            "markdown",
            "documentation",
        ]
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>(),
        ThemePackageKind::Skin => {
            vec!["ox-content", "theme", "skin", "ssg", "markdown", "documentation"]
                .into_iter()
                .map(str::to_string)
                .collect::<Vec<_>>()
        }
    };
    all_keywords.extend(keywords.iter().cloned());

    PackageJson {
        name,
        version: version.to_string(),
        description,
        keywords: all_keywords,
        license: "MIT",
        author: "ubugeeei",
        repository: Repository {
            kind: "git",
            url: "https://github.com/ubugeeei-prod/ox-content.git",
            directory,
        },
        files: vec!["dist"],
        package_type: "module",
        main: "./dist/index.cjs",
        types: "./dist/index.d.mts",
        exports: PackageExports {
            root: ExportCondition {
                import: "./dist/index.mjs",
                require: "./dist/index.cjs",
                types: "./dist/index.d.mts",
            },
        },
        side_effects: false,
        publish_config: PublishConfig { access: "public", provenance: true },
        scripts: Scripts { build: "vp pack", dev: "vp pack --watch", typecheck: "tsgo --noEmit" },
        dev_dependencies: DevDependencies {
            vite_plugin: "workspace:*",
            types_node: "catalog:",
            typescript_native_preview: "catalog:",
            typescript: "catalog:",
            vite_plus: "catalog:",
        },
        peer_dependencies: PeerDependencies { vite_plugin: ">=3.0.0-alpha.1" },
        peer_dependencies_meta: PeerDependenciesMeta {
            vite_plugin: OptionalDependency { optional: true },
        },
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PackageJson {
    name: String,
    version: String,
    description: String,
    keywords: Vec<String>,
    license: &'static str,
    author: &'static str,
    repository: Repository,
    files: Vec<&'static str>,
    #[serde(rename = "type")]
    package_type: &'static str,
    main: &'static str,
    types: &'static str,
    exports: PackageExports,
    side_effects: bool,
    publish_config: PublishConfig,
    scripts: Scripts,
    dev_dependencies: DevDependencies,
    peer_dependencies: PeerDependencies,
    peer_dependencies_meta: PeerDependenciesMeta,
}

#[derive(Serialize)]
struct Repository {
    #[serde(rename = "type")]
    kind: &'static str,
    url: &'static str,
    directory: String,
}

#[derive(Serialize)]
struct PackageExports {
    #[serde(rename = ".")]
    root: ExportCondition,
}

#[derive(Serialize)]
struct ExportCondition {
    import: &'static str,
    require: &'static str,
    types: &'static str,
}

#[derive(Serialize)]
struct PublishConfig {
    access: &'static str,
    provenance: bool,
}

#[derive(Serialize)]
struct Scripts {
    build: &'static str,
    dev: &'static str,
    typecheck: &'static str,
}

#[derive(Serialize)]
struct DevDependencies {
    #[serde(rename = "@ox-content/vite-plugin")]
    vite_plugin: &'static str,
    #[serde(rename = "@types/node")]
    types_node: &'static str,
    #[serde(rename = "@typescript/native-preview")]
    typescript_native_preview: &'static str,
    typescript: &'static str,
    #[serde(rename = "vite-plus")]
    vite_plus: &'static str,
}

#[derive(Serialize)]
struct PeerDependencies {
    #[serde(rename = "@ox-content/vite-plugin")]
    vite_plugin: &'static str,
}

#[derive(Serialize)]
struct PeerDependenciesMeta {
    #[serde(rename = "@ox-content/vite-plugin")]
    vite_plugin: OptionalDependency,
}

#[derive(Serialize)]
struct OptionalDependency {
    optional: bool,
}
