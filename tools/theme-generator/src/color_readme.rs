use crate::colors::Palette;

pub(crate) fn readme(palette: &Palette, export_name: &str) -> String {
    let variants_section = variants_section(palette, export_name);
    format!(
        r##"# @ox-content/theme-color-{id}

{title} — {description} — for [Ox Content](https://github.com/ubugeeei-prod/ox-content).

**Color only.** Light and dark `--octc-*` tokens and nothing else: no layout, no
texture, no typography. That is what lets it drop under any skin package.

```bash
npm install @ox-content/theme-color-{id}
```

```ts
// vite.config.ts
import {{ defineConfig }} from "vite";
import {{ oxContent }} from "@ox-content/vite-plugin";
import {export_name} from "@ox-content/theme-color-{id}";

export default defineConfig({{
  plugins: [
    oxContent({{
      srcDir: "docs",
      ssg: {{ siteName: "My Docs", theme: {export_name} }},
    }}),
  ],
}});
```

Stack a skin on top to change the form as well — layers compose left to right,
so anything you append wins:

```ts
import pixel from "@ox-content/theme-pixel";

theme: [pixel, {export_name}, {{ colors: {{ primary: "#ff5f56" }} }}];
```
{variants_section}

## License

MIT
"##,
        id = palette.id,
        title = palette.title,
        description = palette.description,
    )
}

fn variants_section(palette: &Palette, export_name: &str) -> String {
    let variants = palette.config_entries(export_name);
    let variant_names =
        variants.iter().skip(1).map(|entry| entry.export_name.as_str()).collect::<Vec<_>>();
    if variant_names.is_empty() {
        return String::new();
    }

    let variant_list = palette
        .variants
        .iter()
        .zip(variant_names.iter())
        .map(|(variant, name)| {
            format!(
                "- `{name}` — {}",
                variant.description.as_deref().unwrap_or(&palette.description)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"
## Variants

The default export stays `{export_name}` ({description}). Named variants
are exported from the same package:

```ts
import {{ {imports} }} from "@ox-content/theme-color-{id}";
```

{variant_list}
"#,
        description = palette.description,
        imports = variant_names.join(", "),
        id = palette.id,
    )
}
