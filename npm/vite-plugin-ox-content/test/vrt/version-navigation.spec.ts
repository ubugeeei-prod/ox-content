import { expect, test } from "@playwright/test";
import type { Page, TestInfo } from "@playwright/test";
import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { createDocsResolvedOptions } from "../fixtures/docs-fixture";
import { buildSsg } from "../../src/ssg";
import { resolveVersionsOptions } from "../../src/versions";

const origin = "http://version-navigation.test";

test("navigates across three frozen sidebar pages and browser history", async ({
  page,
}, testInfo) => {
  const fixture = await buildFixture();
  try {
    await routeFixture(page, fixture.pages);
    await page.goto(`${origin}/docs/2.90/alpha/`);
    await expectVersionState(page, "Alpha");
    await attachScreenshot(page, testInfo, "version-navigation-alpha.png");

    await page.getByRole("link", { name: "Beta", exact: true }).click();
    await expect(page).toHaveURL(`${origin}/docs/2.90/beta/`);
    await expectVersionState(page, "Beta");

    await page.getByRole("link", { name: "Gamma", exact: true }).click();
    await expect(page).toHaveURL(`${origin}/docs/2.90/gamma/`);
    await expectVersionState(page, "Gamma");
    await attachScreenshot(page, testInfo, "version-navigation-gamma.png");

    await page.goBack();
    await expect(page).toHaveURL(`${origin}/docs/2.90/beta/`);
    await expectVersionState(page, "Beta");
    await page.goBack();
    await expect(page).toHaveURL(`${origin}/docs/2.90/alpha/`);
    await expectVersionState(page, "Alpha");
    await page.goForward();
    await expect(page).toHaveURL(`${origin}/docs/2.90/beta/`);
    await expectVersionState(page, "Beta");
  } finally {
    await fs.rm(fixture.root, { recursive: true, force: true });
  }
});

async function expectVersionState(page: Page, title: string) {
  await expect(page.locator(".ox-version-switcher > button")).toContainText("2.90.0");
  await expect(page.locator(".nav-link.active")).toHaveText(title);
  await expect(page.locator("html")).toHaveAttribute(
    "data-ox-search-index",
    "/docs/2.90/search-index.json",
  );
}

async function routeFixture(page: Page, pages: Map<string, string>) {
  await page.route(`${origin}/**`, async (route) => {
    const pathname = new URL(route.request().url()).pathname;
    const body = pages.get(pathname);
    await route.fulfill(
      body
        ? { contentType: "text/html", headers: { "cache-control": "no-store" }, body }
        : { status: 404, contentType: "text/plain", body: "not found" },
    );
  });
}

async function buildFixture(): Promise<{ root: string; pages: Map<string, string> }> {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-version-vrt-"));
  const names = ["alpha", "beta", "gamma"];
  for (const sourceRoot of [path.join(root, "content"), path.join(root, "versions", "2.90")]) {
    await fs.mkdir(sourceRoot, { recursive: true });
    await Promise.all(
      names.map((name) =>
        fs.writeFile(
          path.join(sourceRoot, `${name}.md`),
          `---\ntitle: ${title(name)}\n---\n# ${title(name)}\nFrozen navigation fixture.\n`,
          "utf8",
        ),
      ),
    );
  }
  const base = createDocsResolvedOptions();
  const result = await buildSsg(
    createDocsResolvedOptions({
      base: "/docs/",
      versions: resolveVersionsOptions({
        current: "3.0.0-alpha",
        entries: [
          { id: "3.0.0-alpha", label: "3.0.0-alpha", prefix: "" },
          { id: "2.90.0", label: "2.90.0", prefix: "2.90", dir: "versions/2.90" },
        ],
      }),
      ssg: { ...base.ssg, pagination: true, breadcrumbs: true },
    }),
    root,
  );
  expect(result.errors).toEqual([]);
  const pages = new Map<string, string>();
  for (const name of names) {
    pages.set(
      `/docs/2.90/${name}/`,
      await fs.readFile(path.join(root, "dist", "2.90", name, "index.html"), "utf8"),
    );
  }
  return { root, pages };
}

async function attachScreenshot(page: Page, testInfo: TestInfo, name: string) {
  await testInfo.attach(name, {
    body: await page.screenshot({ animations: "disabled", fullPage: true }),
    contentType: "image/png",
  });
}

function title(value: string): string {
  return value[0]!.toUpperCase() + value.slice(1);
}
