import { afterEach, describe, expect, it } from "vite-plus/test";
import { createDocsResolvedOptions } from "../test/fixtures/docs-fixture";
import type { ResolvedOptions } from "./types";
import { renderMarkdown } from "./render-markdown";
import { transformMarkdown } from "./transform";

const originalWarn = console.warn;

afterEach(() => {
  console.warn = originalWarn;
});

describe("cross references", () => {
  it("links labeled sections, figures, and tables with deterministic numbers", async () => {
    const result = await transformMarkdown(
      [
        "# Guide",
        "",
        "## Install {#sec-install}",
        "",
        "See @sec-install, @fig-pipeline, and @tbl-options.",
        "",
        '![Pipeline](/pipeline.png "Render pipeline"){#fig-pipeline}',
        "",
        "| Option | Value |",
        "| --- | --- |",
        "| mode | static |",
        "{#tbl-options}",
      ].join("\n"),
      "/virtual/xrefs.md",
      options(),
    );

    expect(result.html).toContain(
      '<a class="ox-xref ox-xref-section" href="#sec-install" data-ox-xref-id="sec-install" data-ox-xref-kind="section">Section 1.1</a>',
    );
    expect(result.html).toContain(
      '<a class="ox-xref ox-xref-figure" href="#fig-pipeline" data-ox-xref-id="fig-pipeline" data-ox-xref-kind="figure">Figure 1</a>',
    );
    expect(result.html).toContain(
      '<a class="ox-xref ox-xref-table" href="#tbl-options" data-ox-xref-id="tbl-options" data-ox-xref-kind="table">Table 1</a>',
    );
    expect(result.html).toContain(
      'data-ox-xref-kind="figure" data-ox-xref-number="1" data-ox-xref-label="Figure 1"',
    );
    expect(result.html).toContain(
      '<table id="tbl-options" data-ox-xref-kind="table" data-ox-xref-number="1" data-ox-xref-label="Table 1">',
    );
    expect(result.crossReferences).toEqual([
      {
        id: "sec-install",
        kind: "section",
        number: "1.1",
        label: "Section",
        text: "Section 1.1",
        href: "#sec-install",
        title: "Install",
      },
      {
        id: "fig-pipeline",
        kind: "figure",
        number: "1",
        label: "Figure",
        text: "Figure 1",
        href: "#fig-pipeline",
        title: "Render pipeline",
      },
      {
        id: "tbl-options",
        kind: "table",
        number: "1",
        label: "Table",
        text: "Table 1",
        href: "#tbl-options",
      },
    ]);
    expect(result.code).toContain("export const crossReferences =");
  });

  it("keeps reference-looking prose literal when disabled", async () => {
    const result = await renderMarkdown(
      "See @fig-pipeline.\n\n![Pipeline](/pipeline.png){#fig-pipeline}\n",
      "/virtual/disabled.md",
      {
        attrs: true,
        images: true,
        xrefs: false,
        highlight: false,
      },
    );

    expect(result.html).toContain("@fig-pipeline");
    expect(result.html).not.toContain("ox-xref");
  });

  it("does not rewrite references inside code or existing links", async () => {
    const result = await transformMarkdown(
      [
        "Inline `@fig-pipeline` stays code.",
        "",
        "```md",
        "@fig-pipeline",
        "```",
        "",
        '<a href="/manual">@fig-pipeline</a>',
        "",
        "See @fig-pipeline.",
        "",
        "![Pipeline](/pipeline.png){#fig-pipeline}",
      ].join("\n"),
      "/virtual/protected.md",
      options(),
    );

    expect(result.html).toContain("<code>@fig-pipeline</code>");
    expect(result.html).toContain('<pre><code class="language-md">@fig-pipeline\n</code></pre>');
    expect(result.html).toContain('<a href="/manual">@fig-pipeline</a>');
    expect(result.html.match(/class="ox-xref ox-xref-figure"/g)).toHaveLength(1);
  });

  it("reports missing, duplicate, and mismatched labels", async () => {
    await expect(
      transformMarkdown("See @fig-missing.\n", "/virtual/missing.md", options()),
    ).rejects.toThrow('missing cross-reference target "fig-missing"');

    await expect(
      transformMarkdown(
        ["# One {#sec-dup}", "", "## Two {#sec-dup}", "", "See @sec-dup."].join("\n"),
        "/virtual/duplicate.md",
        options(),
      ),
    ).rejects.toThrow('duplicate cross-reference target "sec-dup"');

    await expect(
      transformMarkdown(
        ["# Not a figure {#fig-wrong}", "", "See @fig-wrong."].join("\n"),
        "/virtual/mismatch.md",
        options(),
      ),
    ).rejects.toThrow('cross-reference "fig-wrong" expects figure but found section');
  });

  it("can warn and leave unresolved references literal", async () => {
    const warnings: string[] = [];
    console.warn = (message?: unknown) => warnings.push(String(message));
    const result = await transformMarkdown(
      "See @fig-missing.\n",
      "/virtual/warn.md",
      options({
        crossReferences: {
          ...defaultCrossReferences,
          missing: "warn",
        },
      }),
    );

    expect(result.html).toContain("@fig-missing");
    expect(warnings[0]).toContain('missing cross-reference target "fig-missing"');
  });
});

const defaultCrossReferences = {
  enabled: true,
  missing: "error",
  duplicates: "error",
  mismatches: "error",
  labels: { figure: "Figure", table: "Table", section: "Section" },
} satisfies ResolvedOptions["crossReferences"];

function options(overrides: Partial<ResolvedOptions> = {}): ResolvedOptions {
  return createDocsResolvedOptions({
    highlight: false,
    attrs: { enabled: true },
    images: { enabled: true, lazy: true },
    crossReferences: defaultCrossReferences,
    ...overrides,
  });
}
