import { mkdtemp, rm, writeFile, mkdir } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { describe, expect, it } from "vite-plus/test";
import { createDocsResolvedOptions } from "../test/fixtures/docs-fixture";
import { transformMarkdown } from "./transform";
import type { ResolvedOptions } from "./types";

describe("partials transform", () => {
  it("leaves @partial literal unless opted in, then substitutes escaped values", async () => {
    const root = await mkdtemp(path.join(tmpdir(), "ox-content-partials-"));
    try {
      await mkdir(path.join(root, "_partials"));
      await writeFile(
        path.join(root, "_partials", "install.md"),
        "Install {{ package }} with {{ manager }}.\n",
      );
      const host = path.join(root, "host.md");
      const markdown =
        '<!-- @partial: ./_partials/install.md package="<b>ox</b>" manager="pnpm" -->\n';
      await writeFile(host, markdown);

      const defaultResult = await transformMarkdown(markdown, host, createResolvedOptions());
      expect(defaultResult.html).toContain("@partial");
      expect(defaultResult.html).not.toContain("Install");

      const enabledResult = await transformMarkdown(
        markdown,
        host,
        createResolvedOptions({
          partials: {
            enabled: true,
            rootDir: root,
            root: "_partials",
            missing: "literal",
          },
        }),
      );
      expect(enabledResult.html).toContain("pnpm");
      expect(enabledResult.html).toMatch(/Install (?:&lt;|&#x3C;)b/);
      expect(enabledResult.html).not.toContain("<b>ox</b>");
      expect(enabledResult.html).not.toContain("@partial");
    } finally {
      await rm(root, { recursive: true, force: true });
    }
  });
});

function createResolvedOptions(overrides: Partial<ResolvedOptions> = {}): ResolvedOptions {
  return createDocsResolvedOptions({ highlight: false, ...overrides });
}
