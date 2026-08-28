import { afterEach, describe, expect, it } from "vite-plus/test";
import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { createDocsResolvedOptions } from "../test/fixtures/docs-fixture";
import { buildSsg } from "./ssg";
import { defaultTheme, defineTheme, resolveTheme } from "./theme";

const tempDirs: string[] = [];

afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

const ENTRY_PAGE = `---
layout: home
hero:
  name: Site
  image:
    src: /img/hero.png
    alt: Hero
---

# Home

[architecture](/architecture/)

![icon](/img/icon.png)

[cdn](https://cdn.example.com/x)
`;

async function buildAt(base: string): Promise<string> {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "ox-base-"));
  tempDirs.push(root);
  const srcDir = path.join(root, "content");
  await fs.mkdir(srcDir, { recursive: true });
  await fs.writeFile(path.join(srcDir, "index.md"), ENTRY_PAGE, "utf8");

  const defaults = createDocsResolvedOptions();
  const result = await buildSsg(
    createDocsResolvedOptions({
      base,
      ssg: {
        ...defaults.ssg,
        siteName: "Docs",
        theme: resolveTheme(
          defineTheme({ extends: defaultTheme, header: { logo: "/img/logo.svg", title: "Docs" } }),
        ),
      },
    }),
    root,
  );
  expect(result.errors).toEqual([]);
  return fs.readFile(path.join(root, "dist", "index.html"), "utf8");
}

describe("base prefixing", () => {
  it("prefixes every in-site path when the site is deployed under a sub-path", async () => {
    const html = await buildAt("/team/docs/");

    // Written in Markdown.
    expect(html).toContain('href="/team/docs/architecture/"');
    expect(html).toContain('src="/team/docs/img/icon.png"');
    // Written in theme config.
    expect(html).toContain('src="/team/docs/img/logo.svg"');
    // Written in entry-page frontmatter.
    expect(html).toContain('src="/team/docs/img/hero.png"');
    // Nothing is left at the server root, which is what used to 404.
    expect(html).not.toContain('src="/img/');
    expect(html).not.toContain('href="/architecture/"');
  });

  it("leaves another origin alone", async () => {
    const html = await buildAt("/team/docs/");

    expect(html).toContain('href="https://cdn.example.com/x"');
  });

  it("changes nothing on a site deployed at the root", async () => {
    const html = await buildAt("/");

    expect(html).toContain('href="/architecture/"');
    expect(html).toContain('src="/img/icon.png"');
    expect(html).toContain('src="/img/logo.svg"');
    expect(html).toContain('src="/img/hero.png"');
  });
});
