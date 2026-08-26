import { describe, expect, it } from "vite-plus/test";
import { createDocsResolvedOptions } from "../test/fixtures/docs-fixture";
import { resolveAbbreviationsOptions } from "./index";
import { transformMarkdown } from "./transform";
import type { ResolvedOptions } from "./types";

function options(overrides: Partial<ResolvedOptions> = {}): ResolvedOptions {
  return createDocsResolvedOptions({ highlight: false, ...overrides });
}

describe("resolveAbbreviationsOptions", () => {
  it("omitted => false; true => true; {} => true", () => {
    expect(resolveAbbreviationsOptions(undefined)).toEqual({
      enabled: false,
      terms: {},
      firstUseOnly: false,
    });
    expect(resolveAbbreviationsOptions(true)).toEqual({
      enabled: true,
      terms: {},
      firstUseOnly: false,
    });
    expect(resolveAbbreviationsOptions({})).toEqual({
      enabled: true,
      terms: {},
      firstUseOnly: false,
    });
  });

  it("accepts boolean and object forms", () => {
    expect(resolveAbbreviationsOptions({ enabled: false }).enabled).toBe(false);
    expect(
      resolveAbbreviationsOptions({
        terms: { LSP: "Language Server Protocol" },
        firstUseOnly: true,
      }),
    ).toEqual({
      enabled: true,
      terms: { LSP: "Language Server Protocol" },
      firstUseOnly: true,
    });
  });
});

describe("abbreviations transform", () => {
  it("leaves definitions literal unless opted in", async () => {
    const markdown = "*[LSP]: Language Server Protocol\n\nUse LSP today.\n";

    const defaultResult = await transformMarkdown(markdown, "docs/abbreviations.md", options());
    expect(defaultResult.html).not.toContain("ox-abbr");
    expect(defaultResult.html).toContain("*[LSP]");

    const enabledResult = await transformMarkdown(
      markdown,
      "docs/abbreviations.md",
      options({
        abbreviations: { enabled: true, terms: {}, firstUseOnly: false },
      }),
    );
    expect(enabledResult.html).toContain('<abbr class="ox-abbr" title="Language Server Protocol">');
    expect(enabledResult.html).toContain("LSP");
    expect(enabledResult.html).not.toContain("*[LSP]");
  });
});
