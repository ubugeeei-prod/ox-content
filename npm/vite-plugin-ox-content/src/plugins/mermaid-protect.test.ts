import { spawnSync } from "node:child_process";
import { describe, expect, it } from "vite-plus/test";
import {
  protectMermaidSvgs,
  protectStaticDiagramSvgs,
  restoreMermaidSvgs,
  restoreStaticDiagramSvgs,
} from "./mermaid-protect";

const mermaid =
  '<div class="ox-mermaid"><svg><foreignObject><div>a<br /></div></foreignObject></svg></div>';
const graphviz = '<figure class="ox-graphviz"><svg><g>b</g></svg></figure>';

describe("static diagram protection", () => {
  it("preserves ordered mixed diagrams and all surrounding text", () => {
    const html = `before${mermaid}between${graphviz}after`;
    const result = protectStaticDiagramSvgs(html);
    expect(result.html).toBe(
      "before<!--ox-static-diagram-0-->between<!--ox-static-diagram-1-->after",
    );
    expect([...result.svgs.values()]).toEqual([mermaid, graphviz]);
    expect(restoreStaticDiagramSvgs(result.html, result.svgs)).toBe(html);
  });

  it.each(["İ", "日本語🙂", "AΣİZ", "𝔘nicode"])("keeps original offsets after %s", (prefix) => {
    const html = `${prefix}${mermaid}${prefix}${graphviz}${prefix}`;
    const result = protectStaticDiagramSvgs(html);
    expect([...result.svgs.values()]).toEqual([mermaid, graphviz]);
    expect(restoreStaticDiagramSvgs(result.html, result.svgs)).toBe(html);
  });

  it("matches marker and nested closing tags case-insensitively", () => {
    const diagram = '<DIV CLASS="OX-MERMAID"><div>nested</DIV></div>';
    const result = protectStaticDiagramSvgs(diagram);
    expect([...result.svgs.values()]).toEqual([diagram]);
    expect(result.html).toBe("<!--ox-static-diagram-0-->");
  });

  it("protects an outer diagram together with nested diagrams", () => {
    const outer = `<div class="ox-mermaid">${graphviz}${mermaid}</div>`;
    const result = protectStaticDiagramSvgs(outer + graphviz);
    expect([...result.svgs.values()]).toEqual([outer, graphviz]);
    expect(restoreStaticDiagramSvgs(result.html, result.svgs)).toBe(outer + graphviz);
  });

  it("keeps malformed tails and unrelated marker-like classes unchanged", () => {
    const tail = `<div class="ox-mermaid"><svg>unclosed${graphviz}`;
    const html = `<div class="other ox-mermaid">other</div>${mermaid}${tail}`;
    const result = protectStaticDiagramSvgs(html);
    expect([...result.svgs.values()]).toEqual([mermaid]);
    expect(result.html.endsWith(tail)).toBe(true);
    expect(restoreStaticDiagramSvgs(result.html, result.svgs)).toBe(html);
  });

  it("preserves aliases and unknown placeholders", () => {
    const html = `<!--ox-static-diagram-999-->${graphviz}`;
    const result = protectMermaidSvgs(html);
    expect(restoreMermaidSvgs(result.html, result.svgs)).toBe(html);
    expect(protectStaticDiagramSvgs("plain text")).toEqual({ html: "plain text", svgs: new Map() });
  });

  it("protects and restores 2,000 diagrams within a 64 MiB JS heap", () => {
    const moduleUrl = new URL("./mermaid-protect.ts", import.meta.url).href;
    const program = `
      import { protectStaticDiagramSvgs, restoreStaticDiagramSvgs } from ${JSON.stringify(moduleUrl)};
      const source = (${JSON.stringify("ordinary prose ".repeat(40))} + ${JSON.stringify(mermaid + graphviz)}).repeat(1000);
      const result = protectStaticDiagramSvgs(source);
      if (result.svgs.size !== 2000 || restoreStaticDiagramSvgs(result.html, result.svgs) !== source) process.exit(1);
      process.stdout.write("ok");
    `;
    const result = spawnSync(
      process.execPath,
      ["--max-old-space-size=64", "--input-type=module", "--eval", program],
      {
        encoding: "utf8",
        timeout: 20_000,
        maxBuffer: 1024 * 1024,
      },
    );
    expect(result.error).toBeUndefined();
    expect(result.status, result.stderr.slice(-2000)).toBe(0);
    expect(result.stdout).toBe("ok");
  });
});
