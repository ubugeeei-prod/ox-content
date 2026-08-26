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
  let result = html;
  let idx = 0;

  while (true) {
    const block = findNextStaticDiagramBlock(result, idx);
    if (!block) break;

    const placeholder = `<!--ox-static-diagram-${svgs.size}-->`;
    svgs.set(placeholder, result.substring(block.start, block.end));
    result = result.substring(0, block.start) + placeholder + result.substring(block.end);
    idx = block.start + placeholder.length;
  }

  return { html: result, svgs };
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

function findNextStaticDiagramBlock(
  html: string,
  startAt: number,
): { start: number; end: number } | null {
  const lower = html.toLowerCase();
  const markers = [
    { marker: '<div class="ox-mermaid"', tag: "div" },
    { marker: '<figure class="ox-graphviz"', tag: "figure" },
  ];
  const next = markers
    .map((candidate) => ({ ...candidate, start: lower.indexOf(candidate.marker, startAt) }))
    .filter((candidate) => candidate.start !== -1)
    .sort((a, b) => a.start - b.start)[0];
  if (!next) return null;

  const end = findMatchingElementEnd(lower, next.start, next.tag);
  return end === -1 ? null : { start: next.start, end };
}

function findMatchingElementEnd(html: string, start: number, tag: string): number {
  let depth = 0;
  let pos = start;
  const open = `<${tag}`;
  const close = `</${tag}>`;

  while (pos < html.length) {
    const openIdx = html.indexOf(open, pos);
    const closeIdx = html.indexOf(close, pos);
    if (closeIdx === -1) return -1;

    if (openIdx !== -1 && openIdx < closeIdx) {
      depth++;
      pos = openIdx + open.length;
    } else {
      depth--;
      if (depth === 0) return closeIdx + close.length;
      pos = closeIdx + close.length;
    }
  }

  return -1;
}
