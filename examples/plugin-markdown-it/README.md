# markdown-it Plugin Example

This example demonstrates how to use Ox Content with markdown-it.

## Running

```bash
vp run --filter ./examples/plugin-markdown-it start
```

## How It Works

The plugin replaces markdown-it's default parser with Ox Content's high-performance Rust-based parser while maintaining compatibility with markdown-it's plugin ecosystem.

```javascript
import MarkdownIt from "markdown-it";
import { oxContentPlugin } from "./plugin.js";

const md = new MarkdownIt();
md.use(oxContentPlugin, {
  gfm: true,
});

const html = md.render("# Hello World");
```

## Benefits

- **10x faster parsing** with Ox Content's Rust parser
- **Full plugin compatibility** with markdown-it ecosystem
- **Graceful fallback** to markdown-it if parsing fails
