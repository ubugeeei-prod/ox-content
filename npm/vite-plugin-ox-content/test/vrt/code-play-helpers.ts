import { expect, type Page } from "@playwright/test";
import { resolveCodePlayOptions } from "../../../ox-content-code-play/src/config";
import { enhancePlayHtml } from "../../../ox-content-code-play/src/html";
import { decodePayload, encodePayload } from "../../../ox-content-code-play/src/payload";
import { payloadFromFence } from "../../../ox-content-code-play/src/payload-factory";

export function renderWidget(
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

export async function runWidget(page: Page, title: string, output: string): Promise<void> {
  const widget = page.getByRole("region", { name: new RegExp(title) });
  await widget.getByRole("button", { name: /Run/ }).click();
  await expect(widget.locator("[data-ox-status]")).toHaveText("Done");
  await expect(widget.locator(".ox-code-play__stdio-text")).toContainText(output, {
    timeout: 10_000,
  });
}

export function corsHeaders(): Record<string, string> {
  return {
    "access-control-allow-origin": "*",
    "access-control-allow-methods": "POST, OPTIONS",
    "access-control-allow-headers": "content-type",
  };
}

export async function expectCompactCodePlayChrome(page: Page): Promise<void> {
  const metrics = await page.locator(".ox-code-play").evaluate((widget) => {
    if (!(widget instanceof HTMLElement)) {
      throw new Error("Missing Code Play widget");
    }
    const toolbar = widget.querySelector(".ox-code-play__toolbar");
    const runButton = widget.querySelector('[data-ox-action="run"]');
    const chips = [...widget.querySelectorAll(".ox-code-play__runtime-chip")];
    if (
      !(toolbar instanceof HTMLElement) ||
      !(runButton instanceof HTMLElement) ||
      chips.length === 0
    ) {
      throw new Error("Missing compact Code Play chrome");
    }
    return {
      buttonHeight: runButton.getBoundingClientRect().height,
      maxChipWidth: Math.max(...chips.map((chip) => chip.getBoundingClientRect().width)),
      toolbarHeight: toolbar.getBoundingClientRect().height,
    };
  });
  expect(metrics.buttonHeight).toBeLessThanOrEqual(34);
  expect(metrics.maxChipWidth).toBeLessThanOrEqual(180);
  expect(metrics.toolbarHeight).toBeLessThanOrEqual(54);
}

export async function fitsViewport(page: Page): Promise<boolean> {
  return page.evaluate(() => {
    const doc = document.documentElement;
    return doc.scrollWidth <= doc.clientWidth + 1;
  });
}
