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

// Each skin is shot with the scheme it was designed toward, and in the mode
// that skin is actually itself in. A single scheme across all of them would be
// fairer as a comparison and useless as a portrait: a noir page lit like a
// bright office is not noir, and Liquid Glass over a flat page is not glass.
const PAIRING = {
  pixel: ["Commander", "dark"], // CGA, the palette these pixels came from
  "liquid-glass": ["Arctic", "light"], // cold, so the rim lensing has hue to bend
  "blur-glass": ["Poimandres", "dark"], // frost is invisible without something saturated behind it
  "analog-film": ["Melange", "dark"], // muted earth, where lifted blacks read
  fluid: ["Rosé Pine", "dark"], // the dye field needs saturated accents
  fabric: ["Moss", "dark"], // deep dye; cloth on a pale ground is a swatch, not a bolt
  leather: ["Cacao", "dark"], // tanned hide
  brutalist: ["Mono", "light"], // nothing to hide behind
  terminal: ["Retro", "dark"], // amber phosphor
  blueprint: ["Ink", "dark"], // a blueprint is white lines on blue, not navy on white
  risograph: ["Horizon", "light"], // two vivid drums
  swiss: ["Coral", "light"], // the School was never grey-on-grey; it committed to one hue
  neon: ["Synthwave", "dark"], // the tubes it was drawn for
  clay: ["Catppuccin", "dark"], // extruded solids need saturation to read as volume
  editorial: ["Monokai", "light"], // a magazine has a spot colour
  aurora: ["Nord", "dark"], // the latitude it is named after
  holo: ["Plum", "dark"], // foil needs a dark ground to shift on
  paper: ["Sepia", "light"], // warm stock
  voltage: ["Voltage", "dark"], // its own charge
  manuscript: ["Kanagawa", "light"], // Lotus — real paper tones rather than generic beige
  ledger: ["Solarized", "light"], // the cream of a bound book
  kiosk: ["Stage", "dark"], // poster contrast
  atlas: ["Nord", "light"], // a chart, not a hiking map
  receipt: ["Modus", "light"], // thermal paper is cold white and pure black
  bauhaus: ["Slate", "light"], // neutral ground, primary energy
  zine: ["Stage", "light"], // a zine is loud; two greys is a memo
  noir: ["Kanagawa", "dark"], // ink wash
};

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
  const [scheme, mode] = PAIRING[skin.id] ?? ["Tokyo Night", "light"];
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
    { title: skin.title, scheme, mode },
  );

  // The preview reloads its srcdoc on every change, and the WebGL backdrops
  // need a moment of animation before they have drawn anything.
  await page.waitForTimeout(900);

  // Clip to the hero's own bottom edge. Shooting the whole frame leaves a sliver
  // of the next section along the bottom, which reads as a cropping accident
  // rather than a composition.
  const box = await page.locator(".stage iframe").boundingBox();
  const heroHeight = await page.evaluate(() => {
    const d = document.querySelector(".stage iframe").contentDocument;
    const hero = d && d.querySelector(".hero");
    return hero ? hero.getBoundingClientRect().height : null;
  });
  const clip = {
    x: box.x,
    y: box.y,
    width: box.width,
    height: Math.min(box.height, heroHeight ?? box.height),
  };
  const frame = await page.locator(".stage iframe");
  // JPEG, because these are committed and a full-bleed gradient costs several
  // hundred kilobytes as PNG for no gain at review size. The gallery is the
  // authoritative view; this set is a contact sheet for a pull request.
  await page.screenshot({ path: join(OUT, `${skin.id}.jpg`), type: "jpeg", quality: 88, clip });
  console.log(`  ${skin.id.padEnd(14)} ${scheme} ${mode}`);
}

await browser.close();
console.log(`Captured ${skins.length} skins into ${OUT}`);
