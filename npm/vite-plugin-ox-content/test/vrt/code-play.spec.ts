import { mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { expect, test } from "@playwright/test";
import { bundleBrowserClient } from "../../../ox-content-code-play/src/bundle-browser";
import { resolveCodePlayOptions } from "../../../ox-content-code-play/src/config";
import { enhancePlayHtml } from "../../../ox-content-code-play/src/html";
import { decodePayload, encodePayload } from "../../../ox-content-code-play/src/payload";
import { payloadFromFence } from "../../../ox-content-code-play/src/payload-factory";

test("hydrates written SSG HTML and runs JavaScript in the browser sandbox", async ({ page }) => {
  const outDir = await mkdtemp(path.join(tmpdir(), "ox-code-play-vrt-"));
  try {
    await bundleBrowserClient(outDir);
    const client = await readFile(path.join(outDir, "browser.mjs"), "utf8");
    const options = resolveCodePlayOptions({ languages: { javascript: true } });
    const code = `console.log("ssg-ready");`;
    const payload = encodePayload(
      payloadFromFence(
        {
          language: "js",
          meta: "play",
          code,
          raw: "",
          start: 0,
          end: 0,
          typecheck: false,
        },
        options,
      ),
    );
    const widget = enhancePlayHtml(`<pre><code class="language-js">${code}</code></pre>`, {
      decodePayload,
      encodePayload,
      matchFences: [{ language: "js", code, payload }],
    });
    expect(widget).toContain("data-ox-code-play");
    await page.setContent(
      `<!doctype html><html><head><meta charset="utf-8"></head><body>${widget}<script type="module">${client}</script></body></html>`,
      { waitUntil: "domcontentloaded" },
    );
    const run = page.locator('[data-ox-action="run"]');
    await expect(run).toBeVisible();
    await run.click();
    await expect(page.locator(".ox-code-play__stdio-text")).toContainText("ssg-ready", {
      timeout: 10_000,
    });
    await expect(page.locator(".ox-code-play__stdio-line--stdout")).toBeVisible();
    await expect(page.locator('[data-ox-action="typecheck"]')).toHaveCount(0);
    await expect(page.locator('[data-ox-action="cancel"]')).toBeHidden();
  } finally {
    await rm(outDir, { recursive: true, force: true });
  }
});

test("published TypeScript widgets run without a dead Typecheck button", async ({ page }) => {
  const outDir = await mkdtemp(path.join(tmpdir(), "ox-code-play-vrt-ts-"));
  try {
    await bundleBrowserClient(outDir);
    const client = await readFile(path.join(outDir, "browser.mjs"), "utf8");
    const options = resolveCodePlayOptions({
      languages: { typescript: { execute: true, typecheck: true } },
    });
    const code = `const msg: string = "ssg-ts";\nconsole.log(msg);`;
    const payload = encodePayload(
      payloadFromFence(
        {
          language: "ts",
          meta: "play",
          code,
          raw: "",
          start: 0,
          end: 0,
          typecheck: true,
        },
        options,
      ),
    );
    expect(decodePayload(payload).capabilities.typecheck).toBe(false);
    const widget = enhancePlayHtml(`<pre><code class="language-ts">${code}</code></pre>`, {
      decodePayload,
      encodePayload,
      matchFences: [{ language: "ts", code, payload }],
    });
    await page.setContent(
      `<!doctype html><html><head><meta charset="utf-8"></head><body>${widget}<script type="module">${client}</script></body></html>`,
      { waitUntil: "domcontentloaded" },
    );
    await expect(page.locator('[data-ox-action="typecheck"]')).toHaveCount(0);
    await page.locator('[data-ox-action="run"]').click();
    await expect(page.locator(".ox-code-play__stdio-text")).toContainText("ssg-ts", {
      timeout: 10_000,
    });
  } finally {
    await rm(outDir, { recursive: true, force: true });
  }
});

test("Cancel aborts an in-flight sandbox run and re-enables Run", async ({ page }) => {
  const outDir = await mkdtemp(path.join(tmpdir(), "ox-code-play-vrt-cancel-"));
  try {
    await bundleBrowserClient(outDir);
    const client = await readFile(path.join(outDir, "browser.mjs"), "utf8");
    const options = resolveCodePlayOptions({ languages: { javascript: true } });
    const code = `while (true) {}`;
    const payload = encodePayload(
      payloadFromFence(
        {
          language: "js",
          meta: "play",
          code,
          raw: "",
          start: 0,
          end: 0,
          typecheck: false,
        },
        options,
      ),
    );
    const widget = enhancePlayHtml(`<pre><code class="language-js">${code}</code></pre>`, {
      decodePayload,
      encodePayload,
      matchFences: [{ language: "js", code, payload }],
    });
    await page.setContent(
      `<!doctype html><html><head><meta charset="utf-8"></head><body>${widget}<script type="module">${client}</script></body></html>`,
      { waitUntil: "domcontentloaded" },
    );
    await page.locator('[data-ox-action="run"]').click();
    const cancel = page.locator('[data-ox-action="cancel"]');
    await expect(cancel).toBeVisible();
    await cancel.click();
    await expect(page.getByText(/run cancelled/i).first()).toBeVisible();
    await expect(page.locator('[data-ox-action="run"]')).toBeEnabled();
    await expect(cancel).toBeHidden();
  } finally {
    await rm(outDir, { recursive: true, force: true });
  }
});
