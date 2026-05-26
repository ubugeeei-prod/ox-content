# ox_content_mdc_checker

Static checker and component registry for [MDC](https://mdxjs.com/docs/what-is-mdx/#mdx-syntax) component syntax in Ox Content Markdown.

## Library

```rust
use ox_content_mdc_checker::{check, Registry};

// Tag-balance + attribute diagnostics.
for diagnostic in check(source) {
    eprintln!("{}:{} {}", diagnostic.line, diagnostic.column, diagnostic.message);
}

// Component / attribute completion data, loaded from JSON.
let registry = Registry::from_path(Path::new("ox-content.components.json"))?
    .unwrap_or_default();
for (name, _component) in registry.complete_components("Al") {
    println!("Alert-like component: {name}");
}
```

## CLI

```bash
ox-content-mdc-check docs/**/*.mdc
ox-content-mdc-check --format json docs/page.mdc
```

Exit code is `1` when any diagnostic was emitted or any file failed to read.

## Component registry

Editors load a JSON file describing every MDC component used by the
project. The file powers component name completion after `<` and
attribute name completion inside an opening tag. Editor surfaces for
the registry:

- VS Code: `oxContent.mdc.components` setting (absolute or
  workspace-relative path).
- LSP `initializationOptions.mdcComponents`.
- Workspace config: `"mdc": { "components": "./components.json" }`
  inside `.ox-content.json` or `ox-content.json`.
- Environment variable: `OX_CONTENT_MDC_COMPONENTS`.

### File shape

```json
{
  "components": [
    {
      "name": "Alert",
      "description": "Inline alert callout.",
      "attributes": [
        {
          "name": "tone",
          "type": "info | warn | error",
          "description": "Severity / colour of the alert.",
          "required": true
        },
        { "name": "icon", "description": "Mdi icon name" }
      ]
    },
    {
      "name": "Card",
      "attributes": [{ "name": "title" }]
    }
  ]
}
```

Unknown top-level fields and unknown per-component fields are
tolerated so older editors keep working when the schema grows.

The `required` flag is reserved for a future diagnostic; it does not
change completion behavior today.
