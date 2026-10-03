#!/usr/bin/env node
// Native Mermaid extraction invokes this narrow SVG renderer with the mmdc flags.
import { readFile, writeFile } from "node:fs/promises";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import puppeteer from "puppeteer";

export async function render(source, options = {}) {
  const require = createRequire(import.meta.url);
  const script = join(dirname(require.resolve("mermaid")), "mermaid.min.js");
  const browser = await puppeteer.launch({ headless: "shell", ...options.browser });
  try {
    const page = await browser.newPage();
    await page.setViewport({ width: 1200, height: 900 });
    await page.setContent('<!doctype html><html><body><div id="diagram"></div></body></html>');
    await page.addScriptTag({ path: script });
    return await page.evaluate(
      async ({ source, theme }) => {
        globalThis.mermaid.initialize({ startOnLoad: false, theme, securityLevel: "strict" });
        const { svg } = await globalThis.mermaid.render("ox-content-diagram", source);
        return svg;
      },
      { source, theme: options.theme ?? "neutral" },
    );
  } finally {
    await browser.close();
  }
}

async function main(args) {
  const options = {};
  for (let index = 0; index < args.length; index++) {
    const arg = args[index];
    if (arg === "-q") continue;
    const key = { "-i": "input", "-o": "output", "-t": "theme", "-p": "config" }[arg];
    if (!key || !args[index + 1]) throw new Error(`Invalid Mermaid renderer argument: ${arg}`);
    options[key] = args[++index];
  }
  if (!options.input || !options.output) throw new Error("Mermaid renderer requires -i and -o");
  const browser = options.config ? JSON.parse(await readFile(options.config, "utf8")) : {};
  const svg = await render(await readFile(options.input, "utf8"), {
    theme: options.theme,
    browser,
  });
  await writeFile(options.output, svg);
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  try {
    await main(process.argv.slice(2));
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
  }
}
