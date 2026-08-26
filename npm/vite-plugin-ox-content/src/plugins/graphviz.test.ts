import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, describe, expect, it } from "vite-plus/test";
import { renderMarkdown } from "../render-markdown";
import { clearGraphvizCache, transformGraphvizStatic } from "./graphviz";

const originalWarn = console.warn;
let cleanupDirs: string[] = [];

afterEach(() => {
  console.warn = originalWarn;
  delete process.env.OX_GRAPHVIZ_FAKE_LOG;
  clearGraphvizCache();
  for (const dir of cleanupDirs) {
    rmSync(dir, { recursive: true, force: true });
  }
  cleanupDirs = [];
});

describe("graphviz diagrams", () => {
  it("renders DOT fences to sanitized static SVG and leaves raw HTML code alone", async () => {
    const fake = fakeDot();
    const result = await renderMarkdown(
      [
        "# Graph",
        "",
        "```dot",
        "digraph G { a -> b }",
        "```",
        "",
        "`inline dot`",
        "",
        '<pre><code class="language-dot">digraph Raw { a -> b }</code></pre>',
      ].join("\n"),
      "/virtual/graph.md",
      {
        ssg: false,
        highlight: false,
        graphviz: { command: process.execPath, args: [fake.script] },
      },
    );

    expect(result.html).toContain('<figure class="ox-graphviz" role="img"');
    expect(result.html).toContain("<svg");
    expect(result.html).toContain('id="ox-graphviz-0-');
    expect(result.html).not.toContain("<script");
    expect(result.html).not.toContain("onclick");
    expect(result.html).not.toContain("javascript:");
    expect(result.html).toContain('<code class="language-dot">digraph Raw { a -> b }</code>');
  });

  it("keeps duplicate SVG IDs unique per diagram occurrence", async () => {
    const fake = fakeDot();
    const source = [
      "```graphviz",
      "digraph G { a -> b }",
      "```",
      "",
      "```graphviz",
      "digraph G { a -> b }",
      "```",
    ].join("\n");

    const result = await renderMarkdown(source, "/virtual/dupes.md", {
      ssg: false,
      highlight: false,
      graphviz: { command: process.execPath, args: [fake.script] },
    });
    const graphIds = Array.from(result.html.matchAll(/id="(ox-graphviz-\d-[^"]+-graph0)"/g));
    const markerUrls = Array.from(result.html.matchAll(/marker-end="url\(#([^"]+-arrow)\)"/g));

    expect(graphIds).toHaveLength(2);
    expect(new Set(graphIds.map((match) => match[1])).size).toBe(2);
    expect(graphIds[0]?.[1]).toContain("ox-graphviz-0-");
    expect(graphIds[1]?.[1]).toContain("ox-graphviz-1-");
    expect(markerUrls).toHaveLength(2);
    expect(new Set(markerUrls.map((match) => match[1])).size).toBe(2);
  });

  it("leaves DOT fences as code blocks when disabled", async () => {
    const result = await renderMarkdown("```dot\ndigraph G { a -> b }\n```\n", "/virtual/off.md", {
      ssg: false,
      highlight: false,
      graphviz: false,
    });

    expect(result.html).toContain('<pre><code class="language-dot">');
    expect(result.html).not.toContain("ox-graphviz");
  });

  it("reports a missing renderer according to policy", async () => {
    const source = "```dot\ndigraph G { a -> b }\n```\n";
    await expect(
      renderMarkdown(source, "/virtual/missing.md", {
        ssg: false,
        highlight: false,
        graphviz: { command: "/definitely/missing/dot" },
      }),
    ).rejects.toThrow("Graphviz renderer not found");

    const warnings: string[] = [];
    console.warn = (message?: unknown) => warnings.push(String(message));
    const warned = await renderMarkdown(source, "/virtual/missing-warn.md", {
      ssg: false,
      highlight: false,
      graphviz: { command: "/definitely/missing/dot", missingRenderer: "warn" },
    });

    expect(warnings[0]).toContain("Graphviz renderer not found");
    expect(warned.html).toContain('<pre><code class="language-dot">');
  });

  it("reports invalid DOT sources according to policy", async () => {
    const fake = fakeDot();
    const source = "```dot\nsyntax_error\n```\n";
    await expect(
      renderMarkdown(source, "/virtual/bad.md", {
        ssg: false,
        highlight: false,
        graphviz: { command: process.execPath, args: [fake.script] },
      }),
    ).rejects.toThrow("syntax error near syntax_error");

    const warnings: string[] = [];
    console.warn = (message?: unknown) => warnings.push(String(message));
    const warned = await renderMarkdown(source, "/virtual/bad-warn.md", {
      ssg: false,
      highlight: false,
      graphviz: {
        command: process.execPath,
        args: [fake.script],
        renderErrors: "warn",
      },
    });

    expect(warnings[0]).toContain("syntax error near syntax_error");
    expect(warned.html).toContain('<pre><code class="language-dot">');
  });

  it("caches rendered SVGs by source and renderer options", async () => {
    const fake = fakeDot();
    const log = join(fake.root, "renders.log");
    process.env.OX_GRAPHVIZ_FAKE_LOG = log;
    const html = '<pre><code class="language-dot">digraph G { a -&gt; b }</code></pre>';

    await transformGraphvizStatic(html, { command: process.execPath, args: [fake.script, "a"] });
    await transformGraphvizStatic(html, { command: process.execPath, args: [fake.script, "a"] });
    await transformGraphvizStatic(html, { command: process.execPath, args: [fake.script, "b"] });

    const renders = readFileSync(log, "utf8").trim().split("\n");
    expect(renders).toHaveLength(2);
    expect(renders[0]).toContain("a:digraph G { a -> b }");
    expect(renders[1]).toContain("b:digraph G { a -> b }");
  });
});

function fakeDot(): { root: string; script: string } {
  const root = mkdtempSync(join(tmpdir(), "ox-content-graphviz-"));
  cleanupDirs.push(root);
  const script = join(root, "fake-dot.mjs");
  writeFileSync(script, fakeDotSource(), "utf8");
  return { root, script };
}

function fakeDotSource(): string {
  return `
import { appendFileSync } from "node:fs";

const version = process.argv.includes("-V");
const variant = process.argv.slice(2).find((arg) => !arg.startsWith("-")) ?? "default";
if (version) {
  console.error("fake-dot " + variant);
  process.exit(0);
}

let input = "";
process.stdin.setEncoding("utf8");
process.stdin.on("data", (chunk) => {
  input += chunk;
});
process.stdin.on("end", () => {
  if (input.includes("syntax_error")) {
    console.error("syntax error near syntax_error");
    process.exit(1);
  }
  if (process.env.OX_GRAPHVIZ_FAKE_LOG) {
    appendFileSync(process.env.OX_GRAPHVIZ_FAKE_LOG, variant + ":" + input.trim() + "\\n");
  }
  process.stdout.write([
    "<?xml version=\\"1.0\\"?>",
    "<!DOCTYPE svg>",
    "<svg id=\\"graph0\\" width=\\"10pt\\" height=\\"10pt\\" viewBox=\\"0 0 10 10\\">",
    "<g id=\\"node1\\" class=\\"node\\"><title>Node</title>",
    "<script>alert(1)</script>",
    "<a xlink:href=\\"javascript:alert(1)\\" onclick=\\"evil()\\"><text>Node</text></a>",
    "</g>",
    "<g id=\\"edge1\\" class=\\"edge\\"><path marker-end=\\"url(#arrow)\\" d=\\"M0,0L1,1\\" /></g>",
    "<marker id=\\"arrow\\"></marker>",
    "</svg>",
  ].join(""));
});
`;
}
