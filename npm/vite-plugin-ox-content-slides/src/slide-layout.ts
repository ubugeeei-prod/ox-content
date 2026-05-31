const LAYOUTS = new Set(["stack", "statement", "split", "quote", "code", "canvas"]);
const ALIGNS = new Set(["start", "center", "end"]);
const DENSITIES = new Set(["compact", "balanced", "spacious"]);

interface SlideLayoutSettings {
  layout: string;
  align: string;
  density: string;
  accent?: string;
}

interface SlidePlacement {
  x: number;
  y: number;
  w: number;
  h: number;
}

const PERCENT_MIN = 0;
const PERCENT_MAX = 100;

function stringValue(value: unknown): string | undefined {
  return typeof value === "string" && value.trim() ? value.trim() : undefined;
}

function optionValue(value: unknown, allowed: Set<string>, fallback: string): string {
  const normalized = stringValue(value)?.toLowerCase();
  return normalized && allowed.has(normalized) ? normalized : fallback;
}

function isCssColorToken(value: string): boolean {
  return (
    /^#[\da-f]{3,8}$/i.test(value) ||
    /^rgba?\([\d\s,.%]+\)$/i.test(value) ||
    /^hsla?\([\d\s,.%deg]+\)$/i.test(value)
  );
}

function escapeHtmlAttribute(value: string): string {
  return value
    .replace(/&/g, "&amp;")
    .replace(/"/g, "&quot;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}

function hashString(value: string): string {
  let hash = 0x811c9dc5;
  for (let index = 0; index < value.length; index += 1) {
    hash ^= value.charCodeAt(index);
    hash = Math.imul(hash, 0x01000193);
  }
  return (hash >>> 0).toString(36);
}

function clampPercent(value: unknown, fallback: number): number {
  const number = typeof value === "number" ? value : Number(value);
  if (!Number.isFinite(number)) return fallback;
  return Math.min(PERCENT_MAX, Math.max(PERCENT_MIN, number));
}

function parsePlacement(value: unknown): SlidePlacement | undefined {
  if (!value || typeof value !== "object") return undefined;
  const item = value as Record<string, unknown>;

  return {
    x: clampPercent(item.x, 8),
    y: clampPercent(item.y, 8),
    w: clampPercent(item.w, 84),
    h: clampPercent(item.h, 18),
  };
}

function parsePlacementList(value: unknown): unknown[] {
  if (Array.isArray(value)) return value;
  if (typeof value !== "string" || !value.trim()) return [];

  try {
    const parsed = JSON.parse(value) as unknown;
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}

/**
 * Extracts canvas placements from slide frontmatter.
 */
export function resolveSlidePlacements(frontmatter: Record<string, unknown>): SlidePlacement[] {
  return parsePlacementList(frontmatter.placements)
    .map(parsePlacement)
    .filter((placement) => Boolean(placement)) as SlidePlacement[];
}

function renderPlacementStyle(className: string, placements: SlidePlacement[]): string {
  if (placements.length === 0) return "";

  const rules = placements
    .map((placement, index) => {
      const selector = `.${className}.ox-slide-layout--canvas > :nth-child(${index + 1})`;
      return `${selector}{left:${placement.x.toFixed(3)}%;top:${placement.y.toFixed(3)}%;width:${placement.w.toFixed(3)}%;height:${placement.h.toFixed(3)}%;}`;
    })
    .join("");

  return `<style>${rules}</style>`;
}

/**
 * Extracts layout tokens from slide frontmatter.
 */
export function resolveSlideLayout(frontmatter: Record<string, unknown>): SlideLayoutSettings {
  const accent = stringValue(frontmatter.accent);

  return {
    layout: optionValue(frontmatter.layout, LAYOUTS, "stack"),
    align: optionValue(frontmatter.align, ALIGNS, "start"),
    density: optionValue(frontmatter.density, DENSITIES, "balanced"),
    accent: accent && isCssColorToken(accent) ? accent : undefined,
  };
}

/**
 * Wraps rendered slide HTML with layout metadata controlled by frontmatter.
 */
export function wrapSlideContent(html: string, frontmatter: Record<string, unknown>): string {
  const layout = resolveSlideLayout(frontmatter);
  const placements = layout.layout === "canvas" ? resolveSlidePlacements(frontmatter) : [];
  const placementClass =
    placements.length > 0
      ? `ox-slide-placement-${hashString(`${html}${JSON.stringify(placements)}`)}`
      : "";
  const classes = [
    "ox-slide-layout",
    `ox-slide-layout--${layout.layout}`,
    `ox-slide-align--${layout.align}`,
    `ox-slide-density--${layout.density}`,
    placementClass,
  ];
  const style = layout.accent
    ? ` style="--ox-slide-accent: ${escapeHtmlAttribute(layout.accent)}"`
    : "";
  const placementAttr = placements.length > 0 ? ' data-ox-has-placements="true"' : "";

  return `${renderPlacementStyle(placementClass, placements)}<div class="${classes.filter(Boolean).join(" ")}"${placementAttr}${style}>${html}</div>`;
}
