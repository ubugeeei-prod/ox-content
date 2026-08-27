import { describe, expect, it } from "vite-plus/test";
import { createDocsResolvedOptions } from "../test/fixtures/docs-fixture";
import { transformMarkdown } from "./transform";
import type { ResolvedOptions } from "./types";

function createResolvedOptions(overrides: Partial<ResolvedOptions> = {}): ResolvedOptions {
  return createDocsResolvedOptions(overrides);
}

describe("package-manager tabs", () => {
  // Package-manager tabs are generated after the page's first highlight pass,
  // so without a second pass they reach the reader as plain text beside
  // authored shell fences that are tokenised.
  it("highlights the commands inside package-manager tabs", async () => {
    const result = await transformMarkdown(
      "<pm>npm install -D vite</pm>",
      "docs/package-manager.md",
      createResolvedOptions({
        highlight: true,
        embeds: { github: {}, openGraph: {}, pm: {} },
      } as Partial<ResolvedOptions>),
    );

    expect(result.html).toContain("ox-tabs-container");
    expect(result.html).toContain("--octc-syntax-token-");
    expect(result.html).not.toContain('<code class="language-bash">');
  });
});
