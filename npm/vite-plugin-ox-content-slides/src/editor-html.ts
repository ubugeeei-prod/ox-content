import { renderSlideEditorClientSource } from "./editor-client";
import {
  SLIDE_ACCENT_OPTIONS,
  SLIDE_ALIGN_OPTIONS,
  SLIDE_DENSITY_OPTIONS,
  SLIDE_LAYOUT_OPTIONS,
  type SlideChoice,
} from "./slide-schema";

const EDITOR_UI_THEME = {
  light: {
    bg: "#f3f4f1",
    panel: "#fbfbf8",
    line: "rgba(23, 31, 28, 0.11)",
    lineStrong: "rgba(23, 31, 28, 0.24)",
    text: "#17201e",
    muted: "#6a716b",
    accent: "#1f1f1f",
    accentSoft: "rgba(17, 17, 17, 0.08)",
    code: "#ecefea",
    primary: "#1f1f1f",
    primaryText: "#fff",
  },
  dark: {
    bg: "#101311",
    panel: "#171b18",
    line: "rgba(237, 241, 235, 0.12)",
    lineStrong: "rgba(237, 241, 235, 0.24)",
    text: "#edf1eb",
    muted: "#9fa79f",
    accent: "#e5e5e5",
    accentSoft: "rgba(255, 255, 255, 0.1)",
    code: "#1e231f",
    primary: "#e5e5e5",
    primaryText: "#101311",
  },
} as const;

const EDITOR_ICONS = {
  alignCenter: '<path d="M5 6h14M8 12h8M6 18h12"/>',
  alignEnd: '<path d="M5 6h14M9 12h10M7 18h12"/>',
  alignStart: '<path d="M5 6h14M5 12h10M5 18h12"/>',
  canvas: '<rect x="5" y="5" width="14" height="14" rx="2"/><path d="M9 9h3M9 13h6"/>',
  code: '<path d="m9 8-4 4 4 4M15 8l4 4-4 4"/>',
  densityBalanced: '<path d="M6 7h12M6 12h12M6 17h12"/>',
  densityCompact: '<path d="M6 9h12M6 12h12M6 15h12"/>',
  densitySpacious: '<path d="M6 6h12M6 12h12M6 18h12"/>',
  external:
    '<path d="M14 5h5v5"/><path d="m13 11 6-6"/><path d="M19 14v4a1 1 0 0 1-1 1H6a1 1 0 0 1-1-1V6a1 1 0 0 1 1-1h4"/>',
  plus: '<path d="M12 5v14M5 12h14"/>',
  presenter: '<rect x="3" y="4" width="18" height="12" rx="2"/><path d="M8 20h8M12 16v4"/>',
  quote: '<path d="M8 11h3v6H5v-5c0-3 1.5-5 4-6M18 11h3v6h-6v-5c0-3 1.5-5 4-6"/>',
  save: '<path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2Z"/><path d="M17 21v-8H7v8M7 3v5h8"/>',
  split: '<rect x="4" y="5" width="16" height="14" rx="2"/><path d="M12 5v14"/>',
  stack:
    '<rect x="5" y="6" width="14" height="4" rx="1"/><rect x="5" y="14" width="14" height="4" rx="1"/>',
  statement: '<path d="M7 8h10M9 12h6M10 16h4"/>',
} as const;

type EditorIconName = keyof typeof EDITOR_ICONS;

const SEGMENT_ICONS: Record<string, EditorIconName> = {
  "align:center": "alignCenter",
  "align:end": "alignEnd",
  "align:start": "alignStart",
  "density:balanced": "densityBalanced",
  "density:compact": "densityCompact",
  "density:spacious": "densitySpacious",
  "layout:canvas": "canvas",
  "layout:code": "code",
  "layout:quote": "quote",
  "layout:split": "split",
  "layout:stack": "stack",
  "layout:statement": "statement",
};

function cssVarName(key: string): string {
  return key.replace(/[A-Z]/g, (letter) => `-${letter.toLowerCase()}`);
}

function renderEditorThemeVars(tokens: Record<string, string>): string {
  return Object.entries(tokens)
    .map(([key, value]) => `        --${cssVarName(key)}: ${value};`)
    .join("\n");
}

function escapeHtmlAttribute(value: string): string {
  return value
    .replace(/&/g, "&amp;")
    .replace(/"/g, "&quot;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}

function renderIcon(name: EditorIconName, className = "icon"): string {
  return `<svg class="${className}" viewBox="0 0 24 24" aria-hidden="true">${EDITOR_ICONS[name]}</svg>`;
}

function renderSegmentedOptions(
  kind: "layout" | "align" | "density",
  options: readonly SlideChoice<string>[],
): string {
  return options
    .map((option) => {
      const icon = SEGMENT_ICONS[`${kind}:${option.value}`];
      const iconHtml = icon ? renderIcon(icon, "segment-icon") : "";

      return `<button class="segment" type="button" data-${kind}-value="${escapeHtmlAttribute(option.value)}" title="${escapeHtmlAttribute(option.title)}" aria-label="${escapeHtmlAttribute(option.title)}">${iconHtml}<span class="segment-label sr-only">${option.label}</span></button>`;
    })
    .join("\n              ");
}

function renderAccentControls(): string {
  const swatches = SLIDE_ACCENT_OPTIONS.map(
    (option) =>
      `<button class="swatch" type="button" data-accent-value="${escapeHtmlAttribute(option.value)}" style="--swatch: ${escapeHtmlAttribute(option.value)}; --swatch-foreground: ${escapeHtmlAttribute(option.foreground)}" title="${escapeHtmlAttribute(option.title)}" aria-label="${escapeHtmlAttribute(option.title)}"></button>`,
  ).join("\n              ");
  const defaultAccent = SLIDE_ACCENT_OPTIONS[0].value;

  return `${swatches}
              <input class="color-input" type="color" data-accent-custom value="${escapeHtmlAttribute(defaultAccent)}" title="Custom accent" />`;
}

/**
 * Renders the dev-only browser editor shell.
 */
export function renderSlideEditorHtml(apiPrefix: string): string {
  const apiJson = JSON.stringify(apiPrefix);
  const clientSource = renderSlideEditorClientSource(apiJson);

  return `<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Slide Editor</title>
    <style>
      :root {
        color-scheme: light dark;
${renderEditorThemeVars(EDITOR_UI_THEME.light)}
      }
      @media (prefers-color-scheme: dark) {
        :root {
${renderEditorThemeVars(EDITOR_UI_THEME.dark)}
        }
      }
      * { box-sizing: border-box; }
      html, body { margin: 0; min-height: 100%; }
      body {
        min-height: 100vh;
        background: var(--bg);
        color: var(--text);
        font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
      }
      button, textarea, input {
        color: inherit;
        font: inherit;
      }
      .sr-only {
        position: absolute;
        width: 1px;
        height: 1px;
        padding: 0;
        margin: -1px;
        overflow: hidden;
        clip: rect(0, 0, 0, 0);
        white-space: nowrap;
        border: 0;
      }
      .app {
        display: grid;
        grid-template-columns: 250px minmax(320px, 0.78fr) minmax(380px, 1.22fr);
        min-height: 100vh;
      }
      .sidebar, .editor, .preview {
        min-width: 0;
        min-height: 100vh;
        border-right: 1px solid var(--line);
      }
      .sidebar {
        display: grid;
        grid-template-rows: auto 1fr;
      }
      .editor {
        display: grid;
        grid-template-rows: auto auto minmax(0, 1fr);
      }
      .bar {
        display: flex;
        align-items: center;
        gap: 8px;
        min-height: 58px;
        padding: 10px 12px;
        border-bottom: 1px solid var(--line);
        background: color-mix(in srgb, var(--panel) 94%, var(--bg));
      }
      .title {
        min-width: 0;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
        font-weight: 700;
      }
      .spacer { flex: 1; }
      .deck-list, .editor-body {
        overflow: auto;
      }
      .inspector {
        display: grid;
        gap: 10px;
        padding: 12px;
        border-bottom: 1px solid var(--line);
        background: color-mix(in srgb, var(--panel) 72%, var(--bg));
      }
      .control-row {
        display: grid;
        grid-template-columns: 72px minmax(0, 1fr);
        align-items: center;
        gap: 10px;
      }
      .control-label {
        color: var(--muted);
        font-size: 0.78rem;
        font-weight: 700;
        text-transform: uppercase;
      }
      .segmented {
        display: grid;
        grid-auto-flow: column;
        grid-auto-columns: 1fr;
        min-width: 0;
        border: 1px solid var(--line);
        border-radius: 4px;
        overflow: hidden;
        background: var(--panel);
      }
      .segment {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        gap: 6px;
        min-width: 0;
        min-height: 34px;
        padding: 0;
        border: 0;
        border-right: 1px solid var(--line);
        background: transparent;
        color: var(--muted);
        cursor: pointer;
      }
      .segment-icon {
        width: 14px;
        height: 14px;
        flex: 0 0 auto;
        stroke: currentColor;
        fill: none;
        stroke-width: 1.8;
        stroke-linecap: round;
        stroke-linejoin: round;
      }
      .segment:last-child { border-right: 0; }
      .segment[aria-pressed="true"] {
        background: var(--accent-soft);
        color: var(--text);
        box-shadow: inset 0 0 0 1px var(--accent);
      }
      .swatches {
        display: flex;
        align-items: center;
        gap: 7px;
        min-width: 0;
        flex-wrap: wrap;
      }
      .swatch {
        position: relative;
        width: 28px;
        height: 28px;
        border: 1px solid var(--line-strong);
        border-radius: 4px;
        background: var(--swatch);
        box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.16);
        cursor: pointer;
      }
      .swatch:hover {
        transform: translateY(-1px);
      }
      .swatch[aria-pressed="true"] {
        border-color: color-mix(in srgb, var(--swatch) 80%, var(--text));
        box-shadow:
          inset 0 0 0 1px rgba(255, 255, 255, 0.2),
          0 0 0 2px var(--panel),
          0 0 0 4px color-mix(in srgb, var(--swatch) 72%, var(--text));
      }
      .swatch[aria-pressed="true"]::after {
        position: absolute;
        left: 50%;
        top: 50%;
        width: 9px;
        height: 5px;
        border-left: 2px solid var(--swatch-foreground);
        border-bottom: 2px solid var(--swatch-foreground);
        content: "";
        transform: translate(-50%, -62%) rotate(-45deg);
      }
      .color-input {
        width: 42px;
        height: 28px;
        padding: 0;
        border: 1px solid var(--line-strong);
        border-radius: 4px;
        background: var(--panel);
        cursor: pointer;
      }
      .deck {
        padding: 10px;
        border-bottom: 1px solid var(--line);
      }
      .deck-name {
        margin: 0 0 8px;
        color: var(--muted);
        font-size: 0.78rem;
        font-weight: 700;
        text-transform: uppercase;
      }
      .slide-button {
        width: 100%;
        display: grid;
        grid-template-columns: 34px minmax(0, 1fr);
        align-items: center;
        gap: 8px;
        min-height: 42px;
        padding: 6px 8px;
        border: 1px solid transparent;
        background: transparent;
        border-radius: 4px;
        text-align: left;
        cursor: pointer;
      }
      .slide-button:hover, .slide-button[aria-current="true"] {
        border-color: var(--line-strong);
        background: var(--panel);
      }
      .slide-number {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        width: 30px;
        height: 30px;
        border: 1px solid var(--line);
        border-radius: 4px;
        color: var(--muted);
        font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
        font-size: 0.78rem;
      }
      .slide-name {
        min-width: 0;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
      }
      .button {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        gap: 7px;
        min-height: 36px;
        padding: 0 11px;
        border: 1px solid var(--line-strong);
        border-radius: 4px;
        background: var(--panel);
        text-decoration: none;
        cursor: pointer;
      }
      .button[data-primary="true"] {
        border-color: transparent;
        background: var(--primary);
        color: var(--primary-text);
      }
      .button[data-icon-only="true"] {
        width: 36px;
        padding: 0;
        gap: 0;
      }
      .icon {
        width: 16px;
        height: 16px;
        stroke: currentColor;
        fill: none;
        stroke-width: 1.8;
        stroke-linecap: round;
        stroke-linejoin: round;
      }
      textarea {
        width: 100%;
        min-height: 100%;
        height: 100%;
        padding: 16px;
        border: 0;
        outline: 0;
        resize: none;
        background: var(--code);
        color: var(--text);
        font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
        font-size: 0.94rem;
        line-height: 1.58;
      }
      .preview {
        display: grid;
        grid-template-rows: auto 1fr;
        border-right: 0;
      }
      iframe {
        width: 100%;
        height: 100%;
        border: 0;
        background: var(--panel);
      }
      .status {
        color: var(--muted);
        font-size: 0.88rem;
      }
      @media (max-width: 1100px) {
        .app {
          grid-template-columns: 220px minmax(0, 1fr);
          grid-template-rows: auto minmax(0, 1fr);
        }
        .sidebar { grid-row: 1 / span 2; }
        .editor {
          grid-column: 2;
          min-height: auto;
          border-right: 0;
        }
        .editor-body { display: none; }
        .preview {
          grid-column: 2;
          min-height: 0;
          border-top: 1px solid var(--line);
        }
      }
      @media (max-width: 760px) {
        .app { grid-template-columns: 1fr; }
        .sidebar, .editor, .preview {
          grid-column: auto;
          grid-row: auto;
          min-height: auto;
        }
        .preview { min-height: 70vh; }
        .control-row { grid-template-columns: 1fr; }
      }
    </style>
  </head>
  <body>
    <div class="app">
      <aside class="sidebar">
        <header class="bar">
          <div class="title">Slides</div>
          <button class="button" type="button" data-new data-icon-only="true" title="New slide" aria-label="New slide">
            ${renderIcon("plus")}
            <span class="sr-only">New</span>
          </button>
        </header>
        <div class="deck-list" data-decks></div>
      </aside>
      <section class="editor">
        <header class="bar">
          <div class="title" data-current-file>Editor</div>
          <span class="status" data-status>Clean</span>
          <div class="spacer"></div>
          <button class="button" type="button" data-save data-primary="true" data-icon-only="true" title="Save" aria-label="Save">
            ${renderIcon("save")}
            <span class="sr-only">Save</span>
          </button>
        </header>
        <section class="inspector" data-inspector>
          <div class="control-row">
            <div class="control-label">Layout</div>
            <div class="segmented" data-layout-group>
              ${renderSegmentedOptions("layout", SLIDE_LAYOUT_OPTIONS)}
            </div>
          </div>
          <div class="control-row">
            <div class="control-label">Align</div>
            <div class="segmented" data-align-group>
              ${renderSegmentedOptions("align", SLIDE_ALIGN_OPTIONS)}
            </div>
          </div>
          <div class="control-row">
            <div class="control-label">Density</div>
            <div class="segmented" data-density-group>
              ${renderSegmentedOptions("density", SLIDE_DENSITY_OPTIONS)}
            </div>
          </div>
          <div class="control-row">
            <div class="control-label">Accent</div>
            <div class="swatches">
              ${renderAccentControls()}
            </div>
          </div>
        </section>
        <main class="editor-body">
          <textarea data-source spellcheck="false"></textarea>
        </main>
      </section>
      <section class="preview">
        <header class="bar">
          <div class="title">Preview</div>
          <div class="spacer"></div>
          <a class="button" data-open data-icon-only="true" target="_blank" rel="noreferrer" title="Open" aria-label="Open">${renderIcon("external")}<span class="sr-only">Open</span></a>
          <a class="button" data-presenter data-icon-only="true" target="_blank" rel="noreferrer" title="Presenter" aria-label="Presenter">${renderIcon("presenter")}<span class="sr-only">Presenter</span></a>
        </header>
        <iframe data-preview title="Slide preview"></iframe>
      </section>
    </div>
    <script type="module">${clientSource}</script>
  </body>
</html>`;
}
