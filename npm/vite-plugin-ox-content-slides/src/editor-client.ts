import {
  CANVAS_AUTO_PLACEMENT,
  CANVAS_EDITOR_CLASSES,
  CANVAS_EDITOR_STYLE,
  CANVAS_PLACEMENT_BOUNDS,
  CANVAS_SLIDE_LAYOUT,
  CANVAS_SNAP,
  DEFAULT_CANVAS_PLACEMENT,
  SLIDE_ALIGN_OPTIONS,
  SLIDE_DENSITY_OPTIONS,
  SLIDE_DOM,
  SLIDE_FRONTMATTER_KEYS,
  SLIDE_FRONTMATTER_ORDER,
  SLIDE_LAYOUT_DEFAULTS,
  SLIDE_LAYOUT_OPTIONS,
  type SlideAlign,
  type SlideDensity,
  type SlideLayout,
  type SlidePlacement,
} from "./slide-schema";

interface EditorSlide {
  filePath: string;
  title: string;
  slideNumber: number;
  href: string;
  presenterHref?: string;
}

interface EditorDeck {
  title: string;
  slides: EditorSlide[];
}

interface EditorState {
  decks: EditorDeck[];
  selected: EditorSlide | null;
  dirty: boolean;
  saveTimer: number;
}

interface ParsedFrontmatter {
  data: Record<string, string>;
  order: string[];
  body: string;
}

interface LayoutSettings {
  layout: SlideLayout;
  align: SlideAlign;
  density: SlideDensity;
  accent: string;
}

type CanvasDragMode = "move" | "resize";

interface CanvasSnapResult {
  placement: SlidePlacement;
  guideX?: number;
  guideY?: number;
}

interface CanvasDragState {
  mode: CanvasDragMode;
  index: number;
  startX: number;
  startY: number;
  start: SlidePlacement;
  layoutRect: DOMRect;
  target: HTMLElement;
  box: HTMLElement;
}

interface EditorElements {
  decks: HTMLElement;
  source: HTMLTextAreaElement;
  file: HTMLElement;
  status: HTMLElement;
  preview: HTMLIFrameElement;
  open: HTMLAnchorElement;
  presenter: HTMLAnchorElement;
  save: HTMLButtonElement;
  add: HTMLButtonElement;
  layoutButtons: NodeListOf<HTMLButtonElement>;
  alignButtons: NodeListOf<HTMLButtonElement>;
  densityButtons: NodeListOf<HTMLButtonElement>;
  accentButtons: NodeListOf<HTMLButtonElement>;
  accentCustom: HTMLInputElement;
}

declare global {
  var __OX_SLIDE_EDITOR_API__: string | undefined;
}

interface EditorClientConfig {
  frontmatter: {
    keys: typeof SLIDE_FRONTMATTER_KEYS;
    order: readonly string[];
  };
  defaults: {
    layout: SlideLayout;
    align: SlideAlign;
    density: SlideDensity;
    accent: string;
  };
  values: {
    layout: readonly SlideLayout[];
    align: readonly SlideAlign[];
    density: readonly SlideDensity[];
  };
  dom: typeof SLIDE_DOM;
  canvas: {
    layout: typeof CANVAS_SLIDE_LAYOUT;
    snap: typeof CANVAS_SNAP;
    placementBounds: typeof CANVAS_PLACEMENT_BOUNDS;
    defaultPlacement: SlidePlacement;
    autoPlacement: typeof CANVAS_AUTO_PLACEMENT;
    editorClasses: typeof CANVAS_EDITOR_CLASSES;
    editorStyle: typeof CANVAS_EDITOR_STYLE;
  };
}

const SLIDE_EDITOR_CONFIG: EditorClientConfig = {
  frontmatter: {
    keys: SLIDE_FRONTMATTER_KEYS,
    order: SLIDE_FRONTMATTER_ORDER,
  },
  defaults: SLIDE_LAYOUT_DEFAULTS,
  values: {
    layout: SLIDE_LAYOUT_OPTIONS.map((option) => option.value),
    align: SLIDE_ALIGN_OPTIONS.map((option) => option.value),
    density: SLIDE_DENSITY_OPTIONS.map((option) => option.value),
  },
  dom: SLIDE_DOM,
  canvas: {
    layout: CANVAS_SLIDE_LAYOUT,
    snap: CANVAS_SNAP,
    placementBounds: CANVAS_PLACEMENT_BOUNDS,
    defaultPlacement: DEFAULT_CANVAS_PLACEMENT,
    autoPlacement: CANVAS_AUTO_PLACEMENT,
    editorClasses: CANVAS_EDITOR_CLASSES,
    editorStyle: CANVAS_EDITOR_STYLE,
  },
};

function queryElement<T extends Element>(selector: string): T {
  const element = document.querySelector<T>(selector);
  if (!element) {
    throw new Error(`Missing editor element: ${selector}`);
  }
  return element;
}

function parseFrontmatter(source: string): ParsedFrontmatter {
  const lines = source.split(/\r?\n/);
  if (lines[0]?.trim() !== "---") return { data: {}, order: [], body: source };
  const end = lines.findIndex((line, index) => index > 0 && line.trim() === "---");
  if (end === -1) return { data: {}, order: [], body: source };

  const data: Record<string, string> = {};
  const order: string[] = [];
  for (const line of lines.slice(1, end)) {
    const match = line.match(/^([A-Za-z0-9_-]+):\s*(.*)$/);
    if (!match) continue;
    const key = match[1] ?? "";
    let value = match[2]?.trim() ?? "";
    if (value.startsWith('"') && value.endsWith('"')) {
      value = value.slice(1, -1);
      value = value.replace(/\\(["\\])/g, "$1").replace(/\\n/g, "\n");
    } else if (value.startsWith("'") && value.endsWith("'")) {
      value = value.slice(1, -1);
    }
    data[key] = value;
    order.push(key);
  }

  return {
    data,
    order,
    body: lines
      .slice(end + 1)
      .join("\n")
      .replace(/^\n+/, ""),
  };
}

function formatFrontmatterValue(value: string): string {
  return `"${value.replace(/\\/g, "\\\\").replace(/"/g, '\\"')}"`;
}

function writeFrontmatter(source: string, updates: Record<string, string | undefined>): string {
  const parsed = parseFrontmatter(source);
  const data: Record<string, string | undefined> = { ...parsed.data, ...updates };
  for (const [key, value] of Object.entries(data)) {
    if (value === "" || value === undefined) delete data[key];
  }

  const preferred = SLIDE_EDITOR_CONFIG.frontmatter.order;
  const keys = [...new Set([...preferred, ...parsed.order, ...Object.keys(data)])].filter((key) =>
    Object.prototype.hasOwnProperty.call(data, key),
  );
  const frontmatter = keys.map((key) => `${key}: ${formatFrontmatterValue(data[key] ?? "")}`);

  return frontmatter.length > 0
    ? `---\n${frontmatter.join("\n")}\n---\n\n${parsed.body}`
    : parsed.body;
}

function pressed(buttons: NodeListOf<HTMLButtonElement>, attr: string, value: string): void {
  for (const button of buttons) {
    button.setAttribute("aria-pressed", String(button.getAttribute(attr) === value));
  }
}

function normalizeToken<const TValue extends string>(
  value: string | undefined,
  allowed: ReadonlySet<TValue>,
  fallback: TValue,
): TValue {
  const normalized = value?.toLowerCase();
  return normalized && allowed.has(normalized as TValue) ? (normalized as TValue) : fallback;
}

function clampNumber(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}

function roundedPercent(value: number): number {
  return Number(clampNumber(value, 0, 100).toFixed(3));
}

function finiteNumber(value: unknown): number | undefined {
  const number = typeof value === "number" ? value : Number(value);
  return Number.isFinite(number) ? number : undefined;
}

function normalizePlacement(
  value: Partial<SlidePlacement>,
  fallback: SlidePlacement,
): SlidePlacement {
  const bounds = SLIDE_EDITOR_CONFIG.canvas.placementBounds;
  const w = clampNumber(
    finiteNumber(value.w) ?? fallback.w,
    bounds.minSizePercent,
    bounds.maxPercent,
  );
  const h = clampNumber(
    finiteNumber(value.h) ?? fallback.h,
    bounds.minSizePercent,
    bounds.maxPercent,
  );
  const maxX = Math.max(bounds.minPercent, bounds.maxPercent - w);
  const maxY = Math.max(bounds.minPercent, bounds.maxPercent - h);

  return {
    x: roundedPercent(clampNumber(finiteNumber(value.x) ?? fallback.x, bounds.minPercent, maxX)),
    y: roundedPercent(clampNumber(finiteNumber(value.y) ?? fallback.y, bounds.minPercent, maxY)),
    w: roundedPercent(w),
    h: roundedPercent(h),
  };
}

function defaultPlacement(index: number, count: number): SlidePlacement {
  const auto = SLIDE_EDITOR_CONFIG.canvas.autoPlacement;
  const safeCount = Math.max(1, count);
  const columns = safeCount >= auto.multiColumnMinItems ? auto.multiColumnCount : 1;
  const rows = Math.ceil(safeCount / columns);
  const column = index % columns;
  const row = Math.floor(index / columns);
  const width = auto.widthPercent / columns - auto.gapPercent;
  const height = auto.heightPercent / rows - auto.gapPercent;

  return normalizePlacement(
    {
      x: auto.xPercent + column * (auto.widthPercent / columns),
      y: auto.yPercent + row * (auto.heightPercent / rows),
      w: width,
      h: height,
    },
    SLIDE_EDITOR_CONFIG.canvas.defaultPlacement,
  );
}

function isPlacement(value: unknown): value is Partial<SlidePlacement> {
  return Boolean(value) && typeof value === "object";
}

function parsePlacements(value: string | undefined): SlidePlacement[] {
  if (!value?.trim()) return [];

  try {
    const parsed = JSON.parse(value) as unknown;
    if (!Array.isArray(parsed)) return [];
    return parsed.filter(isPlacement).map((placement, index, items) =>
      normalizePlacement(
        {
          x: Number(placement.x),
          y: Number(placement.y),
          w: Number(placement.w),
          h: Number(placement.h),
        },
        defaultPlacement(index, items.length),
      ),
    );
  } catch {
    return [];
  }
}

function formatPlacements(placements: SlidePlacement[]): string {
  return JSON.stringify(
    placements.map((placement) => ({
      x: roundedPercent(placement.x),
      y: roundedPercent(placement.y),
      w: roundedPercent(placement.w),
      h: roundedPercent(placement.h),
    })),
  );
}

function softSnapPercent(value: number): { value: number; snapped: boolean } {
  const { gridStepPercent, thresholdPercent } = SLIDE_EDITOR_CONFIG.canvas.snap;
  const snapped = Math.round(value / gridStepPercent) * gridStepPercent;
  const shouldSnap = Math.abs(snapped - value) <= thresholdPercent;
  return {
    value: roundedPercent(shouldSnap ? snapped : value),
    snapped: shouldSnap,
  };
}

function snapPlacementToGrid(
  placement: SlidePlacement,
  fallback: SlidePlacement,
  mode: CanvasDragMode,
): CanvasSnapResult {
  if (mode === "move") {
    const x = softSnapPercent(placement.x);
    const y = softSnapPercent(placement.y);
    return {
      placement: normalizePlacement({ ...placement, x: x.value, y: y.value }, fallback),
      guideX: x.snapped ? x.value : undefined,
      guideY: y.snapped ? y.value : undefined,
    };
  }

  const w = softSnapPercent(placement.w);
  const h = softSnapPercent(placement.h);
  const next = normalizePlacement({ ...placement, w: w.value, h: h.value }, fallback);

  return {
    placement: next,
    guideX: w.snapped ? roundedPercent(next.x + next.w) : undefined,
    guideY: h.snapped ? roundedPercent(next.y + next.h) : undefined,
  };
}

function isHtmlElement(value: unknown): value is HTMLElement {
  return (
    value !== null &&
    typeof value === "object" &&
    "style" in value &&
    "getBoundingClientRect" in value
  );
}

function ensurePlacementCount(placements: SlidePlacement[], count: number): SlidePlacement[] {
  return Array.from({ length: count }, (_, index) =>
    placements[index]
      ? normalizePlacement(placements[index], defaultPlacement(index, count))
      : defaultPlacement(index, count),
  );
}

function applyElementPlacement(element: HTMLElement, placement: SlidePlacement): void {
  element.style.position = "absolute";
  element.style.left = `${placement.x}%`;
  element.style.top = `${placement.y}%`;
  element.style.width = `${placement.w}%`;
  element.style.height = `${placement.h}%`;
  element.style.margin = "0";
  element.style.overflow = "auto";
}

function applyCanvasLayoutStyles(layout: HTMLElement, placements: SlidePlacement[]): void {
  const parent = layout.parentElement;
  if (isHtmlElement(parent)) {
    parent.style.display = "grid";
    parent.style.gridTemplateRows = "minmax(0, 1fr)";
    parent.style.height = "100%";
  }
  layout.style.position = "relative";
  layout.style.display = "block";
  layout.style.height = "100%";
  layout.style.minHeight = "100%";
  if (layout.getBoundingClientRect().height <= 0 && isHtmlElement(parent)) {
    const parentRect = parent.getBoundingClientRect();
    if (parentRect.height > 0) {
      layout.style.height = `${parentRect.height}px`;
    }
  }
  Array.from(layout.children).forEach((child, index) => {
    if (isHtmlElement(child) && placements[index]) {
      applyElementPlacement(child, placements[index]);
    }
  });
}

function renderCanvasEditorStyle(): string {
  const { editorClasses: classes, editorStyle: style, snap } = SLIDE_EDITOR_CONFIG.canvas;
  const majorStep = `${snap.majorGridStepPercent}%`;
  const minorStep = `${snap.gridStepPercent}%`;

  return `
    .${classes.overlay} {
      position: fixed;
      z-index: ${style.zIndex};
      pointer-events: auto;
      background-image:
        linear-gradient(to right, ${style.majorGridLine} 1px, transparent 1px),
        linear-gradient(to bottom, ${style.majorGridLine} 1px, transparent 1px),
        linear-gradient(to right, ${style.minorGridLine} 1px, transparent 1px),
        linear-gradient(to bottom, ${style.minorGridLine} 1px, transparent 1px);
      background-size: ${majorStep} ${majorStep}, ${majorStep} ${majorStep}, ${minorStep} ${minorStep}, ${minorStep} ${minorStep};
    }
    .${classes.overlay}::before {
      position: absolute;
      inset: 0;
      content: "";
      pointer-events: none;
      background:
        linear-gradient(to right, transparent calc(50% - 0.5px), ${style.centerGuideLine} calc(50% - 0.5px), ${style.centerGuideLine} calc(50% + 0.5px), transparent calc(50% + 0.5px)),
        linear-gradient(to bottom, transparent calc(50% - 0.5px), ${style.centerGuideLine} calc(50% - 0.5px), ${style.centerGuideLine} calc(50% + 0.5px), transparent calc(50% + 0.5px));
    }
    .${classes.box} {
      position: absolute;
      border: 1.5px solid ${style.selectionBorder};
      background: ${style.selectionFill};
      box-shadow: ${style.selectionShadow};
      cursor: move;
      pointer-events: auto;
      touch-action: none;
    }
    .${classes.box}[data-selected="true"] {
      border-color: ${style.activeBorder};
      background: ${style.activeFill};
    }
    .${classes.label} {
      position: absolute;
      left: -1px;
      top: -25px;
      min-width: 26px;
      height: 23px;
      display: inline-flex;
      align-items: center;
      justify-content: center;
      border-radius: 4px 4px 0 0;
      background: ${style.selectionBorder};
      color: ${style.labelText};
      font: 650 12px/1 system-ui, sans-serif;
    }
    .${classes.box}[data-selected="true"] .${classes.label} {
      background: ${style.activeBorder};
    }
    .${classes.resize} {
      position: absolute;
      right: -7px;
      bottom: -7px;
      width: 14px;
      height: 14px;
      border: 2px solid ${style.handleBorder};
      border-radius: 3px;
      background: ${style.selectionBorder};
      cursor: nwse-resize;
      touch-action: none;
    }
    .${classes.guide} {
      position: absolute;
      z-index: 1;
      display: none;
      pointer-events: none;
      background: ${style.guideLine};
    }
    .${classes.verticalGuide} {
      top: 0;
      bottom: 0;
      width: 1px;
    }
    .${classes.horizontalGuide} {
      left: 0;
      right: 0;
      height: 1px;
    }
  `;
}

function installCanvasEditorOverlay(
  doc: Document,
  layout: HTMLElement,
  placements: SlidePlacement[],
  onCommit: (placements: SlidePlacement[]) => void,
): () => void {
  const win = doc.defaultView;
  if (!win) return () => {};

  const style = doc.createElement("style");
  style.textContent = renderCanvasEditorStyle();
  doc.head.append(style);

  const classes = SLIDE_EDITOR_CONFIG.canvas.editorClasses;
  const overlay = doc.createElement("div");
  overlay.className = classes.overlay;
  doc.body.append(overlay);
  const verticalGuide = doc.createElement("div");
  verticalGuide.className = `${classes.guide} ${classes.verticalGuide}`;
  const horizontalGuide = doc.createElement("div");
  horizontalGuide.className = `${classes.guide} ${classes.horizontalGuide}`;
  overlay.append(verticalGuide, horizontalGuide);

  let current = placements.map((placement) => ({ ...placement }));
  let drag: CanvasDragState | undefined;

  const boxes = current.map((placement, index) => {
    const box = doc.createElement("div");
    box.className = classes.box;
    box.dataset.index = String(index);
    box.style.left = `${placement.x}%`;
    box.style.top = `${placement.y}%`;
    box.style.width = `${placement.w}%`;
    box.style.height = `${placement.h}%`;

    const label = doc.createElement("span");
    label.className = classes.label;
    label.textContent = String(index + 1);

    const handle = doc.createElement("span");
    handle.className = classes.resize;
    handle.title = "Resize";

    box.append(label, handle);
    overlay.append(box);

    function startDrag(event: PointerEvent, mode: CanvasDragMode): void {
      const target = layout.children[index];
      if (!isHtmlElement(target)) return;
      const layoutRect = layout.getBoundingClientRect();
      if (layoutRect.width <= 0 || layoutRect.height <= 0) return;

      event.preventDefault();
      event.stopPropagation();
      for (const item of boxes) item.dataset.selected = "false";
      box.dataset.selected = "true";
      box.setPointerCapture(event.pointerId);
      doc.body.style.userSelect = "none";
      drag = {
        mode,
        index,
        startX: event.clientX,
        startY: event.clientY,
        start: { ...current[index] },
        layoutRect,
        target,
        box,
      };
    }

    box.addEventListener("pointerdown", (event) => {
      startDrag(event, "move");
    });
    handle.addEventListener("pointerdown", (event) => {
      startDrag(event, "resize");
    });

    return box;
  });

  function updateGuides(result: CanvasSnapResult | undefined): void {
    if (result?.guideX !== undefined) {
      verticalGuide.style.left = `${result.guideX}%`;
      verticalGuide.style.display = "block";
    } else {
      verticalGuide.style.display = "none";
    }

    if (result?.guideY !== undefined) {
      horizontalGuide.style.top = `${result.guideY}%`;
      horizontalGuide.style.display = "block";
    } else {
      horizontalGuide.style.display = "none";
    }
  }

  function updateOverlayBounds(): void {
    const rect = layout.getBoundingClientRect();
    overlay.style.left = `${rect.left}px`;
    overlay.style.top = `${rect.top}px`;
    overlay.style.width = `${rect.width}px`;
    overlay.style.height = `${rect.height}px`;

    boxes.forEach((box, index) => {
      const placement = current[index];
      if (!placement) return;
      box.style.left = `${placement.x}%`;
      box.style.top = `${placement.y}%`;
      box.style.width = `${placement.w}%`;
      box.style.height = `${placement.h}%`;
    });
  }

  function updateDraggedPlacement(event: PointerEvent): void {
    if (!drag) return;
    const dx = ((event.clientX - drag.startX) / drag.layoutRect.width) * 100;
    const dy = ((event.clientY - drag.startY) / drag.layoutRect.height) * 100;
    const rawPlacement =
      drag.mode === "move"
        ? normalizePlacement(
            {
              ...drag.start,
              x: drag.start.x + dx,
              y: drag.start.y + dy,
            },
            drag.start,
          )
        : normalizePlacement(
            {
              ...drag.start,
              w: drag.start.w + dx,
              h: drag.start.h + dy,
            },
            drag.start,
          );
    const snapped = snapPlacementToGrid(rawPlacement, drag.start, drag.mode);
    const next = snapped.placement;

    current[drag.index] = next;
    applyElementPlacement(drag.target, next);
    updateGuides(snapped);
    updateOverlayBounds();
  }

  function finishDrag(): void {
    if (!drag) return;
    doc.body.style.userSelect = "";
    drag = undefined;
    updateGuides(undefined);
    onCommit(current.map((placement) => ({ ...placement })));
  }

  function onPointerMove(event: PointerEvent): void {
    updateDraggedPlacement(event);
  }

  function onPointerUp(): void {
    finishDrag();
  }

  win.addEventListener("pointermove", onPointerMove);
  win.addEventListener("pointerup", onPointerUp);
  win.addEventListener("resize", updateOverlayBounds);
  win.addEventListener("scroll", updateOverlayBounds, true);
  updateOverlayBounds();

  return () => {
    doc.body.style.userSelect = "";
    win.removeEventListener("pointermove", onPointerMove);
    win.removeEventListener("pointerup", onPointerUp);
    win.removeEventListener("resize", updateOverlayBounds);
    win.removeEventListener("scroll", updateOverlayBounds, true);
    overlay.remove();
    style.remove();
  };
}

/**
 * Browser client for the dev-only slide editor.
 */
export function createSlideEditorClient(): void {
  const api = globalThis.__OX_SLIDE_EDITOR_API__;
  if (!api) throw new Error("Missing editor API prefix.");

  const state: EditorState = { decks: [], selected: null, dirty: false, saveTimer: 0 };
  let cleanupPreviewEditor: (() => void) | undefined;
  const elements: EditorElements = {
    decks: queryElement("[data-decks]"),
    source: queryElement("[data-source]"),
    file: queryElement("[data-current-file]"),
    status: queryElement("[data-status]"),
    preview: queryElement("[data-preview]"),
    open: queryElement("[data-open]"),
    presenter: queryElement("[data-presenter]"),
    save: queryElement("[data-save]"),
    add: queryElement("[data-new]"),
    layoutButtons: document.querySelectorAll("[data-layout-value]"),
    alignButtons: document.querySelectorAll("[data-align-value]"),
    densityButtons: document.querySelectorAll("[data-density-value]"),
    accentButtons: document.querySelectorAll("[data-accent-value]"),
    accentCustom: queryElement("[data-accent-custom]"),
  };
  const { defaults } = SLIDE_EDITOR_CONFIG;
  const allowed = {
    layout: new Set(SLIDE_EDITOR_CONFIG.values.layout),
    align: new Set(SLIDE_EDITOR_CONFIG.values.align),
    density: new Set(SLIDE_EDITOR_CONFIG.values.density),
  };

  async function request<T>(requestPath: string, init?: RequestInit): Promise<T> {
    const response = await fetch(api + requestPath, init);
    if (!response.ok) throw new Error(await response.text());
    return response.json() as Promise<T>;
  }

  function setDirty(value: boolean): void {
    state.dirty = value;
    elements.status.textContent = value ? "Unsaved" : "Clean";
  }

  function readLayoutSettings(): LayoutSettings {
    const data = parseFrontmatter(elements.source.value).data;
    const keys = SLIDE_EDITOR_CONFIG.frontmatter.keys;

    return {
      layout: normalizeToken(data[keys.layout], allowed.layout, defaults.layout),
      align: normalizeToken(data[keys.align], allowed.align, defaults.align),
      density: normalizeToken(data[keys.density], allowed.density, defaults.density),
      accent: data[keys.accent] ?? defaults.accent,
    };
  }

  function updateInspector(): void {
    const settings = readLayoutSettings();
    pressed(elements.layoutButtons, "data-layout-value", settings.layout);
    pressed(elements.alignButtons, "data-align-value", settings.align);
    pressed(elements.densityButtons, "data-density-value", settings.density);
    pressed(elements.accentButtons, "data-accent-value", settings.accent);
    if (settings.accent && /^#[\da-f]{6}$/i.test(settings.accent)) {
      elements.accentCustom.value = settings.accent;
    }
  }

  function clearPreviewEditor(): void {
    cleanupPreviewEditor?.();
    cleanupPreviewEditor = undefined;
  }

  function readCanvasPlacements(): SlidePlacement[] {
    return parsePlacements(
      parseFrontmatter(elements.source.value).data[SLIDE_EDITOR_CONFIG.frontmatter.keys.placements],
    );
  }

  function getPreviewLayout(): HTMLElement | undefined {
    try {
      return (
        elements.preview.contentDocument?.querySelector<HTMLElement>(
          `.${SLIDE_EDITOR_CONFIG.dom.layoutClass}`,
        ) ?? undefined
      );
    } catch {
      return undefined;
    }
  }

  function measurePreviewPlacements(): SlidePlacement[] {
    const layout = getPreviewLayout();
    if (!layout) return [];
    const children = Array.from(layout.children);
    const layoutRect = layout.getBoundingClientRect();
    if (children.length === 0) return [];
    if (layoutRect.width <= 0 || layoutRect.height <= 0) {
      return ensurePlacementCount([], children.length);
    }

    return children.map((child, index) => {
      const rect = child.getBoundingClientRect();
      return normalizePlacement(
        {
          x: ((rect.left - layoutRect.left) / layoutRect.width) * 100,
          y: ((rect.top - layoutRect.top) / layoutRect.height) * 100,
          w: (rect.width / layoutRect.width) * 100,
          h: (rect.height / layoutRect.height) * 100,
        },
        defaultPlacement(index, children.length),
      );
    });
  }

  function syncCanvasEditor(): void {
    clearPreviewEditor();
    if (readLayoutSettings().layout !== SLIDE_EDITOR_CONFIG.canvas.layout) return;

    const doc = elements.preview.contentDocument;
    const layout = getPreviewLayout();
    if (!doc || !layout) return;

    const placements = ensurePlacementCount(readCanvasPlacements(), layout.children.length);
    applyCanvasLayoutStyles(layout, placements);
    cleanupPreviewEditor = installCanvasEditorOverlay(doc, layout, placements, (nextPlacements) => {
      applyFrontmatterUpdate({
        [SLIDE_EDITOR_CONFIG.frontmatter.keys.layout]: SLIDE_EDITOR_CONFIG.canvas.layout,
        [SLIDE_EDITOR_CONFIG.frontmatter.keys.placements]: formatPlacements(nextPlacements),
      });
    });
  }

  async function save(): Promise<void> {
    if (!state.selected) return;
    clearTimeout(state.saveTimer);
    elements.status.textContent = "Saving";
    const result = await request<{ filePath: string }>("/source", {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ filePath: state.selected.filePath, source: elements.source.value }),
    });
    await loadDecks(result.filePath);
    if (state.selected) {
      elements.preview.src = `${state.selected.href}?editor=${Date.now()}`;
    }
  }

  function scheduleSave(): void {
    clearTimeout(state.saveTimer);
    elements.status.textContent = "Saving";
    state.saveTimer = window.setTimeout(() => {
      save().catch((error: unknown) => {
        elements.status.textContent = error instanceof Error ? error.message : String(error);
      });
    }, 260);
  }

  function applyFrontmatterUpdate(update: Record<string, string | undefined>): void {
    elements.source.value = writeFrontmatter(elements.source.value, update);
    updateInspector();
    setDirty(true);
    scheduleSave();
  }

  function renderDecks(): void {
    elements.decks.replaceChildren(
      ...state.decks.map((deck) => {
        const section = document.createElement("section");
        section.className = "deck";
        const title = document.createElement("p");
        title.className = "deck-name";
        title.textContent = deck.title;
        section.append(title);

        for (const slide of deck.slides) {
          const button = document.createElement("button");
          button.type = "button";
          button.className = "slide-button";
          button.dataset.slideFile = slide.filePath;
          button.setAttribute("aria-current", String(state.selected?.filePath === slide.filePath));
          button.innerHTML = '<span class="slide-number"></span><span class="slide-name"></span>';
          queryElementFrom<HTMLSpanElement>(button, ".slide-number").textContent = String(
            slide.slideNumber,
          );
          queryElementFrom<HTMLSpanElement>(button, ".slide-name").textContent = slide.title;
          button.addEventListener("click", () => {
            void selectSlide(slide.filePath);
          });
          section.append(button);
        }

        return section;
      }),
    );
  }

  async function loadDecks(preferredFile?: string): Promise<void> {
    state.decks = (await request<{ decks: EditorDeck[] }>("/decks")).decks;
    renderDecks();
    const first = state.decks.flatMap((deck) => deck.slides)[0];
    const target = preferredFile ?? state.selected?.filePath ?? first?.filePath;
    if (target) await selectSlide(target, true);
  }

  async function selectSlide(filePath: string, force = false): Promise<void> {
    if (!force && state.dirty && !confirm("Discard unsaved changes?")) return;
    const data = await request<{ slide: EditorSlide; source: string }>(
      `/source?file=${encodeURIComponent(filePath)}`,
    );
    state.selected = data.slide;
    elements.source.value = data.source;
    elements.file.textContent = data.slide.filePath;
    elements.open.href = data.slide.href;
    elements.presenter.href = data.slide.presenterHref ?? data.slide.href;
    clearPreviewEditor();
    elements.preview.src = `${data.slide.href}?editor=${Date.now()}`;
    updateInspector();
    setDirty(false);
    renderDecks();
  }

  async function addSlide(): Promise<void> {
    const result = await request<{ filePath: string }>("/slides", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ after: state.selected?.filePath }),
    });
    await loadDecks(result.filePath);
  }

  elements.source.addEventListener("input", () => {
    setDirty(true);
    updateInspector();
  });
  elements.save.addEventListener("click", () => {
    void save();
  });
  elements.add.addEventListener("click", () => {
    void addSlide();
  });
  for (const button of elements.layoutButtons) {
    button.addEventListener("click", () => {
      const layout = button.dataset.layoutValue;
      if (!layout) return;
      if (layout === SLIDE_EDITOR_CONFIG.canvas.layout) {
        const placements = measurePreviewPlacements();
        applyFrontmatterUpdate({
          [SLIDE_EDITOR_CONFIG.frontmatter.keys.layout]: layout,
          [SLIDE_EDITOR_CONFIG.frontmatter.keys.placements]:
            placements.length > 0 ? formatPlacements(placements) : undefined,
        });
        return;
      }
      applyFrontmatterUpdate({ [SLIDE_EDITOR_CONFIG.frontmatter.keys.layout]: layout });
    });
  }
  for (const button of elements.alignButtons) {
    button.addEventListener("click", () => {
      applyFrontmatterUpdate({
        [SLIDE_EDITOR_CONFIG.frontmatter.keys.align]: button.dataset.alignValue,
      });
    });
  }
  for (const button of elements.densityButtons) {
    button.addEventListener("click", () => {
      applyFrontmatterUpdate({
        [SLIDE_EDITOR_CONFIG.frontmatter.keys.density]: button.dataset.densityValue,
      });
    });
  }
  for (const button of elements.accentButtons) {
    button.addEventListener("click", () => {
      applyFrontmatterUpdate({
        [SLIDE_EDITOR_CONFIG.frontmatter.keys.accent]: button.dataset.accentValue,
      });
    });
  }
  elements.accentCustom.addEventListener("input", () => {
    applyFrontmatterUpdate({
      [SLIDE_EDITOR_CONFIG.frontmatter.keys.accent]: elements.accentCustom.value,
    });
  });
  elements.preview.addEventListener("load", () => {
    syncCanvasEditor();
  });
  window.addEventListener("beforeunload", (event) => {
    if (!state.dirty) return;
    event.preventDefault();
    event.returnValue = "";
  });
  loadDecks().catch((error: unknown) => {
    elements.status.textContent = error instanceof Error ? error.message : String(error);
  });
}

function queryElementFrom<T extends Element>(root: ParentNode, selector: string): T {
  const element = root.querySelector<T>(selector);
  if (!element) {
    throw new Error(`Missing editor child element: ${selector}`);
  }
  return element;
}

/**
 * Serializes the type-checked editor client into a dev-server inline module.
 */
export function renderSlideEditorClientSource(apiJson: string): string {
  return [
    `globalThis.__OX_SLIDE_EDITOR_API__ = ${apiJson};`,
    `const SLIDE_EDITOR_CONFIG = ${JSON.stringify(SLIDE_EDITOR_CONFIG)};`,
    queryElement.toString(),
    parseFrontmatter.toString(),
    formatFrontmatterValue.toString(),
    writeFrontmatter.toString(),
    pressed.toString(),
    normalizeToken.toString(),
    clampNumber.toString(),
    roundedPercent.toString(),
    finiteNumber.toString(),
    normalizePlacement.toString(),
    defaultPlacement.toString(),
    isPlacement.toString(),
    parsePlacements.toString(),
    formatPlacements.toString(),
    softSnapPercent.toString(),
    snapPlacementToGrid.toString(),
    isHtmlElement.toString(),
    ensurePlacementCount.toString(),
    applyElementPlacement.toString(),
    applyCanvasLayoutStyles.toString(),
    renderCanvasEditorStyle.toString(),
    installCanvasEditorOverlay.toString(),
    queryElementFrom.toString(),
    `(${createSlideEditorClient.toString()})();`,
  ].join("\n");
}
