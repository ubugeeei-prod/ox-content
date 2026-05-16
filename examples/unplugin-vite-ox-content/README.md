# Vite mdast Bridge Example

This example demonstrates the new mdast bridge in `@ox-content/unplugin`.

It combines three stages in one pipeline:

1. `defineMdastPlugin()` mutates the parsed mdast tree.
2. An existing remark-style plugin reads `vfile.data.matter`.
3. An `oxContent` HTML plugin wraps the final rendered output.

## Run

```bash
cd examples/unplugin-vite-ox-content
npm install
npm run dev
```

## What to look for

- The first heading is rewritten to include `[mdast bridge]`.
- The final paragraph is appended by the remark plugin using frontmatter data.
- The imported module exposes `frontmatter`, `toc`, and transformed `html`.

## Core configuration

```ts
import { defineConfig } from "vite-plus";
import oxContent, {
  defineMdastPlugin,
  type MdastRoot,
  type OxContentPlugin,
} from "@ox-content/unplugin/vite";

const annotateHeadings = defineMdastPlugin("annotate-headings", (tree, context) => {
  const badge = String(context.frontmatter.badge ?? "mdast bridge");

  for (const node of tree.children) {
    if (node.type === "heading" && node.depth === 1 && Array.isArray(node.children)) {
      node.children.push({ type: "text", value: ` [${badge}]` });
      break;
    }
  }
});

function remarkExposeFrontmatter() {
  return (tree: MdastRoot, file: { data?: { matter?: { title?: string; stage?: string } } }) => {
    tree.children.push({
      type: "paragraph",
      children: [
        {
          type: "text",
          value:
            `remark saw frontmatter title: ${file.data?.matter?.title ?? "missing-title"} ` +
            `and stage: ${file.data?.matter?.stage ?? "missing-stage"}.`,
        },
      ],
    });
  };
}

const addReadingTime: OxContentPlugin = (html) => {
  const wordCount = html.replace(/<[^>]*>/g, "").split(/\s+/).length;
  return `<p class="reading-time">Reading time: ${Math.ceil(wordCount / 200)} min</p>\n${html}`;
};

const wrapInArticle: OxContentPlugin = (html) => {
  return `<article class="ox-content-demo">${html}</article>`;
};

export default defineConfig({
  plugins: [
    oxContent({
      toc: true,
      plugin: {
        mdast: [annotateHeadings],
        remark: [remarkExposeFrontmatter],
        oxContent: [addReadingTime, wrapInArticle],
      },
    }),
  ],
});
```

## Markdown input

```md
---
title: Unified Bridge Demo
badge: mdast bridge
stage: mdast -> remark -> html
---

# Existing unified plugins still run

This page starts as plain Markdown and is then processed by the Ox Content unified bridge.
```

## Resulting HTML excerpt

```html
<article class="ox-content-demo">
  <p class="reading-time">Reading time: 1 min</p>
  <h1>Existing unified plugins still run [mdast bridge]</h1>
  <p>This page starts as plain Markdown and is then processed by the Ox Content unified bridge.</p>
  <p>remark saw frontmatter title: Unified Bridge Demo and stage: mdast -> remark -> html.</p>
</article>
```

## Advanced: markdown-it token bridge

When you configure `plugin.markdownIt`, downstream remark/unified plugins can read the token stream on `file.data.oxContent.markdownIt.tokens`.

```ts
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
```
