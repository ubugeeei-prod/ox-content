# unplugin mdast Bridge Example

This example demonstrates how `@ox-content/unplugin` can keep the native Ox Content parser while still running mdast-shaped custom plugins and existing remark plugins in the same unified pipeline.

The runnable project lives in `examples/unplugin-vite-ox-content`.

## Run

```bash
cd examples/unplugin-vite-ox-content
npm install
npm run dev
```

## What this example covers

- A custom `defineMdastPlugin()` transformer that rewrites the first heading.
- An existing remark-style plugin that reads `vfile.data.matter`.
- A final `oxContent` HTML plugin that prepends reading time and wraps the rendered output.
- A browser view that shows the resulting `html`, `frontmatter`, and `toc` exports from the imported Markdown module.

## Configuration

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
  const minutes = Math.ceil(wordCount / 200);
  return `<p class="reading-time">Reading time: ${minutes} min</p>\n${html}`;
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

## Markdown Input

```md
---
title: Unified Bridge Demo
badge: mdast bridge
stage: mdast -> remark -> html
---

# Existing unified plugins still run

This page starts as plain Markdown and is then processed by the Ox Content unified bridge.

## What the bridge changes

- The custom mdast plugin appends a badge to the first heading.
- An existing remark plugin reads `vfile.data.matter` and appends a summary paragraph.
- A final ox-content HTML plugin wraps the output in an `<article>` and prepends reading time.
```

## Imported Module Usage

```ts
import content from "./content.md";

document.getElementById("app")!.innerHTML = `
  <div class="rendered-stage">${content.html}</div>
  <pre>${JSON.stringify(content.frontmatter, null, 2)}</pre>
  <pre>${JSON.stringify(content.toc, null, 2)}</pre>
`;
```

## Rendered Preview

<article>
  <p><strong>Reading time:</strong> 1 min</p>
  <h3>Existing unified plugins still run [mdast bridge]</h3>
  <p>This page starts as plain Markdown and is then processed by the Ox Content unified bridge.</p>
  <p>remark saw frontmatter title: Unified Bridge Demo and stage: mdast -&gt; remark -&gt; html.</p>
</article>

## Generated HTML Excerpt

```html
<article class="ox-content-demo">
  <p class="reading-time">Reading time: 1 min</p>
  <h1>Existing unified plugins still run [mdast bridge]</h1>
  <p>This page starts as plain Markdown and is then processed by the Ox Content unified bridge.</p>
  <h2>What the bridge changes</h2>
  <ul>
    <li>The custom mdast plugin appends a badge to the first heading.</li>
    <li>
      An existing remark plugin reads <code>vfile.data.matter</code> and appends a summary
      paragraph.
    </li>
    <li>
      A final ox-content HTML plugin wraps the output in an <code>&lt;article&gt;</code> and
      prepends reading time.
    </li>
  </ul>
  <p>remark saw frontmatter title: Unified Bridge Demo and stage: mdast -&gt; remark -&gt; html.</p>
</article>
```

## Notes

- `plugin.mdast` is the most mdast-native authoring surface.
- `plugin.remark` still runs in the same mdast stage, so existing unified plugins remain usable.
- TOC extraction happens after mdast-stage transforms, so heading rewrites stay reflected in `content.toc`.
