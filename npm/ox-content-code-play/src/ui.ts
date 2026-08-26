import { resolveLanguage } from "./catalog";
import { escapeHtml } from "./escape";
import type { CodePlayPreset, LanguageDefinition, PlayPayload, RunResult } from "./types";
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
  const tabs = renderTabs(state, panel);
  const viewers = state.payload.viewers;
  return `<div class="ox-code-play ox-code-play--${preset}" data-ox-code-play-ui>
  <div class="ox-code-play__toolbar">
    <span class="ox-code-play__lang">${escapeHtml(definition?.name ?? state.payload.language)}</span>
    <button type="button" data-ox-action="run"${actionButtonAttrs("run", Boolean(state.busy))}>Run</button>
    ${canTypecheck ? `<button type="button" data-ox-action="typecheck"${actionButtonAttrs("typecheck", Boolean(state.busy))}>Typecheck</button>` : ""}
    <button type="button" data-ox-action="cancel"${actionButtonAttrs("cancel", Boolean(state.busy))}>Cancel</button>
  </div>
  ${renderRuntimeStrip(state.payload, definition)}
  <div class="ox-code-play__source"></div>
  ${tabs}
  ${viewers.stdio ? `<div class="ox-code-play__panel" data-panel="stdio">${renderDiagnosticsHtml(state.result)}${renderStdioHtml(state.result?.stdio ?? [])}</div>` : ""}
  ${viewers.stderr ? `<div class="ox-code-play__panel" data-panel="stderr"${hidden(panel, "stderr", preset)}>${renderStderrHtml(state.result)}</div>` : ""}
  <div class="ox-code-play__panel" data-panel="config"${hidden(panel, "config", preset)}>${renderConfigHtml(definition?.configSchema ?? [], state.payload.config)}</div>
  <div class="ox-code-play__panel" data-panel="provenance"${hidden(panel, "provenance", preset)}>${renderProvenanceHtml(state.result?.provenance)}</div>
  <div class="ox-code-play__panel" data-panel="timing"${hidden(panel, "timing", preset)}>${renderTimingHtml(state.result?.timing)}</div>
</div>`;
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
    viewers.stdio ? tab("stdio", "stdio", panel) : "",
    viewers.stderr ? tab("stderr", "stderr", panel) : "",
    viewers.config ? tab("config", "config", panel) : "",
    viewers.provenance ? tab("provenance", "provenance", panel) : "",
    viewers.timing ? tab("timing", "timing", panel) : "",
  ]
    .filter(Boolean)
    .join("");
  return buttons ? `<div class="ox-code-play__tabs" role="tablist">${buttons}</div>` : "";
}

function tab(id: string, label: string, selected: string): string {
  return `<button type="button" role="tab" data-ox-panel="${id}" aria-selected="${selected === id ? "true" : "false"}">${label}</button>`;
}

export function actionButtonAttrs(action: string, busy: boolean): string {
  const state = actionBusyState(action, busy);
  return `${state.disabled ? " disabled" : ""}${state.hidden ? " hidden" : ""}`;
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
  }
}

function hidden(current: string, id: string, preset: CodePlayPreset): string {
  if (preset === "compact" && (id === "stdio" || id === "stderr")) {
    return "";
  }
  return current === id ? "" : " hidden";
}
