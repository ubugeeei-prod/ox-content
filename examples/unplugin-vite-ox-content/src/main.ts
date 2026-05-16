import "./style.css";
import content from "./content.md";

const configSource = `import { defineConfig } from "vite-plus";
import oxContent, { defineMdastPlugin } from "@ox-content/unplugin/vite";

const annotateHeadings = defineMdastPlugin("annotate-headings", (tree, context) => {
  const badge = String(context.frontmatter.badge ?? "mdast bridge");

  for (const node of tree.children) {
    if (node.type === "heading" && node.depth === 1 && Array.isArray(node.children)) {
      node.children.push({ type: "text", value: \` [\${badge}]\` });
      break;
    }
  }
});

function remarkExposeFrontmatter() {
  return (tree, file) => {
    tree.children.push({
      type: "paragraph",
      children: [
        {
          type: "text",
          value: \`remark saw frontmatter title: \${file.data?.matter?.title ?? "missing"}.\`,
        },
      ],
    });
  };
}

export default defineConfig({
  plugins: [
    oxContent({
      plugin: {
        mdast: [annotateHeadings],
        remark: [remarkExposeFrontmatter],
      },
    }),
  ],
});`;

const markdownSource = `---
title: Unified Bridge Demo
badge: mdast bridge
stage: mdast -> remark -> html
---

# Existing unified plugins still run

This page starts as plain Markdown and is then processed by the Ox Content unified bridge.`;

document.getElementById("app")!.innerHTML = `
  <header class="page-header">
    <h1>Ox Content mdast bridge example</h1>
    <p>
      This example shows a custom <code>defineMdastPlugin()</code>, an existing remark-style plugin,
      and the final HTML output living in the same pipeline.
    </p>
  </header>

  <section class="example-grid">
    <div class="panel">
      <div class="panel-header">
        <h2>Rendered result</h2>
        <p>The imported module already contains transformed HTML, frontmatter, and TOC.</p>
      </div>
      <div class="rendered-stage">${content.html}</div>
    </div>

    <aside class="panel">
      <div class="panel-header">
        <h2>Imported metadata</h2>
      </div>
      <div class="panel-body metadata-list">
        <div>
          <h3>Frontmatter</h3>
          <pre>${JSON.stringify(content.frontmatter, null, 2)}</pre>
        </div>
        <div>
          <h3>TOC</h3>
          <pre>${JSON.stringify(content.toc, null, 2)}</pre>
        </div>
      </div>
    </aside>
  </section>

  <section class="example-grid" style="margin-top: 20px;">
    <div class="panel">
      <div class="panel-header">
        <h2>vite.config.ts</h2>
        <p>The bridge keeps mdast authoring ergonomic while remaining unified-compatible.</p>
      </div>
      <div class="panel-body source-block">
        <pre>${escapeHtml(configSource)}</pre>
      </div>
    </div>

    <div class="panel">
      <div class="panel-header">
        <h2>content.md</h2>
      </div>
      <div class="panel-body source-block">
        <pre>${escapeHtml(markdownSource)}</pre>
      </div>
    </div>
  </section>
`;

function escapeHtml(value: string): string {
  return value.replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;");
}
