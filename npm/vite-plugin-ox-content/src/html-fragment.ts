import type { Root } from "hast";
import { fromParse5 } from "hast-util-from-parse5";
import { parseFragment } from "parse5";
import type { Plugin } from "unified";

/** Parse generated HTML whose source positions are not used by the renderer. */
export function parseHtmlFragment(html: string): Root {
  return fromParse5(
    parseFragment(html, { sourceCodeLocationInfo: false, scriptingEnabled: false }),
  ) as Root;
}

export function createCachedHtmlParser(html: string): () => Root {
  let tree: Root | undefined;
  return () => (tree ??= parseHtmlFragment(html));
}

export const rehypeParseFragment: Plugin<[], string, Root> = function () {
  this.parser = parseHtmlFragment;
};
