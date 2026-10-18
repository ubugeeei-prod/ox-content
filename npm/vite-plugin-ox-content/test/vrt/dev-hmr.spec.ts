import { expect, test } from "@playwright/test";
import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { createServer } from "vite";
import { oxContent } from "../../src/index";

for (const base of ["/", "/docs/"]) {
  test(`refreshes server-rendered pages after repeated edits under ${base}`, async ({ page }) => {
    const root = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-dev-hmr-"));
    const contentDir = path.join(root, "content");
    await fs.mkdir(contentDir);
    const source = path.join(contentDir, "index.md");
    const markdown = (title: string) =>
      `---\ntitle: ${title}\ndescription: ${title}\n---\n# ${title}\n\nPage body.\n`;
    await fs.writeFile(source, markdown("First heading"));
    const errors: string[] = [];
    page.on("pageerror", (error) => errors.push(error.message));
    const server = await createServer({
      root,
      base,
      configFile: false,
      appType: "custom",
      logLevel: "silent",
      plugins: oxContent({
        srcDir: "content",
        docs: false,
        base,
        ssg: true,
        search: false,
        ogViewer: false,
      }),
      server: { host: "127.0.0.1", port: 0 },
    });
    try {
      await server.listen();
      const address = server.httpServer!.address() as { port: number };
      const connected = page.waitForEvent("console", {
        predicate: (message) => message.text().includes("[vite] connected."),
      });
      await page.goto(`http://127.0.0.1:${address.port}${base}`);
      await expect(page.locator("h1")).toHaveText("First heading");
      await connected;
      for (const title of ["Second heading", "Third heading"]) {
        await fs.writeFile(source, markdown(title));
        await expect(page.locator("h1")).toHaveText(title);
        await expect(page).toHaveTitle(new RegExp(title));
        await expect(page.locator('meta[name="description"]')).toHaveAttribute("content", title);
        await page.getByRole("button", { name: "Toggle theme", exact: true }).click();
        await expect(page.locator("html")).toHaveAttribute("data-theme", /light|dark/);
      }
      await fs.writeFile(path.join(contentDir, "added.md"), "# Newly added\n");
      await expect(page.locator(".nav-link", { hasText: "Added" })).toBeVisible();
      await fs.unlink(path.join(contentDir, "added.md"));
      await expect(page.locator(".nav-link", { hasText: "Added" })).toHaveCount(0);
      expect(errors).toEqual([]);
    } finally {
      await server.close();
      await fs.rm(root, { recursive: true, force: true });
    }
  });
}
