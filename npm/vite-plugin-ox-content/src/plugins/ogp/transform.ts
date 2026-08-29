import { unified } from "unified";
import rehypeParsePlugin from "rehype-parse";
import rehypeStringifyPlugin from "rehype-stringify";
import type { Root, Element } from "hast";
import { interopDefault } from "../../interop";
import { fetchOgpData } from "./fetch";
import { createFallbackCard, createOgpCard } from "./render";
import type { OgpData, OgpOptions } from "./types";
import { isSafeOgpUrl } from "./url";

const rehypeParse = interopDefault(rehypeParsePlugin);
const rehypeStringify = interopDefault(rehypeStringifyPlugin);

function getAttribute(el: Element, name: string): string | undefined {
  const value = el.properties?.[name];
  if (typeof value === "string") return value;
  if (Array.isArray(value)) return value.join(" ");
  return undefined;
}

export async function collectOgpUrls(html: string): Promise<string[]> {
  const urls: string[] = [];
  const urlPattern = /<ogcard[^>]*\s+url=["']([^"']+)["']/gi;

  let match;
  while ((match = urlPattern.exec(html)) !== null) {
    if (isSafeOgpUrl(match[1])) {
      urls.push(match[1]);
    }
  }

  return urls;
}

export async function prefetchOgpData(
  urls: string[],
  options?: OgpOptions,
): Promise<Map<string, OgpData | null>> {
  const results = new Map<string, OgpData | null>();

  await Promise.all(
    urls.map(async (url) => {
      results.set(url, await fetchOgpData(url, options));
    }),
  );

  return results;
}

function rehypeOgp(ogpDataMap: Map<string, OgpData | null>) {
  return (tree: Root) => {
    const visit = (node: Root | Element) => {
      if ("children" in node) {
        for (let i = 0; i < node.children.length; i++) {
          const child = node.children[i];

          if (child.type === "element") {
            if (child.tagName.toLowerCase() === "ogcard") {
              const url = getAttribute(child, "url");

              if (url) {
                const ogpData = ogpDataMap.get(url);
                node.children[i] = ogpData ? createOgpCard(ogpData) : createFallbackCard(url);
              }
            } else {
              visit(child);
            }
          }
        }
      }
    };

    visit(tree);
  };
}

export async function transformOgp(
  html: string,
  ogpDataMap?: Map<string, OgpData | null>,
  options?: OgpOptions,
): Promise<string> {
  if (!/<ogcard\b/i.test(html)) {
    return html;
  }

  let dataMap = ogpDataMap;
  if (!dataMap) {
    dataMap = await prefetchOgpData(await collectOgpUrls(html), options);
  }

  const result = await unified()
    .use(rehypeParse, { fragment: true })
    .use(rehypeOgp, dataMap)
    .use(rehypeStringify)
    .process(html);

  return String(result);
}
