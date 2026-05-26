# unplugin markdown-it Token Bridge

This example demonstrates how `@ox-content/unplugin` can run `markdown-it` plugins first and then expose the resulting token stream to downstream remark/unified plugins through `vfile.data.oxContent.markdownIt.tokens`.

## What this example covers

- A `markdown-it` plugin that rewrites heading text.
- A downstream remark plugin that reads the rewritten token stream from `vfile.data`.
- A final HTML output that reflects both stages in one pipeline.

## Configuration

```ts
import { defineConfig } from "vite-plus";
import oxContent, { type MdastRoot } from "@ox-content/unplugin/vite";
import type MarkdownIt from "markdown-it";

function markdownItHeadingPlugin(md: MarkdownIt) {
  md.core.ruler.push("rewrite-heading", (state) => {
    const inline = state.tokens[1];
    if (!inline || inline.type !== "inline") {
      return;
    }

    for (const child of inline.children ?? []) {
      if (child.type === "text") {
        child.content = "Hello from markdown-it tokens";
      }
    }
  });
}

function remarkReadMarkdownItTokens() {
  return (
    tree: MdastRoot,
    file: {
      data?: {
        oxContent?: {
          markdownIt?: {
            tokens?: Array<{
              type?: string;
              children?: Array<{ type?: string; content?: string }>;
            }>;
          };
        };
      };
    },
  ) => {
    const inline = file.data?.oxContent?.markdownIt?.tokens?.find(
      (token) => token.type === "inline",
    );
    const text = inline?.children?.find((token) => token.type === "text")?.content;
    if (!text) {
      return;
    }

    tree.children.push({
      type: "paragraph",
      children: [{ type: "text", value: `From token stream: ${text}` }],
    });
  };
}

export default defineConfig({
  plugins: [
    oxContent({
      plugin: {
        markdownIt: [markdownItHeadingPlugin],
        remark: [remarkReadMarkdownItTokens],
      },
    }),
  ],
});
```

## Markdown Input

```md
# Hello
```

## Rendered Preview

```html
<h1>Hello from markdown-it tokens</h1>
<p>From token stream: Hello from markdown-it tokens</p>
```

## Notes

- `plugin.markdownIt` runs before the unified mdast/remark bridge.
- Downstream unified plugins can inspect the original `markdown-it` result on `file.data.oxContent.markdownIt`.
- This is the intended handoff point for the future Rust-side zero-copy markdown-it token transfer path.
