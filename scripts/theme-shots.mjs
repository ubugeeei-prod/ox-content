#!/usr/bin/env node
// Captures one screenshot per skin from the live theme gallery.
//
// Driven through the gallery rather than by rendering each skin standalone, so
// what is captured is exactly what a reader sees when they click that preset —
// same stylesheet, same iframe, same composition.
//
// Usage: node scripts/theme-shots.mjs [url] [outDir]

import { mkdirSync, rmSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { createRequire } from "node:module";

const HERE = dirname(fileURLToPath(import.meta.url));
const ROOT = join(HERE, "..");
// Playwright is a dependency of the vite plugin, not of this scripts folder, so
// resolve it from there rather than adding a second copy to the tree.
const require = createRequire(join(HERE, "..", "npm", "vite-plugin-ox-content", "package.json"));
const { chromium } = require("playwright");

const URL = process.argv[2] ?? "http://localhost:4173/theme-gallery.html";
const OUT = process.argv[3] ?? join(ROOT, "docs", "public", "screenshots", "themes");

// One scheme for all of them keeps the set comparable, but a few skins are only
// themselves in one mode — a noir page lit like a bright office is not noir.
const DARK_ONLY = new Set(["noir", "neon", "terminal", "voltage", "aurora", "holo"]);
const SCHEME = "Tokyo Night";

rmSync(OUT, { recursive: true, force: true });
mkdirSync(OUT, { recursive: true });

const browser = await chromium.launch();
const page = await browser.newPage({
  // Sized for a README rather than for print: a 2x buffer of a full-bleed
  // gradient runs to several megabytes per skin, and these are committed.
  viewport: { width: 1360, height: 850 },
  deviceScaleFactor: 1,
});

await page.goto(URL, { waitUntil: "networkidle" });

const skins = await page.evaluate(() =>
  JSON.parse(document.getElementById("data").textContent).skins.map((s) => ({
    id: s.id,
    title: s.title,
  })),
);

for (const skin of skins) {
  await page.evaluate(
    ({ title, scheme, mode }) => {
      const lists = [...document.querySelectorAll(".panel .list")];
      const pick = (list, label) =>
        [...list.children].find((b) => b.textContent.trim() === label)?.click();
      pick(lists[0], title);
      pick(lists[1], scheme);
      // Landing page, then the mode that suits this skin.
      document.querySelectorAll(".seg")[0].children[0].click();
      document.querySelectorAll(".seg")[1].children[mode === "dark" ? 1 : 0].click();
    },
    { title: skin.title, scheme: SCHEME, mode: DARK_ONLY.has(skin.id) ? "dark" : "light" },
  );

  // The preview reloads its srcdoc on every change, and the WebGL backdrops
  // need a moment of animation before they have drawn anything.
  await page.waitForTimeout(900);

  const frame = await page.locator(".stage iframe");
    // JPEG, because these are committed and a full-bleed gradient costs several
  // hundred kilobytes as PNG for no gain at review size. The gallery is the
  // authoritative view; this set is a contact sheet for a pull request.
  await frame.screenshot({ path: join(OUT, `${skin.id}.jpg`), type: "jpeg", quality: 88 });
  console.log(`  ${skin.id}`);
}

await browser.close();
console.log(`Captured ${skins.length} skins into ${OUT}`);
