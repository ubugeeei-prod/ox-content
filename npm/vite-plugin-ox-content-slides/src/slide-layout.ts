import {
  CANVAS_PLACEMENT_BOUNDS,
  CANVAS_SLIDE_LAYOUT,
  DEFAULT_CANVAS_PLACEMENT,
  SLIDE_ALIGN_OPTIONS,
  SLIDE_DENSITY_OPTIONS,
  SLIDE_DOM,
  SLIDE_FRONTMATTER_KEYS,
  SLIDE_LAYOUT_DEFAULTS,
  SLIDE_LAYOUT_OPTIONS,
  type SlideAlign,
  type SlideDensity,
  type SlideLayout,
  type SlideLayoutSettings,
  type SlidePlacement,
} from "./slide-schema";

const LAYOUTS = valueSet(SLIDE_LAYOUT_OPTIONS);
const ALIGNS = valueSet(SLIDE_ALIGN_OPTIONS);
const DENSITIES = valueSet(SLIDE_DENSITY_OPTIONS);

function stringValue(value: unknown): string | undefined {
  return typeof value === "string" && value.trim() ? value.trim() : undefined;
}

function valueSet<const TValue extends string>(
  options: readonly { value: TValue }[],
): ReadonlySet<TValue> {
  return new Set(options.map((option) => option.value));
}

function optionValue<const TValue extends string>(
  value: unknown,
  allowed: ReadonlySet<TValue>,
  fallback: TValue,
): TValue {
  const normalized = stringValue(value)?.toLowerCase();
  return normalized && allowed.has(normalized as TValue) ? (normalized as TValue) : fallback;
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
  return Math.min(
    CANVAS_PLACEMENT_BOUNDS.maxPercent,
    Math.max(CANVAS_PLACEMENT_BOUNDS.minPercent, number),
  );
}

function clampScale(value: unknown, fallback: number): number {
  const number = typeof value === "number" ? value : Number(value);
  if (!Number.isFinite(number)) return fallback;
  return Math.min(
    CANVAS_PLACEMENT_BOUNDS.maxScale,
    Math.max(CANVAS_PLACEMENT_BOUNDS.minScale, number),
  );
}

function parsePlacement(value: unknown): SlidePlacement | undefined {
  if (!value || typeof value !== "object") return undefined;
  const item = value as Record<string, unknown>;

  return {
    x: clampPercent(item.x, DEFAULT_CANVAS_PLACEMENT.x),
    y: clampPercent(item.y, DEFAULT_CANVAS_PLACEMENT.y),
    w: clampPercent(item.w, DEFAULT_CANVAS_PLACEMENT.w),
    h: clampPercent(item.h, DEFAULT_CANVAS_PLACEMENT.h),
    scale: clampScale(item.scale, DEFAULT_CANVAS_PLACEMENT.scale),
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
  return parsePlacementList(frontmatter[SLIDE_FRONTMATTER_KEYS.placements])
    .map(parsePlacement)
    .filter((placement) => Boolean(placement)) as SlidePlacement[];
}

function renderPlacementStyle(className: string, placements: SlidePlacement[]): string {
  if (placements.length === 0) return "";

  const rules = placements
    .map((placement, index) => {
      const selector = `.${className}.${SLIDE_DOM.canvasLayoutClass} > :nth-child(${index + 1})`;
      return `${selector}{left:${placement.x.toFixed(3)}%;top:${placement.y.toFixed(3)}%;width:${placement.w.toFixed(3)}%;height:${placement.h.toFixed(3)}%;transform-origin:0 0;transform:scale(${placement.scale.toFixed(3)});}`;
    })
    .join("");

  return `<style>${rules}</style>`;
}

/**
 * Extracts layout tokens from slide frontmatter.
 */
export function resolveSlideLayout(frontmatter: Record<string, unknown>): SlideLayoutSettings {
  const accent = stringValue(frontmatter[SLIDE_FRONTMATTER_KEYS.accent]);

  return {
    layout: optionValue<SlideLayout>(
      frontmatter[SLIDE_FRONTMATTER_KEYS.layout],
      LAYOUTS,
      SLIDE_LAYOUT_DEFAULTS.layout,
    ),
    align: optionValue<SlideAlign>(
      frontmatter[SLIDE_FRONTMATTER_KEYS.align],
      ALIGNS,
      SLIDE_LAYOUT_DEFAULTS.align,
    ),
    density: optionValue<SlideDensity>(
      frontmatter[SLIDE_FRONTMATTER_KEYS.density],
      DENSITIES,
      SLIDE_LAYOUT_DEFAULTS.density,
    ),
    accent: accent && isCssColorToken(accent) ? accent : undefined,
  };
}

/**
 * Wraps rendered slide HTML with layout metadata controlled by frontmatter.
 */
export function wrapSlideContent(html: string, frontmatter: Record<string, unknown>): string {
  const layout = resolveSlideLayout(frontmatter);
  const placements =
    layout.layout === CANVAS_SLIDE_LAYOUT ? resolveSlidePlacements(frontmatter) : [];
  const placementClass =
    placements.length > 0
      ? `${SLIDE_DOM.placementClassPrefix}-${hashString(`${html}${JSON.stringify(placements)}`)}`
      : "";
  const classes = [
    SLIDE_DOM.layoutClass,
    `${SLIDE_DOM.layoutClass}--${layout.layout}`,
    `ox-slide-align--${layout.align}`,
    `ox-slide-density--${layout.density}`,
    placementClass,
  ];
  const style = layout.accent
    ? ` style="--ox-slide-accent: ${escapeHtmlAttribute(layout.accent)}"`
    : "";
  const placementAttr = placements.length > 0 ? ` ${SLIDE_DOM.hasPlacementsAttribute}="true"` : "";

  return `${renderPlacementStyle(placementClass, placements)}<div class="${classes.filter(Boolean).join(" ")}"${placementAttr}${style}>${html}</div>`;
}
