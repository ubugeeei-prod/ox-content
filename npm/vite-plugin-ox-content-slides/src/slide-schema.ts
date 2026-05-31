export const SLIDE_FRONTMATTER_KEYS = {
  title: "title",
  description: "description",
  layout: "layout",
  align: "align",
  density: "density",
  accent: "accent",
  placements: "placements",
} as const;

export const SLIDE_FRONTMATTER_ORDER = [
  SLIDE_FRONTMATTER_KEYS.title,
  SLIDE_FRONTMATTER_KEYS.description,
  SLIDE_FRONTMATTER_KEYS.layout,
  SLIDE_FRONTMATTER_KEYS.align,
  SLIDE_FRONTMATTER_KEYS.density,
  SLIDE_FRONTMATTER_KEYS.accent,
  SLIDE_FRONTMATTER_KEYS.placements,
] as const;

export interface SlideChoice<TValue extends string> {
  value: TValue;
  label: string;
  title: string;
}

export interface SlideAccentChoice<TValue extends string> extends SlideChoice<TValue> {
  foreground: string;
}

export const SLIDE_LAYOUT_OPTIONS = [
  { value: "stack", label: "Stack", title: "Stacked content" },
  { value: "statement", label: "Statement", title: "Centered statement" },
  { value: "split", label: "Split", title: "Two-column split" },
  { value: "quote", label: "Quote", title: "Quote emphasis" },
  { value: "code", label: "Code", title: "Code-first slide" },
  { value: "canvas", label: "Canvas", title: "Directly place slide elements" },
] as const satisfies readonly SlideChoice<string>[];

export const SLIDE_ALIGN_OPTIONS = [
  { value: "start", label: "Start", title: "Align start" },
  { value: "center", label: "Center", title: "Align center" },
  { value: "end", label: "End", title: "Align end" },
] as const satisfies readonly SlideChoice<string>[];

export const SLIDE_DENSITY_OPTIONS = [
  { value: "compact", label: "Compact", title: "Compact spacing" },
  { value: "balanced", label: "Balanced", title: "Balanced spacing" },
  { value: "spacious", label: "Spacious", title: "Spacious spacing" },
] as const satisfies readonly SlideChoice<string>[];

export const SLIDE_ACCENT_OPTIONS = [
  { value: "#111111", label: "Night", title: "Night black", foreground: "#ffffff" },
  { value: "#1f1f1f", label: "Ink", title: "Ink black", foreground: "#ffffff" },
  { value: "#2b2b2b", label: "Graphite", title: "Graphite", foreground: "#ffffff" },
  { value: "#3a3a3a", label: "Charcoal", title: "Charcoal", foreground: "#ffffff" },
  { value: "#4a4a4a", label: "Iron", title: "Iron gray", foreground: "#ffffff" },
  { value: "#5c5c5c", label: "Smoke", title: "Smoke gray", foreground: "#ffffff" },
  { value: "#666666", label: "Ash", title: "Ash gray", foreground: "#ffffff" },
  { value: "#737373", label: "Neutral", title: "Neutral gray", foreground: "#ffffff" },
] as const satisfies readonly SlideAccentChoice<string>[];

export type SlideLayout = (typeof SLIDE_LAYOUT_OPTIONS)[number]["value"];
export type SlideAlign = (typeof SLIDE_ALIGN_OPTIONS)[number]["value"];
export type SlideDensity = (typeof SLIDE_DENSITY_OPTIONS)[number]["value"];

export interface SlideLayoutSettings {
  layout: SlideLayout;
  align: SlideAlign;
  density: SlideDensity;
  accent?: string;
}

export interface SlidePlacement {
  x: number;
  y: number;
  w: number;
  h: number;
}

export const SLIDE_LAYOUT_DEFAULTS = {
  layout: "stack",
  align: "start",
  density: "balanced",
  accent: "",
} as const satisfies {
  layout: SlideLayout;
  align: SlideAlign;
  density: SlideDensity;
  accent: string;
};

export const CANVAS_SLIDE_LAYOUT = "canvas" as const satisfies SlideLayout;
const SLIDE_LAYOUT_CLASS = "ox-slide-layout";

export const SLIDE_DOM = {
  layoutClass: SLIDE_LAYOUT_CLASS,
  canvasLayoutClass: `${SLIDE_LAYOUT_CLASS}--${CANVAS_SLIDE_LAYOUT}`,
  hasPlacementsAttribute: "data-ox-has-placements",
  placementClassPrefix: "ox-slide-placement",
} as const;

export const DEFAULT_CANVAS_PLACEMENT = {
  x: 8,
  y: 8,
  w: 84,
  h: 18,
} as const satisfies SlidePlacement;

export const CANVAS_PLACEMENT_BOUNDS = {
  minPercent: 0,
  maxPercent: 100,
  minSizePercent: 5,
} as const;

export const CANVAS_AUTO_PLACEMENT = {
  multiColumnMinItems: 3,
  multiColumnCount: 2,
  gapPercent: 4,
  widthPercent: 88,
  heightPercent: 84,
  xPercent: 6,
  yPercent: 8,
} as const;

export const CANVAS_SNAP = {
  gridStepPercent: 2.5,
  majorGridStepPercent: 10,
  thresholdPercent: 0.55,
} as const;

export const CANVAS_EDITOR_CLASSES = {
  overlay: "ox-editor-canvas-overlay",
  box: "ox-editor-canvas-box",
  grabHandle: "ox-editor-canvas-grab-handle",
  resize: "ox-editor-canvas-resize",
  guide: "ox-editor-canvas-guide",
  verticalGuide: "ox-editor-canvas-guide--vertical",
  horizontalGuide: "ox-editor-canvas-guide--horizontal",
} as const;

export const CANVAS_EDITOR_STYLE = {
  zIndex: 2147483647,
  majorGridLine: "rgba(78, 93, 104, 0.16)",
  minorGridLine: "rgba(78, 93, 104, 0.08)",
  centerGuideLine: "rgba(65, 83, 94, 0.22)",
  selectionBorder: "#2f3437",
  selectionFill: "rgba(17, 17, 17, 0.035)",
  selectionShadow: "0 0 0 1px rgba(255, 255, 255, 0.78), 0 8px 24px rgba(15, 23, 42, 0.13)",
  activeBorder: "#111111",
  activeFill: "rgba(17, 17, 17, 0.06)",
  handleText: "#ffffff",
  handleBorder: "#fff",
  guideLine: "rgba(17, 17, 17, 0.58)",
} as const;
