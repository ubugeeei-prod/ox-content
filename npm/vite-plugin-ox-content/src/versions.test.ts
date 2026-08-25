import { afterEach, describe, expect, it } from "vite-plus/test";
import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { createDocsResolvedOptions } from "../test/fixtures/docs-fixture";
import { buildSsg } from "./ssg";
import {
  prefixRoutePaths,
  resolveSnapshotDir,
  resolveVersionsOptions,
  sanitizePrefix,
} from "./versions";
import {
  escapeHtml,
  injectVersionChrome,
  searchIndexUrl,
  versionBannerMarkup,
  versionSwitcherMarkup,
} from "./versions-html";

const tempDirs: string[] = [];

afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

async function makeSite(
  current: Record<string, string>,
  snapshot?: Record<string, string>,
): Promise<string> {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-versions-"));
  tempDirs.push(root);
  const srcDir = path.join(root, "content");
  await fs.mkdir(srcDir, { recursive: true });
  for (const [relative, body] of Object.entries(current)) {
    const filePath = path.join(srcDir, relative);
    await fs.mkdir(path.dirname(filePath), { recursive: true });
    await fs.writeFile(filePath, body, "utf8");
  }
  if (snapshot) {
    const snapDir = path.join(root, "versions", "2.90");
    await fs.mkdir(snapDir, { recursive: true });
    for (const [relative, body] of Object.entries(snapshot)) {
      const filePath = path.join(snapDir, relative);
      await fs.mkdir(path.dirname(filePath), { recursive: true });
      await fs.writeFile(filePath, filePath.endsWith(".md") ? body : body, "utf8");
    }
  }
  return root;
}

function enabledVersions() {
  return resolveVersionsOptions({
    current: "3.0.0-alpha",
    entries: [
      { id: "3.0.0-alpha", label: "3.0.0-alpha", prefix: "", banner: "unreleased" },
      { id: "2.90.0", label: "2.90.0", prefix: "2.90", dir: "versions/2.90" },
    ],
  });
}

describe("resolveVersionsOptions", () => {
  it("disables the feature when omitted or false", () => {
    expect(resolveVersionsOptions(undefined).enabled).toBe(false);
    expect(resolveVersionsOptions(false).enabled).toBe(false);
    expect(createDocsResolvedOptions().versions).toBeUndefined();
  });

  it("enables a current-only entry when true, and overrides object fields", () => {
    expect(resolveVersionsOptions(true)).toEqual({
      enabled: true,
      current: "current",
      switcher: true,
      badge: true,
      entries: [{ id: "current", label: "Latest", prefix: "", banner: false }],
    });
    expect(
      resolveVersionsOptions({
        current: "next",
        switcher: false,
        entries: [{ id: "next", label: "Next", prefix: "next", banner: "unreleased" }],
      }),
    ).toMatchObject({
      enabled: true,
      current: "next",
      switcher: false,
      entries: [{ id: "next", label: "Next", prefix: "next", banner: "unreleased" }],
    });
  });
});

describe("sanitizePrefix and snapshot confinement", () => {
  it("keeps [a-z0-9.-] prefixes and drops path traversal", () => {
    expect(sanitizePrefix("2.90")).toBe("2.90");
    expect(sanitizePrefix("/next/")).toBe("next");
    expect(sanitizePrefix("../secret")).toBe("");
    expect(sanitizePrefix("javascript:alert(1)")).toBe("");
    expect(resolveSnapshotDir("/docs", "../etc")).toBeUndefined();
    expect(resolveSnapshotDir("/docs", "versions/2.90")).toBe(
      path.resolve("/docs", "versions/2.90"),
    );
  });
});

describe("prefixRoutePaths", () => {
  it("prefixes output paths that stay inside outDir", () => {
    const prefixed = prefixRoutePaths(
      {
        outputPath: path.join("/site", "dist", "guide", "index.html"),
        urlPath: "guide",
        href: "/guide/",
      },
      "2.90",
      path.join("/site", "dist"),
      "/",
    );
    expect(prefixed.outputPath).toBe(path.join("/site", "dist", "2.90", "guide", "index.html"));
    expect(prefixed.urlPath).toBe("2.90/guide");
    expect(prefixed.href).toBe("/2.90/guide/");
  });
});

describe("version switcher chrome", () => {
  it("uses themed header-select markup without hardcoded colors", () => {
    const html = versionSwitcherMarkup(
      [
        { id: "alpha", label: "3.0.0-alpha", href: "/", current: true, banner: "unreleased" },
        { id: "stable", label: "2.90.0", href: "/2.90/", current: false },
      ],
      true,
    );
    expect(html).toContain('class="ox-header-select ox-version-switcher"');
    expect(html).toContain('class="ox-header-select-menu"');
    expect(html).toContain('aria-expanded="false"');
    expect(html).toContain('aria-haspopup="true"');
    expect(html).toContain("/2.90/");
    expect(html).not.toContain("Canvas");
    expect(html).not.toContain("#fff7ed");
    expect(html).not.toContain("#f4f4f5");
    expect(html).not.toContain("--ox-bg");
    expect(html).not.toContain("<style>");
    expect(versionBannerMarkup("unreleased")).toContain("ox-version-banner--unreleased");
    expect(versionBannerMarkup("unreleased")).not.toContain("#fff7ed");
  });
});

describe("version chrome XSS", () => {
  it("escapes labels and rejects javascript hrefs", () => {
    const html = injectVersionChrome(
      `<html><body><div class="header-actions"></div><script>fetch("/search-index.json")</script></body></html>`,
      `<nav class="ox-version-switcher">${escapeHtml("<script>alert(1)</script>")}</nav>`,
      `<aside class="ox-version-banner">unreleased</aside>`,
      "/search-index.json",
      "/2.90/search-index.json",
    );
    expect(html).toContain("&lt;script&gt;");
    expect(html).not.toContain('<nav class="ox-version-switcher"><script>alert(1)</script>');
    expect(html).toContain("/2.90/search-index.json");
    expect(html).not.toContain('fetch("/search-index.json")');
    expect(searchIndexUrl("/docs/", "2.90")).toBe("/docs/2.90/search-index.json");
  });
});

describe("buildSsg versions", () => {
  it("writes no switcher, banner, or snapshot tree when omitted", async () => {
    const root = await makeSite(
      { "guide.md": "---\ntitle: Guide\n---\n\n# Guide\n" },
      { "old.md": "---\ntitle: Old\n---\n\n# Old\n" },
    );
    await buildSsg(createDocsResolvedOptions(), root);
    const guide = await fs.readFile(path.join(root, "dist", "guide", "index.html"), "utf8");
    expect(guide).not.toContain("ox-version-switcher");
    expect(guide).not.toContain("ox-version-banner");
    await expect(fs.access(path.join(root, "dist", "2.90", "old", "index.html"))).rejects.toThrow();
  });

  it("injects chrome on the live tree and writes a frozen snapshot prefix", async () => {
    const root = await makeSite(
      { "guide.md": "---\ntitle: Guide\n---\n\n# Guide\nLive.\n" },
      { "old.md": "---\ntitle: Old\n---\n\n# Old\n```js\nversions: true\n```\n" },
    );
    await buildSsg(createDocsResolvedOptions({ versions: enabledVersions() }), root);

    const live = await fs.readFile(path.join(root, "dist", "guide", "index.html"), "utf8");
    const snap = await fs.readFile(path.join(root, "dist", "2.90", "old", "index.html"), "utf8");
    expect(live).toContain("ox-version-switcher");
    expect(live).toContain("3.0.0-alpha");
    expect(live).toContain("ox-version-banner--unreleased");
    expect(live).toContain("/2.90/");
    expect(live).toContain('data-search-filter="version"');
    expect(live).toContain('data-index="/2.90/search-index.json"');
    expect(live).toContain('aria-label="Version"');
    expect(snap).toContain("Old");
    expect(snap).toContain("2.90.0");
    expect(snap).toContain("/2.90/search-index.json");
    expect(snap).not.toContain("<script>alert(1)</script>");
    const snapshotSource = await fs.readFile(path.join(root, "versions", "2.90", "old.md"), "utf8");
    expect(snapshotSource).toContain("versions: true");
    await expect(
      fs.access(path.join(root, "dist", "2.90", "search-index.json")),
    ).resolves.toBeUndefined();
  });

  it("omits draft snapshot pages when publishState is on", async () => {
    const root = await makeSite(
      { "guide.md": "---\ntitle: Guide\n---\n\n# Guide\n" },
      {
        "old.md": "---\ntitle: Old\n---\n\n# Old\n",
        "secret.md": "---\ntitle: Secret\ndraft: true\n---\n\n# Secret\n",
      },
    );
    await buildSsg(
      createDocsResolvedOptions({
        versions: enabledVersions(),
        publishState: { enabled: true, includeDrafts: false },
      }),
      root,
    );
    await expect(
      fs.access(path.join(root, "dist", "2.90", "old", "index.html")),
    ).resolves.toBeUndefined();
    await expect(
      fs.access(path.join(root, "dist", "2.90", "secret", "index.html")),
    ).rejects.toThrow();
  });

  it("does not treat fence mentions as a reason to rewrite the snapshot source", async () => {
    const root = await makeSite(
      { "guide.md": "---\ntitle: Guide\n---\n\n# Guide\n" },
      { "old.md": "---\ntitle: Old\n---\n\n# Old\n" },
    );
    await buildSsg(createDocsResolvedOptions({ versions: enabledVersions() }), root);
    const before = await fs.readFile(path.join(root, "versions", "2.90", "old.md"), "utf8");
    expect(before).toBe("---\ntitle: Old\n---\n\n# Old\n");
  });

  it("escapes hostile version labels", async () => {
    const root = await makeSite({ "guide.md": "---\ntitle: Guide\n---\n\n# Guide\n" });
    await buildSsg(
      createDocsResolvedOptions({
        versions: resolveVersionsOptions({
          current: "x",
          entries: [{ id: "x", label: "<script>alert(1)</script>", prefix: "" }],
        }),
      }),
      root,
    );
    const guide = await fs.readFile(path.join(root, "dist", "guide", "index.html"), "utf8");
    expect(guide).toContain("&lt;script&gt;");
    expect(guide).not.toContain("<script>alert(1)</script>");
  });
});
