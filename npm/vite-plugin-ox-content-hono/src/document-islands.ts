import { parseFragment, type DefaultTreeAdapterTypes } from "parse5";

interface IslandRange {
  start: number;
  end: number;
  children: IslandRange[];
}

/** Give the HTML host disjoint ranges so replacing a child cannot move its parent's offsets. */
export function splitNestedIslands(html: string, prefix: string): string[] {
  if (!html.includes("data-ox-island=")) return [html];
  const tree = parseFragment(html, { sourceCodeLocationInfo: true });
  const root: IslandRange = { start: 0, end: html.length, children: [] };

  function collect(node: DefaultTreeAdapterTypes.Node, parent: IslandRange) {
    if ("tagName" in node && (node.tagName === "div" || node.tagName === "span")) {
      const location = node.sourceCodeLocation;
      if (location?.endTag && node.attrs.some((attr) => attr.name === "data-ox-island")) {
        const range = { start: location.startOffset, end: location.endOffset, children: [] };
        parent.children.push(range);
        parent = range;
      }
    }
    if ("childNodes" in node) {
      for (const child of node.childNodes) collect(child, parent);
    }
  }
  collect(tree, root);

  const fragments = [""];
  function replaceChildren(range: IslandRange, replace: (child: IslandRange) => string) {
    let start = range.start;
    let result = "";
    for (const child of range.children) {
      result += html.slice(start, child.start) + replace(child);
      start = child.end;
    }
    return result + html.slice(start, range.end);
  }
  function store(child: IslandRange): string {
    const index = fragments.length;
    fragments.push("");
    fragments[index] = replaceChildren(child, store);
    return `<!--${prefix}slot-${index}-->`;
  }

  fragments[0] = replaceChildren(root, (island) => replaceChildren(island, store));
  return fragments;
}
