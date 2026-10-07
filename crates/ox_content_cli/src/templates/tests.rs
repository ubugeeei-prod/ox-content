use super::*;
use serde_json::Value;
use std::{collections::BTreeSet, path::Path};

const TEMPLATES: [&str; 3] = ["docs", "blog", "minimal"];
const MANAGERS: [&str; 5] = ["vp", "pnpm", "npm", "yarn", "bun"];

fn file<'a>(files: &'a [(String, String)], name: &str) -> &'a str {
    &files.iter().find(|(path, _)| path == name).unwrap_or_else(|| panic!("Missing {name}")).1
}

/// Package directories below `npm/<group>` mapped to their published names.
fn workspace_packages(group: &str) -> Vec<(String, String)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../npm").join(group);
    let mut packages: Vec<_> = std::fs::read_dir(root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.join("package.json").is_file())
        .map(|path| {
            let manifest: Value =
                serde_json::from_str(&std::fs::read_to_string(path.join("package.json")).unwrap())
                    .unwrap();
            let directory = path.file_name().unwrap().to_string_lossy().into_owned();
            (directory, manifest["name"].as_str().unwrap().to_string())
        })
        .collect();
    packages.sort();
    packages
}

#[test]
fn choices_are_sorted_unique_package_name_fragments() {
    for choices in [SKINS, PALETTES] {
        assert!(choices.windows(2).all(|pair| pair[0] < pair[1]), "{choices:?}");
        for choice in choices {
            assert!(
                choice
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-'),
                "{choice}"
            );
            assert!(!choice.starts_with('-') && !choice.ends_with('-'), "{choice}");
        }
    }
}

/// `oxct new` preselects these by position, so reordering the lists must keep them in place.
#[test]
fn interactive_defaults_stay_on_editorial_and_nord() {
    assert_eq!(SKINS[8], "editorial");
    assert_eq!(PALETTES[26], "nord");
}

#[test]
fn every_offered_skin_and_palette_is_a_workspace_package() {
    let skins = workspace_packages("theme");
    assert_eq!(skins.iter().map(|(directory, _)| directory.as_str()).collect::<Vec<_>>(), SKINS);
    for (directory, name) in skins {
        assert_eq!(name, format!("@ox-content/theme-{directory}"));
    }
    let palettes = workspace_packages("theme-color");
    assert_eq!(
        palettes.iter().map(|(directory, _)| directory.as_str()).collect::<Vec<_>>(),
        PALETTES
    );
    for (directory, name) in palettes {
        assert_eq!(name, format!("@ox-content/theme-color-{directory}"));
    }
}

#[test]
fn every_skin_and_palette_combination_generates_a_consistent_project() {
    for skin in SKINS {
        for palette in PALETTES {
            let files = project_files("site", "docs", skin, palette, "vp").unwrap();
            let package: Value = serde_json::from_str(file(&files, "package.json")).unwrap();
            let dependencies = package["devDependencies"].as_object().unwrap();
            assert_eq!(dependencies.len(), 5, "{skin}/{palette}");
            assert_eq!(
                dependencies[&format!("@ox-content/theme-{skin}")],
                env!("CARGO_PKG_VERSION")
            );
            assert_eq!(
                dependencies[&format!("@ox-content/theme-color-{palette}")],
                env!("CARGO_PKG_VERSION")
            );
            let config = file(&files, "vite.config.ts");
            assert!(config.contains(&format!("import skin from \"@ox-content/theme-{skin}\";")));
            assert!(
                config.contains(&format!(
                    "import palette from \"@ox-content/theme-color-{palette}\";"
                ))
            );
        }
    }
}

#[test]
fn every_template_and_manager_resolves_all_placeholders() {
    for template in TEMPLATES {
        for manager in MANAGERS {
            let files = project_files("My Site", template, "paper", "sepia", manager).unwrap();
            let paths: BTreeSet<_> = files.iter().map(|(path, _)| path.as_str()).collect();
            assert_eq!(paths.len(), files.len(), "{template}/{manager} has duplicate paths");
            for (path, content) in &files {
                let label = format!("{template}/{manager}/{path}");
                assert!(
                    !path.starts_with('/') && !path.split('/').any(|part| part == ".."),
                    "{label}"
                );
                assert!(!content.contains("__OX_"), "{label}: {content}");
                assert!(content.ends_with('\n') && !content.contains('\r'), "{label}");
            }
            for required in [
                "package.json",
                "vite.config.ts",
                "index.html",
                "tsconfig.json",
                ".gitignore",
                "README.md",
                "content/index.md",
            ] {
                assert!(paths.contains(required), "{template}/{manager} is missing {required}");
            }
            assert_eq!(paths.contains("content/guide.md"), template == "docs");
            assert_eq!(paths.contains("content/posts/welcome.md"), template == "blog");
            let bundler = if manager == "vp" { "vite-plus" } else { "vite" };
            assert!(
                file(&files, "vite.config.ts")
                    .starts_with(&format!("import {{ defineConfig }} from \"{bundler}\";"))
            );
            assert!(
                file(&files, "README.md")
                    .contains(&format!("An Ox Content {template} site using Vite."))
            );
            for json in ["package.json", "tsconfig.json"] {
                assert!(
                    serde_json::from_str::<Value>(file(&files, json)).is_ok(),
                    "{template}/{manager}/{json}"
                );
            }
        }
    }
}

#[test]
fn generated_ignore_rules_cover_dependencies_and_build_output() {
    let files = project_files("site", "minimal", "paper", "sepia", "npm").unwrap();
    assert_eq!(
        file(&files, ".gitignore").lines().collect::<Vec<_>>(),
        ["node_modules/", "dist/", ".ox-content/", ".vite/"]
    );
    let config = file(&files, "vite.config.ts");
    assert!(config.contains("srcDir: \"content\",") && config.contains("outDir: \"dist\","));
}

#[test]
fn package_names_are_npm_safe_for_any_directory_name() {
    for (name, expected) in [
        ("site", "site"),
        ("My Site", "my-site"),
        ("Docs_v2.1", "docs-v2-1"),
        ("@scope/pkg", "scope-pkg"),
        ("--weird--", "weird"),
        ("a--b", "a--b"),
        ("日本語", "my-content"),
        ("...", "my-content"),
        ("", "my-content"),
        ("Ünïcode", "n-code"),
        ("123", "123"),
    ] {
        let files = project_files(name, "minimal", "paper", "sepia", "vp").unwrap();
        let package: Value = serde_json::from_str(file(&files, "package.json")).unwrap();
        assert_eq!(package["name"], expected, "{name:?}");
    }
}

#[test]
fn display_names_are_embedded_as_valid_string_literals() {
    for name in [
        "plain",
        "it's",
        "say \"hi\"",
        "back\\slash",
        "new\nline",
        "日本語 👩‍💻",
        "${injected}",
        "</title>",
    ] {
        for template in TEMPLATES {
            let files = project_files(name, template, "paper", "sepia", "vp").unwrap();
            let literal = serde_json::to_string(name).unwrap();
            let config = file(&files, "vite.config.ts");
            assert!(config.contains(&format!("siteName: {literal},")), "{name:?}: {config}");
            assert!(config.contains(&format!("footer: {{ copyright: {literal} }}")), "{name:?}");
            assert!(
                file(&files, "content/index.md").starts_with(&format!("---\ntitle: {literal}\n")),
                "{name:?}"
            );
        }
    }
}

#[test]
fn scripts_follow_the_package_manager() {
    for manager in MANAGERS {
        let files = project_files("site", "docs", "paper", "sepia", manager).unwrap();
        let package: Value = serde_json::from_str(file(&files, "package.json")).unwrap();
        let runner = if manager == "vp" { "vp" } else { "vite" };
        assert_eq!(
            package["scripts"],
            serde_json::json!({
                "dev": format!("{runner} dev"),
                "build": format!("{runner} build"),
                "preview": format!("{runner} preview"),
                "lint": "oxct lint \"content/**/*.{md,mdx,mdc}\""
            })
        );
        let dependencies = package["devDependencies"].as_object().unwrap();
        assert_eq!(dependencies.contains_key("vite-plus"), manager == "vp");
        assert_eq!(dependencies.contains_key("vite"), manager != "vp");
        assert_eq!(dependencies["typescript"], "^5.8.0");
    }
}

#[test]
fn unknown_templates_are_rejected() {
    for template in ["", "wiki", "Docs", "docs "] {
        let error = project_files("site", template, "paper", "sepia", "vp").unwrap_err();
        assert_eq!(error.to_string(), "Unknown project template", "{template:?}");
    }
}
