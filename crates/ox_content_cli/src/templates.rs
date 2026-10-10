use crate::Result;
use serde_json::json;

#[cfg(test)]
mod tests;

pub const SKINS: &[&str] = &[
    "analog-film",
    "atlas",
    "aurora",
    "bauhaus",
    "blueprint",
    "blur-glass",
    "brutalist",
    "clay",
    "editorial",
    "fabric",
    "fluid",
    "holo",
    "kiosk",
    "leather",
    "ledger",
    "liquid-glass",
    "manuscript",
    "neon",
    "noir",
    "paper",
    "pixel",
    "receipt",
    "risograph",
    "swiss",
    "terminal",
    "voltage",
    "zine",
];
pub const PALETTES: &[&str] = &[
    "arctic",
    "ayu",
    "cacao",
    "catppuccin",
    "commander",
    "coral",
    "dracula",
    "emerald",
    "everforest",
    "flexoki",
    "fuji",
    "github",
    "graphite",
    "gruvbox",
    "high-contrast",
    "horizon",
    "iceberg",
    "ink",
    "kanagawa",
    "material",
    "melange",
    "modus",
    "mono",
    "monokai",
    "moss",
    "night-owl",
    "nord",
    "oceanic",
    "one-dark",
    "palenight",
    "plum",
    "poimandres",
    "porcelain",
    "retro",
    "rose-pine",
    "sand",
    "sepia",
    "slate",
    "snow",
    "solarized",
    "stage",
    "synthwave",
    "tokyo-night",
    "vitesse",
    "voltage",
    "zenburn",
];

pub fn project_files(
    name: &str,
    template: &str,
    skin: &str,
    palette: &str,
    manager: &str,
) -> Result<Vec<(String, String)>> {
    let vite_plus = manager == "vp";
    let runner = if vite_plus { "vp" } else { "vite" };
    let package_name: String = name
        .to_lowercase()
        .chars()
        .map(
            |ch| if ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' { ch } else { '-' },
        )
        .collect();
    let package_name = package_name.trim_matches('-');
    let version = env!("CARGO_PKG_VERSION");
    let mut dependencies = json!({"@ox-content/vite-plugin": version, "typescript": "^5.8.0"});
    dependencies[format!("@ox-content/theme-{skin}")] = json!(version);
    dependencies[format!("@ox-content/theme-color-{palette}")] = json!(version);
    dependencies[if vite_plus { "vite-plus" } else { "vite" }] =
        json!(if vite_plus { "^0.3.2" } else { "^8.0.0" });
    let pkg = json!({
        "name": if package_name.is_empty() { "my-content" } else { package_name },
        "version": "0.0.0", "private": true, "type": "module",
        "scripts": {"dev": format!("{runner} dev"), "build": format!("{runner} build"), "preview": format!("{runner} preview"), "lint": "oxct lint \"content/**/*.{md,mdx,mdc}\""},
        "devDependencies": dependencies
    });
    let config = match template {
        "docs" => include_str!("templates/docs.ts.txt"),
        "blog" => include_str!("templates/blog.ts.txt"),
        "minimal" => include_str!("templates/minimal.ts.txt"),
        _ => return Err("Unknown project template".into()),
    };
    let mut files = vec![
        ("package.json".to_string(), serde_json::to_string_pretty(&pkg)? + "\n"),
        (
            "vite.config.ts".to_string(),
            config
                .replace("__OX_VITE__", if vite_plus { "vite-plus" } else { "vite" })
                .replace("__OX_SKIN__", skin)
                .replace("__OX_PALETTE__", palette)
                .replace("\"__OX_NAME__\"", &serde_json::to_string(name)?),
        ),
    ];
    let assets: &[(&str, &str)] = match template {
        "docs" => &[
            ("index.html", include_str!("templates/index.html.txt")),
            ("tsconfig.json", include_str!("templates/tsconfig.json.txt")),
            (".gitignore", include_str!("templates/.gitignore.txt")),
            ("README.md", include_str!("templates/README.md.txt")),
            ("content/index.md", include_str!("templates/docs-content-index.md.txt")),
            ("content/guide.md", include_str!("templates/docs-content-guide.md.txt")),
        ],
        "blog" => &[
            ("index.html", include_str!("templates/index.html.txt")),
            ("tsconfig.json", include_str!("templates/tsconfig.json.txt")),
            (".gitignore", include_str!("templates/.gitignore.txt")),
            ("README.md", include_str!("templates/README.md.txt")),
            ("content/index.md", include_str!("templates/blog-content-index.md.txt")),
            (
                "content/posts/welcome.md",
                include_str!("templates/blog-content-posts-welcome.md.txt"),
            ),
        ],
        "minimal" => &[
            ("index.html", include_str!("templates/index.html.txt")),
            ("tsconfig.json", include_str!("templates/tsconfig.json.txt")),
            (".gitignore", include_str!("templates/.gitignore.txt")),
            ("README.md", include_str!("templates/README.md.txt")),
            ("content/index.md", include_str!("templates/minimal-content-index.md.txt")),
        ],
        _ => return Err("Unknown project template".into()),
    };
    for (path, content) in assets {
        files.push((
            (*path).to_string(),
            content
                .replace("\"__OX_NAME__\"", &serde_json::to_string(name)?)
                .replace("__OX_NAME__", name)
                .replace("__OX_TEMPLATE__", template),
        ));
    }
    Ok(files)
}
