/**
 * Synthetic project generator for the scale benchmark.
 *
 * The pages are not lorem ipsum: each one carries the constructs a real
 * documentation page carries — frontmatter, headings, prose, fenced code in
 * several languages, tables, lists, links to sibling pages, images,
 * containers, footnotes — because what a build spends its time on is
 * highlighting, SSG rendering and asset work, not the Markdown parse. A
 * corpus without code blocks would measure the wrong thing.
 */
import { mkdir, writeFile } from "node:fs/promises";
import * as path from "node:path";

const LANGUAGES = ["ts", "rust", "bash", "json", "python", "go"];

const CODE_SAMPLES = {
  ts: `import { defineConfig } from "vite-plus";\nimport { oxContent } from "@ox-content/vite-plugin";\n\nexport default defineConfig({\n  plugins: [oxContent({ srcDir: "content" })],\n});`,
  rust: `pub fn render(source: &str) -> String {\n    let allocator = Allocator::new();\n    let parser = Parser::new(&allocator, source);\n    HtmlRenderer::new().render(&parser.parse().unwrap())\n}`,
  bash: `#!/usr/bin/env bash\nset -euo pipefail\nnpm install\nnpm run build -- --outDir dist`,
  json: `{\n  "name": "example",\n  "version": "1.0.0",\n  "scripts": { "build": "vp build" }\n}`,
  python: `def render(source: str) -> str:\n    tokens = tokenize(source)\n    return "".join(emit(token) for token in tokens)`,
  go: `func Render(source string) string {\n\tdoc := Parse(source)\n\treturn HTML(doc)\n}`,
};

/** A page's body. `index` seeds the deterministic variation. */
function page(index, total, options) {
  const language = LANGUAGES[index % LANGUAGES.length];
  const previous = (index + total - 1) % total;
  const next = (index + 1) % total;

  const lines = [
    "---",
    `title: Page ${index}`,
    `description: Reference material for section ${index % 40}, generated for the scale benchmark.`,
    "---",
    "",
    `# Page ${index}`,
    "",
    `This page documents section ${index % 40}. It links to [the previous page](./page-${previous}.md)`,
    `and [the next one](./page-${next}.md), and mentions \`renderMarkdown\` inline.`,
    "",
    "## Usage",
    "",
    "```" + language,
    CODE_SAMPLES[language],
    "```",
    "",
    "## Options",
    "",
    "| Option | Type | Default | Description |",
    "| --- | --- | --- | --- |",
    "| `srcDir` | `string` | `content` | Where the Markdown lives. |",
    "| `outDir` | `string` | `dist` | Where the HTML is written. |",
    "| `base` | `string` | `/` | Public base path. |",
    "",
    "## Notes",
    "",
    "- The first item mentions **bold** and *emphasis*.",
    "- The second links to <https://example.com/docs>.",
    "- The third has `inline code` and a footnote.[^note]",
    "",
    "[^note]: Footnotes exercise the definition pre-pass.",
    "",
    "> A block quote, so container and quote handling is on the path.",
    "",
  ];

  if (options.containers) {
    lines.push("::: tip", "Containers are a builtin, so the benchmark pays for them.", ":::", "");
  }
  if (options.images) {
    lines.push(`![Diagram for section ${index % 40}](/img/diagram.png)`, "");
  }
  if (options.math) {
    lines.push("The complexity is $O(n \\log n)$ in the number of nodes.", "");
  }
  if (options.mermaid) {
    lines.push(
      "```mermaid",
      "graph LR",
      `  A${index}[Parse] --> B${index}[Transform]`,
      `  B${index} --> C${index}[Render]`,
      "```",
      "",
    );
  }

  return lines.join("\n");
}

/**
 * Writes `pages` Markdown files under `dir`. Returns the content directory.
 */
export async function generateSite(dir, pages, options = {}) {
  const contentDir = path.join(dir, "content");
  await mkdir(contentDir, { recursive: true });

  const writes = [];
  for (let index = 0; index < pages; index++) {
    const name = index === 0 ? "index.md" : `page-${index}.md`;
    writes.push(writeFile(path.join(contentDir, name), page(index, pages, options)));
  }
  await Promise.all(writes);

  return contentDir;
}
