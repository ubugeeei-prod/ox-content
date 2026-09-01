use crate::case::camel;
use crate::color_math::ensure_contrast;
use crate::color_readme::readme;
use crate::color_tokens::tokens_for;
use crate::fs::Workspace;
use crate::package::{ThemePackageKind, theme_package_json};
use crate::ts::{VITE_CONFIG, json_string, string_record, tsconfig_json};
use crate::{Result, print_line};

pub(crate) fn generate(workspace: &Workspace) -> Result<()> {
    let mut manifest: PaletteManifest =
        workspace.read_json(workspace.theme_colors_dir.join("palettes.json"))?;

    for palette in &mut manifest.palettes {
        palette.normalize_contrast();
    }

    for palette in &manifest.palettes {
        let directory = workspace.root.join("npm/theme-color").join(&palette.id);
        let source_directory = directory.join("src");
        let export_name = camel(&palette.id);

        workspace.remove_dir(&source_directory)?;
        workspace.write(source_directory.join("index.ts"), index_ts(palette, &export_name))?;
        for entry in palette.config_entries(&export_name) {
            workspace.write(
                source_directory.join(format!("{}.ts", entry.export_name)),
                config_file_ts(palette, &entry),
            )?;
        }
        workspace.write_json(
            directory.join("package.json"),
            &theme_package_json(
                ThemePackageKind::Color,
                &palette.id,
                &workspace.version,
                &palette.description,
                std::slice::from_ref(&palette.id),
            ),
        )?;
        workspace.write_json(directory.join("tsconfig.json"), &tsconfig_json())?;
        workspace.write(directory.join("vite.config.ts"), VITE_CONFIG)?;
        workspace.write(directory.join("README.md"), readme(palette, &export_name))?;
    }

    print_line(&format!(
        "Generated {} color packages into npm/theme-color-*",
        manifest.palettes.len()
    ));
    Ok(())
}

#[derive(serde::Deserialize)]
struct PaletteManifest {
    palettes: Vec<Palette>,
}

#[derive(serde::Deserialize)]
pub(crate) struct Palette {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) description: String,
    light: ColorMode,
    dark: ColorMode,
    #[serde(default)]
    pub(crate) variants: Vec<PaletteVariant>,
}

impl Palette {
    fn normalize_contrast(&mut self) {
        self.light.normalize_contrast();
        self.dark.normalize_contrast();
        for variant in &mut self.variants {
            if let Some(light) = &mut variant.light {
                light.normalize_contrast();
            }
            if let Some(dark) = &mut variant.dark {
                dark.normalize_contrast();
            }
        }
    }

    pub(crate) fn config_entries(&self, export_name: &str) -> Vec<ColorConfigEntry> {
        let mut entries = vec![ColorConfigEntry {
            export_name: export_name.to_string(),
            light: self.light.clone(),
            dark: self.dark.clone(),
            name: self.id.clone(),
            description: self.description.clone(),
        }];

        entries.extend(self.variants.iter().map(|variant| {
            ColorConfigEntry {
                export_name: variant
                    .export_name
                    .clone()
                    .unwrap_or_else(|| camel(&format!("{}-{}", self.id, variant.id))),
                light: variant.light.clone().unwrap_or_else(|| self.light.clone()),
                dark: variant.dark.clone().unwrap_or_else(|| self.dark.clone()),
                name: variant.name.clone().unwrap_or_else(|| format!("{}-{}", self.id, variant.id)),
                description: variant
                    .description
                    .clone()
                    .unwrap_or_else(|| self.description.clone()),
            }
        }));

        entries
    }
}

pub(crate) struct ColorConfigEntry {
    pub(crate) export_name: String,
    pub(crate) light: ColorMode,
    pub(crate) dark: ColorMode,
    name: String,
    description: String,
}

#[derive(Clone, serde::Deserialize)]
pub(crate) struct PaletteVariant {
    id: String,
    #[serde(default, rename = "exportName")]
    export_name: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    pub(crate) description: Option<String>,
    #[serde(default)]
    light: Option<ColorMode>,
    #[serde(default)]
    dark: Option<ColorMode>,
}

#[derive(Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ColorMode {
    pub(crate) bg: String,
    pub(crate) bg_alt: String,
    pub(crate) text: String,
    muted: String,
    pub(crate) border: String,
    pub(crate) primary: String,
    primary_hover: String,
    pub(crate) code_bg: String,
    pub(crate) code_text: String,
    pub(crate) red: String,
    pub(crate) green: String,
    pub(crate) yellow: String,
    pub(crate) blue: String,
    pub(crate) magenta: String,
    pub(crate) cyan: String,
    #[serde(default)]
    syntax: Option<SyntaxColors>,
}

impl ColorMode {
    fn normalize_contrast(&mut self) {
        self.text = ensure_contrast(&self.text, &self.bg, 4.5);
        self.code_text = ensure_contrast(&self.code_text, &self.code_bg, 4.5);
        self.primary = ensure_contrast(&self.primary, &self.bg, 4.5);
        self.primary_hover = ensure_contrast(&self.primary_hover, &self.bg, 4.5);
        self.muted = ensure_contrast(&self.muted, &self.bg, 4.5);
    }

    fn colors(&self) -> Vec<(&str, String)> {
        vec![
            ("primary", self.primary.clone()),
            ("primaryHover", self.primary_hover.clone()),
            ("background", self.bg.clone()),
            ("backgroundAlt", self.bg_alt.clone()),
            ("text", self.text.clone()),
            ("textMuted", self.muted.clone()),
            ("border", self.border.clone()),
            ("codeBackground", self.code_bg.clone()),
            ("codeBackgroundTop", self.code_bg.clone()),
            ("codeText", self.code_text.clone()),
        ]
    }

    pub(crate) fn syntax_value(
        &self,
        field: impl FnOnce(&SyntaxColors) -> Option<&String>,
    ) -> Option<String> {
        self.syntax.as_ref().and_then(field).cloned()
    }
}

#[derive(Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SyntaxColors {
    #[serde(default)]
    pub(crate) foreground: Option<String>,
    #[serde(default)]
    pub(crate) background: Option<String>,
    #[serde(default)]
    pub(crate) comment: Option<String>,
    #[serde(default)]
    pub(crate) punctuation: Option<String>,
    #[serde(default)]
    pub(crate) keyword: Option<String>,
    #[serde(default)]
    pub(crate) string: Option<String>,
    #[serde(default)]
    pub(crate) string_expression: Option<String>,
    #[serde(default)]
    pub(crate) constant: Option<String>,
    #[serde(default)]
    pub(crate) function: Option<String>,
    #[serde(default)]
    pub(crate) parameter: Option<String>,
    #[serde(default)]
    pub(crate) link: Option<String>,
}

fn index_ts(palette: &Palette, export_name: &str) -> String {
    let entries = palette.config_entries(export_name);
    let mut lines = Vec::with_capacity(entries.len() + 1);
    let default = &entries[0].export_name;
    lines.push(format!("export {{ {default} as default, {default} }} from \"./{default}.js\";"));
    lines.extend(entries[1..].iter().map(|entry| {
        format!("export {{ {} }} from \"./{}.js\";", entry.export_name, entry.export_name)
    }));
    lines.push(String::new());
    lines.join("\n")
}

fn config_file_ts(palette: &Palette, entry: &ColorConfigEntry) -> String {
    format!(
        "import type {{ ThemeConfig }} from \"@ox-content/vite-plugin\";\n\n{}",
        config_ts(palette, entry)
    )
}

fn config_ts(palette: &Palette, entry: &ColorConfigEntry) -> String {
    let light_tokens = tokens_for(&entry.light, &entry.dark, "light");
    let dark_tokens = tokens_for(&entry.dark, &entry.light, "dark");

    format!(
        r"/**
 * {title} — {description}.
 *
 * Color only: no layout, no texture, no typography. Compose it under any
 * `@ox-content/theme-*` skin, or use it on its own over the default theme.
 *
 * Generated from `tools/scripts/theme-colors/palettes.json`; edit that file and run
 * `node tools/scripts/theme-colors/generate.mjs` rather than editing this by hand.
 */
export const {export_name}: ThemeConfig = {{
  name: {name},
  colors: {{
{colors}  }},
  darkColors: {{
{dark_colors}  }},
  tokens: {{
{tokens}  }},
  darkTokens: {{
{dark_tokens}  }},
}};
",
        title = palette.title,
        description = entry.description,
        export_name = entry.export_name,
        name = json_string(&entry.name),
        colors = string_record(&entry.light.colors(), "    "),
        dark_colors = string_record(&entry.dark.colors(), "    "),
        tokens = string_record(&light_tokens, "    "),
        dark_tokens = string_record(&dark_tokens, "    "),
    )
}
