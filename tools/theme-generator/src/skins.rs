use crate::case::camel;
use crate::css::{escape_template_literal, minify};
use crate::fs::Workspace;
use crate::package::{ThemePackageKind, theme_package_json};
use crate::skin_readme::readme;
use crate::ts::{VITE_CONFIG, borrowed_string_record, json_string, tsconfig_json};
use crate::{Result, print_line};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub(crate) fn generate(workspace: &Workspace) -> Result<()> {
    let manifest: SkinManifest =
        workspace.read_json(workspace.theme_skins_dir.join("skins.json"))?;
    let shared = SharedCss::read(workspace)?;
    let mut total_css_bytes = 0;

    for skin in &manifest.skins {
        let export_name = camel(&skin.id);
        let directory = workspace.root.join("npm/theme").join(&skin.id);
        let source_directory = directory.join("src");
        let own_css = read_skin_css(&workspace.theme_skins_dir, &skin.id)?;
        let css = minify(&format!(
            "{}\n{}\n{}\n{}\n{}",
            shared.motion, shared.hero, shared.details, own_css, shared.guards
        ));
        let js = skin_js(workspace, skin, &shared.gl_runtime)?;
        let fonts = manifest.fonts_for(skin)?;
        let bytes = format!("{:.1} kB", kilobytes(css.len()));
        total_css_bytes += css.len();

        workspace.remove_dir(&source_directory)?;
        workspace.write(source_directory.join("skin.ts"), skin_ts(skin, &css))?;
        if let Some(js) = &js {
            workspace.write(source_directory.join("gl.ts"), gl_ts(skin, js))?;
        }
        workspace.write(
            source_directory.join("index.ts"),
            index_ts(skin, &export_name, fonts, js.is_some()),
        )?;
        workspace.write_json(
            directory.join("package.json"),
            &theme_package_json(
                ThemePackageKind::Skin,
                &skin.id,
                &workspace.version,
                &skin.description,
                &skin.keywords,
            ),
        )?;
        workspace.write_json(directory.join("tsconfig.json"), &tsconfig_json())?;
        workspace.write(directory.join("vite.config.ts"), VITE_CONFIG)?;
        workspace.write(
            directory.join("README.md"),
            readme(skin, &export_name, &bytes, js.is_some()),
        )?;
        print_line(&format!("  {:<14} {:>8}", skin.id, bytes));
    }

    let average = kilobytes(total_css_bytes) / manifest.skins.len() as f64;
    print_line(&format!(
        "Generated {} skin packages ({average:.1} kB average)",
        manifest.skins.len()
    ));
    Ok(())
}

#[derive(serde::Deserialize)]
struct SkinManifest {
    #[serde(rename = "fontStacks")]
    font_stacks: BTreeMap<String, String>,
    skins: Vec<Skin>,
}

impl SkinManifest {
    fn fonts_for<'a>(&'a self, skin: &Skin) -> Result<FontRefs<'a>> {
        let sans = self.font_stacks.get(&skin.sans).ok_or_else(|| {
            format!("unknown sans font stack '{}' for skin '{}'", skin.sans, skin.id)
        })?;
        let mono = self.font_stacks.get(&skin.mono).ok_or_else(|| {
            format!("unknown mono font stack '{}' for skin '{}'", skin.mono, skin.id)
        })?;
        Ok(FontRefs { sans, mono })
    }
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Skin {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) description: String,
    #[serde(default)]
    keywords: Vec<String>,
    sans: String,
    mono: String,
    layout: SkinLayout,
    motion: SkinMotion,
    #[serde(default)]
    embed_head: Option<String>,
    #[serde(default)]
    entry_page: Option<String>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct SkinLayout {
    sidebar_width: String,
    header_height: String,
    max_content_width: String,
}

impl SkinLayout {
    fn entries(&self) -> [(&str, &str); 3] {
        [
            ("sidebarWidth", &self.sidebar_width),
            ("headerHeight", &self.header_height),
            ("maxContentWidth", &self.max_content_width),
        ]
    }
}

#[derive(serde::Deserialize)]
struct SkinMotion {
    fast: String,
    base: String,
    slow: String,
    ease: String,
    spring: String,
    rise: String,
}

impl SkinMotion {
    fn token_entries(&self) -> [(&str, &str); 6] {
        [
            ("motion-fast", &self.fast),
            ("motion-base", &self.base),
            ("motion-slow", &self.slow),
            ("motion-ease", &self.ease),
            ("motion-spring", &self.spring),
            ("motion-rise", &self.rise),
        ]
    }
}

struct FontRefs<'a> {
    sans: &'a str,
    mono: &'a str,
}

struct SharedCss {
    motion: String,
    hero: String,
    details: String,
    guards: String,
    gl_runtime: String,
}

impl SharedCss {
    fn read(workspace: &Workspace) -> Result<Self> {
        let base = &workspace.theme_skins_dir;
        Ok(Self {
            motion: workspace.read_to_string(base.join("motion.css"))?,
            hero: workspace.read_to_string(base.join("hero.css"))?,
            details: workspace.read_to_string(base.join("details.css"))?,
            guards: workspace.read_to_string(base.join("guards.css"))?,
            gl_runtime: workspace.read_to_string(base.join("js/runtime.js"))?,
        })
    }
}

fn read_skin_css(base: &Path, id: &str) -> Result<String> {
    let directory = base.join("skins").join(id);
    if directory.is_dir() {
        let mut files = fs::read_dir(directory)?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<std::io::Result<Vec<_>>>()?;
        files.retain(|path| path.extension().is_some_and(|extension| extension == "css"));
        files.sort();
        let mut output = String::new();
        for file in files {
            output.push_str(&fs::read_to_string(file)?);
            output.push('\n');
        }
        return Ok(output);
    }

    Ok(fs::read_to_string(base.join("skins").join(format!("{id}.css")))?)
}

fn skin_js(workspace: &Workspace, skin: &Skin, runtime: &str) -> Result<Option<String>> {
    let path = workspace.theme_skins_dir.join("js").join(format!("{}.js", skin.id));
    if !path.exists() {
        return Ok(None);
    }
    let script = workspace.read_to_string(path)?;
    Ok(Some(format!(
        "(()=>{{try{{\n{runtime}\n{script}\n}}catch(e){{console.warn(\"[ox-content] theme backdrop disabled:\",e)}}}})();"
    )))
}

fn gl_ts(skin: &Skin, js: &str) -> String {
    format!(
        r"/**
 * {title} WebGL2 hero backdrop.
 *
 * Progressive enhancement only: it bails out under prefers-reduced-motion,
 * on a missing WebGL2 context, and whenever the hero scrolls out of view.
 * The CSS backdrop underneath is the design; this layers on top of it.
 *
 * Generated from tools/scripts/theme-skins/js/{id}.js; edit that file and
 * run node tools/scripts/theme-skins/generate.mjs rather than editing this by hand.
 */
export const js = {js};
",
        title = skin.title,
        id = skin.id,
        js = json_string(js),
    )
}

fn skin_ts(skin: &Skin, css: &str) -> String {
    format!(
        r"/**
 * {title} skin stylesheet.
 *
 * {description}.
 *
 * Written entirely against `--octc-*` custom properties and `color-mix()`, plus
 * neutral black/white alpha for depth — no hue is ever named, which is what lets
 * any `@ox-content/theme-color-*` package drive it.
 *
 * Generated from `tools/scripts/theme-skins/skins/{id}.css`; edit that file and
 * run `node tools/scripts/theme-skins/generate.mjs` rather than editing this by hand.
 */
export const css = `{css}`;
",
        title = skin.title,
        description = skin.description,
        id = skin.id,
        css = escape_template_literal(css),
    )
}

fn kilobytes(bytes: usize) -> f64 {
    bytes as f64 / 1024.0
}

fn index_ts(skin: &Skin, export_name: &str, fonts: FontRefs<'_>, has_js: bool) -> String {
    let js_import = if has_js { "\nimport { js } from \"./gl\";" } else { "" };
    let js_field = if has_js { "\n  js," } else { "" };
    let embed = skin
        .embed_head
        .as_ref()
        .map(|head| format!("\n  embed: {{ head: {} }},", json_string(head)))
        .unwrap_or_default();
    format!(
        r#"import type {{ ThemeConfig }} from "@ox-content/vite-plugin";

import {{ css }} from "./skin";{js_import}

/**
 * {title} — {description}.
 *
 * Form only: geometry, texture, typography and motion. It names no colors, so
 * it composes with any `@ox-content/theme-color-*` package:
 *
 * ```ts
 * theme: [{export_name}, tokyoNight]
 * ```
 */
export const {export_name}: ThemeConfig = {{
  name: {name},
  fonts: {{
    sans: {sans},
    mono: {mono},
  }},
  layout: {{
{layout}  }},
  entryPage: {{ mode: {entry_page} }},{embed}
  tokens: {{
{motion}  }},
  css,{js_field}
}};

export default {export_name};
"#,
        title = skin.title,
        description = skin.description,
        name = json_string(&skin.id),
        sans = json_string(fonts.sans),
        mono = json_string(fonts.mono),
        layout = borrowed_string_record(&skin.layout.entries(), "    "),
        entry_page = json_string(skin.entry_page.as_deref().unwrap_or("default")),
        motion = borrowed_string_record(&skin.motion.token_entries(), "    "),
    )
}
