import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { afterEach, describe, expect, it } from "vite-plus/test";
import { buildSsg } from "./ssg";
import type { ResolvedOptions } from "./types";
import { createDocsResolvedOptions } from "../test/fixtures/docs-fixture";

const tempDirs: string[] = [];

afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

describe("SSG optional widget scripts", () => {
  it("does not emit search, code-play, or island script srcs on a plain page", async () => {
    const root = await makeSite({ "guide.md": "# Guide\n\nHello.\n" });
    const built = await buildSsg(themedOptions(), root);
    const html = await fs.readFile(path.join(root, "dist", "guide", "index.html"), "utf8");
    const srcs = scriptSrcs(html);

    expect(built.errors).toEqual([]);
    expect(srcs.join("\n")).not.toMatch(/search|code-play|island|ox-islands|mermaid|tabs/i);
    expect(html).not.toContain("ox-code-play.js");
    expect(html).not.toContain("data-ox-island");
    expect(html).not.toContain("data-ox-tab-group");
  });

  it("emits a tabs chunk only when a page syncs tab groups", async () => {
    const root = await makeSite({
      "plain.md": "# Plain\n\nHello.\n",
      "tabs.md":
        '# Tabs\n\n<div class="ox-tabs" data-ox-tab-group="pkg"><div class="ox-tab-panel">A</div></div>\n',
    });
    const built = await buildSsg(themedOptions(), root);
    expect(built.errors).toEqual([]);

    const plain = await fs.readFile(path.join(root, "dist", "plain", "index.html"), "utf8");
    const tabs = await fs.readFile(path.join(root, "dist", "tabs", "index.html"), "utf8");
    expect(scriptSrcs(plain).join("\n")).not.toMatch(/tabs/i);
    expect(scriptSrcs(tabs).join("\n")).toMatch(/ox-content-tabs-/);
  });
});

function scriptSrcs(html: string): string[] {
  return [...html.matchAll(/<script\b[^>]*\bsrc="([^"]+)"/g)].map((match) => match[1] ?? "");
}

async function makeSite(files: Record<string, string>): Promise<string> {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-lazy-widgets-"));
  tempDirs.push(root);
  for (const [name, source] of Object.entries(files)) {
    const full = path.join(root, "content", name);
    await fs.mkdir(path.dirname(full), { recursive: true });
    await fs.writeFile(full, source);
  }
  return root;
}

function themedOptions(): ResolvedOptions {
  const base = createDocsResolvedOptions({
    ssg: {
      ...createDocsResolvedOptions().ssg,
      bare: false,
      siteUrl: "https://example.com",
    },
    mermaid: false,
  });
  return base;
}
