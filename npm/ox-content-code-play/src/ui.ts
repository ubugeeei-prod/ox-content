import { resolveLanguage } from "./catalog";
import { escapeHtml } from "./escape";
import { idleRunActionState } from "./hydrate-action";
import type {
  CodePlayPreset,
  LanguageDefinition,
  PlayPayload,
  RunActionState,
  RunResult,
} from "./types";
import {
  renderConfigHtml,
  renderDiagnosticsHtml,
  renderProvenanceHtml,
  renderStderrHtml,
  renderStdioHtml,
  renderTimingHtml,
} from "./viewers";

export interface UiState {
  payload: PlayPayload;
  result?: RunResult;
  busy?: boolean;
  runState?: RunActionState;
  panel?: "stdio" | "stderr" | "config" | "provenance" | "timing";
}

export function renderPlayUi(state: UiState): string {
  const definition = resolveLanguage(state.payload.language);
  const preset: CodePlayPreset = state.payload.ui === "headless" ? "headless" : state.payload.ui;
  if (preset === "headless") {
    return "";
  }
  const panel = state.panel ?? "stdio";
  const canTypecheck = state.payload.capabilities.typecheck;
  const runState =
    state.runState ??
    (state.busy ? { phase: "running" as const, action: "execute" as const } : idleRunActionState());
  const isBusy = Boolean(state.busy) || runState.phase === "running";
  const tabs = renderTabs(state, panel);
  const viewers = state.payload.viewers;
  const label = widgetLabel(state.payload, definition);
  return `<div class="ox-code-play ox-code-play--${preset}" data-ox-code-play-ui data-ox-run-state="${runState.phase}" role="region" aria-label="${escapeHtml(label)}" aria-busy="${isBusy ? "true" : "false"}">
  <div class="ox-code-play__toolbar">
    <span class="ox-code-play__summary"><span class="ox-code-play__lang">${escapeHtml(definition?.name ?? state.payload.language)}</span>${state.payload.title ? `<span class="ox-code-play__title">${escapeHtml(state.payload.title)}</span>` : ""}</span>
    <span class="ox-code-play__status" data-ox-status role="status" aria-live="polite">${renderRunStatusText(runState)}</span>
    <button type="button" data-ox-action="run" aria-label="${escapeHtml(`Run ${label}`)}"${actionButtonAttrs("run", Boolean(state.busy))}>Run</button>
    ${canTypecheck ? `<button type="button" data-ox-action="typecheck" aria-label="${escapeHtml(`Typecheck ${label}`)}"${actionButtonAttrs("typecheck", Boolean(state.busy))}>Typecheck</button>` : ""}
    <button type="button" data-ox-action="cancel" aria-label="${escapeHtml(`Cancel ${label}`)}"${actionButtonAttrs("cancel", Boolean(state.busy))}>Cancel</button>
  </div>
  ${renderRuntimeStrip(state.payload, definition)}
  <div class="ox-code-play__source"></div>
  ${tabs}
  ${viewers.stdio ? renderPanel("stdio", "Stdio", panel, preset, `${renderDiagnosticsHtml(state.result)}${renderStdioHtml(state.result?.stdio ?? [])}`) : ""}
  ${viewers.stderr ? renderPanel("stderr", "Stderr", panel, preset, renderStderrHtml(state.result)) : ""}
  ${viewers.config ? renderPanel("config", "Config", panel, preset, renderConfigHtml(definition?.configSchema ?? [], state.payload.config)) : ""}
  ${viewers.provenance ? renderPanel("provenance", "Provenance", panel, preset, renderProvenanceHtml(state.result?.provenance)) : ""}
  ${viewers.timing ? renderPanel("timing", "Timing", panel, preset, renderTimingHtml(state.result?.timing)) : ""}
</div>`;
}

export function renderRunStatusText(state: RunActionState): string {
  if (state.phase === "running") {
    return state.action === "typecheck" ? "Typechecking" : "Running";
  }
  if (state.phase === "offline") {
    return "Offline";
  }
  if (state.phase === "error") {
    if (state.result?.status === "timeout") {
      return "Timed out";
    }
    if (state.result?.status === "unsupported") {
      return "Unsupported";
    }
    return "Error";
  }
  if (state.phase === "result") {
    if (state.result?.status === "cancelled") {
      return "Cancelled";
    }
    return "Done";
  }
  return "Ready";
}

function renderRuntimeStrip(
  payload: PlayPayload,
  definition: LanguageDefinition | undefined,
): string {
  const runtime = runtimeLabel(payload, definition);
  const executor = executorLabel(payload, definition);
  const checks = payload.capabilities.typecheck ? "Typecheck ready" : "Run only";
  const chips = [
    runtimeChip("Runtime", runtime.label, runtime.kind),
    runtimeChip("Executor", executor.label, executor.kind),
    runtimeChip("Checks", checks, payload.capabilities.typecheck ? "ok" : "muted"),
  ].join("");
  return `<div class="ox-code-play__runtime" aria-label="Code Play runtime">${chips}</div>`;
}

function runtimeLabel(
  payload: PlayPayload,
  definition: LanguageDefinition | undefined,
): { label: string; kind: "ok" | "warn" | "muted" } {
  switch (definition?.backend) {
    case "javascript":
      return { label: "Browser sandbox", kind: "ok" };
    case "typescript":
      return { label: "TypeScript sandbox", kind: "ok" };
    case "framework":
      return { label: `${definition.name} iframe preview`, kind: "ok" };
    case "rust-playground":
      return { label: "Rust Playground", kind: payload.endpoints?.rust ? "ok" : "warn" };
    case "go-playground":
      return { label: "Go Playground", kind: payload.endpoints?.go ? "ok" : "warn" };
    case "remote":
      return payload.endpoint
        ? { label: "Piston-compatible", kind: "ok" }
        : { label: "Endpoint missing", kind: "warn" };
    default:
      return { label: definition?.name ?? payload.language, kind: "muted" };
  }
}

function executorLabel(
  payload: PlayPayload,
  definition: LanguageDefinition | undefined,
): { label: string; kind: "ok" | "warn" | "muted" } {
  if (!payload.capabilities.execute) {
    return { label: "Disabled", kind: "warn" };
  }
  if (definition?.backend === "remote" && !payload.endpoint) {
    return { label: "Configure endpoint", kind: "warn" };
  }
  return { label: "On demand", kind: "ok" };
}

function runtimeChip(label: string, value: string, kind: "ok" | "warn" | "muted"): string {
  return `<span class="ox-code-play__runtime-chip ox-code-play__runtime-chip--${kind}"><span>${escapeHtml(label)}</span><strong>${escapeHtml(value)}</strong></span>`;
}

function renderTabs(state: UiState, panel: string): string {
  if (state.payload.ui === "compact") {
    return "";
  }
  const viewers = state.payload.viewers;
  const buttons = [
    viewers.stdio ? tab("stdio", "Stdio", panel) : "",
    viewers.stderr ? tab("stderr", "Stderr", panel) : "",
    viewers.config ? tab("config", "Config", panel) : "",
    viewers.provenance ? tab("provenance", "Provenance", panel) : "",
    viewers.timing ? tab("timing", "Timing", panel) : "",
  ]
    .filter(Boolean)
    .join("");
  return buttons ? `<div class="ox-code-play__tabs" role="tablist">${buttons}</div>` : "";
}

function tab(id: string, label: string, selected: string): string {
  const isSelected = selected === id;
  return `<button type="button" role="tab" data-ox-panel="${id}" aria-selected="${isSelected ? "true" : "false"}" tabindex="${isSelected ? "0" : "-1"}">${label}</button>`;
}

export function actionButtonAttrs(action: string, busy: boolean): string {
  const state = actionBusyState(action, busy);
  return `${state.disabled ? ' disabled aria-disabled="true"' : ""}${state.hidden ? " hidden" : ""}`;
}

export function actionBusyState(
  action: string,
  busy: boolean,
): { disabled: boolean; hidden: boolean } {
  if (action === "cancel") {
    return { disabled: !busy, hidden: !busy };
  }
  return { disabled: busy, hidden: false };
}

export function applyActionBusy(root: ParentNode, busy: boolean): void {
  for (const button of root.querySelectorAll<HTMLButtonElement>("button[data-ox-action]")) {
    const state = actionBusyState(button.dataset.oxAction ?? "", busy);
    button.disabled = state.disabled;
    button.hidden = state.hidden;
    button.setAttribute("aria-disabled", state.disabled ? "true" : "false");
  }
  for (const widget of queryPlayWidgets(root)) {
    widget.setAttribute("aria-busy", busy ? "true" : "false");
  }
}

function renderPanel(
  id: string,
  label: string,
  current: string,
  preset: CodePlayPreset,
  html: string,
): string {
  return `<div class="ox-code-play__panel" role="tabpanel" tabindex="0" aria-label="${escapeHtml(label)}" data-panel="${id}"${hidden(current, id, preset)}>${html}</div>`;
}

function hidden(current: string, id: string, preset: CodePlayPreset): string {
  if (preset === "compact" && (id === "stdio" || id === "stderr")) {
    return "";
  }
  return current === id ? "" : " hidden";
}

function widgetLabel(payload: PlayPayload, definition: LanguageDefinition | undefined): string {
  const language = definition?.name ?? payload.language;
  return payload.title ? `${payload.title} (${language})` : `${language} Code Play`;
}

function queryPlayWidgets(root: ParentNode): HTMLElement[] {
  const widgets = [...root.querySelectorAll<HTMLElement>(".ox-code-play")];
  if (
    typeof HTMLElement !== "undefined" &&
    root instanceof HTMLElement &&
    root.matches(".ox-code-play")
  ) {
    widgets.unshift(root);
  }
  return widgets;
}
