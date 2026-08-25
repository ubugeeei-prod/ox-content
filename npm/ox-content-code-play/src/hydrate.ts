import { createCodePlay, type CodePlayClient } from "./client";
import { readPlayPayload, runPlayAction } from "./hydrate-action";
import { decodePayload, encodePayload } from "./payload";
import { errorMessage, errorResult } from "./result";
import { CODE_PLAY_STYLES } from "./styles";
import { JS_SANDBOX_FLAGS } from "./javascript-sandbox";
import { applyActionBusy, renderPlayUi } from "./ui";
import type { PlayPayload, RunResult } from "./types";
import {
  renderDiagnosticsHtml,
  renderProvenanceHtml,
  renderStderrHtml,
  renderStdioHtml,
  renderTimingHtml,
} from "./viewers";

export interface HydrateOptions {
  client?: CodePlayClient;
  root?: ParentNode;
}

const STYLE_ID = "ox-code-play-styles";

export function hydrateCodePlay(
  root: ParentNode = defaultRoot(),
  options: HydrateOptions = {},
): void {
  ensureStyles();
  const client = options.client ?? createCodePlayFromPayloads(root);
  for (const element of queryWidgets(root)) {
    try {
      mountCodePlay(element, { client });
    } catch {
      // One broken widget must not abort the rest of the page.
    }
  }
}

export function mountCodePlay(element: Element, options: { client?: CodePlayClient } = {}): void {
  if (!(element instanceof HTMLElement) || element.dataset.oxCodePlayMounted === "true") {
    return;
  }
  const payload = readPlayPayload(element.getAttribute("data-ox-code-play") ?? "");
  if (!payload || payload.ui === "headless") {
    return;
  }
  const client =
    options.client ??
    createCodePlay({
      languages: { [payload.language]: true },
      endpoints: payload.endpoints,
    });
  const source = element.innerHTML;
  const ui = document.createElement("div");
  ui.innerHTML = renderPlayUi({ payload });
  const widget = ui.firstElementChild;
  if (!widget) {
    return;
  }
  const sourceSlot = widget.querySelector(".ox-code-play__source");
  if (sourceSlot) {
    sourceSlot.innerHTML = source;
  }
  element.replaceChildren(widget);
  element.dataset.oxCodePlayMounted = "true";
  bindWidget(element, payload, client);
}

function bindWidget(element: HTMLElement, payload: PlayPayload, client: CodePlayClient): void {
  let current = payload;
  const session = client.createSession({
    language: payload.language,
    code: payload.code,
    config: payload.config,
  });
  const runButton = element.querySelector<HTMLButtonElement>('[data-ox-action="run"]');
  const checkButton = element.querySelector<HTMLButtonElement>('[data-ox-action="typecheck"]');
  const cancelButton = element.querySelector<HTMLButtonElement>('[data-ox-action="cancel"]');
  runButton?.addEventListener("click", () => void run("execute"));
  checkButton?.addEventListener("click", () => void run("typecheck"));
  cancelButton?.addEventListener("click", () => session.cancel());
  element.addEventListener("click", (event) => {
    const target = event.target;
    if (!(target instanceof HTMLElement)) {
      return;
    }
    const panel = target.dataset.oxPanel;
    if (panel) {
      showPanel(element, panel);
    }
  });
  element.addEventListener("change", (event) => {
    const form = (event.target as HTMLElement | null)?.closest("form");
    if (!form) {
      return;
    }
    session.setConfig(readForm(form));
    current = { ...current, config: session.config };
  });

  async function run(action: "execute" | "typecheck"): Promise<void> {
    await runPlayAction({
      action: () => (action === "typecheck" ? session.typecheck() : session.run()),
      setBusy: (busy) => applyActionBusy(element, busy),
      onResult: (result) => paintResult(element, current, result),
      onError: (error) => paintResult(element, current, errorResult(errorMessage(error))),
    });
  }
}

function paintResult(element: HTMLElement, _payload: PlayPayload, result: RunResult): void {
  const stdio = element.querySelector('[data-panel="stdio"]');
  if (stdio) {
    stdio.innerHTML = `${renderDiagnosticsHtml(result)}${renderStdioHtml(result.stdio)}`;
    if (result.preview) {
      const frame = document.createElement("iframe");
      frame.setAttribute("sandbox", JS_SANDBOX_FLAGS);
      frame.srcdoc = result.preview.html;
      frame.title = "Code Play preview";
      frame.style.width = "100%";
      frame.style.minHeight = "12rem";
      frame.style.border = "0";
      stdio.append(frame);
    }
  }
  const provenance = element.querySelector('[data-panel="provenance"]');
  if (provenance) {
    provenance.innerHTML = renderProvenanceHtml(result.provenance);
  }
  const timing = element.querySelector('[data-panel="timing"]');
  if (timing) {
    timing.innerHTML = renderTimingHtml(result.timing);
  }
  const stderr = element.querySelector('[data-panel="stderr"]');
  if (stderr) {
    stderr.innerHTML = renderStderrHtml(result);
  }
  showPanel(element, resultPanelToShow(result, Boolean(stderr)));
}

export function resultPanelToShow(result: RunResult, hasStderrPanel: boolean): "stderr" | "stdio" {
  if (!hasStderrPanel) {
    return "stdio";
  }
  const hasErrorDiagnostics = result.diagnostics.some(
    (diagnostic) => diagnostic.severity === "error",
  );
  if (hasErrorDiagnostics || (result.status !== "ok" && Boolean(result.stderr))) {
    return "stderr";
  }
  return "stdio";
}

function showPanel(element: HTMLElement, panel: string): void {
  const compact = Boolean(element.querySelector(".ox-code-play--compact"));
  for (const tab of element.querySelectorAll("[data-ox-panel]")) {
    tab.setAttribute(
      "aria-selected",
      tab.getAttribute("data-ox-panel") === panel ? "true" : "false",
    );
  }
  for (const node of element.querySelectorAll<HTMLElement>(".ox-code-play__panel")) {
    const id = node.dataset.panel;
    if (compact && (id === "stdio" || id === "stderr")) {
      node.hidden = false;
      continue;
    }
    node.hidden = id !== panel;
  }
}

function readForm(form: HTMLFormElement): Record<string, unknown> {
  const data = new FormData(form);
  const config: Record<string, unknown> = {};
  for (const [key, value] of data.entries()) {
    config[key] = value === "on" ? true : value;
  }
  for (const input of form.querySelectorAll<HTMLInputElement>('input[type="checkbox"]')) {
    config[input.name] = input.checked;
  }
  return config;
}

function createCodePlayFromPayloads(root: ParentNode) {
  const languages: Record<string, true> = {};
  let endpoints;
  for (const element of queryWidgets(root)) {
    try {
      const payload = decodePayload(element.getAttribute("data-ox-code-play") ?? "");
      languages[payload.language] = true;
      endpoints = payload.endpoints ?? endpoints;
    } catch {
      // Ignore malformed widgets so one bad payload cannot block the page.
    }
  }
  return createCodePlay({ languages, endpoints });
}

function queryWidgets(root: ParentNode): HTMLElement[] {
  return [...root.querySelectorAll<HTMLElement>("[data-ox-code-play]")];
}

function ensureStyles(): void {
  if (typeof document === "undefined" || document.getElementById(STYLE_ID)) {
    return;
  }
  const style = document.createElement("style");
  style.id = STYLE_ID;
  style.textContent = CODE_PLAY_STYLES;
  document.head.append(style);
}

function defaultRoot(): ParentNode {
  if (typeof document === "undefined") {
    throw new Error("hydrateCodePlay() needs a DOM root.");
  }
  return document;
}

export { encodePayload, decodePayload };
