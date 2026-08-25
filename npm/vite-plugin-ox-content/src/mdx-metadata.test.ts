import { describe, expect, it } from "vite-plus/test";
import { createDocsResolvedOptions } from "../test/fixtures/docs-fixture";
import { transformMarkdown } from "./transform";
import type { ResolvedOptions } from "./types";

describe("MDX module metadata", () => {
  it("exports import lists, export names, and component names", async () => {
    const result = await transformMarkdown(
      [
        "import Alert from './Alert'",
        "import { Chart as Plot } from './Chart'",
        "import * as Icons from './icons'",
        "export const title = 'Guide'",
        "export function helper() {}",
        "",
        "<Alert />",
        "",
        "Hello <Badge /> and <Icons.Star />",
      ].join("\n"),
      "docs/guide.mdx",
      createResolvedOptions(),
    );

    expect(result.imports).toEqual([
      {
        source: "./Alert",
        specifiers: [{ imported: "default", local: "Alert", kind: "default" }],
      },
      {
        source: "./Chart",
        specifiers: [{ imported: "Chart", local: "Plot", kind: "named" }],
      },
      {
        source: "./icons",
        specifiers: [{ imported: "*", local: "Icons", kind: "namespace" }],
      },
    ]);
    expect(result.exports).toEqual(["title", "helper"]);
    expect(result.components).toEqual(["Alert", "Badge", "Icons.Star"]);
    expect(result.html).toContain('data-ox-island="Alert"');
    expect(result.code).toContain("export const html =");
    expect(result.code).toContain("export const imports =");
    expect(result.code).toContain("export const exports =");
    expect(result.code).toContain("export const components =");
    expect(result.code).toContain("imports,");
    expect(result.code).toContain("exports,");
    expect(result.code).toContain("components,");
  });

  it("keeps html and does not execute import side effects", async () => {
    const result = await transformMarkdown(
      [
        "import { boom } from './throws-on-import.js'",
        "export const leaked = (() => { throw new Error('executed') })()",
        "",
        "<Alert />",
      ].join("\n"),
      "docs/side-effects.mdx",
      createResolvedOptions(),
    );

    expect(result.html).toContain('data-ox-island="Alert"');
    expect(result.imports).toEqual([
      {
        source: "./throws-on-import.js",
        specifiers: [{ imported: "boom", local: "boom", kind: "named" }],
      },
    ]);
    expect(result.exports).toEqual(["leaked"]);
    expect(result.code).toContain("export const html =");
    expect(result.code).not.toMatch(/^import\s/m);
    expect(result.code).not.toContain("from './throws-on-import.js'");
    expect(result.code).not.toContain("throw new Error('executed')");
    expect(result.code).toContain('"./throws-on-import.js"');
  });

  it("exports empty metadata arrays when MDX is off", async () => {
    const result = await transformMarkdown(
      "import Alert from './Alert'\n\n<Alert />\n",
      "docs/guide.md",
      createResolvedOptions({ mdx: false }),
    );

    expect(result.imports).toEqual([]);
    expect(result.exports).toEqual([]);
    expect(result.components).toEqual([]);
    expect(result.code).toContain("export const imports = []");
    expect(result.code).toContain("export const exports = []");
    expect(result.code).toContain("export const components = []");
    expect(result.code).toContain("export const html =");
  });
});

function createResolvedOptions(overrides: Partial<ResolvedOptions> = {}): ResolvedOptions {
  return createDocsResolvedOptions({ highlight: false, ...overrides });
}
