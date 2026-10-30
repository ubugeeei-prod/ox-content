import { describe, expect, it } from "vite-plus/test";
import { mkdtemp, mkdir, readFile, rm, writeFile } from "node:fs/promises";
import * as path from "node:path";
import { inflateSync } from "node:zlib";
import { generateOgImages, resolveOgImageOptions, openBrowser } from "./og-image";
import type { OgImageRenderer } from "./og-image";

// The first RGB pixel has no left/up predictor for any PNG filter.
function firstPixel(png: Buffer): number[] {
  expect(png.subarray(0, 8).toString("hex")).toBe("89504e470d0a1a0a");
  expect(png.readUInt32BE(16)).toBe(320);
  expect(png.readUInt32BE(20)).toBe(160);
  expect(png[24]).toBe(8);
  expect([2, 3, 6]).toContain(png[25]);
  const chunks: Buffer[] = [];
  let palette = Buffer.alloc(0);
  for (let offset = 8; offset < png.length;) {
    const size = png.readUInt32BE(offset);
    if (png.toString("ascii", offset + 4, offset + 8) === "IDAT")
      chunks.push(png.subarray(offset + 8, offset + 8 + size));
    if (png.toString("ascii", offset + 4, offset + 8) === "PLTE")
      palette = png.subarray(offset + 8, offset + 8 + size);
    offset += size + 12;
  }
  const decoded = inflateSync(Buffer.concat(chunks));
  return png[25] === 3
    ? [...palette.subarray(decoded[1] * 3, decoded[1] * 3 + 3)]
    : [...decoded.subarray(1, 4)];
}

const browser = await openBrowser();
const hasChromium = browser !== null;
await browser?.[Symbol.asyncDispose]();
if (process.env.OX_CONTENT_REQUIRE_OG_CHROMIUM === "1" && !hasChromium) {
  throw new Error("Chromium must be available for the OG renderer integration matrix.");
}
const css =
  ".card{display:flex;width:100%;height:100%;background:#123456;color:white;font-size:24px}";
const templates = {
  ts:
    "export default (p: {title:string}) => `<style>" +
    css +
    '</style><div class="card">${p.title}</div>`;',
  vue: '<script setup lang="ts">import Card from "./Card.vue";defineProps<{title:string}>();</script><template><Card :title="title" /></template>',
  svelte:
    '<script lang="ts">import Card from "./Card.svelte";let {title}: {title:string} = $props();</script><Card {title} />',
  react:
    "export default function Card(p: {title:string}) { return <><style>{" +
    JSON.stringify(css) +
    '}</style><div className="card">{p.title}</div></> }',
  solid:
    "export default function Card(p: {title:string}) { return <><style>{" +
    JSON.stringify(css) +
    '}</style><div class="card">{p.title}</div></> }',
};

describe.each<OgImageRenderer>(["chromium", "satori", "takumi"])("%s OG images", (renderer) => {
  it.skipIf(renderer === "chromium" && !hasChromium).each(Object.entries(templates))(
    "renders %s templates including component CSS",
    async (framework, source) => {
      // Stay under the workspace so external SSR runtimes resolve from node_modules.
      const root = await mkdtemp(path.resolve(import.meta.dirname, "../.og-renderers-"));
      try {
        await writeFile(
          path.join(root, "Card.vue"),
          '<script setup lang="ts">defineProps<{title:string}>();</script><template><div class="card">{{title}}</div></template><style scoped>' +
            css +
            "</style>",
        );
        await writeFile(
          path.join(root, "Card.svelte"),
          '<script lang="ts">let {title}: {title:string} = $props();</script><div class="card">{title}</div><style>' +
            css +
            "</style>",
        );
        const ext = framework === "react" || framework === "solid" ? "tsx" : framework;
        await writeFile(path.join(root, "og." + ext), source);
        const outputPath = path.join(root, "og.png");
        const options = resolveOgImageOptions({
          renderer,
          template: "og." + ext,
          jsxFramework: framework === "solid" ? "solid" : "react",
          width: 320,
          height: 160,
          cache: false,
          concurrency: 1,
        });
        const [result] = await generateOgImages(
          [{ outputPath, props: { title: "Hello OG" } }],
          options,
          root,
        );
        expect(result).toEqual({ outputPath, cached: false });
        expect(firstPixel(await readFile(outputPath))).toEqual([18, 52, 86]);
      } finally {
        await rm(root, { recursive: true, force: true });
      }
    },
  );
});

describe.each<OgImageRenderer>(["satori", "takumi"])("%s resources and cache", (renderer) => {
  it("invalidates cached images when an imported Vue component's CSS changes", async () => {
    const root = await mkdtemp(path.resolve(import.meta.dirname, "../.og-imports-"));
    try {
      await writeFile(path.join(root, "og.vue"), templates.vue);
      const child = path.join(root, "Card.vue");
      const card = (color: string) =>
        `<script setup lang="ts">defineProps<{title:string}>();</script><template><div class="card">{{title}}</div></template><style>.card{display:flex;width:100%;height:100%;background:${color}}</style>`;
      await writeFile(child, card("#123456"));
      const outputPath = path.join(root, "og.png");
      const pages = [{ outputPath, props: { title: "Imported CSS" } }];
      const options = resolveOgImageOptions({
        renderer,
        template: "og.vue",
        width: 320,
        height: 160,
      });
      expect((await generateOgImages(pages, options, root))[0].cached).toBe(false);
      expect((await generateOgImages(pages, options, root))[0].cached).toBe(true);
      await writeFile(child, card("#abcdef"));
      expect((await generateOgImages(pages, options, root))[0].cached).toBe(false);
      expect(firstPixel(await readFile(outputPath))).toEqual([171, 205, 239]);
    } finally {
      await rm(root, { recursive: true, force: true });
    }
  });
  it("renders public images and invalidates the cache when switching backends", async () => {
    const root = await mkdtemp(path.resolve(import.meta.dirname, "../.og-resources-"));
    try {
      await mkdir(path.join(root, "public"));
      await writeFile(
        path.join(root, "public/logo.svg"),
        '<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20"><rect width="20" height="20" fill="red"/></svg>',
      );
      await writeFile(
        path.join(root, "og.ts"),
        'export default () => `<div style="display:flex;width:100%;height:100%;background:#123456"><img src="/logo.svg" width="20" height="20" /></div>`',
      );
      const outputPath = path.join(root, "og.png");
      const pages = [{ outputPath, props: { title: "Resource test" } }];
      const options = resolveOgImageOptions({
        renderer,
        template: "og.ts",
        width: 320,
        height: 160,
      });
      expect((await generateOgImages(pages, options, root))[0]).toEqual({
        outputPath,
        cached: false,
      });
      expect((await generateOgImages(pages, options, root))[0]).toEqual({
        outputPath,
        cached: true,
      });
      expect(firstPixel(await readFile(outputPath))).toEqual([255, 0, 0]);
      const other = resolveOgImageOptions({
        ...options,
        renderer: renderer === "satori" ? "takumi" : "satori",
      });
      expect((await generateOgImages(pages, other, root))[0]).toEqual({
        outputPath,
        cached: false,
      });
    } finally {
      await rm(root, { recursive: true, force: true });
    }
  });
  it("reports missing font configuration without falling back to Chromium", async () => {
    const options = resolveOgImageOptions({
      renderer,
      cache: false,
      [renderer]: { systemFontFallback: false },
    });
    const [result] = await generateOgImages(
      [{ outputPath: "/unused.png", props: { title: "Font test" } }],
      options,
      import.meta.dirname,
    );
    expect(result.error).toContain(`ogImageOptions.${renderer}.fonts`);
  });
});
