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
  layout: string;
  align: string;
  density: string;
  accent: string;
}

interface SlidePlacement {
  x: number;
  y: number;
  w: number;
  h: number;
}

type CanvasDragMode = "move" | "resize";

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

  const preferred = ["title", "description", "layout", "align", "density", "accent", "placements"];
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

function normalizeToken(
  value: string | undefined,
  allowed: ReadonlySet<string>,
  fallback: string,
): string {
  const normalized = value?.toLowerCase();
  return normalized && allowed.has(normalized) ? normalized : fallback;
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
  const w = clampNumber(finiteNumber(value.w) ?? fallback.w, 5, 100);
  const h = clampNumber(finiteNumber(value.h) ?? fallback.h, 5, 100);
  const maxX = Math.max(0, 100 - w);
  const maxY = Math.max(0, 100 - h);

  return {
    x: roundedPercent(clampNumber(finiteNumber(value.x) ?? fallback.x, 0, maxX)),
    y: roundedPercent(clampNumber(finiteNumber(value.y) ?? fallback.y, 0, maxY)),
    w: roundedPercent(w),
    h: roundedPercent(h),
  };
}

function defaultPlacement(index: number, count: number): SlidePlacement {
  const safeCount = Math.max(1, count);
  const columns = safeCount > 2 ? 2 : 1;
  const rows = Math.ceil(safeCount / columns);
  const column = index % columns;
  const row = Math.floor(index / columns);
  const gap = 4;
  const width = 88 / columns - gap;
  const height = 84 / rows - gap;

  return normalizePlacement(
    {
      x: 6 + column * (88 / columns),
      y: 8 + row * (84 / rows),
      w: width,
      h: height,
    },
    { x: 8, y: 8, w: 84, h: 18 },
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
  return `
    .ox-editor-canvas-overlay {
      position: fixed;
      z-index: 2147483647;
      pointer-events: auto;
    }
    .ox-editor-canvas-box {
      position: absolute;
      border: 2px solid #176b5d;
      background: rgba(23, 107, 93, 0.06);
      box-shadow: 0 0 0 1px rgba(255, 255, 255, 0.84), 0 10px 28px rgba(10, 18, 28, 0.18);
      cursor: move;
      pointer-events: auto;
      touch-action: none;
    }
    .ox-editor-canvas-box[data-selected="true"] {
      border-color: #d15a2f;
      background: rgba(209, 90, 47, 0.08);
    }
    .ox-editor-canvas-label {
      position: absolute;
      left: -2px;
      top: -28px;
      min-width: 26px;
      height: 24px;
      display: inline-flex;
      align-items: center;
      justify-content: center;
      border-radius: 5px 5px 0 0;
      background: #176b5d;
      color: #fff;
      font: 700 12px/1 system-ui, sans-serif;
    }
    .ox-editor-canvas-box[data-selected="true"] .ox-editor-canvas-label {
      background: #d15a2f;
    }
    .ox-editor-canvas-resize {
      position: absolute;
      right: -7px;
      bottom: -7px;
      width: 14px;
      height: 14px;
      border: 2px solid #fff;
      border-radius: 4px;
      background: #176b5d;
      cursor: nwse-resize;
      touch-action: none;
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

  const overlay = doc.createElement("div");
  overlay.className = "ox-editor-canvas-overlay";
  doc.body.append(overlay);

  let current = placements.map((placement) => ({ ...placement }));
  let drag: CanvasDragState | undefined;

  const boxes = current.map((placement, index) => {
    const box = doc.createElement("div");
    box.className = "ox-editor-canvas-box";
    box.dataset.index = String(index);
    box.style.left = `${placement.x}%`;
    box.style.top = `${placement.y}%`;
    box.style.width = `${placement.w}%`;
    box.style.height = `${placement.h}%`;

    const label = doc.createElement("span");
    label.className = "ox-editor-canvas-label";
    label.textContent = String(index + 1);

    const handle = doc.createElement("span");
    handle.className = "ox-editor-canvas-resize";
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
    const next =
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

    current[drag.index] = next;
    applyElementPlacement(drag.target, next);
    updateOverlayBounds();
  }

  function finishDrag(): void {
    if (!drag) return;
    doc.body.style.userSelect = "";
    drag = undefined;
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
  const defaults = { layout: "stack", align: "start", density: "balanced", accent: "" };
  const allowed = {
    layout: new Set(["stack", "statement", "split", "quote", "code", "canvas"]),
    align: new Set(["start", "center", "end"]),
    density: new Set(["compact", "balanced", "spacious"]),
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

    return {
      layout: normalizeToken(data.layout, allowed.layout, defaults.layout),
      align: normalizeToken(data.align, allowed.align, defaults.align),
      density: normalizeToken(data.density, allowed.density, defaults.density),
      accent: data.accent ?? defaults.accent,
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
    return parsePlacements(parseFrontmatter(elements.source.value).data.placements);
  }

  function getPreviewLayout(): HTMLElement | undefined {
    try {
      return (
        elements.preview.contentDocument?.querySelector<HTMLElement>(".ox-slide-layout") ??
        undefined
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
    if (readLayoutSettings().layout !== "canvas") return;

    const doc = elements.preview.contentDocument;
    const layout = getPreviewLayout();
    if (!doc || !layout) return;

    const placements = ensurePlacementCount(readCanvasPlacements(), layout.children.length);
    applyCanvasLayoutStyles(layout, placements);
    cleanupPreviewEditor = installCanvasEditorOverlay(doc, layout, placements, (nextPlacements) => {
      applyFrontmatterUpdate({
        layout: "canvas",
        placements: formatPlacements(nextPlacements),
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
      if (layout === "canvas") {
        const placements = measurePreviewPlacements();
        applyFrontmatterUpdate({
          layout,
          placements: placements.length > 0 ? formatPlacements(placements) : undefined,
        });
        return;
      }
      applyFrontmatterUpdate({ layout });
    });
  }
  for (const button of elements.alignButtons) {
    button.addEventListener("click", () => {
      applyFrontmatterUpdate({ align: button.dataset.alignValue });
    });
  }
  for (const button of elements.densityButtons) {
    button.addEventListener("click", () => {
      applyFrontmatterUpdate({ density: button.dataset.densityValue });
    });
  }
  for (const button of elements.accentButtons) {
    button.addEventListener("click", () => {
      applyFrontmatterUpdate({ accent: button.dataset.accentValue });
    });
  }
  elements.accentCustom.addEventListener("input", () => {
    applyFrontmatterUpdate({ accent: elements.accentCustom.value });
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
