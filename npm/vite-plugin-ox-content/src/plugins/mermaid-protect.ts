/**
 * Protects mermaid SVG content from rehype HTML5 parser corruption.
 *
 * rehypeParse + rehypeStringify converts `<br />` in SVG foreignObject
 * to `<br></br>`, which HTML5 interprets as 2 <br> elements.
 * Each rehype pass doubles them: 1 → 2 → 4 → 8 → 16.
 *
 * This module extracts ox-mermaid SVG blocks into placeholders before
 * rehype processing and restores them after.
 */

export interface MermaidSvgProtection {
  html: string;
  svgs: Map<string, string>;
}

/**
 * Extract `<div class="ox-mermaid">...</div>` blocks and replace
 * with HTML comment placeholders that rehype will preserve.
 */
export function protectMermaidSvgs(html: string): MermaidSvgProtection {
  const svgs = new Map<string, string>();
  let result = html;
  let idx = 0;

  while (true) {
    const marker = `<div class="ox-mermaid">`;
    const start = result.indexOf(marker, idx);
    if (start === -1) break;

    // Find the matching </div> by counting nested divs
    let depth = 0;
    let pos = start;
    let endPos = -1;

    while (pos < result.length) {
      const openIdx = result.indexOf("<div", pos);
      const closeIdx = result.indexOf("</div>", pos);
      if (closeIdx === -1) break;

      if (openIdx !== -1 && openIdx < closeIdx) {
        depth++;
        pos = openIdx + 4;
      } else {
        depth--;
        if (depth === 0) {
          endPos = closeIdx + 6;
          break;
        }
        pos = closeIdx + 6;
      }
    }

    if (endPos === -1) break;

    const svgContent = result.substring(start, endPos);
    const placeholder = `<!--ox-mermaid-${svgs.size}-->`;
    svgs.set(placeholder, svgContent);
    result = result.substring(0, start) + placeholder + result.substring(endPos);
    idx = start + placeholder.length;
  }

  return { html: result, svgs };
}

/**
 * Restore protected mermaid SVG blocks from placeholders.
 */
export function restoreMermaidSvgs(html: string, svgs: Map<string, string>): string {
  if (svgs.size === 0) {
    return html;
  }
  // Single pass over the HTML instead of one full `String.replace` scan per
  // placeholder (O(svgs × html) → O(html)). The function replacer also avoids
  // the `$`-pattern interpretation that the string form of `replace` applies
  // to the SVG replacement content.
  return html.replace(/<!--ox-mermaid-\d+-->/g, (placeholder) => {
    const content = svgs.get(placeholder);
    return content !== undefined ? content : placeholder;
  });
}
