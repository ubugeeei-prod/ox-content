import { readFile } from "node:fs/promises";
import * as path from "node:path";
import { unified } from "unified";
import parse from "rehype-parse";
import stringify from "rehype-stringify";
import type { Root, Element } from "hast";
import { interopDefault } from "../interop";

const processor = unified()
  .use(interopDefault(parse), { fragment: true })
  .use(interopDefault(stringify));
const mime: Record<string, string> = {
  ".png": "image/png",
  ".jpg": "image/jpeg",
  ".jpeg": "image/jpeg",
  ".svg": "image/svg+xml",
  ".gif": "image/gif",
  ".webp": "image/webp",
};

/** Browserless engines have no HTTP server for the project's public assets. */
export async function inlinePublicImages(source: string, root: string): Promise<string> {
  const tree = processor.parse(source) as Root;
  const publicDir = path.resolve(root, "public");
  async function inline(url: string): Promise<string> {
    if (!url || /^(?:[a-z][a-z\d+.-]*:|\/\/|#)/i.test(url)) return url;
    const file = path.resolve(
      publicDir,
      decodeURIComponent(url.split(/[?#]/)[0]).replace(/^\//, ""),
    );
    const relative = path.relative(publicDir, file);
    if (relative.startsWith("..") || path.isAbsolute(relative))
      throw new Error("OG image asset is outside public/: " + url);
    const type = mime[path.extname(file).toLowerCase()];
    if (!type) return url;
    return `data:${type};base64,${(await readFile(file)).toString("base64")}`;
  }
  async function inlineCss(css: string): Promise<string> {
    const matches = [...css.matchAll(/url\(\s*["']?([^"')\s]+)["']?\s*\)/g)];
    const urls = await Promise.all(matches.map((match) => inline(match[1])));
    let index = 0;
    return css.replace(/url\(\s*["']?([^"')\s]+)["']?\s*\)/g, () => `url("${urls[index++]}")`);
  }
  async function visit(node: Root | Element): Promise<void> {
    if (node.type === "element") {
      if (node.tagName === "img" && typeof node.properties.src === "string")
        node.properties.src = await inline(node.properties.src);
      if (typeof node.properties.style === "string")
        node.properties.style = await inlineCss(node.properties.style);
      if (node.tagName === "style")
        for (const child of node.children)
          if (child.type === "text") child.value = await inlineCss(child.value);
    }
    await Promise.all(
      node.children.map((child) => (child.type === "element" ? visit(child) : undefined)),
    );
  }
  await visit(tree);
  return processor.stringify(tree);
}
