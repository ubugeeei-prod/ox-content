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
    accent: "#2f5d50",
    accentSoft: "rgba(47, 93, 80, 0.12)",
    code: "#ecefea",
    primary: "#24483f",
    primaryText: "#fff",
  },
  dark: {
    bg: "#101311",
    panel: "#171b18",
    line: "rgba(237, 241, 235, 0.12)",
    lineStrong: "rgba(237, 241, 235, 0.24)",
    text: "#edf1eb",
    muted: "#9fa79f",
    accent: "#8fb8aa",
    accentSoft: "rgba(143, 184, 170, 0.14)",
    code: "#1e231f",
    primary: "#8fb8aa",
    primaryText: "#101311",
  },
} as const;

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

function renderSegmentedOptions(
  kind: "layout" | "align" | "density",
  options: readonly SlideChoice<string>[],
): string {
  return options
    .map(
      (option) =>
        `<button class="segment" type="button" data-${kind}-value="${escapeHtmlAttribute(option.value)}" title="${escapeHtmlAttribute(option.title)}">${option.label}</button>`,
    )
    .join("\n              ");
}

function renderAccentControls(): string {
  const swatches = SLIDE_ACCENT_OPTIONS.map(
    (option) =>
      `<button class="swatch" type="button" data-accent-value="${escapeHtmlAttribute(option.value)}" style="--swatch: ${escapeHtmlAttribute(option.value)}" title="${escapeHtmlAttribute(option.title)}"></button>`,
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
        min-width: 0;
        min-height: 34px;
        padding: 0 8px;
        border: 0;
        border-right: 1px solid var(--line);
        background: transparent;
        color: var(--muted);
        cursor: pointer;
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
        gap: 8px;
        min-width: 0;
        flex-wrap: wrap;
      }
      .swatch {
        width: 32px;
        height: 32px;
        border: 1px solid var(--line-strong);
        border-radius: 4px;
        background: var(--swatch);
        cursor: pointer;
      }
      .swatch[aria-pressed="true"] {
        outline: 2px solid var(--accent);
        outline-offset: 2px;
      }
      .color-input {
        width: 42px;
        height: 34px;
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
          <button class="button" type="button" data-new>
            <svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><path d="M12 5v14M5 12h14"/></svg>
            <span>New</span>
          </button>
        </header>
        <div class="deck-list" data-decks></div>
      </aside>
      <section class="editor">
        <header class="bar">
          <div class="title" data-current-file>Editor</div>
          <span class="status" data-status>Clean</span>
          <div class="spacer"></div>
          <button class="button" type="button" data-save data-primary="true">
            <svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2Z"/><path d="M17 21v-8H7v8M7 3v5h8"/></svg>
            <span>Save</span>
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
          <a class="button" data-open target="_blank" rel="noreferrer">Open</a>
          <a class="button" data-presenter target="_blank" rel="noreferrer">Presenter</a>
        </header>
        <iframe data-preview title="Slide preview"></iframe>
      </section>
    </div>
    <script type="module">${clientSource}</script>
  </body>
</html>`;
}
