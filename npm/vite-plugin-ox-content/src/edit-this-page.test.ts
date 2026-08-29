import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vite-plus/test";
import { createDocsResolvedOptions } from "../test/fixtures/docs-fixture";
import { transformMarkdown } from "./transform";
import type { ResolvedOptions } from "./types";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");

function createResolvedOptions(overrides: Partial<ResolvedOptions> = {}): ResolvedOptions {
  return createDocsResolvedOptions(overrides);
}

describe("editThisPage", () => {
  it("places edit links inside the repository when rootDir is a repository path", async () => {
    const srcDir = resolve(repoRoot, "docs/content");
    const page = resolve(srcDir, "guide/nested.md");

    const editHref = async (rootDir?: string): Promise<string | undefined> => {
      const result = await transformMarkdown(
        "# Nested\n",
        page,
        createResolvedOptions({
          editThisPage: {
            enabled: true,
            repoUrl: "https://gitlab.example.com/owner/repo",
            branch: "main",
            rootDir,
          },
        }),
        { sourcePath: page, srcDir },
      );
      return /<a href="([^"]+)"/.exec(result.html.split("ox-edit-this-page")[1] ?? "")?.[1];
    };

    // The page path is measured from srcDir and rootDir goes in front of it.
    expect(await editHref("docs/content")).toBe(
      "https://gitlab.example.com/owner/repo/edit/main/docs/content/guide/nested.md",
    );
    expect(await editHref("packages/site/docs")).toBe(
      "https://gitlab.example.com/owner/repo/edit/main/packages/site/docs/guide/nested.md",
    );
    // Never the build machine's own path, whatever rootDir says.
    for (const rootDir of [undefined, "", ".", "docs", "/nowhere/on/disk"]) {
      expect(await editHref(rootDir)).not.toContain(repoRoot);
    }
  });

  it("shapes edit links for the forge the repository is hosted on", async () => {
    const page = resolve(repoRoot, "docs/content/guide.md");

    const editHref = async (editThisPage: Record<string, unknown>): Promise<string | undefined> => {
      const result = await transformMarkdown(
        "# Guide\n",
        page,
        createResolvedOptions({
          editThisPage: { enabled: true, branch: "main", rootDir: "docs", ...editThisPage },
        }),
        { sourcePath: page, srcDir: resolve(repoRoot, "docs") },
      );
      return /<a href="([^"]+)"/.exec(result.html.split("ox-edit-this-page")[1] ?? "")?.[1];
    };

    expect(await editHref({ repoUrl: "https://gitlab.com/owner/repo" })).toBe(
      "https://gitlab.com/owner/repo/-/edit/main/docs/content/guide.md",
    );
    expect(
      await editHref({ repoUrl: "https://git.example.com/owner/repo", provider: "gitlab" }),
    ).toBe("https://git.example.com/owner/repo/-/edit/main/docs/content/guide.md");
    expect(await editHref({ repoUrl: "https://bitbucket.org/owner/repo" })).toBe(
      "https://bitbucket.org/owner/repo/src/main/docs/content/guide.md?mode=edit",
    );
    expect(
      await editHref({
        repoUrl: "https://git.example.com/owner/repo",
        urlPattern: "{repoUrl}/ui/edit?ref={branch}&file={path}",
      }),
      // `&` is escaped for the attribute, as any other href would be.
    ).toBe("https://git.example.com/owner/repo/ui/edit?ref=main&amp;file=docs/content/guide.md");
    // Unchanged for the shape this option has always produced.
    expect(await editHref({ repoUrl: "https://github.com/owner/repo" })).toBe(
      "https://github.com/owner/repo/edit/main/docs/content/guide.md",
    );
  });
});
