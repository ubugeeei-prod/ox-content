import { mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { expect, test, type Page } from "@playwright/test";
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

function renderWidget(
  language: string,
  code: string,
  title: string,
  options: ReturnType<typeof resolveCodePlayOptions>,
): string {
  const payload = encodePayload(
    payloadFromFence(
      {
        language,
        meta: `play play-title="${title}"`,
        code,
        raw: "",
        start: 0,
        end: 0,
        typecheck: false,
        title,
        config: {},
      },
      options,
    ),
  );
  const escapedCode = code.replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;");
  return enhancePlayHtml(`<pre><code class="language-${language}">${escapedCode}</code></pre>`, {
    decodePayload,
    encodePayload,
    matchFences: [{ language, code, payload }],
  });
}

async function runWidget(page: Page, title: string, output: string): Promise<void> {
  const widget = page.getByRole("region", { name: new RegExp(title) });
  await widget.getByRole("button", { name: /Run/ }).click();
  await expect(widget.locator("[data-ox-status]")).toHaveText("Done");
  await expect(widget.locator(".ox-code-play__stdio-text")).toContainText(output, {
    timeout: 10_000,
  });
}

function corsHeaders(): Record<string, string> {
  return {
    "access-control-allow-origin": "*",
    "access-control-allow-methods": "POST, OPTIONS",
    "access-control-allow-headers": "content-type",
  };
}
