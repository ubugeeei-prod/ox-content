import { describe, expect, it } from "vite-plus/test";
import { createDocsResolvedOptions } from "../test/fixtures/docs-fixture";
import { transformMarkdown } from "./transform";
import type { ResolvedOptions } from "./types";

describe("definition-list transform", () => {
  it("leaves definition-list source literal unless opted in", async () => {
    const markdown = "HTTP\n: Hypertext Transfer Protocol\n";

    const defaultResult = await transformMarkdown(
      markdown,
      "docs/definition-lists.md",
      createResolvedOptions(),
    );
    expect(defaultResult.html).not.toContain("ox-definition-list");
    expect(defaultResult.html).toContain(": Hypertext Transfer Protocol");

    const enabledResult = await transformMarkdown(
      markdown,
      "docs/definition-lists.md",
      createResolvedOptions({
        definitionLists: { enabled: true },
      }),
    );
    expect(enabledResult.html).toContain('<dl class="ox-definition-list">');
    expect(enabledResult.html).toContain("<dt>");
    expect(enabledResult.html).toContain("HTTP");
    expect(enabledResult.html).toContain("<dd>");
    expect(enabledResult.html).toContain("Hypertext Transfer Protocol");
    expect(enabledResult.html).not.toContain(": Hypertext");
  });
});

function createResolvedOptions(overrides: Partial<ResolvedOptions> = {}): ResolvedOptions {
  return createDocsResolvedOptions({ highlight: false, ...overrides });
}
