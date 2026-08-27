import { mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { expect, test } from "@playwright/test";
import { bundleBrowserClient } from "../../../ox-content-code-play/src/bundle-browser";
import { resolveCodePlayOptions } from "../../../ox-content-code-play/src/config";
import { enhancePlayHtml } from "../../../ox-content-code-play/src/html";
import { parsePlayFences } from "../../../ox-content-code-play/src/markdown";
import { decodePayload, encodePayload } from "../../../ox-content-code-play/src/payload";
import { payloadFromFence } from "../../../ox-content-code-play/src/payload-factory";
import {
  corsHeaders,
  expectCompactCodePlayChrome,
  fitsViewport,
  renderWidget,
  runWidget,
} from "./code-play-helpers";

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
    await expectCompactCodePlayChrome(page);
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

test("hydrates Rust, Go, and Python runtime widgets and runs them on demand", async ({ page }) => {
  const outDir = await mkdtemp(path.join(tmpdir(), "ox-code-play-vrt-runtimes-"));
  try {
    await bundleBrowserClient(outDir);
    const client = await readFile(path.join(outDir, "browser.mjs"), "utf8");
    const options = resolveCodePlayOptions({
      languages: {
        rust: true,
        go: true,
        python: { endpoint: "https://exec.example/api/v2/piston" },
      },
    });
    const rustCode = `fn main() {\n    println!("rust ok");\n}`;
    const goCode = `package main\nimport "fmt"\nfunc main() { fmt.Println("go ok") }`;
    const pythonCode = `print("python ok")`;

    await page.route("https://play.rust-lang.org/execute", async (route) => {
      if (route.request().method() === "OPTIONS") {
        await route.fulfill({ status: 204, headers: corsHeaders() });
        return;
      }
      await route.fulfill({
        status: 200,
        headers: { ...corsHeaders(), "content-type": "application/json" },
        body: JSON.stringify({ success: true, stdout: "rust ok\n", stderr: "" }),
      });
    });
    await page.route("https://play.golang.org/compile", async (route) => {
      await route.fulfill({
        status: 200,
        headers: { ...corsHeaders(), "content-type": "application/json" },
        body: JSON.stringify({ Events: [{ Kind: "stdout", Message: "go ok\n" }] }),
      });
    });
    await page.route("https://exec.example/api/v2/piston/execute", async (route) => {
      if (route.request().method() === "OPTIONS") {
        await route.fulfill({ status: 204, headers: corsHeaders() });
        return;
      }
      await route.fulfill({
        status: 200,
        headers: { ...corsHeaders(), "content-type": "application/json" },
        body: JSON.stringify({ run: { stdout: "python ok\n", stderr: "", code: 0 } }),
      });
    });

    const html = [
      renderWidget("rust", rustCode, "Rust success", options),
      renderWidget("go", goCode, "Go success", options),
      renderWidget("python", pythonCode, "Python success", options),
    ].join("\n");
    await page.setContent(
      `<!doctype html><html><head><meta charset="utf-8"></head><body>${html}<script type="module">${client}</script></body></html>`,
      { waitUntil: "domcontentloaded" },
    );

    await runWidget(page, "Rust success", "rust ok");
    await runWidget(page, "Go success", "go ok");
    await runWidget(page, "Python success", "python ok");
    await expect(page.getByRole("region", { name: /Rust success/ })).toContainText(
      "Rust Playground",
    );
    await expect(page.getByRole("region", { name: /Go success/ })).toContainText("Go Playground");
    await expect(page.getByRole("region", { name: /Python success/ })).toContainText(
      "Piston-compatible",
    );
  } finally {
    await rm(outDir, { recursive: true, force: true });
  }
});

test("surfaces runtime errors and unsupported remote executors in hydrated widgets", async ({
  page,
}) => {
  const outDir = await mkdtemp(path.join(tmpdir(), "ox-code-play-vrt-runtimes-error-"));
  try {
    await bundleBrowserClient(outDir);
    const client = await readFile(path.join(outDir, "browser.mjs"), "utf8");
    const options = resolveCodePlayOptions({ languages: { go: true, python: true } });
    await page.route("https://play.golang.org/compile", async (route) => {
      await route.fulfill({
        status: 200,
        headers: { ...corsHeaders(), "content-type": "application/json" },
        body: JSON.stringify({ Errors: "prog.go:3:1: undefined: missing\n" }),
      });
    });
    const html = [
      renderWidget(
        "go",
        `package main\nfunc main() {\n    missing()\n}`,
        "Go compile error",
        options,
      ),
      renderWidget("python", `print("needs endpoint")`, "Python unsupported", options),
    ].join("\n");
    await page.setContent(
      `<!doctype html><html><head><meta charset="utf-8"></head><body>${html}<script type="module">${client}</script></body></html>`,
      { waitUntil: "domcontentloaded" },
    );

    const go = page.getByRole("region", { name: /Go compile error/ });
    await go.getByRole("button", { name: /Run/ }).click();
    await expect(go.locator("[data-ox-status]")).toHaveText("Error");
    await expect(go.locator(".ox-code-play__diag--error").first()).toContainText(
      "undefined: missing",
    );

    const python = page.getByRole("region", { name: /Python unsupported/ });
    await expect(python).toContainText("Endpoint missing");
    await python.getByRole("button", { name: /Run/ }).click();
    await expect(python.locator("[data-ox-status]")).toHaveText("Unsupported");
    await expect(python.locator(".ox-code-play__diag--error").first()).toContainText(
      /Python execution needs a configured HTTP executor/,
    );
  } finally {
    await rm(outDir, { recursive: true, force: true });
  }
});

test("surfaces Rust playground transport failures as offline in the browser", async ({ page }) => {
  const outDir = await mkdtemp(path.join(tmpdir(), "ox-code-play-vrt-runtimes-offline-"));
  try {
    await bundleBrowserClient(outDir);
    const client = await readFile(path.join(outDir, "browser.mjs"), "utf8");
    const options = resolveCodePlayOptions({ languages: { rust: true } });
    await page.route("https://play.rust-lang.org/execute", async (route) => {
      await route.abort("failed");
    });
    const html = renderWidget("rust", `fn main() {}`, "Rust offline", options);
    await page.setContent(
      `<!doctype html><html><head><meta charset="utf-8"></head><body>${html}<script type="module">${client}</script></body></html>`,
      { waitUntil: "domcontentloaded" },
    );

    const rust = page.getByRole("region", { name: /Rust offline/ });
    await rust.getByRole("button", { name: /Run/ }).click();
    await expect(rust.locator("[data-ox-status]")).toHaveText("Offline");
    await expect(rust.locator(".ox-code-play__diag--error").first()).toContainText(
      /CORS|unreachable/,
    );
  } finally {
    await rm(outDir, { recursive: true, force: true });
  }
});

test("renders project sandbox fallback links on mobile without provider scripts", async ({
  page,
}) => {
  const outDir = await mkdtemp(path.join(tmpdir(), "ox-code-play-vrt-project-"));
  try {
    await page.setViewportSize({ width: 390, height: 844 });
    await bundleBrowserClient(outDir);
    const client = await readFile(path.join(outDir, "browser.mjs"), "utf8");
    const options = resolveCodePlayOptions({ languages: { typescript: true } });
    const source = [
      '```ts play play-title="Project demo" play-project=stackblitz play-file=src/main.ts play-entry=src/main.ts play-project-url=https://stackblitz.com/edit/ox-content-project',
      'console.log("project");',
      "```",
    ].join("\n");
    const fence = parsePlayFences(source)[0];
    if (!fence) {
      throw new Error("expected a project Code Play fence");
    }
    const payload = encodePayload(
      payloadFromFence(fence, options, {
        files: [{ path: "package.json", code: '{"scripts":{"dev":"vite"}}\n' }],
      }),
    );
    const widget = enhancePlayHtml(`<pre><code class="language-ts">${fence.code}</code></pre>`, {
      decodePayload,
      encodePayload,
      matchFences: [{ language: "ts", code: fence.code, payload }],
    });

    await page.setContent(
      `<!doctype html><html><head><meta charset="utf-8"></head><body>${widget}<script type="module">${client}</script></body></html>`,
      { waitUntil: "domcontentloaded" },
    );

    const project = page.getByRole("region", { name: /Project demo/ });
    await expect(project.locator(".ox-code-play__project")).toBeVisible();
    await expect(project).toContainText("StackBlitz");
    await expect(project).toContainText("2 files");
    await expect(project.getByRole("link", { name: "Open" })).toHaveAttribute(
      "href",
      "https://stackblitz.com/edit/ox-content-project",
    );
    await expect(page.locator('script[src*="stackblitz"], iframe[src*="stackblitz"]')).toHaveCount(
      0,
    );
    expect(await fitsViewport(page)).toBe(true);
  } finally {
    await rm(outDir, { recursive: true, force: true });
  }
});
