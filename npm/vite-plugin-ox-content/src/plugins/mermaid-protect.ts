/**
 * Protects generated static SVG content from rehype HTML5 parser corruption.
 *
 * rehypeParse + rehypeStringify converts `<br />` in SVG foreignObject
 * to `<br></br>`, which HTML5 interprets as 2 <br> elements.
 * Each rehype pass doubles them: 1 → 2 → 4 → 8 → 16.
 *
 * This module extracts ox-mermaid and ox-graphviz SVG blocks into
 * placeholders before rehype processing and restores them after.
 */

export interface MermaidSvgProtection {
  html: string;
  svgs: Map<string, string>;
}

/**
 * Extract generated static diagram blocks and replace them with HTML comment
 * placeholders that rehype will preserve.
 */
export function protectStaticDiagramSvgs(html: string): MermaidSvgProtection {
  const svgs = new Map<string, string>();
  const chunks: string[] = [];
  const markers = /<div class="ox-mermaid"|<figure class="ox-graphviz"/gi;
  let cursor = 0;
  let match: RegExpExecArray | null;

  while ((match = markers.exec(html)) !== null) {
    const tag = match[0].slice(1, 4).toLowerCase() === "div" ? "div" : "figure";
    const end = findMatchingElementEnd(html, match.index, tag);
    if (end === -1) break;

    const placeholder = `<!--ox-static-diagram-${svgs.size}-->`;
    // Retain slices of one source string, not one whole intermediate document
    // for every diagram. Offsets stay in the original UTF-16 string as well.
    svgs.set(placeholder, html.slice(match.index, end));
    chunks.push(html.slice(cursor, match.index), placeholder);
    cursor = end;
    markers.lastIndex = end;
  }

  if (svgs.size === 0) return { html, svgs };
  chunks.push(html.slice(cursor));
  return { html: chunks.join(""), svgs };
}

/**
 * Restore generated static diagram blocks from placeholders.
 */
export function restoreStaticDiagramSvgs(html: string, svgs: Map<string, string>): string {
  if (svgs.size === 0) {
    return html;
  }
  return html.replace(/<!--ox-static-diagram-\d+-->/g, (placeholder) => {
    const content = svgs.get(placeholder);
    return content !== undefined ? content : placeholder;
  });
}

/**
 * Extract `<div class="ox-mermaid">...</div>` blocks and replace
 * with HTML comment placeholders that rehype will preserve.
 */
export function protectMermaidSvgs(html: string): MermaidSvgProtection {
  return protectStaticDiagramSvgs(html);
}

/**
 * Restore protected mermaid SVG blocks from placeholders.
 */
export function restoreMermaidSvgs(html: string, svgs: Map<string, string>): string {
  return restoreStaticDiagramSvgs(html, svgs);
}

function findMatchingElementEnd(html: string, start: number, tag: "div" | "figure"): number {
  const tags = tag === "div" ? /<div|<\/div>/gi : /<figure|<\/figure>/gi;
  tags.lastIndex = start;
  let depth = 0;
  let match: RegExpExecArray | null;
  while ((match = tags.exec(html)) !== null) {
    depth += match[0][1] === "/" ? -1 : 1;
    if (depth === 0) return tags.lastIndex;
  }
  return -1;
}
