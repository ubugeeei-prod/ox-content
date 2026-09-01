# rehype/unified Integration Example

This example demonstrates how to use Ox Content with the unified/rehype ecosystem.

## Running

```bash
vp run --filter ./examples/plugin-rehype start
```

## How It Works

1. Parse Markdown with Ox Content (fast Rust parser)
2. Pass HTML through unified/rehype pipeline
3. Apply rehype transformations
4. Output final HTML

```javascript
import { unified } from "unified";
import rehypeParse from "rehype-parse";
import rehypeStringify from "rehype-stringify";
import { parseAndRender } from "@ox-content/napi";

// Get HTML from Ox Content
const { html } = parseAndRender(markdown, { gfm: true });

// Process with rehype
const result = await unified()
  .use(rehypeParse, { fragment: true })
  .use(rehypeAddClasses)
  .use(rehypeSyntaxHighlight)
  .use(rehypeStringify)
  .process(html);
```

## Benefits

- Fast parsing with Ox Content
- Full rehype plugin compatibility
- Transform and enhance HTML output
