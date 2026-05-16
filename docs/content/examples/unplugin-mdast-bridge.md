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

## Compatibility Model

The bridge is designed for practical mdast and remark plugin reuse without making the no-plugin path pay
for unified. When `plugin.mdast`, `plugin.remark`, `plugin.rehype`, or `plugin.markdownIt` is configured,
Ox Content enters the unified bridge. Without those plugins, Markdown still uses the native Rust parse and
render path.

The supported path covers:

- `defineMdastPlugin()` functions that mutate the Rust-produced mdast tree.
- Existing remark/unified transformer functions, plugin tuples, and preset objects.
- `plugin.remark` and `plugin.mdast` in the same mdast stage, with TOC extraction after those transforms.
- `vfile.data.matter`, `vfile.data.frontmatter`, `file.data.oxContent`, and original-source offsets for diagnostics.
- Explicit `remark-rehype` and `rehype-stringify` bridge plugins when a project wants to own those stages.
- Custom unified parsers and compilers, when they are registered by the user.

There are two important compatibility boundaries:

- remark syntax extensions and custom parsers force a fallback to `remark-parse`, because the Rust parser
  cannot execute micromark extensions.
- `plugin.markdownIt` runs first and exposes tokens at `file.data.oxContent.markdownIt.tokens`; downstream
  remark plugins can inspect those tokens, but that path is a token/HTML bridge rather than a perfect
  source-mdast representation of markdown-it internals.

## Performance Expectations

The native fast path remains the performance target. The mdast bridge is a compatibility path, so it adds
JS mdast materialization plus unified plugin execution.

A local Node `v24.15.0` smoke benchmark over a 45 KB Markdown fixture measured:

| Path               | Time per document | Throughput |
| ------------------ | ----------------: | ---------: |
| native fast path   |          1.215 ms | 34.6 MiB/s |
| mdast no-op plugin |         10.154 ms |  4.1 MiB/s |
| remark no-op       |         10.597 ms |  4.0 MiB/s |

The Rust-side transfer work is much smaller than the end-to-end bridge cost. On the same large fixture,
`transformMdastRaw` measured about 0.594 ms and `transform_html` measured about 0.686 ms in Criterion.
That means the current bottleneck is mostly JS object materialization and unified processing, not the Rust
parser itself. The raw transfer format is intentionally explicit, but its current payload is larger than
the JSON export, so raw format and deserializer tuning remain good follow-up optimization targets.

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
